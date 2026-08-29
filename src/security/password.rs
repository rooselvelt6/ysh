use anyhow::Result;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::{Algorithm, Argon2, Params};

pub fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let params = Params::new(19456, 2, 1, None)
        .map_err(|e| anyhow::anyhow!("Argon2 params error: {}", e))?;
    let argon2 = Argon2::new(Algorithm::Argon2id, argon2::Version::V0x13, params);
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("Argon2 hash error: {}", e))?;
    Ok(hash.to_string())
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
    let parsed_hash =
        PasswordHash::new(hash).map_err(|e| anyhow::anyhow!("Password hash parse error: {}", e))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

pub fn hash_blake3(data: &[u8]) -> blake3::Hash {
    blake3::hash(data)
}

pub fn verify_blake3(data: &[u8], expected: &str) -> Result<bool> {
    let hash = blake3::hash(data);
    Ok(hash.to_hex().as_str() == expected)
}
