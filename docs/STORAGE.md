# OpenPaste Storage Design

## Overview

OpenPaste stores clipboard items efficiently using compression for text, thumbnails for images, and references for files. Storage is designed for low memory usage and fast retrieval.

## Storage Architecture

```
Clipboard Item
    │
    ├── Content Type Detection
    │
    ├── Content Normalization
    │
    ├── Compression (if text)
    │
    ├── Thumbnail Generation (if image)
    │
    ├── Encryption (if enabled)
    │
    ├── Database Storage
    │
    └── File Storage (large items)
```

## Content Types

### Supported Content Types

- **text**: Plain text
- **html**: HTML content
- **image**: Images (PNG, JPEG, GIF, BMP, WebP)
- **file**: File references (paths)
- **binary**: Arbitrary binary data
- **rtf**: Rich Text Format
- **code**: Source code with syntax highlighting info

### Content Type Detection

**Automatic Detection:**
```rust
fn detect_content_type(data: &[u8], format_hint: Option<&str>) -> ContentType {
    match format_hint {
        Some("text/plain") => ContentType::Text,
        Some("text/html") => ContentType::Html,
        Some("image/png") => ContentType::Image,
        Some("image/jpeg") => ContentType::Image,
        Some("file/uri") => ContentType::File,
        _ => detect_from_bytes(data),
    }
}
```

**Byte Pattern Detection:**
- PNG: Magic bytes `89 50 4E 47`
- JPEG: Magic bytes `FF D8 FF`
- GIF: Magic bytes `47 49 46 38`
- HTML: Starts with `<!DOCTYPE` or `<html`
- Text: Valid UTF-8, no control characters

## Text Storage

### Compression Strategy

**Algorithm:** Zstandard (zstd)

**Compression Levels:**
- **Default:** Level 3 (balance speed/ratio)
- **Background:** Level 5 (better ratio)
- **Real-time:** Level 1 (fastest)

**Threshold:** Compress if > 512 bytes

**Implementation:**
```rust
fn compress_text(text: &str) -> Result<Vec<u8>, StorageError> {
    if text.len() <= 512 {
        return Ok(text.as_bytes().to_vec());
    }
    
    let encoder = zstd::encode::Encoder::new(Vec::new(), 3)?;
    encoder.write_all(text.as_bytes())?;
    Ok(encoder.finish()?)
}
```

**Decompression:**
```rust
fn decompress_text(data: &[u8]) -> Result<String, StorageError> {
    // Try to decompress, if fails assume uncompressed
    match zstd::decode_all(data) {
        Ok(decompressed) => Ok(String::from_utf8(decompressed)?),
        Err(_) => Ok(String::from_utf8(data.to_vec())?),
    }
}
```

### Text Preview

**Purpose:** Quick preview without decompression

**Length:** First 200 characters

**Storage:** Stored separately in `content_preview` column

**Implementation:**
```rust
fn generate_preview(text: &str) -> String {
    text.chars().take(200).collect()
}
```

### Code Storage

**Metadata:**
- Language detection (using `tree-sitter` or heuristic)
- Line count
- Indentation type (tabs/spaces)

**Storage:**
- Compressed code in `content` column
- Language in `metadata` JSON
- Syntax highlighting info in `metadata`

## Image Storage

### Image Processing

**Supported Formats:**
- PNG (lossless, preferred for screenshots)
- JPEG (lossy, for photos)
- GIF (animated)
- BMP (legacy)
- WebP (modern, efficient)

### Thumbnail Generation

**Purpose:** Fast preview without loading full image

**Thumbnail Sizes:**
- **Small:** 64x64 pixels (list view)
- **Medium:** 256x256 pixels (grid view)
- **Large:** 512x512 pixels (detail view)

**Storage:** Stored as separate BLOB in database or file system

