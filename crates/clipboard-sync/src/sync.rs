//! Sync manager implementation

use crate::SyncError;

/// Sync manager for clipboard synchronization
pub struct SyncManager {
    // TODO: Add sync provider configuration
}

impl Default for SyncManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncManager {
    /// Create a new sync manager
    pub fn new() -> Self {
        Self {}
    }

    /// Start synchronization
    pub async fn start(&self) -> Result<(), SyncError> {
        // TODO: Implement sync start
        Err(SyncError::ProviderNotConfigured)
    }

    /// Stop synchronization
    pub async fn stop(&self) -> Result<(), SyncError> {
        // TODO: Implement sync stop
        Ok(())
    }

    /// Get sync status
    pub async fn status(&self) -> Result<SyncStatus, SyncError> {
        Ok(SyncStatus::Stopped)
    }
}

/// Sync status
#[derive(Debug, Clone, PartialEq)]
pub enum SyncStatus {
    Stopped,
    Running,
    Error(String),
}
