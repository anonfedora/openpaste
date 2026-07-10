//! IPC (Inter-Process Communication) module

pub mod error;
pub mod ipc;

pub use error::IpcError;
pub use ipc::{AppSettings, ClipboardHistoryItem, IpcClient, IpcMessage, IpcServer, PluginInfoItem, TagItem};
