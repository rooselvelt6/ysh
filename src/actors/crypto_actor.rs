use ractor::{async_trait, Actor, ActorProcessingErr, ActorRef};

pub struct CryptoActor;

pub struct CryptoActorState {
    algorithm: String,
}

#[async_trait]
impl Actor for CryptoActor {
    type Msg = CryptoActorMsg;
    type State = CryptoActorState;
    type Arguments = String;

    async fn pre_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        algorithm: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        tracing::info!("CryptoActor starting with algorithm: {}", algorithm);
        Ok(CryptoActorState { algorithm })
    }

    async fn handle(
        &self,
        _myself: ActorRef<Self::Msg>,
        msg: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match msg {
            CryptoActorMsg::Encrypt { plaintext, aad: _ } => {
                tracing::debug!(
                    "Encrypting {} bytes with {}",
                    plaintext.len(),
                    state.algorithm
                );
            }
            CryptoActorMsg::Decrypt { ciphertext, aad: _ } => {
                tracing::debug!(
                    "Decrypting {} bytes with {}",
                    ciphertext.len(),
                    state.algorithm
                );
            }
            CryptoActorMsg::RotateKeys => {
                tracing::info!("Rotating encryption keys");
            }
        }
        Ok(())
    }
}

pub enum CryptoActorMsg {
    Encrypt { plaintext: Vec<u8>, aad: Vec<u8> },
    Decrypt { ciphertext: Vec<u8>, aad: Vec<u8> },
    RotateKeys,
}
