//! OpenPaste Sync Module
//!
//! This module provides synchronization functionality for clipboard data across devices.

pub mod sync;

pub use sync::SyncManager;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum SyncError {
    #[error("Sync failed: {0}")]
    SyncFailed(String),
    #[error("Provider not configured")]
    ProviderNotConfigured,
    #[error("Conflict detected: {0}")]
    ConflictDetected(String),
}
