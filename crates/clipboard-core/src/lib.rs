//! OpenPaste Core Module
//!
//! This module provides the core clipboard management functionality.

pub mod clipboard;
pub mod item;

pub use clipboard::ClipboardManager;
pub use item::ClipboardItem;

/// Clipboard content types
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ContentType {
    Text,
    Html,
    Image,
    File,
    Binary,
    Rtf,
    Code,
}

impl std::fmt::Display for ContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContentType::Text => write!(f, "text"),
            ContentType::Html => write!(f, "html"),
            ContentType::Image => write!(f, "image"),
            ContentType::File => write!(f, "file"),
            ContentType::Binary => write!(f, "binary"),
            ContentType::Rtf => write!(f, "rtf"),
            ContentType::Code => write!(f, "code"),
        }
    }
}

impl ContentType {
    /// Detect content type from data
    pub fn detect(data: &[u8]) -> Self {
        if data.is_empty() {
            return ContentType::Text;
        }

        // Check for image signatures
        if data.len() >= 8 {
            let header = &data[..8];
            // PNG
            if header.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) {
                return ContentType::Image;
            }
            // JPEG
            if header.starts_with(&[0xFF, 0xD8, 0xFF]) {
                return ContentType::Image;
            }
        }

        // Check for HTML
        let text = String::from_utf8_lossy(data);
        if text.starts_with("<!DOCTYPE") || text.starts_with("<html") || text.starts_with("<HTML") {
            return ContentType::Html;
        }

        // Default to text
        ContentType::Text
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_type_detection() {
        assert_eq!(ContentType::detect(b"Hello, World!"), ContentType::Text);
        assert_eq!(ContentType::detect(b"<!DOCTYPE html>"), ContentType::Html);
    }
}
