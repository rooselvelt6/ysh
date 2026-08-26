#[cfg(test)]
mod password_tests {
    use ysh::security::password::{hash_blake3, hash_password, verify_blake3, verify_password};

    #[test]
    fn hash_and_verify_password() {
        let password = "SuperSecure123!";
        let hash = hash_password(password).unwrap();
        assert!(verify_password(password, &hash).unwrap());
    }

    #[test]
    fn wrong_password_fails() {
        let hash = hash_password("correct_password").unwrap();
        assert!(!verify_password("wrong_password", &hash).unwrap());
    }

    #[test]
    fn different_hashes_for_same_password() {
        let password = "same_password";
        let hash1 = hash_password(password).unwrap();
        let hash2 = hash_password(password).unwrap();
        assert_ne!(hash1, hash2, "Argon2 uses random salts");
        assert!(verify_password(password, &hash1).unwrap());
        assert!(verify_password(password, &hash2).unwrap());
    }

    #[test]
    fn hash_is_argon2id_format() {
        let hash = hash_password("test").unwrap();
        assert!(hash.starts_with("$argon2id$"), "Hash should be Argon2id format");
    }

    #[test]
    fn empty_password() {
        let hash = hash_password("").unwrap();
        assert!(verify_password("", &hash).unwrap());
        assert!(!verify_password("not_empty", &hash).unwrap());
    }

    #[test]
    fn long_password() {
        let password = "a".repeat(1000);
        let hash = hash_password(&password).unwrap();
        assert!(verify_password(&password, &hash).unwrap());
    }

    #[test]
    fn unicode_password() {
        let password = "contraseña_ñ_ü_中文";
        let hash = hash_password(password).unwrap();
        assert!(verify_password(password, &hash).unwrap());
    }

    #[test]
    fn blake3_hash_deterministic() {
        let data = b"test data";
        let h1 = hash_blake3(data);
        let h2 = hash_blake3(data);
        assert_eq!(h1, h2);
    }

    #[test]
    fn blake3_hash_hex_length() {
        let hash = hash_blake3(b"test");
        assert_eq!(hash.to_hex().len(), 64, "Blake3 produces 64-char hex");
    }

    #[test]
    fn verify_blake3_correct() {
        let data = b"hello world";
        let hash = hash_blake3(data);
        assert!(verify_blake3(data, &hash.to_hex()).unwrap());
    }

    #[test]
    fn verify_blake3_wrong() {
        let data = b"hello world";
        let hash = hash_blake3(data);
        assert!(!verify_blake3(b"hello universe", &hash.to_hex()).unwrap());
    }

    #[test]
    fn blake3_different_inputs_different_hashes() {
        let h1 = hash_blake3(b"input1");
        let h2 = hash_blake3(b"input2");
        assert_ne!(h1, h2);
    }
}

#[cfg(test)]
mod totp_tests {
    use ysh::security::totp::{
        base32_encode, generate_recovery_codes, generate_secret, generate_uri,
        hash_recovery_code, verify_code, verify_recovery_code,
    };

    #[test]
    fn base32_encode_basic() {
        assert_eq!(base32_encode(b""), "");
        assert_eq!(base32_encode(b"f"), "MY");
        assert_eq!(base32_encode(b"fo"), "MZXQ");
        assert_eq!(base32_encode(b"foo"), "MZXW6");
    }

    #[test]
    fn base32_encode_all_uppercase() {
        let encoded = base32_encode(b"hello world test");
        assert_eq!(encoded, encoded.to_uppercase());
    }

    #[test]
    fn generate_secret_returns_20_bytes() {
        let (secret, encoded) = generate_secret();
        assert_eq!(secret.len(), 20);
        assert!(!encoded.is_empty());
    }

    #[test]
    fn generate_secret_unique() {
        let (_, enc1) = generate_secret();
        let (_, enc2) = generate_secret();
        assert_ne!(enc1, enc2);
    }

    #[test]
    fn generate_uri_format() {
        let uri = generate_uri("JBSWY3DPEHPK3PXP", "user@example.com", "YSH");
        assert!(uri.starts_with("otpauth://totp/YSH:user@example.com"));
        assert!(uri.contains("secret=JBSWY3DPEHPK3PXP"));
        assert!(uri.contains("issuer=YSH"));
        assert!(uri.contains("algorithm=SHA1"));
        assert!(uri.contains("digits=6"));
        assert!(uri.contains("period=30"));
    }

    #[test]
    fn verify_code_valid() {
        let (secret, _) = generate_secret();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let time_step = now / 30;

        use hmac::Mac;
        let mut mac = <hmac::Hmac::<sha1::Sha1> as hmac::Mac>::new_from_slice(&secret).unwrap();
        mac.update(&time_step.to_be_bytes());
        let result = mac.finalize().into_bytes();
        let offset = (result[19] & 0x0f) as usize;
        let code = u32::from_be_bytes([
            result[offset] & 0x7f,
            result[offset + 1],
            result[offset + 2],
            result[offset + 3],
        ]) % 1_000_000;
        let code_str = format!("{:06}", code);

        assert!(verify_code(&secret, &code_str));
    }

    #[test]
    fn verify_code_invalid() {
        let (secret, _) = generate_secret();
        assert!(!verify_code(&secret, "000000"));
        assert!(!verify_code(&secret, "123456"));
        assert!(!verify_code(&secret, "999999"));
    }

    #[test]
    fn verify_code_wrong_length() {
        let (secret, _) = generate_secret();
        assert!(!verify_code(&secret, "12345"));
        assert!(!verify_code(&secret, "1234567"));
    }

    #[test]
    fn recovery_codes_correct_count() {
        let codes = generate_recovery_codes(10);
        assert_eq!(codes.len(), 10);
    }

    #[test]
    fn recovery_codes_format() {
        let codes = generate_recovery_codes(5);
        for code in &codes {
            let parts: Vec<&str> = code.split('-').collect();
            assert_eq!(parts.len(), 3, "Recovery code should have 3 parts: {}", code);
        }
    }

    #[test]
    fn recovery_codes_unique() {
        let codes = generate_recovery_codes(100);
        let mut set = std::collections::HashSet::new();
        for code in &codes {
            assert!(set.insert(code.clone()), "Duplicate recovery code: {}", code);
        }
    }

    #[test]
    fn hash_and_verify_recovery_code() {
        let codes = generate_recovery_codes(1);
        let code = &codes[0];
        let hash = hash_recovery_code(code);
        assert!(verify_recovery_code(code, &hash));
    }

    #[test]
    fn wrong_recovery_code_fails() {
        let codes = generate_recovery_codes(1);
        let hash = hash_recovery_code(&codes[0]);
        assert!(!verify_recovery_code("XXXXX-XXXX-XXXX", &hash));
    }
}
