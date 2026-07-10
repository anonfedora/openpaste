//! Clipboard management — thin wrapper around arboard.

use crate::item::ClipboardItem;
use crate::ContentType;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClipboardError {
    #[error("Failed to access clipboard: {0}")]
    AccessFailed(String),
    #[error("Content type not supported")]
    UnsupportedContentType,
    #[error("Content too large: {0} bytes")]
    ContentTooLarge(usize),
}

/// Clipboard manager for reading and writing the system clipboard.
pub struct ClipboardManager {
    max_size: usize,
}

impl ClipboardManager {
    /// Create a new clipboard manager.
    /// `max_size` is the maximum content size in bytes that will be accepted.
    pub fn new(max_size: usize) -> Self {
        Self { max_size }
    }

    /// Read the current clipboard content.
    ///
    /// Tries text first; images are not handled here (use `clipboard-platform`
    /// for full image + content-type detection).
    pub async fn capture(&self) -> Result<ClipboardItem, ClipboardError> {
        let mut ctx = arboard::Clipboard::new()
            .map_err(|e| ClipboardError::AccessFailed(e.to_string()))?;

        let text = ctx
            .get_text()
            .map_err(|e| ClipboardError::AccessFailed(e.to_string()))?;

        if text.is_empty() {
            return Err(ClipboardError::AccessFailed("Clipboard is empty".to_string()));
        }

        let bytes = text.into_bytes();
        if bytes.len() > self.max_size {
            return Err(ClipboardError::ContentTooLarge(bytes.len()));
        }

        Ok(ClipboardItem::new(ContentType::Text, bytes))
    }

    /// Write a clipboard item to the system clipboard.
    ///
    /// Only text content types are supported; image writes require the
    /// platform-level API.
    pub async fn set(&self, item: &ClipboardItem) -> Result<(), ClipboardError> {
        match item.content_type {
            ContentType::Image => return Err(ClipboardError::UnsupportedContentType),
            _ => {}
        }

        let text = String::from_utf8_lossy(&item.content).to_string();
        let mut ctx = arboard::Clipboard::new()
            .map_err(|e| ClipboardError::AccessFailed(e.to_string()))?;

        ctx.set_text(&text)
            .map_err(|e| ClipboardError::AccessFailed(e.to_string()))
    }
}
