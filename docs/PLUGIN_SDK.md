# OpenPaste Plugin SDK

## Overview

OpenPaste plugins are WebAssembly (WASM) modules that extend OpenPaste functionality. Plugins run in a sandboxed environment with controlled access to OpenPaste APIs. This document describes the plugin SDK, lifecycle, permissions, and available APIs.

## Plugin Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    OpenPaste Daemon                         │
│  ┌───────────────────────────────────────────────────────┐  │
│  │              Plugin Runtime (Wasmtime)                 │  │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  │  │
│  │  │   Plugin A  │  │   Plugin B  │  │   Plugin C  │  │  │
│  │  │  (WASM)     │  │  (WASM)     │  │  (WASM)     │  │  │
│  │  └─────────────┘  └─────────────┘  └─────────────┘  │  │
│  │                                                       │  │
│  │  ┌───────────────────────────────────────────────┐  │  │
│  │  │              Host API                         │  │  │
│  │  │  - Clipboard API                              │  │  │
│  │  │  - Storage API                                │  │  │
│  │  │  - Search API                                 │  │  │
│  │  │  - Event API                                  │  │  │
│  │  │  - UI API                                     │  │  │
│  │  └───────────────────────────────────────────────┘  │  │
│  └───────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

## Plugin Manifest

### manifest.json

Every plugin requires a manifest file:

```json
{
  "name": "url-detector",
  "version": "1.0.0",
  "description": "Detects and categorizes URLs in clipboard content",
  "author": "OpenPaste Community",
  "license": "MIT",
  "homepage": "https://github.com/openpaste/plugins/url-detector",
  "permissions": [
    "clipboard:read",
    "storage:read",
    "storage:write",
    "event:subscribe"
  ],
  "entrypoint": "url_detector.wasm",
  "min_openpaste_version": "0.1.0",
  "max_openpaste_version": "1.0.0",
  "settings": {
    "auto_categorize": {
      "type": "boolean",
      "default": true,
      "description": "Automatically categorize URLs"
    },
    "custom_categories": {
      "type": "array",
      "default": [],
      "description": "Custom URL categories"
    }
  }
}
```

### Manifest Fields

**Required Fields:**
- `name`: Plugin identifier (kebab-case)
- `version`: SemVer version string
- `description`: Short description
- `entrypoint`: WASM file name
- `permissions`: Array of required permissions

**Optional Fields:**
- `author`: Plugin author
- `license`: Plugin license
- `homepage`: Plugin homepage URL
- `min_openpaste_version`: Minimum OpenPaste version
- `max_openpaste_version`: Maximum OpenPaste version
- `settings`: Plugin settings schema

## Plugin Lifecycle

### Lifecycle States

```
[Loaded] → [Initialized] → [Running] → [Paused] → [Unloaded]
              ↓              ↓
          [Error]        [Error]
```

### Lifecycle Methods

**Exported Functions (WASM):**

```rust
// Called when plugin is loaded
#[no_mangle]
pub extern "C" fn openpaste_init() -> i32;

// Called when plugin is unloaded
#[no_mangle]
pub extern "C" fn openpaste_shutdown();

// Called when clipboard item is added
#[no_mangle]
pub extern "C" fn openpaste_on_clipboard_added(item_id: i64) -> i32;

// Called when search is performed
#[no_mangle]
pub extern "C" fn openpaste_on_search(query_ptr: *const u8, query_len: usize) -> i32;

// Called when plugin settings change
#[no_mangle]
pub extern "C" fn openpaste_on_settings_changed(settings_ptr: *const u8, settings_len: usize) -> i32;
```

### Initialization

```rust
#[no_mangle]
pub extern "C" fn openpaste_init() -> i32 {
    // Initialize plugin state
    // Register event handlers
    // Return 0 on success, non-zero on error
    0
}
```

### Shutdown

```rust
#[no_mangle]
pub extern "C" fn openpaste_shutdown() {
    // Clean up resources
    // Unregister event handlers
    // Save state if needed
}
```

## Permissions

