use std::ops::Deref;
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Zeroize, ZeroizeOnDrop)]
pub struct SecureBuffer {
    data: Vec<u8>,
}

impl SecureBuffer {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl Deref for SecureBuffer {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        &self.data
    }
}

impl From<Vec<u8>> for SecureBuffer {
    fn from(data: Vec<u8>) -> Self {
        Self::new(data)
    }
}

impl From<&[u8]> for SecureBuffer {
    fn from(data: &[u8]) -> Self {
        Self::new(data.to_vec())
    }
}

#[derive(Zeroize, ZeroizeOnDrop)]
pub struct SecureString {
    data: String,
}

impl SecureString {
    pub fn new(data: String) -> Self {
        Self { data }
    }

    pub fn as_str(&self) -> &str {
        &self.data
    }
}

impl Deref for SecureString {
    type Target = str;

    fn deref(&self) -> &str {
        &self.data
    }
}

impl From<String> for SecureString {
    fn from(data: String) -> Self {
        Self::new(data)
    }
}

#[derive(Zeroize, ZeroizeOnDrop)]
pub struct EncryptedKey {
    inner: Vec<u8>,
    algorithm: String,
}

impl EncryptedKey {
    pub fn new(inner: Vec<u8>, algorithm: String) -> Self {
        Self { inner, algorithm }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.inner
    }

    pub fn algorithm(&self) -> &str {
        &self.algorithm
    }
}
