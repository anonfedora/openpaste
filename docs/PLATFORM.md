# OpenPaste Platform Integration

## Overview

OpenPaste integrates with platform-specific clipboard APIs on Windows, Linux (Wayland + X11), and macOS. This document details the clipboard access mechanisms, change detection strategies, and platform-specific considerations.

## Architecture

```
clipboard-platform crate
    │
    ├── Platform Abstraction Layer
    │
    ├── Windows Implementation
    │   ├── clipboard-win
    │   └── windows-rs
    │
    ├── Linux Implementation
    │   ├── Wayland (wl-clipboard-rs)
    │   └── X11 (x11-clipboard)
    │
    └── macOS Implementation
        ├── cocoa
        └── objc
```

## Platform Abstraction

### Trait Definition

```rust
pub trait ClipboardProvider: Send + Sync {
    /// Get current clipboard content
    fn get_content(&self) -> Result<ClipboardContent, PlatformError>;
    
    /// Set clipboard content
    fn set_content(&self, content: &ClipboardContent) -> Result<(), PlatformError>;
    
    /// Start watching for clipboard changes
    fn start_watching(&self) -> Result<ClipboardWatcher, PlatformError>;
    
    /// Check if clipboard has changed
    fn has_changed(&self) -> Result<bool, PlatformError>;
    
    /// Get available formats
    fn get_formats(&self) -> Result<Vec<String>, PlatformError>;
    
    /// Clear clipboard
    fn clear(&self) -> Result<(), PlatformError>;
}
```

### Content Types

```rust
pub enum ClipboardContent {
    Text(String),
    Html(String),
    Image(Vec<u8>),
    Files(Vec<String>),
    Custom { format: String, data: Vec<u8> },
}
```

## Windows

### Clipboard API

**Library:** `clipboard-win`

**API:** Windows Clipboard API (user32.dll)

### Implementation

#### Get Content

```rust
use clipboard_win::{get_clipboard_string, formats, is_format_avail};

fn get_content() -> Result<ClipboardContent, PlatformError> {
    // Check for text
    if is_format_avail(formats::CF_UNICODETEXT) {
        let text = get_clipboard_string()?;
        return Ok(ClipboardContent::Text(text));
    }
    
    // Check for HTML
    if is_format_avail(HtmlFormat) {
        let html = get_clipboard_html()?;
        return Ok(ClipboardContent::Html(html));
    }
    
    // Check for image
    if is_format_avail(formats::CF_DIB) {
        let image = get_clipboard_image()?;
        return Ok(ClipboardContent::Image(image));
    }
    
    // Check for files
    if is_format_avail(FileDescriptorFormat) {
        let files = get_clipboard_files()?;
        return Ok(ClipboardContent::Files(files));
    }
    
    Err(PlatformError::NoContent)
}
```

#### Set Content

```rust
use clipboard_win::{set_clipboard_string, set_clipboard_image};

fn set_content(content: &ClipboardContent) -> Result<(), PlatformError> {
    match content {
        ClipboardContent::Text(text) => {
            set_clipboard_string(text)?;
        }
        ClipboardContent::Html(html) => {
            set_clipboard_html(html)?;
        }
        ClipboardContent::Image(image) => {
            set_clipboard_image(image)?;
        }
        ClipboardContent::Files(files) => {
            set_clipboard_files(files)?;
        }
        _ => return Err(PlatformError::UnsupportedFormat),
    }
    Ok(())
}
```

### Clipboard Watching

**Strategy:** Polling with `GetClipboardSequenceNumber`

**Implementation:**
```rust
use clipboard_win::get_sequence_number;

struct WindowsClipboardWatcher {
    last_sequence: u32,
}

impl WindowsClipboardWatcher {
    fn has_changed(&mut self) -> Result<bool, PlatformError> {
        let current = get_sequence_number()?;
        let changed = current != self.last_sequence;
        self.last_sequence = current;
        Ok(changed)
    }
}
```

**Polling Interval:** 100ms

**Alternative:** Clipboard viewer chain (more complex, less reliable)

### Windows Formats

**Standard Formats:**
- `CF_UNICODETEXT`: Unicode text
- `CF_DIB`: Device-independent bitmap
- `CF_HDROP`: File list

**Custom Formats:**
- `HTML Format`: HTML content (MS HTML format)
- `FileGroupDescriptor`: File metadata

### Permissions

**Required:** None (clipboard access is unrestricted on Windows)

**Considerations:**
- UAC elevation may be required for some operations
- Background access works without special permissions

### Windows-Specific Issues

**Clipboard Viewer Chain:**
- Can be broken by other applications
- Not reliable for change detection
- Use sequence number instead

