//! Platform-specific clipboard provider

use crate::PlatformError;
use async_trait::async_trait;
use clipboard_core::{ClipboardItem, ContentType};

/// Platform-agnostic clipboard provider trait
#[async_trait]
pub trait ClipboardProvider: Send + Sync {
    /// Get clipboard content — tries image first, then text
    async fn get_content(&self) -> Result<ClipboardItem, PlatformError>;

    /// Set clipboard content
    async fn set_content(&self, item: &ClipboardItem) -> Result<(), PlatformError>;

    /// Watch for clipboard changes
    async fn watch_changes(&self) -> Result<(), PlatformError>;
}

// ── Shared helpers ────────────────────────────────────────────────────────────

/// Encode raw RGBA pixels as a PNG byte vector.
fn rgba_to_png(width: usize, height: usize, bytes: &[u8]) -> Result<Vec<u8>, String> {
    use image::{ImageBuffer, RgbaImage};
    let img: RgbaImage = ImageBuffer::from_raw(width as u32, height as u32, bytes.to_vec())
        .ok_or_else(|| "Invalid image dimensions".to_string())?;
    let mut buf = std::io::Cursor::new(Vec::new());
    img.write_to(&mut buf, image::ImageFormat::Png)
        .map_err(|e| e.to_string())?;
    Ok(buf.into_inner())
}

/// Detect a richer content type from plain text.
fn detect_text_type(text: &str) -> ContentType {
    let trimmed = text.trim();

    // HTML
    if trimmed.starts_with("<!DOCTYPE")
        || trimmed.starts_with("<html")
        || trimmed.starts_with("<HTML")
    {
        return ContentType::Html;
    }

    // JSON
    if (trimmed.starts_with('{') && trimmed.ends_with('}'))
        || (trimmed.starts_with('[') && trimmed.ends_with(']'))
    {
        if serde_json::from_str::<serde_json::Value>(trimmed).is_ok() {
            return ContentType::Code;
        }
    }

    // Code heuristics
    let code_indicators = [
        "fn ", "def ", "class ", "import ", "use ", "const ", "let ", "var ",
        "function ", "return ", "if (", "for (", "while (", "#include", "public ",
        "private ", "async ", "await ", "=>", "->", "::", "$(", "#!/",
    ];
    let code_score: usize = code_indicators
        .iter()
        .filter(|&&pat| trimmed.contains(pat))
        .count();
    let lines: Vec<&str> = trimmed.lines().collect();
    let indented_lines = lines
        .iter()
        .filter(|l| l.starts_with("    ") || l.starts_with('\t'))
        .count();
    if code_score >= 2 || (lines.len() > 3 && indented_lines > 1) {
        return ContentType::Code;
    }

    ContentType::Text
}

/// On macOS, check whether the pasteboard currently advertises an image type,
/// without actually reading the image data. This avoids the ~50 ms arboard
/// penalty on every poll when the clipboard contains only text.
///
/// Returns `true`  → image type is present, go ahead and call `get_image()`.
/// Returns `false` → skip image attempt entirely.
#[cfg(target_os = "macos")]
#[allow(unexpected_cfgs)]
fn pasteboard_has_image() -> bool {
    // Use the Objective-C runtime to call
    // [[NSPasteboard generalPasteboard] canReadObjectForClasses:...] or
    // more simply check the types array for common image UTIs.
    //
    // We do this via a tiny inline Objective-C call using the `objc` crate
    // that is already a transitive dependency.
    use std::ffi::CStr;
    unsafe {
        use objc::runtime::{Class, Object};
        use objc::{msg_send, sel, sel_impl};

        let cls = match Class::get("NSPasteboard") {
            Some(c) => c,
            None => return false,
        };
        let pb: *mut Object = msg_send![cls, generalPasteboard];
        if pb.is_null() {
            return false;
        }
        let types: *mut Object = msg_send![pb, types];
        if types.is_null() {
            return false;
        }
        let count: usize = msg_send![types, count];
        for i in 0..count {
            let type_str: *mut Object = msg_send![types, objectAtIndex: i];
            if type_str.is_null() {
                continue;
            }
            let utf8: *const std::os::raw::c_char = msg_send![type_str, UTF8String];
            if utf8.is_null() {
                continue;
            }
            let s = CStr::from_ptr(utf8).to_string_lossy();
            // Common image UTIs advertised by macOS apps
            if s.contains("image")
                || s == "public.png"
                || s == "public.jpeg"
                || s == "public.tiff"
                || s == "com.apple.pict"
            {
                return true;
            }
        }
        false
    }
}