### Permission Model

Plugins must declare required permissions in manifest. Users grant permissions at install time.

### Permission Categories

**Clipboard Permissions:**
- `clipboard:read` - Read clipboard content
- `clipboard:write` - Write to clipboard
- `clipboard:watch` - Watch for clipboard changes

**Storage Permissions:**
- `storage:read` - Read clipboard items from storage
- `storage:write` - Write clipboard items to storage
- `storage:delete` - Delete clipboard items

**Search Permissions:**
- `search:execute` - Execute searches
- `search:index` - Modify search index

**Event Permissions:**
- `event:subscribe` - Subscribe to events
- `event:publish` - Publish events

**Network Permissions:**
- `network:http` - Make HTTP requests
- `network:https` - Make HTTPS requests

**File Permissions:**
- `file:read` - Read files
- `file:write` - Write files

**UI Permissions:**
- `ui:notify` - Show notifications
- `ui:dialog` - Show dialogs

### Permission Checks

Runtime checks enforce permissions:

```rust
fn check_permission(plugin: &Plugin, permission: &str) -> bool {
    plugin.manifest.permissions.contains(&permission)
}
```

## Host API

### Clipboard API

**Get Clipboard Content:**

```rust
#[no_mangle]
pub extern "C" fn openpaste_clipboard_get(item_id: i64, out_ptr: *mut u8, out_len: *mut usize) -> i32 {
    if !has_permission("clipboard:read") {
        return -1; // Permission denied
    }
    
    let content = host_clipboard_get(item_id);
    // Copy content to out_ptr
    // Set out_len to content length
    0
}
```

**Set Clipboard Content:**

```rust
#[no_mangle]
pub extern "C" fn openpaste_clipboard_set(content_ptr: *const u8, content_len: usize) -> i64 {
    if !has_permission("clipboard:write") {
        return -1;
    }
    
    let content = unsafe { std::slice::from_raw_parts(content_ptr, content_len) };
    host_clipboard_set(content)
}
```

### Storage API

**Get Item:**

```rust
#[no_mangle]
pub extern "C" fn openpaste_storage_get(item_id: i64, out_ptr: *mut u8, out_len: *mut usize) -> i32 {
    if !has_permission("storage:read") {
        return -1;
    }
    
    let item = host_storage_get(item_id);
    // Serialize item to JSON
    // Copy to out_ptr
    0
}
```

**Store Item:**

```rust
#[no_mangle]
pub extern "C" fn openpaste_storage_store(item_ptr: *const u8, item_len: usize) -> i64 {
    if !has_permission("storage:write") {
        return -1;
    }
    
    let item_json = unsafe { std::slice::from_raw_parts(item_ptr, item_len) };
    let item: ClipboardItem = serde_json::from_slice(item_json).unwrap();
    host_storage_store(item)
}
```

**Add Tag:**

```rust
#[no_mangle]
pub extern "C" fn openpaste_storage_add_tag(item_id: i64, tag_ptr: *const u8, tag_len: usize) -> i32 {
    if !has_permission("storage:write") {
        return -1;
    }
    
    let tag = unsafe { std::slice::from_raw_parts(tag_ptr, tag_len) };
    let tag_str = std::str::from_utf8(tag).unwrap();
    host_storage_add_tag(item_id, tag_str)
}
```

### Search API

**Execute Search:**

```rust
#[no_mangle]
pub extern "C" fn openpaste_search_execute(
    query_ptr: *const u8,
    query_len: usize,
    out_ptr: *mut u8,
    out_len: *mut usize
) -> i32 {
    if !has_permission("search:execute") {
        return -1;
    }
    
    let query = unsafe { std::slice::from_raw_parts(query_ptr, query_len) };
    let query_str = std::str::from_utf8(query).unwrap();
    let results = host_search_execute(query_str);
    // Serialize results to JSON
    // Copy to out_ptr
    0
}
```

### Event API

**Subscribe to Event:**

