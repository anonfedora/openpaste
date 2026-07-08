# OpenPaste Database Design

## Overview

OpenPaste uses SQLite as its primary database, leveraging FTS5 for full-text search and WAL mode for performance. The database stores clipboard items, metadata, collections, tags, and user settings.

## Database Location

### Platform-Specific Paths

**Windows:**
```
%APPDATA%\OpenPaste\openpaste.db
```

**Linux:**
```
~/.config/openpaste/openpaste.db
```

**macOS:**
```
~/Library/Application Support/OpenPaste/openpaste.db
```

### Development Mode

During development, the database is stored in the project root:
```
./openpaste-dev.db
```

## SQLite Configuration

### Pragma Settings

```sql
-- Enable WAL mode for better concurrency
PRAGMA journal_mode = WAL;

-- Set synchronous mode to NORMAL (balance between safety and performance)
PRAGMA synchronous = NORMAL;

-- Set cache size to -10MB (10MB cache)
PRAGMA cache_size = -10000;

-- Enable foreign key constraints
PRAGMA foreign_keys = ON;

-- Set temp store to memory (faster for temporary operations)
PRAGMA temp_store = MEMORY;

-- Set mmap size to 300MB (memory-mapped I/O for better performance)
PRAGMA mmap_size = 300000000;

-- Enable query optimizer
PRAGMA optimize;
```

### Connection Pool

- **Pool Size**: 5 connections
- **Timeout**: 30 seconds
- **Idle Timeout**: 5 minutes

## Schema

### clipboard_items

Stores all clipboard items.

```sql
CREATE TABLE clipboard_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    content_type TEXT NOT NULL,           -- 'text', 'image', 'html', 'file', 'binary'
    content BLOB NOT NULL,                 -- Actual content (may be compressed/encrypted)
    content_preview TEXT,                  -- Text preview for UI (first 200 chars)
    hash TEXT NOT NULL UNIQUE,             -- SHA-256 hash for duplicate detection
    size_bytes INTEGER NOT NULL,           -- Size of content in bytes
    source_app TEXT,                       -- Application that copied the content
    source_window TEXT,                    -- Window title where copied
    created_at INTEGER NOT NULL,           -- Unix timestamp (milliseconds)
    last_accessed_at INTEGER,              -- Unix timestamp (milliseconds)
    access_count INTEGER DEFAULT 0,        -- Number of times accessed
    pinned BOOLEAN DEFAULT 0,              -- Whether item is pinned
    favorite BOOLEAN DEFAULT 0,            -- Whether item is favorited
    is_encrypted BOOLEAN DEFAULT 0,        -- Whether content is encrypted
    encryption_nonce BLOB,                  -- Nonce for AES-GCM (if encrypted)
    collection_id INTEGER,                  -- Foreign key to collections
    metadata JSON,                         -- Additional metadata as JSON
    deleted_at INTEGER,                     -- Soft delete timestamp (NULL if not deleted)
    
    FOREIGN KEY (collection_id) REFERENCES collections(id) ON DELETE SET NULL
);
```

**Indexes:**
```sql
-- Index on created_at for time-based queries
CREATE INDEX idx_clipboard_items_created_at ON clipboard_items(created_at DESC);

-- Index on hash for duplicate detection
CREATE INDEX idx_clipboard_items_hash ON clipboard_items(hash);

-- Index on content_type for filtering
CREATE INDEX idx_clipboard_items_content_type ON clipboard_items(content_type);

-- Index on pinned for quick access to pinned items
CREATE INDEX idx_clipboard_items_pinned ON clipboard_items(pinned) WHERE pinned = 1;

-- Index on favorite for quick access to favorites
CREATE INDEX idx_clipboard_items_favorite ON clipboard_items(favorite) WHERE favorite = 1;

-- Index on access_count for frequently accessed items
CREATE INDEX idx_clipboard_items_access_count ON clipboard_items(access_count DESC);

-- Index on collection_id for collection queries
CREATE INDEX idx_clipboard_items_collection_id ON clipboard_items(collection_id);

-- Index on deleted_at for soft delete queries
CREATE INDEX idx_clipboard_items_deleted_at ON clipboard_items(deleted_at);
```

### collections

Organizes clipboard items into collections.

