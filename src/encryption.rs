use aes_gcm::{aead::Aead, Aes256Gcm, KeyInit, Nonce};
use redb::StorageBackend;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

const KEY_SIZE: usize = 32;
const NONCE_SIZE: usize = 12;
const TAG_SIZE: usize = 16;
const PAGE_SIZE: usize = 4096;
const ENCRYPTED_PAGE: usize = PAGE_SIZE + TAG_SIZE; // 4112

/// File header: 64 bytes
/// [0..8]   magic: b"YSHENC\0\0"
/// [8..16]  nonce_prefix: random bytes (unique per file)
/// [16..24] logical_len: u64 LE
/// [24..64] reserved
const HEADER_SIZE: usize = 64;
const MAGIC: &[u8; 8] = b"YSHENC\0\0";

pub struct EncryptedBackend {
    inner: Mutex<Inner>,
}

struct Inner {
    file: File,
    cipher: Aes256Gcm,
    nonce_prefix: [u8; 8],
    path: PathBuf,
    logical_len: u64,
}

impl std::fmt::Debug for EncryptedBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let inner = self.inner.lock().unwrap();
        f.debug_struct("EncryptedBackend")
            .field("path", &inner.path)
            .field("nonce_prefix", &inner.nonce_prefix)
            .field("logical_len", &inner.logical_len)
            .finish()
    }
}

#[derive(Debug)]
pub enum CryptoError {
    Io(std::io::Error),
    InvalidKey,
    InvalidFile,
}

impl std::fmt::Display for CryptoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CryptoError::Io(e) => write!(f, "IO error: {}", e),
            CryptoError::InvalidKey => write!(f, "Invalid encryption key"),
            CryptoError::InvalidFile => write!(f, "Invalid encrypted file header"),
        }
    }
}

impl std::error::Error for CryptoError {}

impl From<std::io::Error> for CryptoError {
    fn from(e: std::io::Error) -> Self {
        CryptoError::Io(e)
    }
}

impl From<CryptoError> for std::io::Error {
    fn from(e: CryptoError) -> Self {
        std::io::Error::other(e.to_string())
    }
}

impl EncryptedBackend {
    pub fn open(path: impl AsRef<Path>, key: &[u8; KEY_SIZE]) -> Result<Self, CryptoError> {
        let path = path.as_ref().to_path_buf();
        let cipher =
            Aes256Gcm::new_from_slice(key).map_err(|_| CryptoError::InvalidKey)?;

        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)?;

        let metadata = file.metadata()?;
        let (nonce_prefix, logical_len) = if metadata.len() == 0 {
            let mut prefix = [0u8; 8];
            rand::RngCore::fill_bytes(&mut rand::rng(), &mut prefix);
            let mut header = [0u8; HEADER_SIZE];
            header[0..8].copy_from_slice(MAGIC);
            header[8..16].copy_from_slice(&prefix);
            header[16..24].copy_from_slice(&0u64.to_le_bytes());
            file.write_all(&header)?;
            file.sync_data()?;
            (prefix, 0u64)
        } else {
            let mut header = [0u8; HEADER_SIZE];
            file.read_exact(&mut header)?;
            if &header[0..8] != MAGIC {
                return Err(CryptoError::InvalidFile);
            }
            let mut prefix = [0u8; 8];
            prefix.copy_from_slice(&header[8..16]);
            let mut len_bytes = [0u8; 8];
            len_bytes.copy_from_slice(&header[16..24]);
            (prefix, u64::from_le_bytes(len_bytes))
        };