```rust
#[no_mangle]
pub extern "C" fn openpaste_event_subscribe(event_type_ptr: *const u8, event_type_len: usize) -> i32 {
    if !has_permission("event:subscribe") {
        return -1;
    }
    
    let event_type = unsafe { std::slice::from_raw_parts(event_type_ptr, event_type_len) };
    let event_type_str = std::str::from_utf8(event_type).unwrap();
    host_event_subscribe(event_type_str)
}
```

**Publish Event:**

```rust
#[no_mangle]
pub extern "C" fn openpaste_event_publish(
    event_type_ptr: *const u8,
    event_type_len: usize,
    data_ptr: *const u8,
    data_len: usize
) -> i32 {
    if !has_permission("event:publish") {
        return -1;
    }
    
    let event_type = unsafe { std::slice::from_raw_parts(event_type_ptr, event_type_len) };
    let event_type_str = std::str::from_utf8(event_type).unwrap();
    let data = unsafe { std::slice::from_raw_parts(data_ptr, data_len) };
    host_event_publish(event_type_str, data)
}
```

### UI API

**Show Notification:**

```rust
#[no_mangle]
pub extern "C" fn openpaste_ui_notify(
    title_ptr: *const u8,
    title_len: usize,
    message_ptr: *const u8,
    message_len: usize
) -> i32 {
    if !has_permission("ui:notify") {
        return -1;
    }
    
    let title = unsafe { std::slice::from_raw_parts(title_ptr, title_len) };
    let title_str = std::str::from_utf8(title).unwrap();
    let message = unsafe { std::slice::from_raw_parts(message_ptr, message_len) };
    let message_str = std::str::from_utf8(message).unwrap();
    host_ui_notify(title_str, message_str)
}
```

### Logging API

**Log Message:**

```rust
#[no_mangle]
pub extern "C" fn openpaste_log(
    level: i32,
    message_ptr: *const u8,
    message_len: usize
) {
    let message = unsafe { std::slice::from_raw_parts(message_ptr, message_len) };
    let message_str = std::str::from_utf8(message).unwrap();
    host_log(level, message_str)
}
```

**Log Levels:**
- 0: Debug
- 1: Info
- 2: Warning
- 3: Error

## Plugin Development

### Rust Plugin Template

**Cargo.toml:**
```toml
[package]
name = "my-plugin"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

**lib.rs:**
```rust
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

#[no_mangle]
pub extern "C" fn openpaste_init() -> i32 {
    log_message(1, "Plugin initialized");
    0
}

#[no_mangle]
pub extern "C" fn openpaste_shutdown() {
    log_message(1, "Plugin shutdown");
}

#[no_mangle]
pub extern "C" fn openpaste_on_clipboard_added(item_id: i64) -> i32 {
    log_message(1, &format!("Clipboard item added: {}", item_id));
    0
}

fn log_message(level: i32, message: &str) {
    let c_message = CString::new(message).unwrap();
    unsafe {
        openpaste_log(level, c_message.as_ptr());
    }
}

// External host functions
extern "C" {
    fn openpaste_log(level: i32, message: *const c_char);
}
```

### Building Plugin

**Build for WASM:**
```bash
cargo build --target wasm32-unknown-unknown --release
```

**Optimize WASM:**
```bash
wasm-opt -O3 target/wasm32-unknown-unknown/release/my_plugin.wasm -o my_plugin.wasm
```

### Go Plugin Template

**main.go:**
```go
package main

import "C"

//export openpaste_init
func openpaste_init() C.int {
    logMessage(1, "Plugin initialized")
    return 0
}

//export openpaste_shutdown
func openpaste_shutdown() {
    logMessage(1, "Plugin shutdown")
}

//export openpaste_on_clipboard_added
func openpaste_on_clipboard_added(itemId C.long) C.int {
    logMessage(1, "Clipboard item added")
    return 0
}

func logMessage(level int, message string) {
    cMessage := C.CString(message)
    defer C.free(unsafe.Pointer(cMessage))
    C.openpaste_log(C.int(level), cMessage)
}

