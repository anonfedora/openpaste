//! Storage implementation

use crate::StorageError;
use clipboard_core::ContentType;
use std::path::Path;
use uuid::Uuid;

/// Storage manager for clipboard items
pub struct Storage {
    #[allow(dead_code)]
    base_path: std::path::PathBuf,
}

impl Storage {
    /// Create a new storage manager
    pub fn new(base_path: &Path) -> Self {
        Self {
            base_path: base_path.to_path_buf(),
        }
    }

    /// Store clipboard content
    pub async fn store(
        &self,
        content_type: ContentType,
        data: &[u8],
    ) -> Result<String, StorageError> {
        // Generate a unique filename using UUID to avoid collisions
        let uuid = Uuid::new_v4();
        let filename = format!("{}_{}.zst", content_type as i32, uuid);
        let path = self.base_path.join(&filename);

        // Compress data using zstd
        let compressed = zstd::encode_all(data, 3) // Level 3 compression
            .map_err(|e| StorageError::CompressionFailed(e.to_string()))?;

        // Write to file using async I/O
        tokio::fs::write(&path, compressed)
            .await
            .map_err(|e| StorageError::OperationFailed(e.to_string()))?;

        Ok(filename)
    }

    /// Retrieve clipboard content
    pub async fn retrieve(&self, path: &str) -> Result<Vec<u8>, StorageError> {
        let full_path = self.base_path.join(path);

        // Validate path is within base_path to prevent traversal
        let canonical_full = full_path
            .canonicalize()
            .map_err(|e| StorageError::FileNotFound(e.to_string()))?;
        let canonical_base = self
            .base_path
            .canonicalize()
            .map_err(|e| StorageError::OperationFailed(e.to_string()))?;

        if !canonical_full.starts_with(&canonical_base) {
            return Err(StorageError::OperationFailed(
                "Path traversal detected".to_string(),
            ));
        }

        // Read compressed data using async I/O
        let compressed = tokio::fs::read(&full_path)
            .await
            .map_err(|e| StorageError::FileNotFound(e.to_string()))?;

        // Decompress
        let decompressed = zstd::decode_all(&*compressed)
            .map_err(|e| StorageError::DecompressionFailed(e.to_string()))?;

        Ok(decompressed)
    }

    /// Delete stored content
    pub async fn delete(&self, path: &str) -> Result<(), StorageError> {
        let full_path = self.base_path.join(path);

        // Validate path is within base_path to prevent traversal
        let canonical_full = full_path
            .canonicalize()
            .map_err(|e| StorageError::OperationFailed(e.to_string()))?;
        let canonical_base = self
            .base_path
            .canonicalize()
            .map_err(|e| StorageError::OperationFailed(e.to_string()))?;

        if !canonical_full.starts_with(&canonical_base) {
            return Err(StorageError::OperationFailed(
                "Path traversal detected".to_string(),
            ));
        }

        // Delete file using async I/O
        tokio::fs::remove_file(&full_path)
            .await
            .map_err(|e| StorageError::OperationFailed(e.to_string()))?;

        Ok(())
    }
}
