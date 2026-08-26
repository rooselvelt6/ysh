pub fn compute_fingerprint(user_agent: &str, accept_language: &str, accept_encoding: &str) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(user_agent.as_bytes());
    hasher.update(b"|");
    hasher.update(accept_language.as_bytes());
    hasher.update(b"|");
    hasher.update(accept_encoding.as_bytes());
    hasher.finalize().to_hex().to_string()
}
