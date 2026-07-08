//! Clipboard item representation

use crate::ContentType;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A clipboard item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardItem {
    pub id: Uuid,
    pub content_type: ContentType,
    pub content: Vec<u8>,
    pub hash: String,
    pub created_at: DateTime<Utc>,
    pub accessed_at: Option<DateTime<Utc>>,
    pub pinned: bool,
    pub favorite: bool,
}

impl Default for ClipboardItem {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            content_type: ContentType::Text,
            content: Vec::new(),
            hash: String::new(),
            created_at: Utc::now(),
            accessed_at: None,
            pinned: false,
            favorite: false,
        }
    }
}

impl ClipboardItem {
    /// Create a new clipboard item
    pub fn new(content_type: ContentType, content: Vec<u8>) -> Self {
        let hash = Self::compute_hash(&content);
        Self {
            id: Uuid::new_v4(),
            content_type,
            content,
            hash,
            created_at: Utc::now(),
            ..Default::default()
        }
    }

    /// Compute hash of content
    fn compute_hash(content: &[u8]) -> String {
        // TODO: Implement proper hashing (e.g., SHA-256)
        format!("{:x}", md5::compute(content))
    }
}