**Implementation:**
```rust
fn generate_thumbnail(image_data: &[u8], size: u32) -> Result<Vec<u8>, StorageError> {
    let img = image::load_from_memory(image_data)?;
    let thumbnail = img.thumbnail(size, size);
    let mut buffer = Vec::new();
    thumbnail.write_to(&mut std::io::Cursor::new(&mut buffer), image::ImageFormat::Png)?;
    Ok(buffer)
}
```

### Full Image Storage

**Strategy:**
- **Small images (< 1MB):** Store in database BLOB
- **Large images (≥ 1MB):** Store in file system

**File System Storage:**
```
~/.local/share/openpaste/images/
├── abc123def456.png
├── 789ghi012jkl.png
└── ...
```

**Database Reference:**
```json
{
  "storage": "file",
  "path": "/home/user/.local/share/openpaste/images/abc123def456.png",
  "size": 2048576,
  "format": "png"
}
```

### Image Metadata

**Extracted Metadata:**
- Width and height
- Format (PNG, JPEG, etc.)
- File size
- Color space
- EXIF data (if available)

**Storage:** In `metadata` JSON column

## HTML Storage

### HTML Processing

**Sanitization:**
- Remove scripts
- Remove dangerous tags
- Remove event handlers
- Keep safe HTML for formatting

**Implementation:** Use `ammonia` or similar library

**Storage:**
- Sanitized HTML in `content` column
- Plain text preview in `content_preview`
- Original URL (if from browser) in `metadata`

### HTML Preview

**Strategy:** Extract text content for preview

**Implementation:**
```rust
fn html_to_text_preview(html: &str) -> String {
    let html = ammonia::clean(html);
    let document = scraper::Html::parse_document(&html);
    document.root().text().collect::<Vec<_>>().join(" ")
}
```

## File Reference Storage

### File Detection

