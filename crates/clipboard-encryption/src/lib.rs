//! OpenPaste Encryption Module
//!
//! This module provides encryption functionality using AES-256-GCM and Argon2id.

pub mod encryption;
pub mod key_derivation;

pub use encryption::{EncryptedData, Encryption};
pub use key_derivation::KeyDerivation;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum EncryptionError {
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),
    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),
    #[error("Key derivation failed: {0}")]
    KeyDerivationFailed(String),
    #[error("Invalid password")]
    InvalidPassword,
}
