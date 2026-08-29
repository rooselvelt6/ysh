use ractor::{Actor, ActorProcessingErr, ActorRef, async_trait};
use std::sync::Arc;

use crate::config::settings::JobsConfig;
use crate::db::Database;

pub struct JobsActor;

pub struct JobsActorArguments {
    pub db: Arc<Database>,
    pub config: JobsConfig,
}

pub struct JobsActorState {
    db: Arc<Database>,
    config: JobsConfig,
    runs: serde_json::Map<String, serde_json::Value>,
    last_run_at: std::collections::HashMap<String, String>,
}

#[async_trait]
impl Actor for JobsActor {
    type Msg = JobsActorMsg;
    type State = JobsActorState;
    type Arguments = JobsActorArguments;

    async fn pre_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        tracing::info!(
            "JobsActor starting (interval: {}s, payouts: {}, staking: {}, moderation: {}, cleanup: {}, notifications: {}, analytics: {})",
            args.config.interval_secs,
            args.config.payouts,
            args.config.staking,
            args.config.moderation,
            args.config.cleanup,
            args.config.notifications,
            args.config.analytics
        );
        Ok(JobsActorState {
            db: args.db,
            config: args.config,
            runs: serde_json::Map::new(),
            last_run_at: std::collections::HashMap::new(),
        })
    }

    async fn handle(
        &self,
        _myself: ActorRef<Self::Msg>,
        msg: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match msg {
            JobsActorMsg::RunPayouts => {
                let result = state.db.auto_process_payouts()?;
                state.record("payouts", &result);
            }
            JobsActorMsg::RunStaking => {
                let result = state.db.compute_staking_interest()?;
                state.record("staking", &result);
            }
            JobsActorMsg::RunModeration => {
                let result = state.db.auto_resolve_moderation(
                    state.config.moderation_auto_resolve_secs,
                    state.config.moderation_dismiss_below,
                    state.config.moderation_action_above,
                )?;
                state.record("moderation", &result);
            }
            JobsActorMsg::RunCleanup => {
                let result = state.db.cleanup_expired(
                    state.config.analytics_retention_days,
                    state.config.quality_retention_days,
                )?;
                state.record("cleanup", &result);
            }
            JobsActorMsg::RunNotifications => {
                let result = state.db.flush_pending_notifications()?;
                state.record("notifications", &result);
            }
            JobsActorMsg::RunAnalyticsSnapshot => {
                let result = state.db.compute_analytics_snapshot()?;
                state.record("analytics", &result);
            }
            JobsActorMsg::RunAll => {
                if state.config.enabled {
                    if state.config.payouts {
                        let _ = _myself.send_message(JobsActorMsg::RunPayouts);
                    }
                    if state.config.staking {
                        let _ = _myself.send_message(JobsActorMsg::RunStaking);
                    }
                    if state.config.moderation {
                        let _ = _myself.send_message(JobsActorMsg::RunModeration);
                    }
                    if state.config.cleanup {
                        let _ = _myself.send_message(JobsActorMsg::RunCleanup);
                    }
                    if state.config.notifications {
                        let _ = _myself.send_message(JobsActorMsg::RunNotifications);
                    }
                    if state.config.analytics {
                        let _ = _myself.send_message(JobsActorMsg::RunAnalyticsSnapshot);
                    }
                }
            }
            JobsActorMsg::GetStats { reply_to } => {
                let _ = reply_to.send(serde_json::json!({
                    "enabled": state.config.enabled,
                    "interval_secs": state.config.interval_secs,
                    "last_run_at": state.last_run_at,
                    "jobs": state.runs,
                }));
            }
        }
        Ok(())
    }
}

impl JobsActorState {
    fn record(&mut self, name: &str, result: &serde_json::Value) {
        let mut entry = serde_json::json!({
            "result": result,
            "run_at": chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            "runs": 1,
        });
        if let Some(prev) = self.runs.get(name) {
            entry["last_result"] = prev["result"].clone();
            entry["runs"] = serde_json::Value::from(prev["runs"].as_i64().unwrap_or(0) + 1);
        }
        self.runs.insert(name.to_string(), entry);
        self.last_run_at
            .insert(name.to_string(), chrono::Utc::now().to_rfc3339());
    }
}

pub enum JobsActorMsg {
    RunPayouts,
    RunStaking,
    RunModeration,
    RunCleanup,
    RunNotifications,
    RunAnalyticsSnapshot,
    RunAll,
    GetStats {
        reply_to: tokio::sync::oneshot::Sender<serde_json::Value>,
    },
}
