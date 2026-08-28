use ractor::{async_trait, Actor, ActorProcessingErr, ActorRef};
use std::sync::Arc;

use crate::config::settings::WebRtcConfig;
use crate::db::Database;

pub struct WebRTCActor;

pub struct WebRTCActorArguments {
    pub db: Arc<Database>,
    pub config: WebRtcConfig,
}

pub struct WebRTCActorState {
    db: Arc<Database>,
    config: WebRtcConfig,
    calls_started: u64,
    calls_ended: u64,
    quality_samples: u64,
    recordings_started: u64,
    billed_total: i64,
}

#[async_trait]
impl Actor for WebRTCActor {
    type Msg = WebRTCActorMsg;
    type State = WebRTCActorState;
    type Arguments = WebRTCActorArguments;

    async fn pre_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        tracing::info!(
            "WebRTCActor starting (mode: {}, p2p: {}, duo: {}, group: {})",
            args.config.signal_mode,
            args.config.p2p_capacity,
            args.config.duo_capacity,
            args.config.group_capacity
        );
        Ok(WebRTCActorState {
            db: args.db,
            config: args.config,
            calls_started: 0,
            calls_ended: 0,
            quality_samples: 0,
            recordings_started: 0,
            billed_total: 0,
        })
    }

    async fn handle(
        &self,
        _myself: ActorRef<Self::Msg>,
        msg: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match msg {
            WebRTCActorMsg::CallStart { call_id, caller_id, callee_id, call_type } => {
                state.calls_started += 1;
                state.db.create_call_record(&call_id, &call_id, caller_id, &call_type, &{
                    if callee_id > 0 { vec![caller_id, callee_id] } else { vec![caller_id] }
                })?;
                let host_id = if callee_id > 0 { callee_id } else { caller_id };
                let _ = state.db.start_call_billing(caller_id, host_id, &call_type, state.config.cost_per_minute);
                tracing::info!("WebRTC call started: {} ({})", call_id, call_type);
            }
            WebRTCActorMsg::CallEnd { call_id, caller_id } => {
                let duration = state.db.end_call_record(&call_id).unwrap_or(0);
                if let Some(cb_id) = state.db.find_active_call_billing(caller_id).ok().flatten().map(|cb| cb.id)
                    && let Some(res) = state
                        .db
                        .end_call_billing(cb_id)
                        .ok()
                        .and_then(|_| state.db.finalize_call_payment(cb_id).ok())
                {
                    state.billed_total += res["total_cost"].as_i64().unwrap_or(0);
                }
                state.calls_ended += 1;
                tracing::info!("WebRTC call ended: {} ({}s)", call_id, duration);
            }
            WebRTCActorMsg::QualityReport { call_id, user_id, bitrate_kbps, packet_loss_pct, rtt_ms, resolution, simulcast_tier } => {
                state.db.add_quality_sample(&call_id, user_id, bitrate_kbps, packet_loss_pct, rtt_ms, &resolution, &simulcast_tier)?;
                state.quality_samples += 1;
            }
            WebRTCActorMsg::RecordingStart { call_id } => {
                let encrypted = state.config.recording_encryption;
                let storage_key = if encrypted {
                    format!("enc://recordings/{}/{}", call_id, chrono::Utc::now().timestamp())
                } else {
                    format!("recordings/{}/{}", call_id, chrono::Utc::now().timestamp())
                };
                let _ = state.db.start_call_recording(&call_id, &storage_key, encrypted, 0);
                state.db.set_call_recording(&call_id, true, encrypted)?;
                state.recordings_started += 1;
            }
            WebRTCActorMsg::RecordingStop { call_id } => {
                let segments = state.db.list_call_recordings(&call_id).unwrap_or_default();
                for seg in segments.iter().filter(|s| s.status == "recording") {
                    let _ = state.db.finalize_call_recording(&call_id, seg.segment_id);
                }
            }
            WebRTCActorMsg::ScreenShare { call_id, user_id, active } => {
                state.db.set_call_screen_share(&call_id, user_id, active)?;
            }
            WebRTCActorMsg::GetStats { reply_to } => {
                let _ = reply_to.send(serde_json::json!({
                    "calls_started": state.calls_started,
                    "calls_ended": state.calls_ended,
                    "active_calls": state.calls_started.saturating_sub(state.calls_ended),
                    "quality_samples": state.quality_samples,
                    "recordings_started": state.recordings_started,
                    "billed_total": state.billed_total,
                    "signal_mode": state.config.signal_mode,
                }));
            }
        }
        Ok(())
    }
}

// NOTE: billing lookup is resolved by caller within CallEnd since the billing
// row is keyed by host in redb; the actor keeps it simple by scanning once.
pub enum WebRTCActorMsg {
    CallStart { call_id: String, caller_id: i64, callee_id: i64, call_type: String },
    CallEnd { call_id: String, caller_id: i64 },
    QualityReport { call_id: String, user_id: i64, bitrate_kbps: f64, packet_loss_pct: f64, rtt_ms: f64, resolution: String, simulcast_tier: String },
    RecordingStart { call_id: String },
    RecordingStop { call_id: String },
    ScreenShare { call_id: String, user_id: i64, active: bool },
    GetStats { reply_to: tokio::sync::oneshot::Sender<serde_json::Value> },
}