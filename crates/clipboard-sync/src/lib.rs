//! OpenPaste Sync Module
//!
//! HTTP-based clipboard sync. Each device connects to a shared relay server.
//! Items are identified by content hash — the server is a simple append-only
//! log; clients push new items and pull anything newer than their last sync.

pub mod sync;

pub use sync::{SyncClient, SyncConfig, SyncStatus};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum SyncError {
    #[error("Sync failed: {0}")]
    SyncFailed(String),
    #[error("Provider not configured")]
    ProviderNotConfigured,
    #[error("HTTP error: {0}")]
    Http(String),
    #[error("Conflict detected: {0}")]
    ConflictDetected(String),
    #[error("Database error: {0}")]
    Database(String),
}
