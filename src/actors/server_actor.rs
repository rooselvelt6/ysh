use ractor::{async_trait, Actor, ActorProcessingErr, ActorRef};

pub struct ServerActor;

pub struct ServerActorState {
    host: String,
    port: u16,
}

#[async_trait]
impl Actor for ServerActor {
    type Msg = ServerActorMsg;
    type State = ServerActorState;
    type Arguments = (String, u16);

    async fn pre_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        (host, port): Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        tracing::info!("ServerActor starting on {}:{}", host, port);
        Ok(ServerActorState { host, port })
    }

    async fn handle(
        &self,
        myself: ActorRef<Self::Msg>,
        msg: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match msg {
            ServerActorMsg::Start => {
                tracing::info!("Server starting on {}:{}", state.host, state.port);
            }
            ServerActorMsg::Stop => {
                tracing::info!("Server stopping");
                myself.stop(None);
            }
        }
        Ok(())
    }
}

pub enum ServerActorMsg {
    Start,
    Stop,
}
