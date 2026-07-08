//! OpenPaste AI Module
//!
//! This module provides optional AI features for content categorization, smart search, and analysis.

pub mod ai;

pub use ai::AiEngine;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum AiError {
    #[error("AI request failed: {0}")]
    RequestFailed(String),
    #[error("Provider not configured")]
    ProviderNotConfigured,
    #[error("API key invalid")]
    InvalidApiKey,
}
