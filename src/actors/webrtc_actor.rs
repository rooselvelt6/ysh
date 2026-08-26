use ractor::{async_trait, Actor, ActorProcessingErr, ActorRef};

pub struct WebRTCActor;

pub struct WebRTCActorState {
    max_concurrent_calls: u32,
}

#[async_trait]
impl Actor for WebRTCActor {
    type Msg = WebRTCActorMsg;
    type State = WebRTCActorState;
    type Arguments = u32;

    async fn pre_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        max_concurrent_calls: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        tracing::info!(
            "WebRTCActor starting, max_concurrent_calls: {}",
            max_concurrent_calls
        );
        Ok(WebRTCActorState {
            max_concurrent_calls,
        })
    }

    async fn handle(
        &self,
        _myself: ActorRef<Self::Msg>,
        msg: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match msg {
            WebRTCActorMsg::CallStart { caller, callee } => {
                tracing::info!(
                    "Call starting: {} -> {} (max: {})",
                    caller,
                    callee,
                    state.max_concurrent_calls
                );
            }
            WebRTCActorMsg::CallEnd { caller, callee } => {
                tracing::info!("Call ended: {} -> {}", caller, callee);
            }
        }
        Ok(())
    }
}

pub enum WebRTCActorMsg {
    CallStart { caller: String, callee: String },
    CallEnd { caller: String, callee: String },
}