func main() {}
```

**Build for WASM:**
```bash
tinygo build -o my_plugin.wasm -target wasm ./...
```

## Plugin Distribution

### Plugin Package Structure

```
my-plugin/
├── manifest.json
├── my-plugin.wasm
├── README.md
└── icon.png (optional)
```

### Plugin Registry

Plugins can be published to the OpenPaste plugin registry:

```json
{
  "plugins": [
    {
      "name": "url-detector",
      "version": "1.0.0",
      "description": "Detects and categorizes URLs",
      "download_url": "https://github.com/openpaste/plugins/releases/download/v1.0.0/url-detector.zip",
      "checksum": "sha256:abc123...",
      "author": "OpenPaste Community",
      "license": "MIT"
    }
  ]
}
```

### Installation

**From Registry:**
```bash
openpaste plugin install url-detector
```

**From File:**
```bash
openpaste plugin install ./my-plugin.zip
```

## Plugin Settings

### Settings Schema

Defined in manifest.json:

```json
{
  "settings": {
    "enabled": {
      "type": "boolean",
      "default": true,
      "description": "Enable plugin"
    },
    "api_key": {
      "type": "string",
      "default": "",
      "description": "API key for external service",
      "secret": true
    },
    "max_items": {
      "type": "integer",
      "default": 100,
      "min": 1,
      "max": 1000,
      "description": "Maximum items to process"
    }
  }
}
```

### Accessing Settings

**Get Setting:**

```rust
#[no_mangle]
pub extern "C" fn openpaste_settings_get(
    key_ptr: *const u8,
    key_len: usize,
    out_ptr: *mut u8,
    out_len: *mut usize
) -> i32 {
    let key = unsafe { std::slice::from_raw_parts(key_ptr, key_len) };
    let key_str = std::str::from_utf8(key).unwrap();
    let value = host_settings_get(key_str);
    // Copy value to out_ptr
    0
}
```

**Set Setting:**

```rust
#[no_mangle]
pub extern "C" fn openpaste_settings_set(
    key_ptr: *const u8,
    key_len: usize,
    value_ptr: *const u8,
    value_len: usize
) -> i32 {
    let key = unsafe { std::slice::from_raw_parts(key_ptr, key_len) };
    let key_str = std::str::from_utf8(key).unwrap();
    let value = unsafe { std::slice::from_raw_parts(value_ptr, value_len) };
    let value_str = std::str::from_utf8(value).unwrap();
    host_settings_set(key_str, value_str)
}
```

## Sandboxing

### Sandboxing Model

**Memory Isolation:**
- Separate WASM memory space
- No direct access to host memory
- All access via host API

**Resource Limits:**
- Memory limit: 128MB
- CPU time limit: 1 second per operation
- Network rate limit: 10 requests/second

**Filesystem Access:**
- No direct filesystem access
- Only via host API with permissions

**Network Access:**
- Only with network permission
- HTTPS only (unless http permission granted)
- Rate limited

### Security Considerations

**Input Validation:**
- Validate all plugin inputs
- Sanitize data before processing
- Limit buffer sizes

**Output Sanitization:**
- Sanitize plugin output
- Validate data structures
- Prevent injection attacks

**Error Handling:**
- Catch plugin errors
- Prevent plugin crashes
- Log errors for debugging

## Plugin Examples

### URL Detector Plugin

Detects and categorizes URLs in clipboard content.

```rust
#[no_mangle]
pub extern "C" fn openpaste_on_clipboard_added(item_id: i64) -> i32 {
    // Get item content
    let content = get_item_content(item_id);
    
    // Detect URLs
    let urls = detect_urls(&content);
    
    // Categorize URLs
    for url in urls {
        let category = categorize_url(&url);
        add_tag(item_id, &category);
    }
    
    0
}

fn detect_urls(text: &str) -> Vec<String> {
    // URL detection logic
    vec![]
}

