//! IPC (Inter-Process Communication) module

pub mod error;
pub mod ipc;

pub use error::IpcError;
pub use ipc::{ClipboardHistoryItem, IpcClient, IpcMessage, IpcServer};
