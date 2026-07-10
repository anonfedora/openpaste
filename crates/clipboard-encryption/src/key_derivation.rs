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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_key_is_deterministic() {
        let kd = KeyDerivation::new();
        let salt = KeyDerivation::generate_salt();
        let key1 = kd.derive_key("password123", &salt).unwrap();
        let key2 = kd.derive_key("password123", &salt).unwrap();
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_derive_key_differs_for_different_passwords() {
        let kd = KeyDerivation::new();
        let salt = KeyDerivation::generate_salt();
        let key1 = kd.derive_key("password123", &salt).unwrap();
        let key2 = kd.derive_key("different!", &salt).unwrap();
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_derive_key_differs_for_different_salts() {
        let kd = KeyDerivation::new();
        let salt1 = KeyDerivation::generate_salt();
        let salt2 = KeyDerivation::generate_salt();
        let key1 = kd.derive_key("password", &salt1).unwrap();
        let key2 = kd.derive_key("password", &salt2).unwrap();
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_hash_and_verify_password() {
        let kd = KeyDerivation::new();
        let hash = kd.hash_password("my_secure_password").unwrap();
        assert!(!hash.is_empty());
        // Correct password verifies
        assert!(kd.verify("my_secure_password", &hash).unwrap());
    }

    #[test]
    fn test_wrong_password_fails_verification() {
        let kd = KeyDerivation::new();
        let hash = kd.hash_password("correct_password").unwrap();
        let result = kd.verify("wrong_password", &hash);
        // Should return Err(InvalidPassword)
        assert!(result.is_err());
    }

    #[test]
    fn test_derived_key_is_32_bytes() {
        let kd = KeyDerivation::new();
        let salt = KeyDerivation::generate_salt();
        let key = kd.derive_key("anything", &salt).unwrap();
        assert_eq!(key.len(), 32);
    }
}