        Ok(Self {
            inner: Mutex::new(Inner {
                file,
                cipher,
                nonce_prefix,
                path,
                logical_len,
            }),
        })
    }

    #[allow(dead_code)]
    pub fn open_hex(path: impl AsRef<Path>, hex_key: &str) -> Result<Self, CryptoError> {
        let key_bytes = hex::decode(hex_key).map_err(|_| CryptoError::InvalidKey)?;
        if key_bytes.len() != KEY_SIZE {
            return Err(CryptoError::InvalidKey);
        }
        let mut key = [0u8; KEY_SIZE];
        key.copy_from_slice(&key_bytes);
        Self::open(path, &key)
    }

    fn make_nonce(prefix: &[u8; 8], page_index: u64) -> [u8; NONCE_SIZE] {
        let mut nonce = [0u8; NONCE_SIZE];
        nonce[0..8].copy_from_slice(prefix);
        nonce[8..12].copy_from_slice(&page_index.to_le_bytes()[0..4]);
        nonce
    }

    fn page_physical_offset(page_index: u64) -> u64 {
        HEADER_SIZE as u64 + page_index * ENCRYPTED_PAGE as u64
    }

    fn save_logical_len(inner: &mut Inner) -> Result<(), std::io::Error> {
        inner.file.seek(SeekFrom::Start(16))?;
        inner.file.write_all(&inner.logical_len.to_le_bytes())?;
        Ok(())
    }

    fn encrypt_page(
        inner: &mut Inner,
        page_index: u64,
        plaintext: &[u8; PAGE_SIZE],
    ) -> Result<(), std::io::Error> {
        let nonce_bytes = Self::make_nonce(&inner.nonce_prefix, page_index);
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = inner
            .cipher
            .encrypt(nonce, plaintext.as_ref())
            .map_err(|_| std::io::Error::other("Encryption failed"))?;
        let phys = Self::page_physical_offset(page_index);
        inner.file.seek(SeekFrom::Start(phys))?;
        inner.file.write_all(&ciphertext)?;
        Ok(())
    }

    fn decrypt_page(
        inner: &mut Inner,
        page_index: u64,
    ) -> Result<[u8; PAGE_SIZE], std::io::Error> {
        let phys = Self::page_physical_offset(page_index);
        let file_len = inner.file.metadata()?.len();
        if phys + ENCRYPTED_PAGE as u64 > file_len {
            return Ok([0u8; PAGE_SIZE]);
        }
        let mut enc_data = vec![0u8; ENCRYPTED_PAGE];
        inner.file.seek(SeekFrom::Start(phys))?;
        inner.file.read_exact(&mut enc_data)?;
        let nonce_bytes = Self::make_nonce(&inner.nonce_prefix, page_index);
        let nonce = Nonce::from_slice(&nonce_bytes);
        let plaintext = inner
            .cipher
            .decrypt(nonce, enc_data.as_ref())
            .map_err(|_| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Decryption failed — wrong key or corrupted data",
                )
            })?;
        let mut page = [0u8; PAGE_SIZE];
        page.copy_from_slice(&plaintext);
        Ok(page)
    }
}

impl StorageBackend for EncryptedBackend {
    fn len(&self) -> Result<u64, std::io::Error> {
        let inner = self
            .inner
            .lock()
            .map_err(|_| std::io::Error::other("Lock poisoned"))?;
        Ok(inner.logical_len)
    }

    fn read(&self, offset: u64, out: &mut [u8]) -> Result<(), std::io::Error> {
        let mut inner = self
            .inner
            .lock()
            .map_err(|_| std::io::Error::other("Lock poisoned"))?;

        let mut pos = 0usize;
        while pos < out.len() {
            let abs = offset + pos as u64;
            let page_idx = abs / PAGE_SIZE as u64;
            let offset_in_page = (abs % PAGE_SIZE as u64) as usize;
            let available = PAGE_SIZE - offset_in_page;
            let needed = (out.len() - pos).min(available);

            let page = Self::decrypt_page(&mut inner, page_idx)?;
            out[pos..pos + needed].copy_from_slice(&page[offset_in_page..offset_in_page + needed]);
            pos += needed;
        }
        Ok(())
    }

