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
            INSERT INTO clipboard_items (content_type, content, hash, created_at, accessed_at, pinned, favorite)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
        )
        .bind(&item.content_type)
        .bind(&item.content)
        .bind(&item.hash)
        .bind(item.created_at)
        .bind(item.accessed_at)
        .bind(item.pinned)
        .bind(item.favorite)
        .execute(&self.pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        Ok(result.last_insert_rowid())
    }

    /// Get a clipboard item by ID
    pub async fn get_item(&self, id: i64) -> Result<ClipboardItem, DatabaseError> {
        let item = sqlx::query_as::<_, ClipboardItem>(
            "SELECT id, content_type, content, hash, created_at, accessed_at, pinned, favorite FROM clipboard_items WHERE id = ?"
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
            "SELECT id, content_type, content, hash, created_at, accessed_at, pinned, favorite FROM clipboard_items ORDER BY created_at DESC LIMIT ? OFFSET ?"
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
            SELECT ci.id, ci.content_type, ci.content, ci.hash, ci.created_at, ci.accessed_at, ci.pinned, ci.favorite
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
}