**Large Data:**
- Windows clipboard has size limits
- Use delayed rendering for large data
- Fall back to file storage for very large items

**COM Initialization:**
- Clipboard API requires COM initialization
- Must initialize COM on each thread
- Use `OleInitialize` or `CoInitializeEx`

## Linux

### Wayland

**Library:** `wl-clipboard-rs`

**Protocol:** Wayland clipboard protocol (wl-clipboard)

### Implementation

#### Get Content

```rust
use wl_clipboard_rs::copy::{MimeType, Options, Source};

fn get_content() -> Result<ClipboardContent, PlatformError> {
    let mut opts = Options::new();
    opts.foreground(false);
    
    let source = Source::new(&opts)?;
    
    // Try text/plain
    if let Ok(text) = source.load(MimeType::Text, "text/plain") {
        return Ok(ClipboardContent::Text(String::from_utf8(text)?));
    }
    
    // Try text/html
    if let Ok(html) = source.load(MimeType::Text, "text/html") {
        return Ok(ClipboardContent::Html(String::from_utf8(html)?));
    }
    
    // Try image/png
    if let Ok(image) = source.load(MimeType::Image, "image/png") {
        return Ok(ClipboardContent::Image(image));
    }
    
    Err(PlatformError::NoContent)
}
```

#### Set Content

```rust
use wl_clipboard_rs::copy::{MimeType, Options, Source};

fn set_content(content: &ClipboardContent) -> Result<(), PlatformError> {
    let mut opts = Options::new();
    opts.foreground(false);
    opts.copy(true);
    
    let mut source = Source::new(&opts)?;
    
    match content {
        ClipboardContent::Text(text) => {
            source.add(MimeType::Text, "text/plain", text.as_bytes())?;
        }
        ClipboardContent::Html(html) => {
            source.add(MimeType::Text, "text/html", html.as_bytes())?;
        }
        ClipboardContent::Image(image) => {
            source.add(MimeType::Image, "image/png", image)?;
        }
        _ => return Err(PlatformError::UnsupportedFormat),
    }
    
    source.serve()?;
    Ok(())
}
```

### Clipboard Watching

**Strategy:** Polling with content hash comparison

**Implementation:**
```rust
struct WaylandClipboardWatcher {
    last_hash: Option<String>,
}

impl WaylandClipboardWatcher {
    fn has_changed(&mut self) -> Result<bool, PlatformError> {
        let content = get_content()?;
        let hash = compute_hash(&content)?;
        
        let changed = self.last_hash.as_ref() != Some(&hash);
        self.last_hash = Some(hash);
        
        Ok(changed)
    }
}
```

**Polling Interval:** 200ms

**Note:** Wayland doesn't provide change notifications, polling is required

### X11

**Library:** `x11-clipboard`

**Protocol:** X11 clipboard protocol

### Implementation

#### Get Content

```rust
use x11_clipboard::Clipboard;

fn get_content() -> Result<ClipboardContent, PlatformError> {
    let clipboard = Clipboard::new(Arc::new(Atom::new(display, "CLIPBOARD")))?;
    
    // Try text/plain
    if let Ok(text) = clipboard.load(display, window, Atom::new(display, "UTF8_STRING"), 
                                     Atom::new(display, "CLIPBOARD"), 
                                     clipboard.waiter(display, window)) {
        return Ok(ClipboardContent::Text(String::from_utf8(text)?));
    }
    
    // Try image/png
    if let Ok(image) = clipboard.load(display, window, Atom::new(display, "image/png"),
                                     Atom::new(display, "CLIPBOARD"),
                                     clipboard.waiter(display, window)) {
        return Ok(ClipboardContent::Image(image));
    }
    
    Err(PlatformError::NoContent)
}
```

#### Set Content

```rust
use x11_clipboard::Clipboard;

fn set_content(content: &ClipboardContent) -> Result<(), PlatformError> {
    let clipboard = Clipboard::new(Arc::new(Atom::new(display, "CLIPBOARD")))?;
    
    match content {
        ClipboardContent::Text(text) => {
            clipboard.store(
                display,
                window,
                Atom::new(display, "UTF8_STRING"),
                Atom::new(display, "CLIPBOARD"),
                text.as_bytes(),
            )?;
        }
        ClipboardContent::Image(image) => {
            clipboard.store(
                display,
                window,
                Atom::new(display, "image/png"),
                Atom::new(display, "CLIPBOARD"),
                image,
            )?;
        }
        _ => return Err(PlatformError::UnsupportedFormat),
    }
    
    Ok(())
}
```

### Clipboard Watching

**Strategy:** X11 Selection events

