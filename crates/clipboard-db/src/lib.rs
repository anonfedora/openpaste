//! OpenPaste Database Module
//!
//! This module provides database functionality using SQLite.

pub mod database;
pub mod models;
pub mod schema;

pub use database::Database;
pub use models::*;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Database connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Query failed: {0}")]
    QueryFailed(String),
    #[error("Migration failed: {0}")]
    MigrationFailed(String),
    #[error("Item not found: {0}")]
    NotFound(String),
}
