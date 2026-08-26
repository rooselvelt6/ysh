use axum::extract::ws::{Message, WebSocket};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::broadcast;

use crate::security::token::validate_token;

pub type WsSender = tokio::sync::mpsc::UnboundedSender<String>;
pub type UserBroadcast = broadcast::Sender<String>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum ClientMessage {
    ChatMessage {
        session_id: i64,
        content: String,
        encrypted: bool,
    },
    ChatHistory {
        session_id: i64,
        limit: Option<i64>,
        before_id: Option<i64>,
    },
    Typing {
        session_id: i64,
        is_typing: bool,
    },
    ReadReceipt {
        session_id: i64,
        message_id: i64,
    },
    MatchJoin {
        mode: String,
        preferences: serde_json::Value,
    },
    MatchCancel,
    MatchTimerExtend,
    KnockKnockAccept {
        session_id: i64,
    },
    SetStatus {
        status: String,
    },
    Ping,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum ServerMessage {
    ChatMessage {
        session_id: i64,
        message_id: i64,
        sender_id: i64,
        content: String,
        encrypted: bool,
        created_at: String,
    },
    ChatHistory {
        session_id: i64,
        messages: Vec<serde_json::Value>,
    },
    Typing {
        session_id: i64,
        user_id: i64,
        is_typing: bool,
    },
    ReadReceipt {
        session_id: i64,
        message_id: i64,
        user_id: i64,
    },
    ReadBatch {
        session_id: i64,
        count: usize,
        user_id: i64,
    },
    MatchFound {
        session_id: i64,
        user_id: i64,
        username: String,
        mode: String,
    },
    MatchQueued {
        queue_id: i64,
        mode: String,
        position: i64,
        timer_seconds: u64,
    },
    MatchCancelled,
    MatchTimerTick {
        remaining: u64,
    },
    KnockKnockInvite {
        session_id: i64,
        from_user_id: i64,
        from_username: String,
        timer_seconds: u64,
    },
    MatchDeclined {
        reason: String,
    },
    PresenceUpdate {
        user_id: i64,
        status: String,
    },
    UserOnline {
        user_id: i64,
    },
    UserOffline {
        user_id: i64,
    },
    Error {
        message: String,
    },
    Pong,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsAuthQuery {
    pub token: String,
}

pub struct ConnectionManager {
    pub connections: HashMap<i64, WsSender>,
    pub online_users: HashMap<i64, String>,
    pub broadcast: UserBroadcast,
}

impl ConnectionManager {
    pub fn new() -> Self {
        let (broadcast, _) = broadcast::channel(1024);
        Self {
            connections: HashMap::new(),
            online_users: HashMap::new(),
            broadcast,
        }
    }

    pub fn add(&mut self, user_id: i64, sender: WsSender, status: String) {
        self.connections.insert(user_id, sender);
        self.online_users.insert(user_id, status);
    }

    pub fn remove(&mut self, user_id: i64) {
        self.connections.remove(&user_id);
        self.online_users.remove(&user_id);
        let _ = self.broadcast.send(serde_json::to_string(&ServerMessage::UserOffline { user_id }).unwrap());
    }

    #[allow(dead_code)]
    pub fn get(&self, user_id: i64) -> Option<&WsSender> {
        self.connections.get(&user_id)
    }

    pub fn send_to(&self, user_id: i64, msg: &ServerMessage) {
        if let Some(sender) = self.connections.get(&user_id) {
            if let Ok(json) = serde_json::to_string(msg) {
                let _ = sender.send(json);
            }
        }
    }

    pub fn broadcast(&self, msg: &ServerMessage) {
        if let Ok(json) = serde_json::to_string(msg) {
            let _ = self.broadcast.send(json);
        }
    }

    #[allow(dead_code)]
    pub fn is_online(&self, user_id: i64) -> bool {
        self.connections.contains_key(&user_id)
    }

    #[allow(dead_code)]
    pub fn online_count(&self) -> usize {
        self.connections.len()
    }

    pub fn update_status(&mut self, user_id: i64, status: String) {
        if let Some(s) = self.online_users.get_mut(&user_id) {
            *s = status.clone();
        }
        self.broadcast(&ServerMessage::PresenceUpdate {
            user_id,
            status,
        });
    }
}

pub async fn handle_ws(socket: WebSocket, query: WsAuthQuery, state: crate::server::AppState) {
    let secret = match std::env::var("YSH_JWT_SECRET") {
        Ok(s) => s,
        Err(_) => return,
    };

    let claims = match validate_token(&query.token, secret.as_bytes()) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("WS auth failed: {}", e);
            return;
        }
    };

    if claims.kind == "2fa_pending" {
        tracing::warn!("WS rejected: 2fa_pending token");
        return;
    }

    let user_id: i64 = match claims.sub.parse() {
        Ok(id) => id,
        Err(_) => return,
    };

    tracing::info!("WebSocket connected: user_id={}", user_id);

    let (mut ws_sender, mut ws_receiver) = socket.split();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();

    {
        let mut mgr = state.ws_connections.lock().await;
        mgr.add(user_id, tx.clone(), "online".into());
        let _ = mgr.broadcast.send(
            serde_json::to_string(&ServerMessage::UserOnline { user_id }).unwrap(),
        );
    }

    let read_receipts = state.read_receipts.clone();
    let db = state.db.clone();
    let ws_connections = state.ws_connections.clone();
    let ws_connections_clone = ws_connections.clone();
    let match_tx = state.match_tx.clone();

    let mut send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_sender.send(Message::Text(msg.into())).await.is_err() {
                break;
            }
        }
    });

    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = ws_receiver.next().await {
            match msg {
                Message::Text(text) => {
                    let text_str: &str = &text;
                    let client_msg: ClientMessage = match serde_json::from_str(text_str) {
                        Ok(m) => m,
                        Err(_) => {
                            let err = ServerMessage::Error { message: "Invalid message format".into() };
                            if let Ok(json) = serde_json::to_string(&err) {
                                let _ = tx.send(json);
                            }
                            continue;
                        }
                    };

                    match client_msg {
                        ClientMessage::ChatMessage { session_id, content, encrypted } => {
                            let msg_id = match db.send_message(session_id, user_id, &content, "text", encrypted) {
                                Ok(id) => id,
                                Err(e) => {
                                    tracing::error!("Failed to save message: {}", e);
                                    let err = ServerMessage::Error { message: e.to_string() };
                                    if let Ok(json) = serde_json::to_string(&err) {
                                        let _ = tx.send(json);
                                    }
                                    continue;
                                }
                            };

                            let participants = match db.get_session_participants(session_id) {
                                Ok(p) => p,
                                Err(_) => continue,
                            };

                            let created_at = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
                            let server_msg = ServerMessage::ChatMessage {
                                session_id,
                                message_id: msg_id,
                                sender_id: user_id,
                                content,
                                encrypted,
                                created_at,
                            };

                            let mgr = ws_connections_clone.lock().await;
                            for p in &participants {
                                if let Some(uid) = p["user_id"].as_i64() {
                                    mgr.send_to(uid, &server_msg);
                                }
                            }
                        }
                        ClientMessage::ChatHistory { session_id, limit, before_id } => {
                            let lim = limit.unwrap_or(50);
                            match db.get_messages(session_id, lim, before_id) {
                                Ok(messages) => {
                                    let server_msg = ServerMessage::ChatHistory {
                                        session_id,
                                        messages,
                                    };
                                    let mgr = ws_connections_clone.lock().await;
                                    mgr.send_to(user_id, &server_msg);
                                }
                                Err(e) => {
                                    let err = ServerMessage::Error { message: e.to_string() };
                                    if let Ok(json) = serde_json::to_string(&err) {
                                        let _ = tx.send(json);
                                    }
                                }
                            }
                        }
                        ClientMessage::Typing { session_id, is_typing } => {
                            let participants = match db.get_session_participants(session_id) {
                                Ok(p) => p,
                                Err(_) => continue,
                            };
                            let typing_msg = ServerMessage::Typing {
                                session_id,
                                user_id,
                                is_typing,
                            };
                            let mgr = ws_connections_clone.lock().await;
                            for p in &participants {
                                if let Some(uid) = p["user_id"].as_i64() {
                                    if uid != user_id {
                                        mgr.send_to(uid, &typing_msg);
                                    }
                                }
                            }
                        }
                        ClientMessage::ReadReceipt { session_id, message_id } => {
                            let _ = db.mark_messages_read(session_id, user_id);
                            let participants = match db.get_session_participants(session_id) {
                                Ok(p) => p,
                                Err(_) => continue,
                            };
                            {
                                let mut receipts = read_receipts.lock().await;
                                receipts.insert((session_id, user_id), message_id);
                            }
                            let read_msg = ServerMessage::ReadReceipt {
                                session_id,
                                message_id,
                                user_id,
                            };
                            let mgr = ws_connections_clone.lock().await;
                            for p in &participants {
                                if let Some(uid) = p["user_id"].as_i64() {
                                    if uid != user_id {
                                        mgr.send_to(uid, &read_msg);
                                    }
                                }
                            }
                        }
                        ClientMessage::MatchJoin { mode, preferences } => {
                            let prefs_str = preferences.to_string();
                            match db.enqueue_match(user_id, &mode, &prefs_str) {
                                Ok(queue_id) => {
                                    let queue_size = db.get_queue_size().unwrap_or(0);
                                    let timer = 15u64;
                                    let queued_msg = ServerMessage::MatchQueued {
                                        queue_id,
                                        mode: mode.clone(),
                                        position: queue_size,
                                        timer_seconds: timer,
                                    };
                                    {
                                        let mgr = ws_connections_clone.lock().await;
                                        mgr.send_to(user_id, &queued_msg);
                                    }

                                    let match_tx2 = match_tx.clone();
                                    let db2 = db.clone();
                                    let ws2 = ws_connections_clone.clone();
                                    let uid = user_id;
                                    let m = mode.clone();
                                    tokio::spawn(async move {
                                        match_attempt_loop(uid, &m, db2, ws2, match_tx2, timer).await;
                                    });
                                }
                                Err(e) => {
                                    let err = ServerMessage::Error { message: e.to_string() };
                                    if let Ok(json) = serde_json::to_string(&err) {
                                        let _ = tx.send(json);
                                    }
                                }
                            }
                        }
                        ClientMessage::MatchCancel => {
                            let _ = db.dequeue_match(user_id);
                            let mgr = ws_connections_clone.lock().await;
                            mgr.send_to(user_id, &ServerMessage::MatchCancelled);
                        }
                        ClientMessage::MatchTimerExtend => {
                            let timer = 30u64;
                            let mgr = ws_connections_clone.lock().await;
                            mgr.send_to(user_id, &ServerMessage::MatchTimerTick { remaining: timer });
                        }
                        ClientMessage::KnockKnockAccept { session_id } => {
                            let participants = match db.get_session_participants(session_id) {
                                Ok(p) => p,
                                Err(_) => continue,
                            };
                            let msg = ServerMessage::ChatHistory {
                                session_id,
                                messages: Vec::new(),
                            };
                            let mgr = ws_connections_clone.lock().await;
                            for p in &participants {
                                if let Some(uid) = p["user_id"].as_i64() {
                                    mgr.send_to(uid, &msg);
                                }
                            }
                        }
                        ClientMessage::SetStatus { status } => {
                            let mut mgr = ws_connections_clone.lock().await;
                            mgr.update_status(user_id, status);
                        }
                        ClientMessage::Ping => {
                            let mgr = ws_connections_clone.lock().await;
                            mgr.send_to(user_id, &ServerMessage::Pong);
                        }
                    }
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
    });

    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    }

    let mut mgr = ws_connections.lock().await;
    mgr.remove(user_id);
    tracing::info!("WebSocket disconnected: user_id={}", user_id);
}