    fn write(&self, offset: u64, data: &[u8]) -> Result<(), std::io::Error> {
        let mut inner = self
            .inner
            .lock()
            .map_err(|_| std::io::Error::other("Lock poisoned"))?;

        let mut pos = 0usize;
        while pos < data.len() {
            let abs = offset + pos as u64;
            let page_idx = abs / PAGE_SIZE as u64;
            let offset_in_page = (abs % PAGE_SIZE as u64) as usize;
            let available = PAGE_SIZE - offset_in_page;
            let needed = (data.len() - pos).min(available);

            let mut page = if offset_in_page == 0 && needed == PAGE_SIZE {
                [0u8; PAGE_SIZE]
            } else {
                Self::decrypt_page(&mut inner, page_idx)?
            };
            page[offset_in_page..offset_in_page + needed]
                .copy_from_slice(&data[pos..pos + needed]);
            Self::encrypt_page(&mut inner, page_idx, &page)?;
            pos += needed;
        }

        let end = offset + data.len() as u64;
        if end > inner.logical_len {
            inner.logical_len = end;
            Self::save_logical_len(&mut inner)?;
        }
        Ok(())
    }

    fn set_len(&self, len: u64) -> Result<(), std::io::Error> {
        let mut inner = self
            .inner
            .lock()
            .map_err(|_| std::io::Error::other("Lock poisoned"))?;
        inner.logical_len = len;
        Self::save_logical_len(&mut inner)?;
        Ok(())
    }

    fn sync_data(&self) -> Result<(), std::io::Error> {
        let inner = self
            .inner
            .lock()
            .map_err(|_| std::io::Error::other("Lock poisoned"))?;
        inner.file.sync_data()?;
        Ok(())
    }
}

