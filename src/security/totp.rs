use anyhow::Result;
use hmac::{Hmac, Mac};
use sha1::Sha1;
use std::time::{SystemTime, UNIX_EPOCH};

type HmacSha1 = Hmac<Sha1>;

const BASE32_ALPHABET: &[u8; 32] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

pub fn base32_encode(data: &[u8]) -> String {
    let mut result = String::new();
    let mut bits = 0u32;
    let mut value = 0u32;

    for &byte in data {
        value = (value << 8) | byte as u32;
        bits += 8;
        while bits >= 5 {
            result.push(BASE32_ALPHABET[((value >> (bits - 5)) & 31) as usize] as char);
            bits -= 5;
        }
    }

    if bits > 0 {
        result.push(BASE32_ALPHABET[((value << (5 - bits)) & 31) as usize] as char);
    }

    result
}

pub fn generate_secret() -> ([u8; 20], String) {
    use rand_core::OsRng;
    use rand_core::RngCore;
    let mut secret = [0u8; 20];
    OsRng.fill_bytes(&mut secret);
    let encoded = base32_encode(&secret);
    (secret, encoded)
}

pub fn generate_uri(secret_base32: &str, email: &str, issuer: &str) -> String {
    format!(
        "otpauth://totp/{}:{}?secret={}&issuer={}&algorithm=SHA1&digits=6&period=30",
        issuer, email, secret_base32, issuer
    )
}

fn compute_code(secret: &[u8], time_step: u64) -> Result<String> {
    let mut mac =
        HmacSha1::new_from_slice(secret).map_err(|e| anyhow::anyhow!("HMAC init: {}", e))?;
    mac.update(&time_step.to_be_bytes());
    let result = mac.finalize().into_bytes();

    let offset = (result[19] & 0x0f) as usize;
    let code = u32::from_be_bytes([
        result[offset] & 0x7f,
        result[offset + 1],
        result[offset + 2],
        result[offset + 3],
    ]) % 1_000_000;
    Ok(format!("{:06}", code))
}

pub fn verify_code(secret: &[u8], code: &str) -> bool {
    let Ok(now) = SystemTime::now().duration_since(UNIX_EPOCH) else {
        return false;
    };
    let time_step = now.as_secs() / 30;

    for offset in -1i64..=1 {
        let step = (time_step as i64 + offset) as u64;
        if let Ok(c) = compute_code(secret, step)
            && c == code
        {
            return true;
        }
    }
    false
}

pub fn generate_recovery_codes(count: usize) -> Vec<String> {
    use rand_core::OsRng;
    use rand_core::RngCore;
    let mut codes = Vec::with_capacity(count);
    for _ in 0..count {
        let mut buf = [0u8; 10];
        OsRng.fill_bytes(&mut buf);
        let a = base32_encode(&buf[..4]);
        let b = base32_encode(&buf[4..7]);
        let c = base32_encode(&buf[7..]);
        codes.push(format!("{}-{}-{}", a, b, c));
    }
    codes
}

pub fn hash_recovery_code(code: &str) -> String {
    blake3::hash(code.as_bytes()).to_hex().to_string()
}

pub fn verify_recovery_code(code: &str, expected_hash: &str) -> bool {
    blake3::hash(code.as_bytes()).to_hex().as_str() == expected_hash
}
