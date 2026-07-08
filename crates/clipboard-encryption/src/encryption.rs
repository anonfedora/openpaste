//! Encryption implementation using AES-256-GCM

use crate::EncryptionError;
use aes_gcm::aead::{AeadCore, OsRng};
use aes_gcm::{Aes256Gcm, Nonce};

/// Encrypted data with nonce
#[derive(Debug, Clone)]
pub struct EncryptedData {
    pub nonce: Vec<u8>,
    pub ciphertext: Vec<u8>,
}

/// Encryption manager
pub struct Encryption {
    key: [u8; 32],
}

impl Encryption {
    /// Create a new encryption manager with a 32-byte key
    pub fn new(key: [u8; 32]) -> Self {
        Self { key }
    }

    /// Encrypt data and return nonce + ciphertext
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<EncryptedData, EncryptionError> {
        use aes_gcm::aead::{Aead, KeyInit};
        let key = <aes_gcm::Key<Aes256Gcm> as From<[u8; 32]>>::from(self.key);
        let cipher = Aes256Gcm::new(&key);
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

        let ciphertext = cipher
            .encrypt(&nonce, plaintext)
            .map_err(|e: aes_gcm::aead::Error| EncryptionError::EncryptionFailed(e.to_string()))?;

        Ok(EncryptedData {
            nonce: nonce.to_vec(),
            ciphertext,
        })
    }

    /// Decrypt data using provided nonce
    pub fn decrypt(&self, encrypted: &EncryptedData) -> Result<Vec<u8>, EncryptionError> {
        use aes_gcm::aead::{Aead, KeyInit};
        let key = <aes_gcm::Key<Aes256Gcm> as From<[u8; 32]>>::from(self.key);
        let cipher = Aes256Gcm::new(&key);
        let nonce = Nonce::from_slice(&encrypted.nonce);

        cipher
            .decrypt(nonce, encrypted.ciphertext.as_ref())
            .map_err(|e: aes_gcm::aead::Error| EncryptionError::DecryptionFailed(e.to_string()))
    }
}