```sql
CREATE TABLE collections (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    color TEXT,                            -- Hex color code for UI
    icon TEXT,                             -- Icon identifier
    created_at INTEGER NOT NULL,           -- Unix timestamp (milliseconds)
    updated_at INTEGER NOT NULL,           -- Unix timestamp (milliseconds)
    is_system BOOLEAN DEFAULT 0,           -- Whether this is a system collection
    sort_order INTEGER DEFAULT 0           -- Display order
);
```

**Indexes:**
```sql
-- Index on name for lookups
CREATE INDEX idx_collections_name ON collections(name);

-- Index on sort_order for display
CREATE INDEX idx_collections_sort_order ON collections(sort_order);
```

### tags

Tags for categorizing clipboard items.

```sql
CREATE TABLE tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    color TEXT,                            -- Hex color code for UI
    created_at INTEGER NOT NULL            -- Unix timestamp (milliseconds)
);
```

**Indexes:**
```sql
-- Index on name for lookups
CREATE INDEX idx_tags_name ON tags(name);
```

### clipboard_item_tags

Many-to-many relationship between clipboard items and tags.

```sql
CREATE TABLE clipboard_item_tags (
    clipboard_item_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,
    created_at INTEGER NOT NULL,           -- Unix timestamp (milliseconds)
    
    PRIMARY KEY (clipboard_item_id, tag_id),
    FOREIGN KEY (clipboard_item_id) REFERENCES clipboard_items(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);
```

**Indexes:**
```sql
-- Index on tag_id for finding items by tag
CREATE INDEX idx_clipboard_item_tags_tag_id ON clipboard_item_tags(tag_id);

-- Index on clipboard_item_id for finding tags of an item
CREATE INDEX idx_clipboard_item_tags_clipboard_item_id ON clipboard_item_tags(clipboard_item_id);
```

### settings

Application settings.

```sql
CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    value_type TEXT NOT NULL,              -- 'string', 'integer', 'boolean', 'json'
    updated_at INTEGER NOT NULL            -- Unix timestamp (milliseconds)
);
```

### sync_state

Tracks synchronization state for cloud sync.

```sql
CREATE TABLE sync_state (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    sync_provider TEXT NOT NULL,           -- 'webdav', 's3', 'git', etc.
    last_sync_at INTEGER,                  -- Unix timestamp (milliseconds)
    last_sync_hash TEXT,                   -- Hash of last synced state
    sync_status TEXT NOT NULL,             -- 'idle', 'syncing', 'error'
    sync_error TEXT,                       -- Last error message
    config JSON NOT NULL                   -- Provider-specific config
);
```

### plugins

Installed plugins and their state.

```sql
CREATE TABLE plugins (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    version TEXT NOT NULL,
    enabled BOOLEAN DEFAULT 1,
    config JSON,                           -- Plugin-specific configuration
    installed_at INTEGER NOT NULL,         -- Unix timestamp (milliseconds)
    last_used_at INTEGER                   -- Unix timestamp (milliseconds)
);
```

### audit_log

Optional audit log for clipboard access events.

```sql
CREATE TABLE audit_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event_type TEXT NOT NULL,              -- 'access', 'copy', 'delete', 'export'
    clipboard_item_id INTEGER,
    source TEXT NOT NULL,                  -- 'daemon', 'desktop', 'cli', 'plugin'
    details JSON,                          -- Additional event details
    created_at INTEGER NOT NULL,           -- Unix timestamp (milliseconds)
    
    FOREIGN KEY (clipboard_item_id) REFERENCES clipboard_items(id) ON DELETE SET NULL
);
```

**Indexes:**
```sql
-- Index on created_at for time-based queries
CREATE INDEX idx_audit_log_created_at ON audit_log(created_at DESC);

-- Index on event_type for filtering
CREATE INDEX idx_audit_log_event_type ON audit_log(event_type);

-- Index on clipboard_item_id for item-specific audit trails
CREATE INDEX idx_audit_log_clipboard_item_id ON audit_log(clipboard_item_id);
```

## Full-Text Search (FTS5)

### clipboard_items_fts

Virtual table for full-text search of clipboard items.

```sql
CREATE VIRTUAL TABLE clipboard_items_fts USING fts5(
    content_preview,
    source_app,
    source_window,
    metadata,
    content="clipboard_items",
    content_rowid="id",
    tokenize="unicode61 remove_diacritics 1"
);
```

**Triggers:**

Keep FTS index in sync with main table.