**Implementation:**
```rust
struct X11ClipboardWatcher {
    window: Window,
}

impl X11ClipboardWatcher {
    fn has_changed(&mut self) -> Result<bool, PlatformError> {
        // Check for SelectionNotify events
        let event = next_event(display)?;
        
        match event {
            Event::SelectionNotify(_) => Ok(true),
            _ => Ok(false),
        }
    }
}
```

**Alternative:** Polling with content hash (fallback)

### Linux Format Handling

**MIME Types:**
- `text/plain`: Plain text
- `text/html`: HTML content
- `image/png`: PNG images
- `image/jpeg`: JPEG images
- `text/uri-list`: File URIs

**Selections:**
- `CLIPBOARD`: Standard clipboard (Ctrl+C)
- `PRIMARY`: Selection highlight (middle-click)
- `SECONDARY`: Secondary selection

**Strategy:** Monitor CLIPBOARD only (PRIMARY is optional)

### Permissions

**Required:** None for basic clipboard access

**Wayland Considerations:**
- Some Wayland compositors restrict clipboard access
- May require portal for file access
- Background access may be limited

**X11 Considerations:**
- No restrictions
- Full access to all selections

### Linux-Specific Issues

**Wayland vs X11:**
- Detect compositor type
- Use appropriate library
- Fallback to X11 if Wayland fails

**Clipboard Ownership:**
- Clipboard is lost when application exits
- Must keep daemon running
- Use clipboard manager daemon if available

**Large Data:**
- X11 has size limits
- Use incremental transfer for large data
- Fall back to file storage

## macOS

### Clipboard API

**Library:** `cocoa` + `objc`

**Framework:** AppKit (NSPasteboard)

### Implementation

#### Get Content

```rust
use cocoa::appkit::{NSPasteboard, NSString};
use cocoa::base::nil;

fn get_content() -> Result<ClipboardContent, PlatformError> {
    let pasteboard = NSPasteboard::generalPasteboard(nil);
    
    // Try text
    if let Some(text) = pasteboard.stringForType_(NSString::alloc(nil).init_str("public.utf8-plain-text")) {
        return Ok(ClipboardContent::Text(text.to_string()));
    }
    
    // Try HTML
    if let Some(html) = pasteboard.stringForType_(NSString::alloc(nil).init_str("public.html")) {
        return Ok(ClipboardContent::Html(html.to_string()));
    }
    
    // Try image
    if let Some(image) = pasteboard.dataForType_(NSString::alloc(nil).init_str("public.png")) {
        return Ok(ClipboardContent::Image(image.to_vec()));
    }
    
    // Try files
    if let Some(files) = pasteboard.propertyListForType_(NSString::alloc(nil).init_str("public.file-url")) {
        return Ok(ClipboardContent::Files(parse_file_urls(files)));
    }
    
    Err(PlatformError::NoContent)
}
```

#### Set Content

```rust
use cocoa::appkit::NSPasteboard;

fn set_content(content: &ClipboardContent) -> Result<(), PlatformError> {
    let pasteboard = NSPasteboard::generalPasteboard(nil);
    pasteboard.clearContents(nil);
    
    match content {
        ClipboardContent::Text(text) => {
            let ns_string = NSString::alloc(nil).init_str(text);
            pasteboard.setString_forType_(ns_string, NSString::alloc(nil).init_str("public.utf8-plain-text"));
        }
        ClipboardContent::Html(html) => {
            let ns_string = NSString::alloc(nil).init_str(html);
            pasteboard.setString_forType_(ns_string, NSString::alloc(nil).init_str("public.html"));
        }
        ClipboardContent::Image(image) => {
            let ns_data = NSData::dataWithBytes_length_(nil, image.as_ptr(), image.len());
            pasteboard.setData_forType_(ns_data, NSString::alloc(nil).init_str("public.png"));
        }
        _ => return Err(PlatformError::UnsupportedFormat),
    }
    
    Ok(())
}
```

### Clipboard Watching

**Strategy:** Polling with change count

**Implementation:**
```rust
struct MacClipboardWatcher {
    last_change_count: i64,
}

impl MacClipboardWatcher {
    fn has_changed(&mut self) -> Result<bool, PlatformError> {
        let pasteboard = NSPasteboard::generalPasteboard(nil);
        let current_count = pasteboard.changeCount(nil);
        
        let changed = current_count != self.last_change_count;
        self.last_change_count = current_count;
        
        Ok(changed)
    }
}
```

**Polling Interval:** 100ms

**Alternative:** NSPasteboardDidChangeNotification (event-based)

### macOS Formats

**UTI (Uniform Type Identifiers):**
- `public.utf8-plain-text`: Plain text
- `public.html`: HTML content
- `public.png`: PNG images
- `public.jpeg`: JPEG images
- `public.file-url`: File URLs
- `public.rtf`: Rich Text Format