#[cfg(not(target_os = "macos"))]
fn pasteboard_has_image() -> bool {
    // On Linux/Windows we always attempt get_image(); arboard is fast there.
    true
}

/// Try to read image from clipboard, encode as PNG.
fn try_get_image() -> Option<Vec<u8>> {
    if !pasteboard_has_image() {
        return None;
    }
    let mut ctx = arboard::Clipboard::new().ok()?;
    let img = ctx.get_image().ok()?;
    rgba_to_png(img.width, img.height, &img.bytes).ok()
}

/// Read text from clipboard.
fn try_get_text() -> Option<String> {
    let mut ctx = arboard::Clipboard::new().ok()?;
    let text = ctx.get_text().ok()?;
    if text.is_empty() { None } else { Some(text) }
}

/// Build a ClipboardItem from the current clipboard state.
/// Prefers images over text.
pub(crate) fn read_clipboard_item() -> Result<ClipboardItem, PlatformError> {
    if let Some(png_bytes) = try_get_image() {
        return Ok(ClipboardItem::new(ContentType::Image, png_bytes));
    }
    match try_get_text() {
        Some(text) => {
            let content_type = detect_text_type(&text);
            Ok(ClipboardItem::new(content_type, text.into_bytes()))
        }
        None => Err(PlatformError::AccessFailed("Clipboard is empty".to_string())),
    }
}

/// Write text to the system clipboard.
pub(crate) fn write_text(text: &str) -> Result<(), PlatformError> {
    let mut ctx =
        arboard::Clipboard::new().map_err(|e| PlatformError::AccessFailed(e.to_string()))?;
    ctx.set_text(text)
        .map_err(|e| PlatformError::AccessFailed(e.to_string()))
}

// ── Platform enum ─────────────────────────────────────────────────────────────

#[derive(Clone)]
pub enum PlatformProvider {
    #[cfg(target_os = "macos")]
    Macos,
    #[cfg(target_os = "linux")]
    Linux,
    #[cfg(windows)]
    Windows,
}

#[async_trait]
impl ClipboardProvider for PlatformProvider {
    async fn get_content(&self) -> Result<ClipboardItem, PlatformError> {
        read_clipboard_item()
    }

    async fn set_content(&self, item: &ClipboardItem) -> Result<(), PlatformError> {
        match item.content_type {
            ContentType::Image => {
                let img = image::load_from_memory(&item.content)
                    .map_err(|e| PlatformError::AccessFailed(e.to_string()))?
                    .to_rgba8();
                let (w, h) = img.dimensions();
                let mut ctx = arboard::Clipboard::new()
                    .map_err(|e| PlatformError::AccessFailed(e.to_string()))?;
                ctx.set_image(arboard::ImageData {
                    width: w as usize,
                    height: h as usize,
                    bytes: img.into_raw().into(),
                })
                .map_err(|e| PlatformError::AccessFailed(e.to_string()))
            }
            _ => {
                let text = String::from_utf8_lossy(&item.content).to_string();
                write_text(&text)
            }
        }
    }

    async fn watch_changes(&self) -> Result<(), PlatformError> {
        Err(PlatformError::WatchFailed("Not implemented".to_string()))
    }
}

pub fn get_provider() -> Result<PlatformProvider, PlatformError> {
    #[cfg(target_os = "macos")]
    return Ok(PlatformProvider::Macos);
    #[cfg(target_os = "linux")]
    return Ok(PlatformProvider::Linux);
    #[cfg(windows)]
    return Ok(PlatformProvider::Windows);
    #[allow(unreachable_code)]
    Err(PlatformError::UnsupportedPlatform)
}
