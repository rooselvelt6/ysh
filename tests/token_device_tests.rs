#[cfg(test)]
mod token_tests {
    use ysh::security::token::{
        create_refresh_token, create_token, create_token_with_kind, validate_token, Claims,
    };

    const SECRET: &[u8] = b"test_secret_key_32_bytes_long!!!";

    #[test]
    fn create_and_validate_token() {
        let token = create_token("1", "user", SECRET).unwrap();
        let claims = validate_token(&token, SECRET).unwrap();
        assert_eq!(claims.sub, "1");
        assert_eq!(claims.role, "user");
        assert_eq!(claims.kind, "access");
    }

    #[test]
    fn token_with_custom_kind() {
        let token =
            create_token_with_kind("42", "admin", SECRET, "2fa_pending", 1).unwrap();
        let claims = validate_token(&token, SECRET).unwrap();
        assert_eq!(claims.sub, "42");
        assert_eq!(claims.kind, "2fa_pending");
        assert_eq!(claims.role, "admin");
    }

    #[test]
    fn token_expiry_in_future() {
        let token = create_token("1", "user", SECRET).unwrap();
        let claims = validate_token(&token, SECRET).unwrap();
        let now = chrono::Utc::now().timestamp() as usize;
        assert!(claims.exp > now, "Token should expire in the future");
    }

    #[test]
    fn token_iat_is_recent() {
        let token = create_token("1", "user", SECRET).unwrap();
        let claims = validate_token(&token, SECRET).unwrap();
        let now = chrono::Utc::now().timestamp() as usize;
        assert!(
            claims.iat <= now && now - claims.iat < 5,
            "iat should be within 5 seconds of now"
        );
    }

    #[test]
    fn wrong_secret_fails() {
        let token = create_token("1", "user", SECRET).unwrap();
        let wrong_secret = b"wrong_secret_key_32_bytes_long!!";
        assert!(validate_token(&token, wrong_secret).is_err());
    }

    #[test]
    fn invalid_token_string_fails() {
        assert!(validate_token("not.a.jwt.token", SECRET).is_err());
    }

    #[test]
    fn empty_token_fails() {
        assert!(validate_token("", SECRET).is_err());
    }

    #[test]
    fn refresh_token_kind() {
        let token = create_refresh_token("1", "user", SECRET, 30).unwrap();
        let claims = validate_token(&token, SECRET).unwrap();
        assert_eq!(claims.kind, "refresh");
        assert_eq!(claims.sub, "1");
    }

    #[test]
    fn refresh_token_long_expiry() {
        let token = create_refresh_token("1", "user", SECRET, 30).unwrap();
        let claims = validate_token(&token, SECRET).unwrap();
        let now = chrono::Utc::now().timestamp() as usize;
        let days_30_secs = 30 * 24 * 3600;
        assert!(claims.exp > now + days_30_secs - 10, "Refresh token should expire ~30 days out");
    }

    #[test]
    fn different_seeds_different_tokens() {
        let token1 = create_token("1", "user", SECRET).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(1100));
        let token2 = create_token("1", "user", SECRET).unwrap();
        assert_ne!(token1, token2, "Tokens should differ (different iat)");
    }

    #[test]
    fn claims_default_kind() {
        let claims = Claims {
            sub: "1".into(),
            exp: 9999999999,
            iat: 1000000000,
            role: "user".into(),
            kind: "access".into(),
        };
        assert_eq!(claims.kind, "access");
    }

    #[test]
    fn custom_expiry_hours() {
        let token =
            create_token_with_kind("1", "user", SECRET, "access", 1).unwrap();
        let claims = validate_token(&token, SECRET).unwrap();
        let now = chrono::Utc::now().timestamp() as usize;
        let hour_secs = 3600;
        assert!(claims.exp <= now + hour_secs + 5);
        assert!(claims.exp > now + hour_secs - 5);
    }
}

#[cfg(test)]
mod device_tests {
    use ysh::security::device::compute_fingerprint;

    #[test]
    fn fingerprint_deterministic() {
        let f1 = compute_fingerprint("Mozilla/5.0", "en-US", "gzip");
        let f2 = compute_fingerprint("Mozilla/5.0", "en-US", "gzip");
        assert_eq!(f1, f2);
    }

    #[test]
    fn fingerprint_hex_length() {
        let fp = compute_fingerprint("ua", "lang", "enc");
        assert_eq!(fp.len(), 64, "Blake3 hex should be 64 chars");
    }

    #[test]
    fn different_inputs_different_fingerprints() {
        let f1 = compute_fingerprint("Chrome", "en", "gzip");
        let f2 = compute_fingerprint("Firefox", "es", "br");
        assert_ne!(f1, f2);
    }

    #[test]
    fn fingerprint_empty_inputs() {
        let fp = compute_fingerprint("", "", "");
        assert_eq!(fp.len(), 64);
    }
}
