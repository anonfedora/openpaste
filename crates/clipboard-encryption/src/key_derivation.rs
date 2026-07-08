//! Key derivation using Argon2id

use crate::EncryptionError;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Algorithm;
use argon2::Argon2;
use argon2::Params;
use argon2::Version;
use sha2::Digest;

/// Key derivation using Argon2id
pub struct KeyDerivation {
    argon2: Argon2<'static>,
}

impl Default for KeyDerivation {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyDerivation {
    /// Create a new key derivation instance with secure defaults
    pub fn new() -> Self {
        // Use Argon2id with secure parameters
        let params = Params::new(65536, 3, 4, None).expect("Invalid params");
        let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

        Self { argon2 }
    }

    /// Generate a random salt
    pub fn generate_salt() -> String {
        let salt = SaltString::generate(&mut argon2::password_hash::rand_core::OsRng);
        salt.as_str().to_string()
    }

    /// Derive a key from password using Argon2id
    pub fn derive_key(&self, password: &str, salt: &str) -> Result<[u8; 32], EncryptionError> {
        let salt = SaltString::from_b64(salt)
            .map_err(|e| EncryptionError::KeyDerivationFailed(e.to_string()))?;

        let password_hash = self
            .argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| EncryptionError::KeyDerivationFailed(e.to_string()))?;

        // Extract the hash bytes and use SHA-256 to derive a 32-byte key
        let hash_str = password_hash.to_string();
        let hash_bytes = hash_str.as_bytes();
        let mut key = [0u8; 32];

        // Use SHA-256 to derive a 32-byte key from the hash string
        let digest = sha2::Sha256::digest(hash_bytes);
        key.copy_from_slice(&digest);

        Ok(key)
    }

    /// Hash a password for storage
    pub fn hash_password(&self, password: &str) -> Result<String, EncryptionError> {
        let salt = SaltString::generate(&mut argon2::password_hash::rand_core::OsRng);
        let password_hash = self
            .argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| EncryptionError::KeyDerivationFailed(e.to_string()))?;

        Ok(password_hash.to_string())
    }

    /// Verify password against hash
    pub fn verify(&self, password: &str, hash: &str) -> Result<bool, EncryptionError> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| EncryptionError::KeyDerivationFailed(e.to_string()))?;

        self.argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .map(|_| true)
            .map_err(|_| EncryptionError::InvalidPassword)
    }
}
