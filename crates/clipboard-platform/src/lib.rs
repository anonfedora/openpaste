//! OpenPaste Platform Module
//!
//! This module provides platform-specific clipboard integration for Windows, Linux, and macOS.

pub mod provider;

pub use provider::{get_provider, ClipboardProvider, PlatformProvider};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum PlatformError {
    #[error("Platform not supported")]
    UnsupportedPlatform,
    #[error("Clipboard access failed: {0}")]
    AccessFailed(String),
    #[error("Clipboard watch failed: {0}")]
    WatchFailed(String),
}
