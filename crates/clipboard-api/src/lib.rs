//! OpenPaste API Module
//!
//! This module provides the REST API for external integration.

pub mod api;
pub mod handlers;

pub use api::ApiServer;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Request failed: {0}")]
    RequestFailed(String),
    #[error("Authentication failed")]
    AuthenticationFailed,
    #[error("Unauthorized")]
    Unauthorized,
}