### Permissions

**Required:** None for clipboard access

**Accessibility:**
- May require accessibility permission for some operations
- Request permission on first use
- Handle permission denial gracefully

### macOS-Specific Issues

**Sandboxing:**
- If sandboxed, clipboard access restricted
- Must request entitlements
- May need user approval

**Notarization:**
- Required for distribution
- Must sign binary
- Include in Info.plist

**Pasteboard Types:**
- macOS uses UTIs, not MIME types
- Must convert between formats
- Some formats are macOS-specific

## Platform Detection

### Auto-Detection

```rust
pub fn get_platform_provider() -> Box<dyn ClipboardProvider> {
    #[cfg(target_os = "windows")]
    {
        Box::new(WindowsClipboardProvider::new())
    }
    
    #[cfg(target_os = "linux")]
    {
        if is_wayland() {
            Box::new(WaylandClipboardProvider::new())
        } else {
            Box::new(X11ClipboardProvider::new())
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        Box::new(MacClipboardProvider::new())
    }
}
```

### Wayland Detection

```rust
fn is_wayland() -> bool {
    std::env::var("WAYLAND_DISPLAY").is_ok()
}
```

### X11 Detection

```rust
fn is_x11() -> bool {
    std::env::var("DISPLAY").is_ok()
}
```

## Clipboard Watching Strategy

### Polling vs Event-Based

**Polling:**
- Pros: Simple, reliable, works everywhere
- Cons: Continuous CPU usage, latency

**Event-Based:**
- Pros: Efficient, low latency
- Cons: Complex, platform-specific, less reliable

**Decision:** Use polling for reliability, event-based where available

### Polling Intervals

**Windows:** 100ms (sequence number check is fast)

**Linux Wayland:** 200ms (no event API)

**Linux X11:** 100ms (or use events)

**macOS:** 100ms (change count check is fast)

### Debouncing

**Purpose:** Avoid duplicate captures

**Strategy:** Wait 500ms after change before capturing

**Implementation:**
```rust
struct DebouncedWatcher {
    watcher: Box<dyn ClipboardWatcher>,
    last_change: Instant,
    debounce_duration: Duration,
}

impl DebouncedWatcher {
    fn has_changed(&mut self) -> Result<bool, PlatformError> {
        if self.watcher.has_changed()? {
            let now = Instant::now();
            if now - self.last_change > self.debounce_duration {
                self.last_change = now;
                return Ok(true);
            }
        }
        Ok(false)
    }
}
```

## Error Handling

### Platform-Specific Errors

**Windows:**
- Clipboard not opened
- Format not available
- COM initialization failed

**Linux:**
- Wayland compositor not available
- X11 display not available
- Selection not owned

**macOS:**
- Pasteboard not available
- Permission denied
- Sandbox restriction

### Fallback Strategy

**Primary Fails:** Try alternative method

**All Fail:** Log error, continue without clipboard watching

**Recovery:** Retry after delay

## Performance

### Performance Targets

- **Get Content:** < 50ms
- **Set Content:** < 50ms
- **Change Detection:** < 10ms
- **Polling Overhead:** < 1% CPU

### Optimization

**Caching:**
- Cache format availability
- Cache clipboard content (briefly)
- Avoid redundant reads

**Batching:**
- Batch format checks
- Batch content retrieval

**Async:**
- Async clipboard operations
- Non-blocking change detection

## Testing

### Unit Tests

- Mock clipboard provider
- Test content parsing
- Test format detection

### Integration Tests

- Test on each platform
- Test with various content types
- Test error handling

### Manual Testing

- Test with real applications
- Test with large content
- Test with special characters

## Platform-Specific Features

### Windows

**Rich Text:** RTF format support

**Delayed Rendering:** For large content

**Clipboard History:** Windows 10+ clipboard history

### Linux

**Primary Selection:** Optional support for middle-click paste

**Clipboard Managers:** Integration with clipboard manager daemons

**Portal Integration:** File access via portals

### macOS

**Finder Sync:** Integration with Finder

**Spotlight:** Integration with Spotlight search

**Quick Look:** Preview clipboard content

## Future Enhancements

### Remote Clipboard

**Purpose:** Sync clipboard across devices

**Implementation:**
- Network clipboard protocol
- End-to-end encryption
- Conflict resolution

### Clipboard Filters

**Purpose:** Transform clipboard content

**Examples:**
- Remove formatting
- Sanitize HTML
- Convert formats

### Clipboard Actions

**Purpose:** Automatic actions on clipboard change

**Examples:**
- Auto-save images
- Auto-extract URLs
- Auto-detect code
