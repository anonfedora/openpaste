//! Storage implementation

use crate::StorageError;
use clipboard_core::ContentType;
use std::path::Path;

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
        // Generate a unique filename based on content type and timestamp
        let timestamp = chrono::Utc::now().timestamp();
        let filename = format!("{}_{}.zst", content_type as i32, timestamp);
        let path = self.base_path.join(&filename);

        // Compress data using zstd
        let compressed = zstd::encode_all(data, 3) // Level 3 compression
            .map_err(|e| StorageError::CompressionFailed(e.to_string()))?;

        // Write to file
        std::fs::write(&path, compressed)
            .map_err(|e| StorageError::OperationFailed(e.to_string()))?;

        Ok(filename)
    }

    /// Retrieve clipboard content
    pub async fn retrieve(&self, path: &str) -> Result<Vec<u8>, StorageError> {
        let full_path = self.base_path.join(path);

        // Read compressed data
        let compressed =
            std::fs::read(&full_path).map_err(|e| StorageError::FileNotFound(e.to_string()))?;

        // Decompress
        let decompressed = zstd::decode_all(&*compressed)
            .map_err(|e| StorageError::DecompressionFailed(e.to_string()))?;

        Ok(decompressed)
    }

    /// Delete stored content
    pub async fn delete(&self, path: &str) -> Result<(), StorageError> {
        let full_path = self.base_path.join(path);

        std::fs::remove_file(&full_path)
            .map_err(|e| StorageError::OperationFailed(e.to_string()))?;

        Ok(())
    }
}
