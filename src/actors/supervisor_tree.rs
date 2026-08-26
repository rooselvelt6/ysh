use ractor::{async_trait, Actor, ActorProcessingErr, ActorRef};

pub struct SupervisorTree;

pub struct SupervisorTreeState {
    config_path: String,
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
        _state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match msg {
            SupervisorTreeMsg::GetConfig => {
                tracing::debug!("Config requested from supervisor tree");
            }
            SupervisorTreeMsg::Shutdown => {
                tracing::info!("SupervisorTree shutting down");
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
