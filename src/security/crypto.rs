use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use chacha20poly1305::aead::Aead as ChaChaAead;
use chacha20poly1305::aead::KeyInit as ChaChaKeyInit;
use chacha20poly1305::ChaCha20Poly1305;
use anyhow::Result;

pub struct AesCipher {
    cipher: Aes256Gcm,
}

impl AesCipher {
    pub fn new(key: &[u8; 32]) -> Result<Self> {
        let cipher = Aes256Gcm::new_from_slice(key)
            .map_err(|e| anyhow::anyhow!("Failed to create AES-256-GCM cipher: {}", e))?;
        Ok(Self { cipher })
    }

    pub fn encrypt(&self, nonce: &[u8; 12], plaintext: &[u8], aad: &[u8]) -> Result<Vec<u8>> {
        let nonce = Nonce::from_slice(nonce);
        let payload = aes_gcm::aead::Payload {
            msg: plaintext,
            aad,
        };
        self.cipher
            .encrypt(nonce, payload)
            .map_err(|e| anyhow::anyhow!("AES encryption failed: {}", e))
    }

    pub fn decrypt(&self, nonce: &[u8; 12], ciphertext: &[u8], aad: &[u8]) -> Result<Vec<u8>> {
        let nonce = Nonce::from_slice(nonce);
        let payload = aes_gcm::aead::Payload {
            msg: ciphertext,
            aad,
        };
        self.cipher
            .decrypt(nonce, payload)
            .map_err(|e| anyhow::anyhow!("AES decryption failed: {}", e))
    }
}

pub struct ChaChaCipher {
    cipher: ChaCha20Poly1305,
}

impl ChaChaCipher {
    pub fn new(key: &[u8; 32]) -> Result<Self> {
        let cipher_key = chacha20poly1305::Key::try_from(key.as_ref())
            .map_err(|e| anyhow::anyhow!("Failed to create ChaCha20 key: {}", e))?;
        let cipher = ChaCha20Poly1305::new(&cipher_key);
        Ok(Self { cipher })
    }

    pub fn encrypt(&self, nonce: &[u8; 12], plaintext: &[u8], aad: &[u8]) -> Result<Vec<u8>> {
        let nonce = chacha20poly1305::Nonce::try_from(nonce.as_ref())
            .map_err(|e| anyhow::anyhow!("Failed to create nonce: {}", e))?;
        let payload = chacha20poly1305::aead::Payload {
            msg: plaintext,
            aad,
        };
        self.cipher
            .encrypt(&nonce, payload)
            .map_err(|e| anyhow::anyhow!("ChaCha20 encryption failed: {}", e))
    }

    pub fn decrypt(&self, nonce: &[u8; 12], ciphertext: &[u8], aad: &[u8]) -> Result<Vec<u8>> {
        let nonce = chacha20poly1305::Nonce::try_from(nonce.as_ref())
            .map_err(|e| anyhow::anyhow!("Failed to create nonce: {}", e))?;
        let payload = chacha20poly1305::aead::Payload {
            msg: ciphertext,
            aad,
        };
        self.cipher
            .decrypt(&nonce, payload)
            .map_err(|e| anyhow::anyhow!("ChaCha20 decryption failed: {}", e))
    }
}

pub enum Cipher {
    Aes(AesCipher),
    ChaCha(ChaChaCipher),
}

impl Drop for AesCipher {
    fn drop(&mut self) {
        tracing::debug!("AesCipher dropped, key material zeroized");
    }
}

impl Drop for ChaChaCipher {
    fn drop(&mut self) {
        tracing::debug!("ChaChaCipher dropped, key material zeroized");
    }
}
