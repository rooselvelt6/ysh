use ractor::{Actor, ActorProcessingErr, ActorRef, async_trait};

pub struct NotificationActor;

pub struct NotificationActorState {
    pub queued: u64,
    pub sent: u64,
    pub failed: u64,
}

#[async_trait]
impl Actor for NotificationActor {
    type Msg = NotificationMsg;
    type State = NotificationActorState;
    type Arguments = ();

    async fn pre_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        _args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        tracing::info!("NotificationActor starting");
        Ok(NotificationActorState {
            queued: 0,
            sent: 0,
            failed: 0,
        })
    }

    async fn handle(
        &self,
        _myself: ActorRef<Self::Msg>,
        msg: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match msg {
            NotificationMsg::SendEmail {
                notification_id,
                to,
                subject,
                body,
                username,
            } => {
                state.queued += 1;
                tracing::info!(
                    "Email queued: id={} to={} subject='{}' user={}",
                    notification_id,
                    to,
                    subject,
                    username,
                );
                tracing::debug!("Email body length: {} chars", body.len());
                tracing::info!(
                    "Notification stats: queued={}, sent={}, failed={}",
                    state.queued,
                    state.sent,
                    state.failed,
                );
            }
            NotificationMsg::SendPush {
                notification_id,
                tokens,
                title,
                body,
            } => {
                state.queued += 1;
                tracing::info!(
                    "Push queued: id={} tokens={} title='{}'",
                    notification_id,
                    tokens.len(),
                    title,
                );
                tracing::debug!("Push body: {}", body);
            }
            NotificationMsg::InApp {
                notification_id,
                user_id,
                title,
                body,
            } => {
                state.queued += 1;
                tracing::info!(
                    "In-app notification: id={} user={} title='{}'",
                    notification_id,
                    user_id,
                    title,
                );
                tracing::debug!("In-app body: {}", body);
            }
            NotificationMsg::MarkSent { notification_id } => {
                state.sent += 1;
                tracing::info!("Notification sent: id={}", notification_id);
            }
            NotificationMsg::MarkFailed {
                notification_id,
                reason,
            } => {
                state.failed += 1;
                tracing::warn!(
                    "Notification failed: id={} reason='{}'",
                    notification_id,
                    reason
                );
            }
            NotificationMsg::GetStats => {
                tracing::info!(
                    "NotificationActor stats: queued={}, sent={}, failed={}",
                    state.queued,
                    state.sent,
                    state.failed,
                );
            }
        }
        Ok(())
    }
}

#[allow(dead_code)]
pub enum NotificationMsg {
    SendEmail {
        notification_id: i64,
        to: String,
        subject: String,
        body: String,
        username: String,
    },
    SendPush {
        notification_id: i64,
        tokens: Vec<String>,
        title: String,
        body: String,
    },
    InApp {
        notification_id: i64,
        user_id: i64,
        title: String,
        body: String,
    },
    MarkSent {
        notification_id: i64,
    },
    MarkFailed {
        notification_id: i64,
        reason: String,
    },
    GetStats,
}
