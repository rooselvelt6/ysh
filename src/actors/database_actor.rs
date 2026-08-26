use ractor::{async_trait, Actor, ActorProcessingErr, ActorRef};

use crate::db::Database;
use std::sync::Arc;

pub struct DatabaseActor;

pub struct DatabaseActorState {
    pub db: Arc<Database>,
    _url: String,
    _max_connections: u32,
    pub queries_executed: u64,
}

#[async_trait]
impl Actor for DatabaseActor {
    type Msg = DatabaseActorMsg;
    type State = DatabaseActorState;
    type Arguments = (Arc<Database>, String, u32);

    async fn pre_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        (db, url, max_connections): Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        tracing::info!(
            "DatabaseActor starting, url: {}, max_connections: {}",
            url,
            max_connections
        );
        Ok(DatabaseActorState {
            db,
            _url: url,
            _max_connections: max_connections,
            queries_executed: 0,
        })
    }

    async fn handle(
        &self,
        _myself: ActorRef<Self::Msg>,
        msg: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match msg {
            DatabaseActorMsg::HealthCheck => {
                match state.db.health_check() {
                    Ok(()) => tracing::debug!("DB health check passed"),
                    Err(e) => tracing::error!("DB health check failed: {}", e),
                }
            }
            DatabaseActorMsg::QueryCount => {
                state.queries_executed += 1;
                match state.db.user_count() {
                    Ok(count) => {
                        tracing::info!(
                            "Users: {}, queries executed: {}",
                            count,
                            state.queries_executed
                        );
                    }
                    Err(e) => tracing::error!("User count failed: {}", e),
                }
            }
            DatabaseActorMsg::GetStats => {
                let users = state.db.user_count().unwrap_or(0);
                let devices = state.db.session_count().unwrap_or(0);
                tracing::info!(
                    "DB stats - users: {}, devices: {}, queries: {}",
                    users,
                    devices,
                    state.queries_executed
                );
            }
        }
        Ok(())
    }
}

pub enum DatabaseActorMsg {
    HealthCheck,
    QueryCount,
    GetStats,
}
