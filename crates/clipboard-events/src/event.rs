//! Event types

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {
    /// Clipboard events
    ClipboardAdded {
        id: Uuid,
    },
    ClipboardChanged {
        id: Uuid,
    },
    ClipboardAccessed {
        id: Uuid,
    },
    ClipboardPinned {
        id: Uuid,
    },
    ClipboardDeleted {
        id: Uuid,
    },

    /// Search events
    SearchPerformed {
        query: String,
    },
    SearchResultSelected {
        id: Uuid,
    },

    /// Storage events
    StorageThresholdReached {
        size: usize,
    },
    ItemRetentionTriggered {
        id: Uuid,
    },

    /// Encryption events
    EncryptionStateChanged {
        locked: bool,
    },
    VaultLocked,
    VaultUnlocked,

    /// Sync events
    SyncStarted,
    SyncCompleted,
    ConflictDetected {
        id: Uuid,
    },

    /// Plugin events
    PluginLoaded {
        name: String,
    },
    PluginUnloaded {
        name: String,
    },
    PluginError {
        name: String,
        error: String,
    },

    /// System events
    DaemonStarted,
    DaemonShutdown,
    ClientConnected {
        id: Uuid,
    },
    ClientDisconnected {
        id: Uuid,
    },
}
