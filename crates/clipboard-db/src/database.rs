//! Database connection and operations

use crate::{models::ClipboardItem, DatabaseError};
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::path::Path;

/// Database connection pool
pub struct Database {
    #[allow(dead_code)]
    pool: SqlitePool,
}

impl Database {
    /// Create a new database connection
    pub async fn new(path: &Path) -> Result<Self, DatabaseError> {
        let db_url = format!("sqlite:{}?mode=rwc", path.display());

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&db_url)
            .await
            .map_err(|e| DatabaseError::ConnectionFailed(e.to_string()))?;

        // Run migrations
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .map_err(|e| DatabaseError::MigrationFailed(e.to_string()))?;

        Ok(Self { pool })
    }

    /// Create an in-memory database for testing
    pub async fn in_memory() -> Result<Self, DatabaseError> {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect("sqlite::memory:")
            .await
            .map_err(|e| DatabaseError::ConnectionFailed(e.to_string()))?;

        // Run migrations
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .map_err(|e| DatabaseError::MigrationFailed(e.to_string()))?;

        Ok(Self { pool })
    }

    /// Insert a clipboard item
    pub async fn insert_item(&self, item: &ClipboardItem) -> Result<i64, DatabaseError> {
        let result = sqlx::query(
            r#"
            INSERT INTO clipboard_items (content_type, content, hash, created_at, accessed_at, pinned, favorite, nonce, encrypted)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            "#,
        )
        .bind(&item.content_type)
        .bind(&item.content)
        .bind(&item.hash)
        .bind(item.created_at)
        .bind(item.accessed_at)
        .bind(item.pinned)
        .bind(item.favorite)
        .bind(&item.nonce)
        .bind(item.encrypted)
        .execute(&self.pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(result.last_insert_rowid())
    }

    /// Get a clipboard item by ID
    pub async fn get_item(&self, id: i64) -> Result<ClipboardItem, DatabaseError> {
        let item = sqlx::query_as::<_, ClipboardItem>(
            "SELECT id, content_type, content, hash, created_at, accessed_at, pinned, favorite, nonce, encrypted FROM clipboard_items WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?
        .ok_or_else(|| DatabaseError::NotFound(id.to_string()))?;

        Ok(item)
    }

    /// List clipboard items
    pub async fn list_items(
        &self,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<ClipboardItem>, DatabaseError> {
        let items = sqlx::query_as::<_, ClipboardItem>(
            "SELECT id, content_type, content, hash, created_at, accessed_at, pinned, favorite, nonce, encrypted FROM clipboard_items ORDER BY created_at DESC LIMIT ? OFFSET ?"
        )
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(items)
    }

    /// Delete a clipboard item
    pub async fn delete_item(&self, id: i64) -> Result<(), DatabaseError> {
        sqlx::query("DELETE FROM clipboard_items WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(())
    }

    /// Toggle pin status of a clipboard item
    pub async fn toggle_pin(&self, id: i64) -> Result<bool, DatabaseError> {
        let result = sqlx::query("UPDATE clipboard_items SET pinned = NOT pinned WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(result.rows_affected() > 0)
    }

    /// Toggle favorite status of a clipboard item
    pub async fn toggle_favorite(&self, id: i64) -> Result<bool, DatabaseError> {
        let result = sqlx::query("UPDATE clipboard_items SET favorite = NOT favorite WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(result.rows_affected() > 0)
    }

    /// Search clipboard items using FTS5
    pub async fn search_items(
        &self,
        query: &str,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<ClipboardItem>, DatabaseError> {
        let items = sqlx::query_as::<_, ClipboardItem>(
            r#"
            SELECT ci.id, ci.content_type, ci.content, ci.hash, ci.created_at, ci.accessed_at, ci.pinned, ci.favorite, ci.nonce, ci.encrypted
            FROM clipboard_items ci
            INNER JOIN clipboard_items_fts fts ON ci.id = fts.rowid
            WHERE clipboard_items_fts MATCH ?
            ORDER BY ci.created_at DESC
            LIMIT ? OFFSET ?
            "#
        )
        .bind(query)
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(items)
    }

    /// Get a setting value by key
    pub async fn get_setting(&self, key: &str) -> Result<Option<String>, DatabaseError> {
        let result = sqlx::query_as::<_, (String,)>(
            "SELECT value FROM settings WHERE key = ?"
        )
        .bind(key)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(result.map(|r| r.0))
    }

    /// Upsert a setting value
    pub async fn upsert_setting(&self, key: &str, value: &str) -> Result<(), DatabaseError> {
        sqlx::query(
            r#"
            INSERT INTO settings (key, value, value_type, updated_at)
            VALUES (?1, ?2, 'string', strftime('%s', 'now') * 1000)
            ON CONFLICT(key) DO UPDATE SET
                value = ?2,
                updated_at = strftime('%s', 'now') * 1000
            "#
        )
        .bind(key)
        .bind(value)
        .execute(&self.pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(())
    }

    /// Delete items older than retention days
    pub async fn cleanup_old_items(&self, retention_days: i32) -> Result<u64, DatabaseError> {
        let cutoff_time = chrono::Utc::now() - chrono::Duration::days(retention_days as i64);
        let cutoff_str = cutoff_time.to_rfc3339();

        let result = sqlx::query(
            "DELETE FROM clipboard_items WHERE created_at < ?"
        )
        .bind(cutoff_str)
        .execute(&self.pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(result.rows_affected())
    }

    /// Enforce max items limit by deleting oldest items
    pub async fn enforce_max_items(&self, max_items: i32) -> Result<u64, DatabaseError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM clipboard_items")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        let max_items_i64 = max_items as i64;

        if count <= max_items_i64 {
            return Ok(0);
        }

        let to_delete = count - max_items_i64;

        let result = sqlx::query(
            r#"
            DELETE FROM clipboard_items
            WHERE id IN (
                SELECT id FROM clipboard_items
                ORDER BY created_at ASC
                LIMIT ?
            )
            "#
        )
        .bind(to_delete)
        .execute(&self.pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(result.rows_affected())
    }

    // ── Tag operations ────────────────────────────────────────────────────────

    /// List all tags
    pub async fn list_tags(&self) -> Result<Vec<crate::models::Tag>, DatabaseError> {
        let tags = sqlx::query_as::<_, crate::models::Tag>(
            "SELECT id, name, color FROM tags ORDER BY name ASC"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;
        Ok(tags)
    }

    /// Create a tag (upsert by name)
    pub async fn create_tag(&self, name: &str, color: Option<&str>) -> Result<i64, DatabaseError> {
        let result = sqlx::query(
            "INSERT INTO tags (name, color) VALUES (?1, ?2) ON CONFLICT(name) DO UPDATE SET color = ?2"
        )
        .bind(name)
        .bind(color)
        .execute(&self.pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;
        Ok(result.last_insert_rowid())
    }

    /// Delete a tag and all its assignments
    pub async fn delete_tag(&self, id: i64) -> Result<(), DatabaseError> {
        sqlx::query("DELETE FROM tags WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;
        Ok(())
    }

    /// Get tags assigned to a clipboard item
    pub async fn get_item_tags(&self, item_id: i64) -> Result<Vec<crate::models::Tag>, DatabaseError> {
        let tags = sqlx::query_as::<_, crate::models::Tag>(
            r#"
            SELECT t.id, t.name, t.color
            FROM tags t
            INNER JOIN clipboard_item_tags cit ON t.id = cit.tag_id
            WHERE cit.clipboard_item_id = ?
            ORDER BY t.name ASC
            "#
        )
        .bind(item_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;
        Ok(tags)
    }

    /// Add a tag to a clipboard item (no-op if already assigned)
    pub async fn add_tag_to_item(&self, item_id: i64, tag_id: i64) -> Result<(), DatabaseError> {
        sqlx::query(
            "INSERT OR IGNORE INTO clipboard_item_tags (clipboard_item_id, tag_id) VALUES (?1, ?2)"
        )
        .bind(item_id)
        .bind(tag_id)
        .execute(&self.pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;
        Ok(())
    }

    /// Remove a tag from a clipboard item
    pub async fn remove_tag_from_item(&self, item_id: i64, tag_id: i64) -> Result<(), DatabaseError> {
        sqlx::query(
            "DELETE FROM clipboard_item_tags WHERE clipboard_item_id = ?1 AND tag_id = ?2"
        )
        .bind(item_id)
        .bind(tag_id)
        .execute(&self.pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;
        Ok(())
    }

    /// List clipboard items that have a given tag
    pub async fn list_items_by_tag(
        &self,
        tag_id: i64,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<ClipboardItem>, DatabaseError> {
        let items = sqlx::query_as::<_, ClipboardItem>(
            r#"
            SELECT ci.id, ci.content_type, ci.content, ci.hash, ci.created_at,
                   ci.accessed_at, ci.pinned, ci.favorite, ci.nonce, ci.encrypted
            FROM clipboard_items ci
            INNER JOIN clipboard_item_tags cit ON ci.id = cit.clipboard_item_id
            WHERE cit.tag_id = ?
            ORDER BY ci.created_at DESC
            LIMIT ? OFFSET ?
            "#
        )
        .bind(tag_id)
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;
        Ok(items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ClipboardItem;
    use chrono::Utc;

    fn make_item(content: &str, content_type: &str) -> ClipboardItem {
        let content_bytes = content.as_bytes().to_vec();
        // Simple hash: hex of a basic checksum, sufficient for test uniqueness
        let hash = format!("{:016x}", content_bytes.iter().fold(0u64, |acc, &b| acc.wrapping_mul(31).wrapping_add(b as u64)));
        ClipboardItem {
            id: 0,
            content_type: content_type.to_string(),
            content: content_bytes,
            hash,
            created_at: Utc::now(),
            accessed_at: None,
            pinned: false,
            favorite: false,
            nonce: None,
            encrypted: false,
        }
    }

    #[tokio::test]
    async fn test_insert_and_list() {
        let db = Database::in_memory().await.unwrap();
        let item = make_item("hello world", "text");
        let id = db.insert_item(&item).await.unwrap();
        assert!(id > 0);

        let items = db.list_items(10, 0).await.unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].content, b"hello world");
        assert_eq!(items[0].content_type, "text");
    }

    #[tokio::test]
    async fn test_get_item() {
        let db = Database::in_memory().await.unwrap();
        let id = db.insert_item(&make_item("get me", "text")).await.unwrap();
        let item = db.get_item(id).await.unwrap();
        assert_eq!(item.id, id);
        assert_eq!(item.content, b"get me");
    }

    #[tokio::test]
    async fn test_delete_item() {
        let db = Database::in_memory().await.unwrap();
        let id = db.insert_item(&make_item("delete me", "text")).await.unwrap();
        db.delete_item(id).await.unwrap();
        let items = db.list_items(10, 0).await.unwrap();
        assert!(items.iter().all(|i| i.id != id));
    }

    #[tokio::test]
    async fn test_toggle_pin_and_favorite() {
        let db = Database::in_memory().await.unwrap();
        let id = db.insert_item(&make_item("pin me", "text")).await.unwrap();

        db.toggle_pin(id).await.unwrap();
        let item = db.get_item(id).await.unwrap();
        assert!(item.pinned);

        db.toggle_pin(id).await.unwrap();
        let item = db.get_item(id).await.unwrap();
        assert!(!item.pinned);

        db.toggle_favorite(id).await.unwrap();
        let item = db.get_item(id).await.unwrap();
        assert!(item.favorite);
    }

    #[tokio::test]
    async fn test_duplicate_hash_rejected() {
        let db = Database::in_memory().await.unwrap();
        let item = make_item("unique content", "text");
        db.insert_item(&item).await.unwrap();
        // Second insert with same hash should fail with UNIQUE constraint
        let result = db.insert_item(&item).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("UNIQUE"));
    }

    #[tokio::test]
    async fn test_settings_upsert_and_get() {
        let db = Database::in_memory().await.unwrap();
        db.upsert_setting("test_key", "value1").await.unwrap();
        let val = db.get_setting("test_key").await.unwrap();
        assert_eq!(val, Some("value1".to_string()));

        // Upsert again — should overwrite
        db.upsert_setting("test_key", "value2").await.unwrap();
        let val = db.get_setting("test_key").await.unwrap();
        assert_eq!(val, Some("value2".to_string()));
    }

    #[tokio::test]
    async fn test_get_missing_setting_returns_none() {
        let db = Database::in_memory().await.unwrap();
        let val = db.get_setting("nonexistent").await.unwrap();
        assert_eq!(val, None);
    }

    #[tokio::test]
    async fn test_tag_crud() {
        let db = Database::in_memory().await.unwrap();

        // Create tag
        db.create_tag("work", Some("#ff0000")).await.unwrap();
        let tags = db.list_tags().await.unwrap();
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].name, "work");
        assert_eq!(tags[0].color.as_deref(), Some("#ff0000"));

        // Upsert same name — should not duplicate
        db.create_tag("work", Some("#00ff00")).await.unwrap();
        let tags = db.list_tags().await.unwrap();
        assert_eq!(tags.len(), 1);

        // Delete
        let tag_id = tags[0].id;
        db.delete_tag(tag_id).await.unwrap();
        let tags = db.list_tags().await.unwrap();
        assert!(tags.is_empty());
    }

    #[tokio::test]
    async fn test_item_tags() {
        let db = Database::in_memory().await.unwrap();
        let item_id = db.insert_item(&make_item("tagged item", "text")).await.unwrap();

        db.create_tag("personal", None).await.unwrap();
        let tags = db.list_tags().await.unwrap();
        let tag_id = tags[0].id;

        db.add_tag_to_item(item_id, tag_id).await.unwrap();
        let item_tags = db.get_item_tags(item_id).await.unwrap();
        assert_eq!(item_tags.len(), 1);
        assert_eq!(item_tags[0].name, "personal");

        db.remove_tag_from_item(item_id, tag_id).await.unwrap();
        let item_tags = db.get_item_tags(item_id).await.unwrap();
        assert!(item_tags.is_empty());
    }

    #[tokio::test]
    async fn test_list_items_by_tag() {
        let db = Database::in_memory().await.unwrap();
        let id1 = db.insert_item(&make_item("item one", "text")).await.unwrap();
        let _id2 = db.insert_item(&make_item("item two", "text")).await.unwrap();

        db.create_tag("filtered", None).await.unwrap();
        let tags = db.list_tags().await.unwrap();
        let tag_id = tags[0].id;

        db.add_tag_to_item(id1, tag_id).await.unwrap();
        let results = db.list_items_by_tag(tag_id, 10, 0).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, id1);
    }

    #[tokio::test]
    async fn test_cleanup_old_items() {
        let db = Database::in_memory().await.unwrap();
        // Insert an item with a very old timestamp
        let old_item = ClipboardItem {
            id: 0,
            content_type: "text".to_string(),
            content: b"old".to_vec(),
            hash: "oldhash123".to_string(),
            created_at: Utc::now() - chrono::Duration::days(100),
            accessed_at: None,
            pinned: false,
            favorite: false,
            nonce: None,
            encrypted: false,
        };
        db.insert_item(&old_item).await.unwrap();
        let new_id = db.insert_item(&make_item("new item", "text")).await.unwrap();

        let deleted = db.cleanup_old_items(30).await.unwrap();
        assert_eq!(deleted, 1);

        let items = db.list_items(10, 0).await.unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id, new_id);
    }

    #[tokio::test]
    async fn test_enforce_max_items() {
        let db = Database::in_memory().await.unwrap();
        for i in 0..5 {
            db.insert_item(&make_item(&format!("item {}", i), "text")).await.unwrap();
        }
        let deleted = db.enforce_max_items(3).await.unwrap();
        assert_eq!(deleted, 2);
        let items = db.list_items(10, 0).await.unwrap();
        assert_eq!(items.len(), 3);
    }

    #[tokio::test]
    async fn test_search_items() {
        let db = Database::in_memory().await.unwrap();
        db.insert_item(&make_item("the quick brown fox", "text")).await.unwrap();
        db.insert_item(&make_item("hello world", "text")).await.unwrap();

        let results = db.search_items("quick", 10, 0).await.unwrap();
        assert_eq!(results.len(), 1);
        assert!(String::from_utf8_lossy(&results[0].content).contains("quick"));
    }
}
