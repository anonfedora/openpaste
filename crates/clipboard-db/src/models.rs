//! Database models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Tag model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Tag {
    pub id: i64,
    pub name: String,
    pub color: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ClipboardItem {
    pub id: i64,
    pub content_type: String,
    pub content: Vec<u8>,
    pub hash: String,
    pub created_at: DateTime<Utc>,
    pub accessed_at: Option<DateTime<Utc>>,
    pub pinned: bool,
    pub favorite: bool,
    pub nonce: Option<Vec<u8>>,
    pub encrypted: bool,
}
