use anyhow::Result;
use ed25519_dalek::{Signer, SigningKey, Verifier, VerifyingKey};
use x25519_dalek::{EphemeralSecret, PublicKey, SharedSecret};

pub struct X25519KeyPair {
    secret: EphemeralSecret,
    pub public: PublicKey,
}

impl X25519KeyPair {
    pub fn generate() -> Self {
        use rand_core::OsRng;
        let secret = EphemeralSecret::random_from_rng(OsRng);
        let public = PublicKey::from(&secret);
        Self { secret, public }
    }

    pub fn agree(self, peer_public: &PublicKey) -> SharedSecret {
        self.secret.diffie_hellman(peer_public)
    }
}

pub struct Ed25519KeyPair {
    signing_key: SigningKey,
    pub verifying_key: VerifyingKey,
}

impl Ed25519KeyPair {
    pub fn generate() -> Self {
        use rand_core::OsRng;
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let verifying_key = signing_key.verifying_key();
        Self {
            signing_key,
            verifying_key,
        }
    }

    pub fn sign(&self, message: &[u8]) -> ed25519_dalek::Signature {
        self.signing_key.sign(message)
    }

    pub fn verify(
        &self,
        message: &[u8],
        signature: &ed25519_dalek::Signature,
    ) -> Result<(), ed25519_dalek::SignatureError> {
        self.verifying_key.verify(message, signature)
    }
}