```sql
-- Insert trigger
CREATE TRIGGER clipboard_items_fts_insert AFTER INSERT ON clipboard_items BEGIN
    INSERT INTO clipboard_items_fts(
        rowid,
        content_preview,
        source_app,
        source_window,
        metadata
    ) VALUES (
        NEW.id,
        NEW.content_preview,
        COALESCE(NEW.source_app, ''),
        COALESCE(NEW.source_window, ''),
        COALESCE(NEW.metadata, '{}')
    );
END;

-- Update trigger
CREATE TRIGGER clipboard_items_fts_update AFTER UPDATE ON clipboard_items BEGIN
    UPDATE clipboard_items_fts SET
        content_preview = NEW.content_preview,
        source_app = COALESCE(NEW.source_app, ''),
        source_window = COALESCE(NEW.source_window, ''),
        metadata = COALESCE(NEW.metadata, '{}')
    WHERE rowid = NEW.id;
END;

-- Delete trigger
CREATE TRIGGER clipboard_items_fts_delete AFTER DELETE ON clipboard_items BEGIN
    DELETE FROM clipboard_items_fts WHERE rowid = OLD.id;
END;
```

**FTS5 Configuration:**

```sql
-- Configure BM25 ranking
INSERT INTO clipboard_items_fts(clipboard_items_fts, rank) VALUES('rank', 'bm25(1.0, 0.75, 0.0, 0.0)');

-- Enable phrase queries
-- (default in FTS5)
```

## Migrations

### Migration Strategy

- **Versioned Migrations**: Numbered SQL files in `crates/clipboard-db/migrations/`
- **Applied Migrations**: Tracked in `schema_migrations` table
- **Rollback**: Supported for development, not production

### schema_migrations

Tracks applied migrations.

```sql
CREATE TABLE schema_migrations (
    version INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    applied_at INTEGER NOT NULL           -- Unix timestamp (milliseconds)
);
```

### Migration Files

#### 001_initial.sql

Initial schema creation.

```sql
-- Create all tables
-- (see schema definitions above)

-- Insert default collections
INSERT INTO collections (name, description, is_system, sort_order) VALUES
    ('All Items', 'All clipboard items', 1, 0),
    ('Pinned', 'Pinned clipboard items', 1, 1),
    ('Favorites', 'Favorite clipboard items', 1, 2);

-- Insert default settings
INSERT INTO settings (key, value, value_type, updated_at) VALUES
    ('max_items', '10000', 'integer', strftime('%s', 'now') * 1000),
    ('retention_days', '90', 'integer', strftime('%s', 'now') * 1000),
    ('encryption_enabled', 'false', 'boolean', strftime('%s', 'now') * 1000),
    ('auto_paste_enabled', 'true', 'boolean', strftime('%s', 'now') * 1000);

-- Record migration
INSERT INTO schema_migrations (version, name, applied_at) VALUES
    (1, 'initial', strftime('%s', 'now') * 1000);
```

#### 002_fts.sql

Add full-text search support.

```sql
-- Create FTS virtual table
CREATE VIRTUAL TABLE clipboard_items_fts USING fts5(
    content_preview,
    source_app,
    source_window,
    metadata,
    content="clipboard_items",
    content_rowid="id",
    tokenize="unicode61 remove_diacritics 1"
);

-- Create triggers
CREATE TRIGGER clipboard_items_fts_insert AFTER INSERT ON clipboard_items BEGIN
    INSERT INTO clipboard_items_fts(
        rowid,
        content_preview,
        source_app,
        source_window,
        metadata
    ) VALUES (
        NEW.id,
        NEW.content_preview,
        COALESCE(NEW.source_app, ''),
        COALESCE(NEW.source_window, ''),
        COALESCE(NEW.metadata, '{}')
    );
END;

CREATE TRIGGER clipboard_items_fts_update AFTER UPDATE ON clipboard_items BEGIN
    UPDATE clipboard_items_fts SET
        content_preview = NEW.content_preview,
        source_app = COALESCE(NEW.source_app, ''),
        source_window = COALESCE(NEW.source_window, ''),
        metadata = COALESCE(NEW.metadata, '{}')
    WHERE rowid = NEW.id;
END;

CREATE TRIGGER clipboard_items_fts_delete AFTER DELETE ON clipboard_items BEGIN
    DELETE FROM clipboard_items_fts WHERE rowid = OLD.id;
END;

-- Populate FTS with existing data
INSERT INTO clipboard_items_fts(rowid, content_preview, source_app, source_window, metadata)
SELECT id, content_preview, COALESCE(source_app, ''), COALESCE(source_window, ''), COALESCE(metadata, '{}')
FROM clipboard_items;

-- Record migration
INSERT INTO schema_migrations (version, name, applied_at) VALUES
    (2, 'fts', strftime('%s', 'now') * 1000);
```

