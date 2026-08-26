use ractor::{async_trait, Actor, ActorProcessingErr, ActorRef};

pub struct ConfigActor;

pub struct ConfigActorState {
    config_path: String,
}

#[async_trait]
impl Actor for ConfigActor {
    type Msg = ConfigActorMsg;
    type State = ConfigActorState;
    type Arguments = String;

    async fn pre_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        config_path: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        tracing::info!("ConfigActor started, watching: {}", config_path);
        Ok(ConfigActorState { config_path })
    }

    async fn handle(
        &self,
        _myself: ActorRef<Self::Msg>,
        msg: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match msg {
            ConfigActorMsg::Reload => {
                tracing::info!("Reloading config from: {}", state.config_path);
            }
            ConfigActorMsg::ConfigChanged(path) => {
                tracing::info!("Config file changed: {}", path);
            }
        }
        Ok(())
    }
}

pub enum ConfigActorMsg {
    Reload,
    ConfigChanged(String),
}
