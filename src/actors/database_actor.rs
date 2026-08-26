use ractor::{async_trait, Actor, ActorProcessingErr, ActorRef};

pub struct DatabaseActor;

pub struct DatabaseActorState {
    pub url: String,
    pub max_connections: u32,
}

#[async_trait]
impl Actor for DatabaseActor {
    type Msg = DatabaseActorMsg;
    type State = DatabaseActorState;
    type Arguments = (String, u32);

    async fn pre_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        (url, max_connections): Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        tracing::info!(
            "DatabaseActor starting, url: {}, max_connections: {}",
            url,
            max_connections
        );
        Ok(DatabaseActorState {
            url,
            max_connections,
        })
    }

    async fn handle(
        &self,
        myself: ActorRef<Self::Msg>,
        msg: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match msg {
            DatabaseActorMsg::Query(sql) => {
                tracing::debug!(
                    "Query (pool max={}): {}",
                    state.max_connections,
                    sql
                );
            }
            DatabaseActorMsg::Connect => {
                tracing::info!(
                    "Connecting to database: {} (max_connections: {})",
                    state.url,
                    state.max_connections
                );
            }
            DatabaseActorMsg::Disconnect => {
                tracing::info!("Disconnecting from database");
                myself.stop(None);
            }
        }
        Ok(())
    }
}

pub enum DatabaseActorMsg {
    Query(String),
    Connect,
    Disconnect,
}