fn categorize_url(url: &str) -> String {
    // URL categorization logic
    "url".to_string()
}
```

### Code Formatter Plugin

Formats code snippets in clipboard content.

```rust
#[no_mangle]
pub extern "C" fn openpaste_on_clipboard_added(item_id: i64) -> i32 {
    let content = get_item_content(item_id);
    
    if is_code(&content) {
        let formatted = format_code(&content);
        update_item_content(item_id, &formatted);
    }
    
    0
}
```

### Auto-Tagger Plugin

Automatically tags clipboard items based on content.

```rust
#[no_mangle]
pub extern "C" fn openpaste_on_clipboard_added(item_id: i64) -> i32 {
    let content = get_item_content(item_id);
    let tags = generate_tags(&content);
    
    for tag in tags {
        add_tag(item_id, &tag);
    }
    
    0
}
```

## Plugin Testing

### Unit Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_detection() {
        let text = "Check out https://example.com";
        let urls = detect_urls(text);
        assert_eq!(urls.len(), 1);
    }
}
```

### Integration Testing

Test plugin with mock host API:

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_plugin_lifecycle() {
        assert_eq!(openpaste_init(), 0);
        openpaste_shutdown();
    }
}
```

## Plugin Debugging

### Logging

Use the logging API to debug:

```rust
log_message(0, "Debug message");
log_message(1, "Info message");
log_message(2, "Warning message");
log_message(3, "Error message");
```

### Error Handling

Return error codes from functions:

```rust
#[no_mangle]
pub extern "C" fn openpaste_on_clipboard_added(item_id: i64) -> i32 {
    match process_item(item_id) {
        Ok(_) => 0,
        Err(e) => {
            log_message(3, &format!("Error: {}", e));
            -1
        }
    }
}
```

## Plugin Performance

### Performance Guidelines

**Memory:**
- Keep memory usage low
- Reuse allocations
- Avoid large buffers

**CPU:**
- Optimize hot paths
- Use efficient algorithms
- Avoid blocking operations

**Network:**
- Cache responses
- Use connection pooling
- Implement rate limiting

### Profiling

Use WASM profiling tools:
- Chrome DevTools
- wasm-bindgen
- custom instrumentation

## Plugin Security Best Practices

### Input Validation

```rust
fn validate_input(input: &str) -> bool {
    input.len() < MAX_INPUT_LENGTH && input.is_ascii()
}
```

### Output Sanitization

```rust
fn sanitize_output(output: &str) -> String {
    output.chars()
        .filter(|c| c.is_ascii() && !c.is_control())
        .collect()
}
```

### Secret Management

Never log secrets:
```rust
fn process_api_key(key: &str) {
    // Don't log the key
    log_message(1, "Processing API key");
    // Use key
}
```

## Plugin API Reference

### Complete API List

**Clipboard:**
- `openpaste_clipboard_get`
- `openpaste_clipboard_set`
- `openpaste_clipboard_get_formats`

**Storage:**
- `openpaste_storage_get`
- `openpaste_storage_store`
- `openpaste_storage_delete`
- `openpaste_storage_add_tag`
- `openpaste_storage_remove_tag`

**Search:**
- `openpaste_search_execute`
- `openpaste_search_get_suggestions`

**Event:**
- `openpaste_event_subscribe`
- `openpaste_event_publish`
- `openpaste_event_unsubscribe`

**UI:**
- `openpaste_ui_notify`
- `openpaste_ui_show_dialog`

**Settings:**
- `openpaste_settings_get`
- `openpaste_settings_set`

**Logging:**
- `openpaste_log`

## Plugin Versioning

### Semantic Versioning

Follow SemVer for plugin versions:
- MAJOR: Breaking changes
- MINOR: New features (backward compatible)
- PATCH: Bug fixes (backward compatible)

### Compatibility

Specify compatible OpenPaste versions in manifest:
```json
{
  "min_openpaste_version": "0.1.0",
  "max_openpaste_version": "1.0.0"
}
```

## Plugin Migration

### Migrating Between Versions

**Breaking Changes:**
- Update manifest version
- Document breaking changes
- Provide migration guide

**Backward Compatibility:**
- Maintain old API if possible
- Deprecate before removing
- Provide migration path