**Detection Methods:**
- File list from clipboard (Windows/Linux)
- File path string
- URI scheme (file://)

### File Metadata

**Extracted Metadata:**
- File path
- File name
- File size
- File type (extension)
- File modification time

**Storage:** In `metadata` JSON column

**Example:**
```json
{
  "files": [
    {
      "path": "/home/user/document.pdf",
      "name": "document.pdf",
      "size": 1048576,
      "type": "application/pdf",
      "modified": 1704067200000
    }
  ]
}
```

### File Content

**Strategy:** Store references only, not file content

**Reasons:**
- Privacy (don't copy user files)
- Storage efficiency
- Avoid permission issues

**Future Enhancement:** Optional file content copying

## Binary Data Storage

### Binary Detection

**Detection:** Content doesn't match known types

### Storage Strategy

**Small binaries (< 100KB):** Store in database BLOB

**Large binaries (≥ 100KB):** Store in file system

**File System Storage:**
```
~/.local/share/openpaste/binary/
├── abc123def456.bin
└── ...
```

### Binary Metadata

**Extracted Metadata:**
- Size
- Magic bytes (for type hint)
- SHA-256 hash

**Storage:** In `metadata` JSON column

## Compression Configuration

### Compression Settings

**User Configurable:**
```json
{
  "storage": {
    "compression": {
      "enabled": true,
      "level": 3,
      "threshold_bytes": 512
    }
  }
}
```

### Compression by Type

| Content Type | Compress | Threshold |
|-------------|----------|-----------|
| text | Yes | 512 bytes |
| html | Yes | 512 bytes |
| code | Yes | 512 bytes |
| image | No | N/A |
| file | No | N/A |
| binary | Yes | 1KB |

## File System Storage

### Directory Structure

**Platform-Specific Paths:**

**Linux:**
```
~/.local/share/openpaste/
├── images/
├── binary/
└── data/
```

**Windows:**
```
%LOCALAPPDATA%\OpenPaste\
├── images\
├── binary\
└── data\
```

**macOS:**
```
~/Library/Application Support/OpenPaste/
├── images/
├── binary/
└── data/
```

### File Naming

**Strategy:** Use hash-based filenames

**Format:** `{hash}.{extension}`

**Benefits:**
- No collisions
- Content-addressable
- Easy deduplication

**Implementation:**
```rust
fn storage_filename(hash: &str, extension: &str) -> String {
    format!("{}.{}", hash, extension)
}
```

### File Cleanup

**Strategy:** Clean up orphaned files

**Trigger:** On startup and periodically

**Implementation:**
```rust
fn cleanup_orphaned_files() {
    let db_files = get_all_file_references_from_db();
    let fs_files = list_all_files_in_storage();
    
    for file in fs_files {
        if !db_files.contains(&file) {
            delete_file(file);
        }
    }
}
```

## Metadata Storage

### Metadata Schema

**Common Metadata:**
```json
{
  "source": {
    "app": "Chrome",
    "window": "GitHub - Pull Request #123",
    "url": "https://github.com/..."
  },
  "extracted": {
    "language": "rust",
    "line_count": 42,
    "has_urls": true,
    "urls": ["https://example.com"]
  },
  "storage": {
    "compressed": true,
    "original_size": 1024,
    "compressed_size": 256,
    "compression_ratio": 0.25
  }
}
```

### Metadata Extraction

**Text Metadata:**
- Word count
- Line count
- URL detection
- Email detection
- Language detection

**Image Metadata:**
- Dimensions
- Format
- Color space
- EXIF data

**Code Metadata:**
- Language
- Line count
- Function count
- Import count

## Storage Performance

### Performance Targets

- **Store Text:** < 10ms for 1KB text
- **Store Image:** < 50ms for 1MB image
- **Retrieve Text:** < 5ms for 1KB text
- **Retrieve Image:** < 20ms for 1MB image
- **Compress Text:** < 5ms for 10KB text

### Optimization Strategies

**Lazy Loading:**
- Load content on demand
- Load thumbnails first
- Load full content when needed

**Caching:**
- Cache frequently accessed items
- Cache thumbnails
- LRU eviction policy

**Batch Operations:**
- Batch database writes
- Batch file operations
- Use transactions

**Async Operations:**
- Async compression
- Async thumbnail generation
- Async file I/O

## Storage Limits

### Configurable Limits

**Per Item:**
- Max text size: 10MB
- Max image size: 50MB
- Max binary size: 100MB

**Total Storage:**
- Max database size: 1GB
- Max file storage: 10GB

**Retention:**
- Max items: 10,000
- Max age: 90 days

### Limit Enforcement

**On Store:**
```rust
fn check_storage_limits(item: &ClipboardItem) -> Result<(), StorageError> {
    if item.size_bytes > MAX_ITEM_SIZE {
        return Err(StorageError::ItemTooLarge);
    }
    
    let total_items = get_total_item_count()?;
    if total_items >= MAX_ITEMS {
        return Err(StorageError::TooManyItems);
    }
    
    Ok(())
}
```

**On Exceed:**
- Delete oldest items
- Notify user
- Log event

## Storage Encryption

### Encryption Scope

**Encrypted:**
- Database content column (if enabled)
- File system files (if enabled)

**Not Encrypted:**
- Metadata
- Indexes
- Thumbnails (optional)

### Encryption Integration

Encryption handled by `clipboard-encryption` crate.

See [ENCRYPTION.md](ENCRYPTION.md) for details.

## Storage API

### Store Item

```rust
pub async fn store_item(item: ClipboardItem) -> Result<i64, StorageError> {
    // Detect content type
    let content_type = detect_content_type(&item.content, item.format_hint.as_deref());
    
    // Generate preview
    let preview = generate_preview(&item.content, &content_type);
    
    // Compress if needed
    let content = compress_if_needed(&item.content, &content_type)?;
    
    // Generate thumbnail if image
    let thumbnail = generate_thumbnail_if_image(&content, &content_type)?;
    
    // Encrypt if enabled
    let (content, nonce) = encrypt_if_enabled(&content)?;
    
    // Store in database
    let id = db::insert_clipboard_item(
        content_type,
        content,
        preview,
        item.hash,
        item.size_bytes,
        item.source_app,
        item.source_window,
        nonce,
    ).await?;
    
    // Store large files in file system
    if content.size_bytes > FILE_STORAGE_THRESHOLD {
        store_in_filesystem(id, &content)?;
    }
    
    Ok(id)
}
```

### Retrieve Item

```rust
pub async fn retrieve_item(id: i64) -> Result<ClipboardItem, StorageError> {
    // Get from database
    let item = db::get_clipboard_item(id).await?;
    
    // Load from file system if needed
    if item.storage == StorageType::File {
        item.content = load_from_filesystem(&item.path)?;
    }
    
    // Decrypt if needed
    let content = decrypt_if_needed(&item.content, &item.nonce)?;
    
    // Decompress if needed
    let content = decompress_if_needed(&content)?;
    
    Ok(ClipboardItem { content, ..item })
}
```

### Delete Item

```rust
pub async fn delete_item(id: i64) -> Result<(), StorageError> {
    // Get item
    let item = db::get_clipboard_item(id).await?;
    
    // Delete from file system if needed
    if item.storage == StorageType::File {
        delete_from_filesystem(&item.path)?;
    }
    
    // Soft delete from database
    db::soft_delete_clipboard_item(id).await?;
    
    Ok(())
}
```

## Storage Monitoring

### Metrics

**Storage Size:** Total bytes used

**Item Count:** Total items stored

**Compression Ratio:** Average compression ratio

**Storage Distribution:** By content type

**File System Usage:** Disk space used

### Alerts

**Low Disk Space:** Alert if < 1GB free

**High Storage Usage:** Alert if > 80% of limit

**Storage Growth:** Alert if rapid growth detected

## Storage Testing

### Unit Tests

- Compression/decompression
- Thumbnail generation
- Metadata extraction
- Content type detection

### Integration Tests

- End-to-end storage pipeline
- File system operations
- Database storage
- Encryption integration

### Performance Tests

- Storage latency benchmarks
- Retrieval latency benchmarks
- Compression benchmarks
- Large file handling

## Storage Migration

### Schema Changes

**Strategy:** Use database migrations

**Data Migration:**
- Migrate old format to new format
- Backward compatibility where possible
- Migration progress tracking

### File System Changes

**Strategy:** Migrate on startup

**Implementation:**
- Detect old structure
- Migrate to new structure
- Clean up old files
- Update database references

## Storage Best Practices

### Error Handling

**Disk Full:**
- Return clear error
- Suggest cleanup
- Prevent data loss

**Corruption:**
- Detect corruption
- Recover from backup
- Log error

**Permission Issues:**
- Check permissions on startup
- Request permissions if needed
- Graceful degradation

### Data Integrity

**Hash Verification:**
- Verify hash on retrieval
- Detect corruption
- Log mismatches

**Atomic Operations:**
- Use transactions
- Atomic file writes
- Rollback on error

### Backup

**Database Backup:**
- Regular backups
- Before migrations
- On user request

**File System Backup:**
- Optional file backup
- User-configurable
- Manual trigger

## Future Enhancements

### Deduplication

**Strategy:** Content-addressable storage

**Implementation:**
- Hash-based storage
- Reference counting
- Automatic cleanup

### Tiered Storage

**Hot Storage:** Frequently accessed items (in memory)

**Warm Storage:** Recently accessed items (SSD)

**Cold Storage:** Old items (slower storage)

### Cloud Storage

**Optional:** Store large items in cloud

**Providers:**
- S3
- WebDAV
- Custom

**Benefits:**
- Reduce local storage
- Sync across devices
- Backup

### Compression Improvements

**Better Algorithms:**
- LZ4 for speed
- Brotli for ratio
- Adaptive compression

**Machine Learning:**
- Learn optimal compression per type
- Predict compression ratio
