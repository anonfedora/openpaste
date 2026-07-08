//! Search engine implementation

use crate::SearchError;
use sqlx::SqlitePool;

/// Search engine for clipboard items
pub struct SearchEngine {
    pool: SqlitePool,
}

impl SearchEngine {
    /// Create a new search engine
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Search for clipboard items
    pub async fn search(&self, query: &str, limit: usize) -> Result<Vec<i64>, SearchError> {
        let results = sqlx::query_scalar::<_, i64>(
            "SELECT rowid FROM clipboard_items_fts WHERE clipboard_items_fts MATCH ? LIMIT ?",
        )
        .bind(query)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SearchError::QueryFailed(e.to_string()))?;

        Ok(results)
    }

    /// Get search suggestions
    pub async fn suggestions(
        &self,
        prefix: &str,
        limit: usize,
    ) -> Result<Vec<String>, SearchError> {
        let suggestions = sqlx::query_scalar::<_, String>(
            "SELECT DISTINCT substr(content, 1, 50) FROM clipboard_items_fts WHERE content MATCH ?* LIMIT ?"
        )
        .bind(prefix)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SearchError::QueryFailed(e.to_string()))?;

        Ok(suggestions)
    }
}