async fn match_attempt_loop(
    user_id: i64,
    mode: &str,
    db: Arc<crate::db::Database>,
    ws_connections: Arc<tokio::sync::Mutex<ConnectionManager>>,
    _match_tx: tokio::sync::mpsc::UnboundedSender<MatchEvent>,
    timer_seconds: u64,
) {
    let mut tick = tokio::time::interval(std::time::Duration::from_secs(1));
    let mut remaining = timer_seconds;

    loop {
        tick.tick().await;

        if remaining == 0 {
            let _ = db.dequeue_match(user_id);
            let mgr = ws_connections.lock().await;
            mgr.send_to(user_id, &ServerMessage::MatchDeclined {
                reason: "Timer expired".into(),
            });
            break;
        }

        {
            let mgr = ws_connections.lock().await;
            mgr.send_to(user_id, &ServerMessage::MatchTimerTick { remaining });
        }

        let matched_user = match mode {
            "random" => db.find_random_match(user_id).ok().flatten(),
            _ => db.find_match(user_id, mode).ok().flatten(),
        };

        if let Some(other_id) = matched_user {
            let session_id = match db.find_direct_session(user_id, other_id) {
                Ok(Some(sid)) => sid,
                Ok(None) => {
                    match db.create_chat_session("direct", &[user_id, other_id]) {
                        Ok(sid) => sid,
                        Err(e) => {
                            tracing::error!("Failed to create session: {}", e);
                            break;
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to find session: {}", e);
                    break;
                }
            };

            let other_username = db.find_user_by_id(other_id)
                .ok()
                .flatten()
                .map(|u| u.username)
                .unwrap_or_else(|| "unknown".into());

            let my_username = db.find_user_by_id(user_id)
                .ok()
                .flatten()
                .map(|u| u.username)
                .unwrap_or_else(|| "unknown".into());

            let mgr = ws_connections.lock().await;
            mgr.send_to(user_id, &ServerMessage::MatchFound {
                session_id,
                user_id: other_id,
                username: other_username.clone(),
                mode: mode.to_string(),
            });
            mgr.send_to(other_id, &ServerMessage::MatchFound {
                session_id,
                user_id,
                username: my_username,
                mode: mode.to_string(),
            });
            break;
        }

        remaining -= 1;
    }
}

#[allow(dead_code)]
pub enum MatchEvent {
    Found(i64, i64),
    Cancelled(i64),
}
