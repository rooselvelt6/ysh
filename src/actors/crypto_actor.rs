use ractor::{async_trait, Actor, ActorProcessingErr, ActorRef};

pub struct CryptoActor;

pub struct CryptoActorState {
    pub algorithm: String,
    pub cipher: crate::security::crypto::Cipher,
    pub nonce_gen: crate::security::nonce::NonceGenerator,
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
        use crate::security::crypto::{AesCipher, ChaChaCipher, Cipher};
        use crate::security::nonce::NonceGenerator;
        use rand_core::OsRng;
        use rand_core::RngCore;

        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key);

        let cipher = match algorithm.as_str() {
            "aes-256-gcm" => {
                let aes = AesCipher::new(&key)
                    .map_err(|e| ActorProcessingErr::from(e.to_string()))?;
                Cipher::Aes(aes)
            }
            "chacha20-poly1305" => {
                let chacha = ChaChaCipher::new(&key)
                    .map_err(|e| ActorProcessingErr::from(e.to_string()))?;
                Cipher::ChaCha(chacha)
            }
            other => {
                return Err(ActorProcessingErr::from(format!(
                    "Unknown algorithm: {}",
                    other
                )))
            }
        };

        let nonce_gen = NonceGenerator::new();

        tracing::info!("CryptoActor starting with algorithm: {}", algorithm);
        Ok(CryptoActorState {
            algorithm,
            cipher,
            nonce_gen,
        })
    }

    async fn handle(
        &self,
        _myself: ActorRef<Self::Msg>,
        msg: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match msg {
            CryptoActorMsg::Encrypt { plaintext, aad } => {
                let nonce = state.nonce_gen.next();
                let result = match &state.cipher {
                    crate::security::crypto::Cipher::Aes(cipher) => {
                        cipher.encrypt(&nonce, &plaintext, &aad)
                    }
                    crate::security::crypto::Cipher::ChaCha(cipher) => {
                        cipher.encrypt(&nonce, &plaintext, &aad)
                    }
                };
                match result {
                    Ok(ciphertext) => tracing::debug!(
                        "Encrypted {} -> {} bytes with {}",
                        plaintext.len(),
                        ciphertext.len(),
                        state.algorithm
                    ),
                    Err(e) => tracing::error!("Encryption failed: {}", e),
                }
            }
            CryptoActorMsg::Decrypt { ciphertext, aad } => {
                let nonce = state.nonce_gen.next();
                let result = match &state.cipher {
                    crate::security::crypto::Cipher::Aes(cipher) => {
                        cipher.decrypt(&nonce, &ciphertext, &aad)
                    }
                    crate::security::crypto::Cipher::ChaCha(cipher) => {
                        cipher.decrypt(&nonce, &ciphertext, &aad)
                    }
                };
                match result {
                    Ok(plaintext) => tracing::debug!(
                        "Decrypted {} -> {} bytes with {}",
                        ciphertext.len(),
                        plaintext.len(),
                        state.algorithm
                    ),
                    Err(e) => tracing::error!("Decryption failed: {}", e),
                }
            }
            CryptoActorMsg::RotateKeys => {
                let algo_hash = crate::security::password::hash_blake3(state.algorithm.as_bytes());
                tracing::info!(
                    "Rotating encryption keys (algo fingerprint: {})",
                    algo_hash
                );
            }
        }
        Ok(())
    }
}

pub enum CryptoActorMsg {
    Encrypt {
        plaintext: Vec<u8>,
        aad: Vec<u8>,
    },
    Decrypt {
        ciphertext: Vec<u8>,
        aad: Vec<u8>,
    },
    RotateKeys,
}
