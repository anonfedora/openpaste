-- Initial database schema for OpenPaste

-- Clipboard items table
CREATE TABLE IF NOT EXISTS clipboard_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    content_type TEXT NOT NULL,
    content BLOB NOT NULL,
    hash TEXT NOT NULL UNIQUE,
    created_at TEXT NOT NULL,
    accessed_at TEXT,
    pinned INTEGER NOT NULL DEFAULT 0,
    favorite INTEGER NOT NULL DEFAULT 0
);

-- FTS5 virtual table for full-text search
CREATE VIRTUAL TABLE IF NOT EXISTS clipboard_items_fts USING fts5(
    content,
    content_type,
    tokenize='porter unicode61'
);

-- Triggers to keep FTS5 table in sync with main table
CREATE TRIGGER IF NOT EXISTS clipboard_items_fts_insert AFTER INSERT ON clipboard_items BEGIN
    INSERT INTO clipboard_items_fts(rowid, content, content_type)
    VALUES (NEW.id, NEW.content, NEW.content_type);
END;

CREATE TRIGGER IF NOT EXISTS clipboard_items_fts_delete AFTER DELETE ON clipboard_items BEGIN
    DELETE FROM clipboard_items_fts WHERE rowid = OLD.id;
END;

CREATE TRIGGER IF NOT EXISTS clipboard_items_fts_update AFTER UPDATE ON clipboard_items BEGIN
    UPDATE clipboard_items_fts SET content = NEW.content, content_type = NEW.content_type
    WHERE rowid = NEW.id;
END;

-- Collections table
CREATE TABLE IF NOT EXISTS collections (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Tags table
CREATE TABLE IF NOT EXISTS tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    color TEXT
);

-- Clipboard item tags junction table
CREATE TABLE IF NOT EXISTS clipboard_item_tags (
    clipboard_item_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,
    PRIMARY KEY (clipboard_item_id, tag_id),
    FOREIGN KEY (clipboard_item_id) REFERENCES clipboard_items(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);

-- Settings table
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Sync state table
CREATE TABLE IF NOT EXISTS sync_state (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Plugins table
CREATE TABLE IF NOT EXISTS plugins (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    version TEXT NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 0,
    installed_at TEXT NOT NULL,
    config TEXT
);

-- Audit log table
CREATE TABLE IF NOT EXISTS audit_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    action TEXT NOT NULL,
    entity_type TEXT NOT NULL,
    entity_id TEXT,
    details TEXT,
    created_at TEXT NOT NULL
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_clipboard_items_created_at ON clipboard_items(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_clipboard_items_pinned ON clipboard_items(pinned);
CREATE INDEX IF NOT EXISTS idx_clipboard_items_favorite ON clipboard_items(favorite);