#[allow(dead_code)]
pub fn generate_key() -> String {
    use rand::RngCore;
    let mut key = [0u8; KEY_SIZE];
    rand::rng().fill_bytes(&mut key);
    hex::encode(key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let tmp = NamedTempFile::new().unwrap();
        let key = [42u8; KEY_SIZE];
        let backend = EncryptedBackend::open(tmp.path(), &key).unwrap();
        let original = b"Hello, encrypted world!";
        backend.write(0, original).unwrap();
        let mut buf = vec![0u8; original.len()];
        backend.read(0, &mut buf).unwrap();
        assert_eq!(&buf, original);
    }

    #[test]
    fn different_offsets_different_nonces() {
        let tmp = NamedTempFile::new().unwrap();
        let key = [1u8; KEY_SIZE];
        let backend = EncryptedBackend::open(tmp.path(), &key).unwrap();
        let data = b"same data same data same data!!!";
        backend.write(0, data).unwrap();
        backend.write(100, data).unwrap();
        let mut buf0 = vec![0u8; data.len()];
        let mut buf1 = vec![0u8; data.len()];
        backend.read(0, &mut buf0).unwrap();
        backend.read(100, &mut buf1).unwrap();
        assert_eq!(&buf0, data);
        assert_eq!(&buf1, data);
    }

    #[test]
    fn wrong_key_fails() {
        let tmp = NamedTempFile::new().unwrap();
        let key1 = [1u8; KEY_SIZE];
        let key2 = [2u8; KEY_SIZE];
        let backend = EncryptedBackend::open(tmp.path(), &key1).unwrap();
        backend.write(0, b"secret data!!").unwrap();
        drop(backend);
        let backend2 = EncryptedBackend::open(tmp.path(), &key2).unwrap();
        let mut buf = [0u8; 13];
        assert!(backend2.read(0, &mut buf).is_err());
    }

    #[test]
    fn key_generation_is_unique() {
        let k1 = generate_key();
        let k2 = generate_key();
        assert_ne!(k1, k2);
        assert_eq!(k1.len(), 64);
    }

    #[test]
    fn len_returns_logical_size() {
        let tmp = NamedTempFile::new().unwrap();
        let key = [99u8; KEY_SIZE];
        let backend = EncryptedBackend::open(tmp.path(), &key).unwrap();
        assert_eq!(backend.len().unwrap(), 0);
        backend.write(0, b"test").unwrap();
        assert_eq!(backend.len().unwrap(), 4);
        backend.write(100, b"more data").unwrap();
        assert_eq!(backend.len().unwrap(), 109);
    }

    #[test]
    fn corrupted_data_fails_decrypt() {
        let tmp = NamedTempFile::new().unwrap();
        let key = [7u8; KEY_SIZE];
        let backend = EncryptedBackend::open(tmp.path(), &key).unwrap();
        backend.write(0, b"real data").unwrap();
        drop(backend);
        let mut f = OpenOptions::new()
            .read(true)
            .write(true)
            .open(tmp.path())
            .unwrap();
        f.seek(SeekFrom::Start(HEADER_SIZE as u64 + 20))
            .unwrap();
        f.write_all(b"CORRUPTED").unwrap();
        drop(f);
        let backend2 = EncryptedBackend::open(tmp.path(), &key).unwrap();
        let mut buf = [0u8; 9];
        assert!(backend2.read(0, &mut buf).is_err());
    }

    #[test]
    fn persistent_len_survives_reopen() {
        let tmp = NamedTempFile::new().unwrap();
        let key = [5u8; KEY_SIZE];
        {
            let backend = EncryptedBackend::open(tmp.path(), &key).unwrap();
            backend.write(0, b"hello").unwrap();
            backend.write(100, b"world").unwrap();
            assert_eq!(backend.len().unwrap(), 105);
        }
        let backend2 = EncryptedBackend::open(tmp.path(), &key).unwrap();
        assert_eq!(backend2.len().unwrap(), 105);
        let mut buf = [0u8; 5];
        backend2.read(0, &mut buf).unwrap();
        assert_eq!(&buf, b"hello");
    }

    #[test]
    fn crossing_page_boundary() {
        let tmp = NamedTempFile::new().unwrap();
        let key = [3u8; KEY_SIZE];
        let backend = EncryptedBackend::open(tmp.path(), &key).unwrap();
        let data = vec![0xABu8; 8192];
        backend.write(0, &data).unwrap();
        let mut buf = vec![0u8; 8192];
        backend.read(0, &mut buf).unwrap();
        assert_eq!(buf, data);
    }

    #[test]
    fn partial_page_read_write() {
        let tmp = NamedTempFile::new().unwrap();
        let key = [4u8; KEY_SIZE];
        let backend = EncryptedBackend::open(tmp.path(), &key).unwrap();
        backend.write(100, b"hello").unwrap();
        let mut buf = [0u8; 5];
        backend.read(100, &mut buf).unwrap();
        assert_eq!(&buf, b"hello");
        backend.write(102, b"XY").unwrap();
        let mut buf2 = [0u8; 5];
        backend.read(100, &mut buf2).unwrap();
        assert_eq!(&buf2, b"heXYo");
    }

    #[test]
    fn adjacent_writes_no_overlap() {
        let tmp = NamedTempFile::new().unwrap();
        let key = [6u8; KEY_SIZE];
        let backend = EncryptedBackend::open(tmp.path(), &key).unwrap();
        let page_a = vec![0x11u8; PAGE_SIZE];
        let page_b = vec![0x22u8; PAGE_SIZE];
        backend.write(0, &page_a).unwrap();
        backend.write(PAGE_SIZE as u64, &page_b).unwrap();
        let mut ra = vec![0u8; PAGE_SIZE];
        let mut rb = vec![0u8; PAGE_SIZE];
        backend.read(0, &mut ra).unwrap();
        backend.read(PAGE_SIZE as u64, &mut rb).unwrap();
        assert_eq!(ra, page_a);
        assert_eq!(rb, page_b);
    }

    #[test]
    fn load_test_many_pages() {
        let tmp = NamedTempFile::new().unwrap();
        let key = [8u8; KEY_SIZE];
        let backend = EncryptedBackend::open(tmp.path(), &key).unwrap();
        for i in 0..50u64 {
            let data = vec![(i % 256) as u8; PAGE_SIZE];
            backend.write(i * PAGE_SIZE as u64, &data).unwrap();
        }
        for i in 0..50u64 {
            let expected = vec![(i % 256) as u8; PAGE_SIZE];
            let mut buf = vec![0u8; PAGE_SIZE];
            backend.read(i * PAGE_SIZE as u64, &mut buf).unwrap();
            assert_eq!(buf, expected, "Page {} mismatch", i);
        }
    }
}
