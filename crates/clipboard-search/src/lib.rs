//! OpenPaste Search Module
//!
//! This module provides full-text search functionality using SQLite FTS5.

pub mod search;

pub use search::SearchEngine;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum SearchError {
    #[error("Search query failed: {0}")]
    QueryFailed(String),
    #[error("Invalid search syntax: {0}")]
    InvalidSyntax(String),
}
