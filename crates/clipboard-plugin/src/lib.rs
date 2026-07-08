//! OpenPaste Plugin Module
//!
//! This module provides plugin functionality using WebAssembly (WASM).

pub mod plugin;
pub mod sandbox;

pub use plugin::PluginManager;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum PluginError {
    #[error("Plugin load failed: {0}")]
    LoadFailed(String),
    #[error("Plugin execution failed: {0}")]
    ExecutionFailed(String),
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("Invalid manifest")]
    InvalidManifest,
}
