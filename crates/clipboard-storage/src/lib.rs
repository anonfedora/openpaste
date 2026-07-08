//! OpenPaste Storage Module
//!
//! This module handles storage of clipboard items including compression and file system operations.

pub mod storage;

pub use storage::Storage;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Storage operation failed: {0}")]
    OperationFailed(String),
    #[error("Compression failed: {0}")]
    CompressionFailed(String),
    #[error("Decompression failed: {0}")]
    DecompressionFailed(String),
    #[error("File not found: {0}")]
    FileNotFound(String),
}
