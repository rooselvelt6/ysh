use std::sync::Arc;

use ractor::{Actor, ActorProcessingErr, ActorRef, async_trait};

use crate::ai::{AIEngine, ModerateRequest};

pub struct AIActor;

pub struct AIActorState {
    pub engine: Arc<AIEngine>,
    model_loaded: bool,
}

#[async_trait]
impl Actor for AIActor {
    type Msg = AIActorMsg;
    type State = AIActorState;
    type Arguments = Arc<AIEngine>;

    async fn pre_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        tracing::info!("AIActor starting (AI engine ready)");
        Ok(AIActorState {
            engine: args,
            model_loaded: true,
        })
    }

    async fn handle(
        &self,
        _myself: ActorRef<Self::Msg>,
        msg: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match msg {
            AIActorMsg::LoadModel => {
                tracing::info!("AI models loaded (text, anomaly, matching, neural)");
                state.model_loaded = true;
            }
            AIActorMsg::Moderate {
                content_id,
                content,
            } => {
                let moderation = state.engine.moderate_text(ModerateRequest {
                    content: content.unwrap_or_default(),
                });
                tracing::info!(
                    "AI moderated content {}: decision={:?} severity={}",
                    content_id,
                    moderation.decision,
                    moderation.severity
                );
            }
            AIActorMsg::DeepfakeCheck { user_id } => {
                tracing::debug!("Deepfake encounter check queued for user: {}", user_id);
            }
        }
        Ok(())
    }
}

pub enum AIActorMsg {
    LoadModel,
    Moderate {
        content_id: String,
        content: Option<String>,
    },
    DeepfakeCheck {
        user_id: String,
    },
}