#### 003_encryption.sql

Add encryption support.

```sql
-- Add encryption columns to clipboard_items
ALTER TABLE clipboard_items ADD COLUMN is_encrypted BOOLEAN DEFAULT 0;
ALTER TABLE clipboard_items ADD COLUMN encryption_nonce BLOB;

-- Add encryption setting
INSERT INTO settings (key, value, value_type, updated_at) VALUES
    ('encryption_key_hash', '', 'string', strftime('%s', 'now') * 1000);

-- Record migration
INSERT INTO schema_migrations (version, name, applied_at) VALUES
    (3, 'encryption', strftime('%s', 'now') * 1000);
```

#### 004_audit_log.sql

Add audit log support.

```sql
-- Create audit_log table
CREATE TABLE audit_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event_type TEXT NOT NULL,
    clipboard_item_id INTEGER,
    source TEXT NOT NULL,
    details JSON,
    created_at INTEGER NOT NULL,
    
    FOREIGN KEY (clipboard_item_id) REFERENCES clipboard_items(id) ON DELETE SET NULL
);

-- Create indexes
CREATE INDEX idx_audit_log_created_at ON audit_log(created_at DESC);
CREATE INDEX idx_audit_log_event_type ON audit_log(event_type);
CREATE INDEX idx_audit_log_clipboard_item_id ON audit_log(clipboard_item_id);

-- Add audit log setting
INSERT INTO settings (key, value, value_type, updated_at) VALUES
    ('audit_log_enabled', 'false', 'boolean', strftime('%s', 'now') * 1000);

-- Record migration
INSERT INTO schema_migrations (version, name, applied_at) VALUES
    (4, 'audit_log', strftime('%s', 'now') * 1000);
```

#### 005_sync_state.sql

Add sync state tracking.

```sql
-- Create sync_state table
CREATE TABLE sync_state (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    sync_provider TEXT NOT NULL,
    last_sync_at INTEGER,
    last_sync_hash TEXT,
    sync_status TEXT NOT NULL,
    sync_error TEXT,
    config JSON NOT NULL
);

-- Record migration
INSERT INTO schema_migrations (version, name, applied_at) VALUES
    (5, 'sync_state', strftime('%s', 'now') * 1000);
```

## Encryption

### Encryption at Rest

When encryption is enabled:
- `content` column is encrypted with AES-256-GCM
- `encryption_nonce` stores the nonce used for encryption
- `is_encrypted` flag indicates encrypted content
- `content_preview` is also encrypted (or stored separately)

### Key Management

- Master password derived using Argon2id
- Database encryption key derived from master password
- Key hash stored in settings for verification
- Keys never stored in plaintext

See [ENCRYPTION.md](ENCRYPTION.md) for detailed encryption design.

## Backup and Recovery

### Backup Strategy

**Automatic Backups:**
- Daily backup to `openpaste.db.backup`
- Keep last 7 backups
- Backup before schema migrations

**Manual Backup:**
```bash
# Copy database file
cp openpaste.db openpaste.db.manual-backup-$(date +%Y%m%d)
```

### Recovery

**From Backup:**
```bash
# Stop daemon
# Restore from backup
cp openpaste.db.backup openpaste.db
# Start daemon
```

**From WAL:**
```bash
# Checkpoint WAL to main database
sqlite3 openpaste.db "PRAGMA wal_checkpoint(TRUNCATE);"
```

### Corruption Recovery

**SQLite Integrity Check:**
```sql
PRAGMA integrity_check;
```

**Dump and Restore:**
```bash
# Dump database to SQL
sqlite3 openpaste.db .dump > openpaste-dump.sql

# Restore from dump
sqlite3 openpaste-new.db < openpaste-dump.sql
```

## Performance Optimization

### Query Optimization

**Use Indexes:**
- All WHERE clauses should use indexed columns
- Avoid LIKE on unindexed columns
- Use FTS for text search, not LIKE

**Batch Operations:**
- Use transactions for bulk inserts
- Batch delete operations

**Connection Pooling:**
- Reuse connections
- Set appropriate pool size

### Maintenance

**VAL Mode:**
- WAL mode enabled by default
- Periodic checkpoint: `PRAGMA wal_checkpoint(TRUNCATE);`

