use ractor::{Actor, ActorProcessingErr, ActorRef, async_trait};

pub struct SessionSupervisor;

pub struct SessionSupervisorState {
    pub active_sessions: u32,
    pub max_sessions: u32,
}

#[async_trait]
impl Actor for SessionSupervisor {
    type Msg = SessionSupervisorMsg;
    type State = SessionSupervisorState;
    type Arguments = u32;

    async fn pre_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        max_sessions: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        tracing::info!("SessionSupervisor starting, max_sessions: {}", max_sessions);
        Ok(SessionSupervisorState {
            active_sessions: 0,
            max_sessions,
        })
    }

    async fn handle(
        &self,
        _myself: ActorRef<Self::Msg>,
        msg: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match msg {
            SessionSupervisorMsg::SessionStarted { user_id } => {
                state.active_sessions += 1;
                tracing::info!(
                    "Session started for user: {} (active: {}/{})",
                    user_id,
                    state.active_sessions,
                    state.max_sessions
                );
            }
            SessionSupervisorMsg::SessionEnded { user_id } => {
                state.active_sessions = state.active_sessions.saturating_sub(1);
                tracing::info!(
                    "Session ended for user: {} (active: {}/{})",
                    user_id,
                    state.active_sessions,
                    state.max_sessions
                );
            }
            SessionSupervisorMsg::GetActiveCount => {
                tracing::debug!(
                    "Active sessions: {}/{}",
                    state.active_sessions,
                    state.max_sessions
                );
            }
        }
        Ok(())
    }
}

pub enum SessionSupervisorMsg {
    SessionStarted { user_id: String },
    SessionEnded { user_id: String },
    GetActiveCount,
}
