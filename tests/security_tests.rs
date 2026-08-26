#[cfg(test)]
mod crypto_tests {
    use ysh::security::crypto::{AesCipher, ChaChaCipher};
    use ysh::security::nonce::NonceGenerator;

    #[test]
    fn aes_encrypt_decrypt_roundtrip() {
        let key = [0x42u8; 32];
        let cipher = AesCipher::new(&key).unwrap();
        let nonce = NonceGenerator::new().next();
        let plaintext = b"Hello, YSH encryption!";
        let aad = b"context";

        let ciphertext = cipher.encrypt(&nonce, plaintext, aad).unwrap();
        assert_ne!(&ciphertext, plaintext);

        let decrypted = cipher.decrypt(&nonce, &ciphertext, aad).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn aes_wrong_key_fails() {
        let key1 = [0x42u8; 32];
        let key2 = [0x43u8; 32];
        let cipher1 = AesCipher::new(&key1).unwrap();
        let cipher2 = AesCipher::new(&key2).unwrap();
        let nonce = NonceGenerator::new().next();
        let plaintext = b"secret data";

        let ciphertext = cipher1.encrypt(&nonce, plaintext, b"").unwrap();
        let result = cipher2.decrypt(&nonce, &ciphertext, b"");
        assert!(result.is_err());
    }

    #[test]
    fn aes_wrong_aad_fails() {
        let key = [0x42u8; 32];
        let cipher = AesCipher::new(&key).unwrap();
        let nonce = NonceGenerator::new().next();
        let plaintext = b"secret data";

        let ciphertext = cipher.encrypt(&nonce, plaintext, b"correct_aad").unwrap();
        let result = cipher.decrypt(&nonce, &ciphertext, b"wrong_aad");
        assert!(result.is_err());
    }

    #[test]
    fn aes_empty_plaintext() {
        let key = [0x42u8; 32];
        let cipher = AesCipher::new(&key).unwrap();
        let nonce = NonceGenerator::new().next();

        let ciphertext = cipher.encrypt(&nonce, b"", b"").unwrap();
        let decrypted = cipher.decrypt(&nonce, &ciphertext, b"").unwrap();
        assert!(decrypted.is_empty());
    }

    #[test]
    fn chacha_encrypt_decrypt_roundtrip() {
        let key = [0x42u8; 32];
        let cipher = ChaChaCipher::new(&key).unwrap();
        let nonce = NonceGenerator::new().next();
        let plaintext = b"ChaCha20 Poly1305 test!";
        let aad = b"additional";

        let ciphertext = cipher.encrypt(&nonce, plaintext, aad).unwrap();
        assert_ne!(&ciphertext, plaintext);

        let decrypted = cipher.decrypt(&nonce, &ciphertext, aad).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn chacha_wrong_key_fails() {
        let key1 = [0x42u8; 32];
        let key2 = [0x43u8; 32];
        let cipher1 = ChaChaCipher::new(&key1).unwrap();
        let cipher2 = ChaChaCipher::new(&key2).unwrap();
        let nonce = NonceGenerator::new().next();

        let ciphertext = cipher1.encrypt(&nonce, b"test", b"").unwrap();
        let result = cipher2.decrypt(&nonce, &ciphertext, b"");
        assert!(result.is_err());
    }

    #[test]
    fn chacha_wrong_aad_fails() {
        let key = [0x42u8; 32];
        let cipher = ChaChaCipher::new(&key).unwrap();
        let nonce = NonceGenerator::new().next();

        let ciphertext = cipher.encrypt(&nonce, b"test", b"aad1").unwrap();
        let result = cipher.decrypt(&nonce, &ciphertext, b"aad2");
        assert!(result.is_err());
    }

    #[test]
    fn aes_large_data() {
        let key = [0xABu8; 32];
        let cipher = AesCipher::new(&key).unwrap();
        let nonce = NonceGenerator::new().next();
        let plaintext = vec![0x55u8; 10_000];

        let ciphertext = cipher.encrypt(&nonce, &plaintext, b"").unwrap();
        let decrypted = cipher.decrypt(&nonce, &ciphertext, b"").unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn chacha_large_data() {
        let key = [0xABu8; 32];
        let cipher = ChaChaCipher::new(&key).unwrap();
        let nonce = NonceGenerator::new().next();
        let plaintext = vec![0x55u8; 10_000];

        let ciphertext = cipher.encrypt(&nonce, &plaintext, b"").unwrap();
        let decrypted = cipher.decrypt(&nonce, &ciphertext, b"").unwrap();
        assert_eq!(decrypted, plaintext);
    }
}

#[cfg(test)]
mod nonce_tests {
    use ysh::security::nonce::NonceGenerator;

    #[test]
    fn nonce_is_12_bytes() {
        let ng = NonceGenerator::new();
        let nonce = ng.next();
        assert_eq!(nonce.len(), 12);
    }

    #[test]
    fn nonces_are_unique() {
        let ng = NonceGenerator::new();
        let mut seen = std::collections::HashSet::new();
        for _ in 0..1000 {
            let nonce = ng.next();
            assert!(seen.insert(nonce), "Duplicate nonce generated");
        }
    }

    #[test]
    fn counter_increments() {
        let ng = NonceGenerator::new();
        assert_eq!(ng.current_counter(), 0);
        ng.next();
        assert_eq!(ng.current_counter(), 1);
        ng.next();
        assert_eq!(ng.current_counter(), 2);
    }

    #[test]
    fn nonce_base_is_random() {
        let ng1 = NonceGenerator::new();
        let ng2 = NonceGenerator::new();
        let n1 = ng1.next();
        let n2 = ng2.next();
        assert_ne!(&n1[..4], &n2[..4], "Random bases should differ");
    }

    #[test]
    fn nonce_counter_in_bytes() {
        let ng = NonceGenerator::new();
        let nonce = ng.next();
        assert_eq!(&nonce[4..], &0u64.to_le_bytes());
        ng.next();
        let nonce2 = ng.next();
        assert_eq!(&nonce2[4..], &2u64.to_le_bytes());
    }
}

#[cfg(test)]
mod zeroize_tests {
    use ysh::security::zeroize::{EncryptedKey, SecureBuffer, SecureString};

    #[test]
    fn secure_buffer_operations() {
        let buf = SecureBuffer::new(vec![1, 2, 3, 4]);
        assert_eq!(buf.len(), 4);
        assert!(!buf.is_empty());
        assert_eq!(buf.as_bytes(), &[1, 2, 3, 4]);
    }

    #[test]
    fn secure_buffer_empty() {
        let buf = SecureBuffer::new(vec![]);
        assert!(buf.is_empty());
        assert_eq!(buf.len(), 0);
    }

    #[test]
    fn secure_buffer_from_slice() {
        let buf = SecureBuffer::from(b"hello".as_slice());
        assert_eq!(buf.as_bytes(), b"hello");
        assert_eq!(buf.len(), 5);
    }

    #[test]
    fn secure_buffer_from_vec() {
        let buf = SecureBuffer::from(vec![10, 20, 30]);
        assert_eq!(buf.as_bytes(), &[10, 20, 30]);
    }

    #[test]
    fn secure_buffer_deref() {
        let buf = SecureBuffer::new(vec![0x41, 0x42]);
        let slice: &[u8] = &buf;
        assert_eq!(slice, &[0x41, 0x42]);
    }

    #[test]
    fn secure_buffer_clone() {
        let buf = SecureBuffer::new(vec![1, 2, 3]);
        let cloned = buf.clone();
        assert_eq!(cloned.as_bytes(), &[1, 2, 3]);
    }

    #[test]
    fn secure_string_operations() {
        let s = SecureString::new("secret_key".to_string());
        assert_eq!(s.as_str(), "secret_key");
        assert_eq!(s.len(), 10);
    }

    #[test]
    fn secure_string_deref() {
        let s = SecureString::new("hello".to_string());
        let deref: &str = &s;
        assert_eq!(deref, "hello");
    }

    #[test]
    fn secure_string_from() {
        let s = SecureString::from("test".to_string());
        assert_eq!(s.as_str(), "test");
    }

    #[test]
    fn encrypted_key_operations() {
        let key = EncryptedKey::new(vec![1, 2, 3], "aes-256-gcm".to_string());
        assert_eq!(key.as_bytes(), &[1, 2, 3]);
        assert_eq!(key.algorithm(), "aes-256-gcm");
    }

    #[test]
    fn encrypted_key_clone() {
        let key = EncryptedKey::new(vec![10, 20], "chacha20".to_string());
        let cloned = key.clone();
        assert_eq!(cloned.as_bytes(), &[10, 20]);
        assert_eq!(cloned.algorithm(), "chacha20");
    }
}

#[cfg(test)]
mod keys_tests {
    use ysh::security::keys::{Ed25519KeyPair, X25519KeyPair};

    #[test]
    fn x25519_key_agreement() {
        let alice = X25519KeyPair::generate();
        let bob = X25519KeyPair::generate();
        let alice_pub = alice.public;

        let alice_shared = alice.agree(&bob.public);
        let bob_shared = bob.agree(&alice_pub);

        assert_eq!(alice_shared.as_bytes(), bob_shared.as_bytes());
    }

    #[test]
    fn x25519_different_keys_different_shared() {
        let alice = X25519KeyPair::generate();
        let bob1 = X25519KeyPair::generate();
        let bob2 = X25519KeyPair::generate();

        let shared1 = alice.agree(&bob1.public);
        // Can't reuse alice (consumed), so test with different setup
        let carol = X25519KeyPair::generate();
        let shared2 = carol.agree(&bob2.public);

        assert_ne!(shared1.as_bytes(), shared2.as_bytes());
    }

    #[test]
    fn x25519_public_key_32_bytes() {
        let keys = X25519KeyPair::generate();
        assert_eq!(keys.public.as_bytes().len(), 32);
    }

    #[test]
    fn ed25519_sign_verify() {
        let keys = Ed25519KeyPair::generate();
        let message = b"Hello, YSH!";

        let signature = keys.sign(message);
        assert!(keys.verify(message, &signature).is_ok());
    }

    #[test]
    fn ed25519_wrong_message_fails() {
        let keys = Ed25519KeyPair::generate();
        let signature = keys.sign(b"correct message");

        assert!(keys.verify(b"wrong message", &signature).is_err());
    }

    #[test]
    fn ed25519_different_keys_different_signature() {
        let keys1 = Ed25519KeyPair::generate();
        let keys2 = Ed25519KeyPair::generate();
        let message = b"test";

        let sig1 = keys1.sign(message);
        let sig2 = keys2.sign(message);
        assert_ne!(sig1.to_bytes(), sig2.to_bytes());
    }

    #[test]
    fn ed25519_verifying_key_32_bytes() {
        let keys = Ed25519KeyPair::generate();
        assert_eq!(keys.verifying_key.as_bytes().len(), 32);
    }

    #[test]
    fn ed25519_large_message() {
        let keys = Ed25519KeyPair::generate();
        let message = vec![0x42u8; 100_000];

        let signature = keys.sign(&message);
        assert!(keys.verify(&message, &signature).is_ok());
    }
}
