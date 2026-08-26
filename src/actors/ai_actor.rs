use ractor::{async_trait, Actor, ActorProcessingErr, ActorRef};

pub struct AIActor;

pub struct AIActorState {
    model_loaded: bool,
}

#[async_trait]
impl Actor for AIActor {
    type Msg = AIActorMsg;
    type State = AIActorState;
    type Arguments = ();

    async fn pre_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        _args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        tracing::info!("AIActor starting");
        Ok(AIActorState { model_loaded: false })
    }

    async fn handle(
        &self,
        _myself: ActorRef<Self::Msg>,
        msg: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match msg {
            AIActorMsg::LoadModel => {
                tracing::info!("Loading AI model");
                state.model_loaded = true;
            }
            AIActorMsg::Moderate { content_id } => {
                tracing::debug!("Moderating content: {}", content_id);
            }
            AIActorMsg::DeepfakeCheck { user_id } => {
                tracing::debug!("Deepfake check for user: {}", user_id);
            }
        }
        Ok(())
    }
}

pub enum AIActorMsg {
    LoadModel,
    Moderate { content_id: String },
    DeepfakeCheck { user_id: String },
}
