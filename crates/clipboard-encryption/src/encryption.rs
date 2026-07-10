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

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key() -> [u8; 32] {
        let mut key = [0u8; 32];
        for (i, b) in key.iter_mut().enumerate() {
            *b = i as u8;
        }
        key
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let enc = Encryption::new(test_key());
        let plaintext = b"hello, OpenPaste!";
        let encrypted = enc.encrypt(plaintext).unwrap();
        assert!(!encrypted.ciphertext.is_empty());
        assert_eq!(encrypted.nonce.len(), 12); // AES-GCM nonce is 12 bytes
        let decrypted = enc.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_produces_different_ciphertexts() {
        // Each call should produce a different nonce → different ciphertext
        let enc = Encryption::new(test_key());
        let plaintext = b"same plaintext";
        let e1 = enc.encrypt(plaintext).unwrap();
        let e2 = enc.encrypt(plaintext).unwrap();
        // Nonces should differ (probabilistically guaranteed)
        assert_ne!(e1.nonce, e2.nonce);
    }

    #[test]
    fn test_decrypt_with_wrong_key_fails() {
        let enc = Encryption::new(test_key());
        let encrypted = enc.encrypt(b"secret").unwrap();

        let mut wrong_key = test_key();
        wrong_key[0] ^= 0xff;
        let enc2 = Encryption::new(wrong_key);
        assert!(enc2.decrypt(&encrypted).is_err());
    }

    #[test]
    fn test_decrypt_with_tampered_ciphertext_fails() {
        let enc = Encryption::new(test_key());
        let mut encrypted = enc.encrypt(b"integrity check").unwrap();
        // Flip a byte in the ciphertext — AEAD should detect this
        if let Some(b) = encrypted.ciphertext.last_mut() {
            *b ^= 0x01;
        }
        assert!(enc.decrypt(&encrypted).is_err());
    }

    #[test]
    fn test_empty_plaintext() {
        let enc = Encryption::new(test_key());
        let encrypted = enc.encrypt(b"").unwrap();
        let decrypted = enc.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, b"");
    }

    #[test]
    fn test_large_plaintext() {
        let enc = Encryption::new(test_key());
        let plaintext = vec![0xABu8; 1024 * 1024]; // 1 MB
        let encrypted = enc.encrypt(&plaintext).unwrap();
        let decrypted = enc.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }
}
