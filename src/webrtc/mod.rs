//! FASE 8 — WebRTC + Streaming: call room coordinate.
//!
//! Pure in-memory room state machine. Persistence/Billing happen
//! through the [`crate::actors::webrtc_actor`] (DB) and the REST/WS layers.

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum CallType {
    P2P,
    Flash,
    Duo,
    Group,
    Live,
}

impl CallType {
    pub fn as_str(self) -> &'static str {
        match self {
            CallType::P2P => "p2p",
            CallType::Flash => "flash",
            CallType::Duo => "duo",
            CallType::Group => "group",
            CallType::Live => "live",
        }
    }

    pub fn parse(s: &str) -> Option<CallType> {
        match s {
            "p2p" => Some(CallType::P2P),
            "flash" => Some(CallType::Flash),
            "duo" => Some(CallType::Duo),
            "group" => Some(CallType::Group),
            "live" => Some(CallType::Live),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Room {
    pub id: String,
    pub call_type: CallType,
    pub host_id: i64,
    pub participants: Vec<i64>,
    pub viewers: Vec<i64>,
    pub screen_share: Option<i64>,
    pub recording: bool,
    pub recording_encrypted: bool,
    pub title: Option<String>,
    pub started_at: i64,
}

pub fn now_secs() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

#[derive(Debug, Clone)]
pub struct JoinOutcome {
    pub accepted: bool,
    pub reason: Option<String>,
    pub participants: Vec<i64>,
    pub viewer_count: usize,
}

#[derive(Debug, Clone)]
pub struct LeaveOutcome {
    pub room_empty: bool,
    pub participants: Vec<i64>,
    pub viewer_count: usize,
}

pub struct RoomManager {
    rooms: HashMap<String, Room>,
    p2p_capacity: usize,
    duo_capacity: usize,
    group_capacity: usize,
    max_live_viewers: usize,
}

impl RoomManager {
    pub fn new(
        p2p_capacity: u32,
        duo_capacity: u32,
        group_capacity: u32,
        max_live_viewers: u32,
    ) -> Self {
        Self {
            rooms: HashMap::new(),
            p2p_capacity: p2p_capacity as usize,
            duo_capacity: duo_capacity as usize,
            group_capacity: group_capacity as usize,
            max_live_viewers: max_live_viewers as usize,
        }
    }

    pub fn capacity(&self, call_type: CallType) -> usize {
        match call_type {
            CallType::P2P | CallType::Flash => self.p2p_capacity,
            CallType::Duo => self.duo_capacity,
            CallType::Group => self.group_capacity,
            CallType::Live => self.max_live_viewers,
        }
    }

    pub fn create_room(
        &mut self,
        id: &str,
        call_type: CallType,
        host_id: i64,
        title: Option<String>,
    ) {
        self.rooms.insert(
            id.to_string(),
            Room {
                id: id.to_string(),
                call_type,
                host_id,
                participants: vec![host_id],
                viewers: Vec::new(),
                screen_share: None,
                recording: false,
                recording_encrypted: false,
                title,
                started_at: now_secs(),
            },
        );
    }

    pub fn get_room(&self, room_id: &str) -> Option<&Room> {
        self.rooms.get(room_id)
    }

    pub fn room_exists(&self, room_id: &str) -> bool {
        self.rooms.contains_key(room_id)
    }

    pub fn join(&mut self, room_id: &str, user_id: i64) -> JoinOutcome {
        let Some(room) = self.rooms.get(room_id) else {
            return JoinOutcome {
                accepted: false,
                reason: Some("Room not found".into()),
                participants: vec![],
                viewer_count: 0,
            };
        };
        let call_type = room.call_type;
        let cap = self.capacity(call_type);
        let room = self.rooms.get_mut(room_id).expect("room exists");

        if call_type != CallType::Live {
            if room.participants.contains(&user_id) {
                return JoinOutcome {
                    accepted: false,
                    reason: Some("Already in this room".into()),
                    participants: room.participants.clone(),
                    viewer_count: room.viewers.len(),
                };
            }
            if room.participants.len() >= cap {
                return JoinOutcome {
                    accepted: false,
                    reason: Some("Room capacity reached".into()),
                    participants: room.participants.clone(),
                    viewer_count: room.viewers.len(),
                };
            }
            room.participants.push(user_id);
            JoinOutcome {
                accepted: true,
                reason: None,
                participants: room.participants.clone(),
                viewer_count: room.viewers.len(),
            }
        } else {
            if room.participants.contains(&user_id) || room.viewers.contains(&user_id) {
                return JoinOutcome {
                    accepted: false,
                    reason: Some("Already joined".into()),
                    participants: room.participants.clone(),
                    viewer_count: room.viewers.len(),
                };
            }
            if room.viewers.len() >= self.max_live_viewers {
                return JoinOutcome {
                    accepted: false,
                    reason: Some("Max live viewers reached".into()),
                    participants: room.participants.clone(),
                    viewer_count: room.viewers.len(),
                };
            }
            room.viewers.push(user_id);
            JoinOutcome {
                accepted: true,
                reason: None,
                participants: room.participants.clone(),
                viewer_count: room.viewers.len(),
            }
        }
    }

    pub fn leave(&mut self, room_id: &str, user_id: i64) -> Option<LeaveOutcome> {
        let room = self.rooms.get_mut(room_id)?;
        room.participants.retain(|p| *p != user_id);
        room.viewers.retain(|v| *v != user_id);
        if room.screen_share == Some(user_id) {
            room.screen_share = None;
        }
        Some(LeaveOutcome {
            room_empty: room.participants.is_empty() && room.viewers.is_empty(),
            participants: room.participants.clone(),
            viewer_count: room.viewers.len(),
        })
    }

    pub fn end_room(&mut self, room_id: &str) -> Option<Room> {
        self.rooms.remove(room_id)
    }

    pub fn set_screen_share(
        &mut self,
        room_id: &str,
        user_id: i64,
        active: bool,
    ) -> Result<bool, String> {
        let Some(room) = self.rooms.get_mut(room_id) else {
            return Err("Room not found".into());
        };
        if !room.participants.contains(&user_id) {
            return Err("Only participants can share screen".into());
        }
        room.screen_share = if active { Some(user_id) } else { None };
        Ok(active)
    }

    pub fn set_recording(
        &mut self,
        room_id: &str,
        active: bool,
        encrypted: bool,
    ) -> Result<bool, String> {
        let Some(room) = self.rooms.get_mut(room_id) else {
            return Err("Room not found".into());
        };
        if active && room.participants.first() != Some(&room.host_id) {
            return Err("Only the host can start a recording".into());
        }
        room.recording = active;
        room.recording_encrypted = encrypted;
        Ok(active)
    }

    pub fn set_title(&mut self, room_id: &str, title: String) -> Result<(), String> {
        let Some(room) = self.rooms.get_mut(room_id) else {
            return Err("Room not found".into());
        };
        room.title = Some(title);
        Ok(())
    }

    pub fn list_live(&self) -> Vec<serde_json::Value> {
        self.rooms
            .values()
            .filter(|r| r.call_type == CallType::Live)
            .map(|r| {
                serde_json::json!({
                    "room_id": r.id,
                    "host_id": r.host_id,
                    "title": r.title,
                    "viewers": r.viewers.len(),
                    "screen_share": r.screen_share.is_some(),
                    "recording": r.recording,
                    "started_at": r.started_at,
                })
            })
            .collect()
    }

    pub fn active_rooms(&self) -> Vec<serde_json::Value> {
        self.rooms
            .values()
            .map(|r| {
                serde_json::json!({
                    "room_id": r.id,
                    "call_type": r.call_type.as_str(),
                    "host_id": r.host_id,
                    "participants": r.participants,
                    "viewers": r.viewers.len(),
                    "screen_share": r.screen_share,
                    "recording": r.recording,
                    "title": r.title,
                })
            })
            .collect()
    }

    #[allow(dead_code)]
    pub fn room_count(&self) -> usize {
        self.rooms.len()
    }
}

pub fn valid_simulcast_tier(configured: &[String], tier: &str) -> bool {
    configured.iter().any(|t| t == tier)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mgr() -> RoomManager {
        RoomManager::new(2, 3, 8, 1000)
    }

    #[test]
    fn p2p_capacity_is_two() {
        let mut m = mgr();
        m.create_room("c1", CallType::P2P, 1, None);
        assert!(m.join("c1", 2).accepted);
        assert!(!m.join("c1", 3).accepted);
        assert_eq!(
            m.join("c1", 3).reason.as_deref(),
            Some("Room capacity reached")
        );
    }

    #[test]
    fn flash_pair_auto_creates_room() {
        let mut m = mgr();
        m.create_room("f1", CallType::Flash, 10, None);
        m.join("f1", 11);
        let r = m.get_room("f1").unwrap();
        assert_eq!(r.call_type, CallType::Flash);
        assert_eq!(r.participants.len(), 2);
    }

    #[test]
    fn duo_holds_three_group_holds_eight() {
        let mut m = mgr();
        m.create_room("d1", CallType::Duo, 1, None);
        m.join("d1", 2);
        assert!(m.join("d1", 3).accepted);
        assert!(!m.join("d1", 4).accepted);

        m.create_room("g1", CallType::Group, 1, None);
        for u in 2..=8 {
            assert!(m.join("g1", u).accepted, "user {} should fit in group", u);
        }
        assert!(!m.join("g1", 9).accepted);
    }

    #[test]
    fn live_streams_viewers_capped() {
        let mut m = RoomManager::new(2, 3, 8, 3);
        m.create_room("l1", CallType::Live, 1, Some("Concierto".into()));
        for v in 10..13 {
            assert!(m.join("l1", v).accepted);
        }
        assert!(!m.join("l1", 13).accepted);
        assert_eq!(m.get_room("l1").unwrap().viewers.len(), 3);
        let live = m.list_live();
        assert_eq!(live.len(), 1);
        assert_eq!(live[0]["viewers"], 3);
        assert_eq!(live[0]["title"], "Concierto");
    }

    #[test]
    fn leave_clears_screen_share_and_tracks_emptiness() {
        let mut m = mgr();
        m.create_room("c1", CallType::Group, 1, None);
        m.join("c1", 2);
        m.set_screen_share("c1", 2, true).unwrap();
        let out = m.leave("c1", 2).unwrap();
        assert!(!out.room_empty);
        assert!(m.get_room("c1").unwrap().screen_share.is_none());

        let out = m.leave("c1", 1).unwrap();
        assert!(out.room_empty);

        m.end_room("c1");
        assert!(
            m.leave("c1", 3).is_none(),
            "leaving an ended room is a no-op"
        );
    }

    #[test]
    fn end_room_removes_it() {
        let mut m = mgr();
        m.create_room("c1", CallType::P2P, 1, None);
        assert_eq!(m.room_count(), 1);
        m.end_room("c1");
        assert_eq!(m.room_count(), 0);
    }

    #[test]
    fn screen_share_requires_participant() {
        let mut m = mgr();
        m.create_room("c1", CallType::Duo, 1, None);
        assert!(m.set_screen_share("c1", 99, true).is_err());
        assert!(m.set_screen_share("c1", 1, true).is_ok());
    }

    #[test]
    fn simulcast_tier_validation() {
        let tiers = vec!["q".to_string(), "h".to_string(), "f".to_string()];
        assert!(valid_simulcast_tier(&tiers, "q"));
        assert!(valid_simulcast_tier(&tiers, "f"));
        assert!(!valid_simulcast_tier(&tiers, "x"));
    }
}