**Vacuum:**
- Run VACUUM monthly to reclaim space
- Or use auto_vacuum setting

**Analyze:**
- Run ANALYZE after bulk operations
- Updates query planner statistics

**Optimize:**
- Run PRAGMA optimize periodically
- Updates internal statistics

## Data Retention

### Retention Policy

Configurable via settings:
- `retention_days`: Default 90 days
- `max_items`: Default 10,000 items

### Cleanup Strategy

**Scheduled Cleanup:**
- Run daily at 3 AM
- Delete items older than retention_days
- Delete oldest items if exceeding max_items
- Soft delete (set deleted_at timestamp)

**Hard Delete:**
- Permanently delete soft-deleted items after 30 days
- Run weekly

### Manual Cleanup

**By Date:**
```sql
DELETE FROM clipboard_items
WHERE created_at < ? AND deleted_at IS NOT NULL;
```

**By Count:**
```sql
DELETE FROM clipboard_items
WHERE id IN (
    SELECT id FROM clipboard_items
    ORDER BY created_at ASC
    LIMIT ?
);
```

## Database Size Estimates

### Per Item

- **Text (1KB)**: ~2KB (with metadata and indexes)
- **Image (100KB)**: ~150KB (with thumbnail and metadata)
- **File reference**: ~5KB (path and metadata)

### Total Estimates

- **1,000 text items**: ~2MB
- **10,000 text items**: ~20MB
- **1,000 images**: ~150MB
- **Mixed (5,000 items)**: ~50MB

### WAL File

- Typically 10-20% of main database size
- Checkpointed periodically

## Query Examples

### Insert Clipboard Item

```sql
INSERT INTO clipboard_items (
    content_type,
    content,
    content_preview,
    hash,
    size_bytes,
    source_app,
    source_window,
    created_at
) VALUES (?, ?, ?, ?, ?, ?, ?, ?);
```

### Search by Content

```sql
SELECT ci.id, ci.content_preview, ci.created_at, ci.source_app
FROM clipboard_items ci
JOIN clipboard_items_fts fts ON ci.id = fts.rowid
WHERE clipboard_items_fts MATCH ?
AND ci.deleted_at IS NULL
ORDER BY ci.created_at DESC
LIMIT 50;
```

### Get Recent Items

```sql
SELECT id, content_preview, created_at, source_app
FROM clipboard_items
WHERE deleted_at IS NULL
ORDER BY created_at DESC
LIMIT 50;
```

### Get Pinned Items

```sql
SELECT id, content_preview, created_at
FROM clipboard_items
WHERE pinned = 1 AND deleted_at IS NULL
ORDER BY created_at DESC;
```

### Get Items by Collection

```sql
SELECT id, content_preview, created_at
FROM clipboard_items
WHERE collection_id = ? AND deleted_at IS NULL
ORDER BY created_at DESC;
```

### Get Items by Tag

```sql
SELECT ci.id, ci.content_preview, ci.created_at
FROM clipboard_items ci
JOIN clipboard_item_tags cit ON ci.id = cit.clipboard_item_id
JOIN tags t ON cit.tag_id = t.id
WHERE t.name = ? AND ci.deleted_at IS NULL
ORDER BY ci.created_at DESC;
```

### Update Access Count

```sql
UPDATE clipboard_items
SET access_count = access_count + 1,
    last_accessed_at = ?
WHERE id = ?;
```

### Soft Delete

```sql
UPDATE clipboard_items
SET deleted_at = ?
WHERE id = ?;
```

### Hard Delete Old Items

```sql
DELETE FROM clipboard_items
WHERE deleted_at IS NOT NULL
AND deleted_at < ?;
```

## Security Considerations

### SQL Injection Prevention

- Always use parameterized queries
- Never concatenate user input into SQL
- Use rusqlite's prepared statements

### Access Control

- Database file permissions: 600 (user read/write only)
- Directory permissions: 700 (user access only)
- Daemon runs as user, not root

### Sensitive Data

- Never log clipboard content
- Never log encryption keys
- Sanitize error messages before logging

## Testing

### Test Database

Use separate test database:
```
./openpaste-test.db
```

### Test Fixtures

- Sample clipboard items
- Sample collections
- Sample tags
- Test migrations

### Test Coverage

- All CRUD operations
- All indexes
- All triggers
- All migrations
- FTS search
- Encryption operations
