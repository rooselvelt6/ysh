use ractor::{Actor, ActorProcessingErr, ActorRef, async_trait};

pub struct SupervisorTree;

pub struct SupervisorTreeState {
    pub config_path: String,
}

#[async_trait]
impl Actor for SupervisorTree {
    type Msg = SupervisorTreeMsg;
    type State = SupervisorTreeState;
    type Arguments = String;

    async fn pre_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        config_path: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        tracing::info!("SupervisorTree starting with OTP supervision");
        Ok(SupervisorTreeState { config_path })
    }

    async fn handle(
        &self,
        myself: ActorRef<Self::Msg>,
        msg: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match msg {
            SupervisorTreeMsg::GetConfig => {
                tracing::debug!(
                    "Config requested from supervisor tree (watching: {})",
                    state.config_path
                );
            }
            SupervisorTreeMsg::Shutdown => {
                tracing::info!(
                    "SupervisorTree shutting down (was watching: {})",
                    state.config_path
                );
                myself.stop(None);
            }
        }
        Ok(())
    }
}

pub enum SupervisorTreeMsg {
    GetConfig,
    Shutdown,
}
