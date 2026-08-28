use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};

use crate::actors::webrtc_actor::WebRTCActorMsg;
use crate::auth::jwt::AuthUser;
use crate::server::AppState;
use crate::webrtc::{CallType, valid_simulcast_tier};

fn parse_uid(auth: &AuthUser) -> Result<i64, (StatusCode, String)> {
    auth.user_id.parse().map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token".into()))
}

fn http_err(e: &anyhow::Error) -> (StatusCode, String) {
    (StatusCode::BAD_REQUEST, format!("{:#}", e))
}

fn new_call_id(caller_id: i64) -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("call-{}-{}", caller_id, nanos)
}

pub async fn start_call(
    auth: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let caller_id = parse_uid(&auth)?;
    let _ = state.db.log_activity(caller_id, "call");
    let call_type_s = req["call_type"].as_str().ok_or((StatusCode::BAD_REQUEST, "call_type required".into()))?;
    let call_type = CallType::parse(call_type_s).ok_or((StatusCode::BAD_REQUEST, "Invalid call_type".into()))?;
    if !state.config.webrtc.enabled {
        return Err((StatusCode::FORBIDDEN, "WebRTC streaming is disabled".into()));
    }

    let title = req["title"].as_str().map(|s| s.to_string());
    let call_id = new_call_id(caller_id);

    let (callee_id, participant_ids) = match call_type {
        CallType::Flash => {
            let peer = state.db.find_random_peer(caller_id).map_err(|e| http_err(&e))?
                .ok_or((StatusCode::NOT_FOUND, "No available peer for flash call".into()))?;
            (peer, vec![caller_id, peer])
        }
        CallType::Live => (0, vec![caller_id]),
        CallType::P2P => {
            let target = req["target_user_id"].as_i64().ok_or((StatusCode::BAD_REQUEST, "target_user_id required".into()))?;
            if target == caller_id {
                return Err((StatusCode::BAD_REQUEST, "Cannot call yourself".into()));
            }
            (target, vec![caller_id, target])
        }
        CallType::Duo | CallType::Group => {
            let target = req["target_user_id"].as_i64();
            let mut ids = vec![caller_id];
            if let Some(t) = target {
                ids.push(t);
            }
            (target.unwrap_or(0), ids)
        }
    };

    {
        let mut rooms = state.webrtc_rooms.lock().await;
        if rooms.room_exists(&call_id) {
            return Err((StatusCode::CONFLICT, "Room id collision".into()));
        }
        rooms.create_room(&call_id, call_type, caller_id, title);
    }

    state
        .webrtc_actor
        .cast(WebRTCActorMsg::CallStart {
            call_id: call_id.clone(),
            caller_id,
            callee_id,
            call_type: call_type.as_str().to_string(),
        })
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{:#}", e)))?;

    Ok(Json(serde_json::json!({
        "call_id": call_id,
        "call_type": call_type.as_str(),
        "host_id": caller_id,
        "participants": participant_ids,
        "simulcast_tiers": state.config.webrtc.simulcast_tiers,
    })))
}

pub async fn join_call(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(call_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id = parse_uid(&auth)?;
    if state.db.get_call_record(&call_id).map_err(|e| http_err(&e))?.is_none()
        && !{
            let rooms = state.webrtc_rooms.lock().await;
            rooms.room_exists(&call_id)
        }
    {
        return Err((StatusCode::NOT_FOUND, "Call not found".into()));
    }

    let mut rooms = state.webrtc_rooms.lock().await;
    let outcome = rooms.join(&call_id, user_id);
    if !outcome.accepted {
        return Err((StatusCode::CONFLICT, outcome.reason.unwrap_or_default()));
    }
    drop(rooms);

    state.db.join_call(&call_id, user_id).map_err(|e| http_err(&e))?;

    Ok(Json(serde_json::json!({
        "call_id": call_id,
        "mode": "sfu_passthrough",
        "participants": outcome.participants,
        "viewer_count": outcome.viewer_count,
    })))
}

pub async fn leave_call(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(call_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id = parse_uid(&auth)?;
    let mut rooms = state.webrtc_rooms.lock().await;
    if let Some(outcome) = rooms.leave(&call_id, user_id) {
        drop(rooms);
        state.db.leave_call(&call_id, user_id).map_err(|e| http_err(&e))?;
        Ok(Json(serde_json::json!({ "left": true, "room_empty": outcome.room_empty, "participants": outcome.participants, "viewer_count": outcome.viewer_count })))
    } else {
        Err((StatusCode::NOT_FOUND, "Room not found".into()))
    }
}

pub async fn end_call(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(call_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let host_id = parse_uid(&auth)?;
    let mut rooms = state.webrtc_rooms.lock().await;
    let room = rooms.get_room(&call_id).cloned().ok_or((StatusCode::NOT_FOUND, "Room not found".into()))?;
    if room.host_id != host_id {
        return Err((StatusCode::FORBIDDEN, "Only the host can end the call".into()));
    }
    rooms.end_room(&call_id);
    drop(rooms);

    let participants = room.participants.clone();
    state
        .webrtc_actor
        .cast(WebRTCActorMsg::CallEnd { call_id: call_id.clone(), caller_id: room.host_id })
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{:#}", e)))?;

    Ok(Json(serde_json::json!({ "ended": true, "call_id": call_id, "participants": participants })))
}

pub async fn toggle_screen_share(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(call_id): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id = parse_uid(&auth)?;
    let active = req["active"].as_bool().ok_or((StatusCode::BAD_REQUEST, "active required".into()))?;

    let mut rooms = state.webrtc_rooms.lock().await;
    rooms.set_screen_share(&call_id, user_id, active).map_err(|e| (StatusCode::CONFLICT, e))?;
    drop(rooms);

    state
        .webrtc_actor
        .cast(WebRTCActorMsg::ScreenShare { call_id: call_id.clone(), user_id, active })
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{:#}", e)))?;

    Ok(Json(serde_json::json!({ "screen_share": active })))
}

pub async fn start_recording(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(call_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id = parse_uid(&auth)?;
    if !state.config.webrtc.recording_enabled {
        return Err((StatusCode::FORBIDDEN, "Call recording is disabled".into()));
    }

    {
        let mut rooms = state.webrtc_rooms.lock().await;
        rooms.set_recording(&call_id, true, state.config.webrtc.recording_encryption).map_err(|e| (StatusCode::CONFLICT, e))?;
    }

    state
        .webrtc_actor
        .cast(WebRTCActorMsg::RecordingStart { call_id: call_id.clone() })
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{:#}", e)))?;

    let _ = user_id;
    Ok(Json(serde_json::json!({
        "recording": true,
        "encrypted": state.config.webrtc.recording_encryption,
    })))
}

pub async fn stop_recording(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(call_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let host_id = parse_uid(&auth)?;
    let mut rooms = state.webrtc_rooms.lock().await;
    let room = rooms.get_room(&call_id).cloned().ok_or((StatusCode::NOT_FOUND, "Room not found".into()))?;
    if room.host_id != host_id {
        return Err((StatusCode::FORBIDDEN, "Only the host can stop the recording".into()));
    }
    rooms.set_recording(&call_id, false, false).map_err(|e| (StatusCode::CONFLICT, e))?;
    drop(rooms);

    state
        .webrtc_actor
        .cast(WebRTCActorMsg::RecordingStop { call_id: call_id.clone() })
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{:#}", e)))?;

    Ok(Json(serde_json::json!({ "recording": false })))
}

pub async fn report_quality(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(call_id): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id = parse_uid(&auth)?;
    let tier = req["simulcast_tier"].as_str().unwrap_or("q").to_string();
    if !valid_simulcast_tier(&state.config.webrtc.simulcast_tiers, &tier) {
        return Err((StatusCode::BAD_REQUEST, "Invalid simulcast tier".into()));
    }

    state
        .webrtc_actor
        .cast(WebRTCActorMsg::QualityReport {
            call_id: call_id.clone(),
            user_id,
            bitrate_kbps: req["bitrate_kbps"].as_f64().unwrap_or(0.0),
            packet_loss_pct: req["packet_loss_pct"].as_f64().unwrap_or(0.0),
            rtt_ms: req["rtt_ms"].as_f64().unwrap_or(0.0),
            resolution: req["resolution"].as_str().unwrap_or("0x0").to_string(),
            simulcast_tier: tier,
        })
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{:#}", e)))?;

    Ok(Json(serde_json::json!({ "recorded": true })))
}

pub async fn get_call(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(call_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    parse_uid(&auth)?;
    let record = state.db.get_call_record(&call_id).map_err(|e| http_err(&e))?;
    let live_room = {
        let rooms = state.webrtc_rooms.lock().await;
        rooms.get_room(&call_id).map(|r| serde_json::to_value(r).unwrap_or_default())
    };
    let quality = state.db.aggregate_quality(&call_id).map_err(|e| http_err(&e))?;
    let recordings = state.db.list_call_recordings(&call_id).map_err(|e| http_err(&e))?;

    Ok(Json(serde_json::json!({
        "call_id": call_id,
        "record": record,
        "live_room": live_room,
        "quality": quality,
        "recordings": recordings.len(),
    })))
}

pub async fn call_quality(
    State(state): State<AppState>,
    Path(call_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let metrics = state.db.aggregate_quality(&call_id).map_err(|e| http_err(&e))?;
    let samples = state.db.get_quality_metrics(&call_id).map_err(|e| http_err(&e))?;
    Ok(Json(serde_json::json!({ "aggregate": metrics, "last_n": samples.len().min(20) })))
}

pub async fn call_history(
    auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id = parse_uid(&auth)?;
    let history = state.db.get_call_history(user_id, 100).map_err(|e| http_err(&e))?;
    Ok(Json(serde_json::json!({ "calls": history })))
}

pub async fn live_streams(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let rooms = state.webrtc_rooms.lock().await;
    Ok(Json(serde_json::json!({ "live": rooms.list_live() })))
}

pub async fn active_rooms(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let rooms = state.webrtc_rooms.lock().await;
    Ok(Json(serde_json::json!({ "rooms": rooms.active_rooms() })))
}

pub async fn call_stats(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let db_stats = state.db.get_call_stats().map_err(|e| http_err(&e))?;
    Ok(Json(db_stats))
}

pub async fn webrtc_stats(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    state
        .webrtc_actor
        .cast(WebRTCActorMsg::GetStats { reply_to: tx })
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{:#}", e)))?;
    let stats = rx.await.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Actor stopped".into()))?;
    Ok(Json(stats))
}

pub async fn room_peers(
    State(state): State<AppState>,
    Path(call_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let rooms = state.webrtc_rooms.lock().await;
    let room = rooms.get_room(&call_id).ok_or((StatusCode::NOT_FOUND, "Room not found".into()))?;
    Ok(Json(serde_json::json!({
        "participants": room.participants,
        "viewers": room.viewers,
        "screen_share": room.screen_share,
        "call_type": room.call_type.as_str(),
    })))
}

pub async fn update_live_title(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(call_id): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let host_id = parse_uid(&auth)?;
    let title = req["title"].as_str().ok_or((StatusCode::BAD_REQUEST, "title required".into()))?.to_string();

    let mut rooms = state.webrtc_rooms.lock().await;
    let room = rooms.get_room(&call_id).cloned().ok_or((StatusCode::NOT_FOUND, "Room not found".into()))?;
    if room.host_id != host_id {
        return Err((StatusCode::FORBIDDEN, "Only the host can set the title".into()));
    }
    rooms.set_title(&call_id, title.clone()).map_err(|e| (StatusCode::CONFLICT, e))?;
    Ok(Json(serde_json::json!({ "title": title })))
}