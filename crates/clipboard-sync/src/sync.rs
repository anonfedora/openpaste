//! Sync client implementation
//!
//! Protocol (all JSON over HTTP):
//!
//! POST /api/v1/sync/push
//!   Body:  { device_id, items: [SyncItem] }
//!   Reply: { accepted: [hash], rejected: [hash] }
//!
//! GET  /api/v1/sync/pull?device_id=&since=<rfc3339>
//!   Reply: { items: [SyncItem] }
//!
//! SyncItem mirrors clipboard_items columns except id (server-assigned).

use crate::SyncError;
use clipboard_db::Database;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info};

// ── Wire types ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncItem {
    pub hash: String,
    pub content_type: String,
    /// Base-64 encoded raw bytes (may be ciphertext when encrypted=true)
    pub content_b64: String,
    pub created_at: String,
    pub pinned: bool,
    pub favorite: bool,
    pub encrypted: bool,
    /// Base-64 encoded nonce, present when encrypted=true
    pub nonce_b64: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PushRequest {
    device_id: String,
    items: Vec<SyncItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PushResponse {
    accepted: Vec<String>,
    #[serde(default)]
    rejected: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PullResponse {
    items: Vec<SyncItem>,
}

// ── Config ────────────────────────────────────────────────────────────────────

/// Configuration required to use the sync client.
#[derive(Debug, Clone)]
pub struct SyncConfig {
    /// Base URL of the sync relay server, e.g. `https://sync.example.com`
    pub server_url: String,
    /// Stable identifier for this machine (UUID stored in settings)
    pub device_id: String,
    /// Optional bearer token for authenticated servers
    pub api_token: Option<String>,
}

// ── Status ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum SyncStatus {
    Idle,
    Syncing,
    Error(String),
}

// ── Client ────────────────────────────────────────────────────────────────────

pub struct SyncClient {
    config: SyncConfig,
    http: Client,
    db: Arc<Database>,
}

impl SyncClient {
    pub fn new(config: SyncConfig, db: Arc<Database>) -> Self {
        Self {
            config,
            http: Client::new(),
            db,
        }
    }

    // ── Push ─────────────────────────────────────────────────────────────────

    /// Push items created after `since_rfc3339` to the server.
    /// Pass `None` to push everything (initial sync).
    pub async fn push(&self, since_rfc3339: Option<&str>) -> Result<usize, SyncError> {
        use base64::{engine::general_purpose::STANDARD, Engine as _};

        // Fetch items from local DB
        let items = self
            .db
            .list_items(1000, 0)
            .await
            .map_err(|e| SyncError::Database(e.to_string()))?;

        let to_push: Vec<SyncItem> = items
            .into_iter()
            .filter(|item| {
                if let Some(since) = since_rfc3339 {
                    item.created_at.to_rfc3339().as_str() > since
                } else {
                    true
                }
            })
            .map(|item| SyncItem {
                hash: item.hash,
                content_type: item.content_type,
                content_b64: STANDARD.encode(&item.content),
                created_at: item.created_at.to_rfc3339(),
                pinned: item.pinned,
                favorite: item.favorite,
                encrypted: item.encrypted,
                nonce_b64: item.nonce.as_deref().map(|n| STANDARD.encode(n)),
            })
            .collect();

        if to_push.is_empty() {
            return Ok(0);
        }

        let count = to_push.len();
        let url = format!("{}/api/v1/sync/push", self.config.server_url);

        let mut req = self
            .http
            .post(&url)
            .json(&PushRequest {
                device_id: self.config.device_id.clone(),
                items: to_push,
            });

        if let Some(ref token) = self.config.api_token {
            req = req.bearer_auth(token);
        }

        let resp = req
            .send()
            .await
            .map_err(|e| SyncError::Http(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(SyncError::Http(format!("{}: {}", status, body)));
        }

        let push_resp: PushResponse = resp
            .json()
            .await
            .map_err(|e| SyncError::Http(e.to_string()))?;

        info!(
            "Sync push: {} accepted, {} rejected",
            push_resp.accepted.len(),
            push_resp.rejected.len()
        );
        Ok(count)
    }

    // ── Pull ─────────────────────────────────────────────────────────────────

    /// Pull items from the server newer than `since_rfc3339` and insert them
    /// into the local DB. Already-known hashes (UNIQUE constraint) are silently
    /// skipped.
    pub async fn pull(&self, since_rfc3339: Option<&str>) -> Result<usize, SyncError> {
        use base64::{engine::general_purpose::STANDARD, Engine as _};

        let since_param = since_rfc3339.unwrap_or("1970-01-01T00:00:00Z");
        let url = format!(
            "{}/api/v1/sync/pull?device_id={}&since={}",
            self.config.server_url,
            urlencoding::encode(&self.config.device_id),
            urlencoding::encode(since_param)
        );

        let mut req = self.http.get(&url);
        if let Some(ref token) = self.config.api_token {
            req = req.bearer_auth(token);
        }

        let resp = req
            .send()
            .await
            .map_err(|e| SyncError::Http(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(SyncError::Http(format!("{}: {}", status, body)));
        }

        let pull_resp: PullResponse = resp
            .json()
            .await
            .map_err(|e| SyncError::Http(e.to_string()))?;

        let mut inserted = 0usize;
        for sync_item in pull_resp.items {
            let content = STANDARD
                .decode(&sync_item.content_b64)
                .map_err(|e| SyncError::SyncFailed(format!("base64 decode: {}", e)))?;

            let nonce = sync_item
                .nonce_b64
                .as_deref()
                .and_then(|n| STANDARD.decode(n).ok());

            let created_at = chrono::DateTime::parse_from_rfc3339(&sync_item.created_at)
                .map_err(|e| SyncError::SyncFailed(format!("date parse: {}", e)))?
                .with_timezone(&chrono::Utc);

            let db_item = clipboard_db::models::ClipboardItem {
                id: 0,
                content_type: sync_item.content_type,
                content,
                hash: sync_item.hash,
                created_at,
                accessed_at: None,
                pinned: sync_item.pinned,
                favorite: sync_item.favorite,
                nonce,
                encrypted: sync_item.encrypted,
            };

            match self.db.insert_item(&db_item).await {
                Ok(_) => inserted += 1,
                Err(e) => {
                    // UNIQUE violation = we already have it, not an error
                    if !e.to_string().contains("UNIQUE") {
                        error!("Failed to insert synced item: {}", e);
                    }
                }
            }
        }

        info!("Sync pull: {} new items inserted", inserted);
        Ok(inserted)
    }

    // ── Full sync cycle ───────────────────────────────────────────────────────

    /// Run one full push → pull cycle.
    /// `last_sync` should be persisted to the `settings` table as `last_sync_at`.
    pub async fn sync_once(&self, last_sync: Option<&str>) -> Result<(usize, usize), SyncError> {
        let pushed = self.push(last_sync).await?;
        let pulled = self.pull(last_sync).await?;
        Ok((pushed, pulled))
    }

    // ── Background sync loop ──────────────────────────────────────────────────

    /// Spawn a background task that syncs every `interval_secs` seconds.
    pub fn start_background(client: Arc<Self>, interval_secs: u64) {
        tokio::spawn(async move {
            let mut ticker =
                tokio::time::interval(tokio::time::Duration::from_secs(interval_secs));

            loop {
                ticker.tick().await;

                // Load last sync timestamp from settings
                let last_sync = client
                    .db
                    .get_setting("last_sync_at")
                    .await
                    .ok()
                    .flatten();

                match client.sync_once(last_sync.as_deref()).await {
                    Ok((pushed, pulled)) => {
                        // Persist new sync timestamp
                        let now = chrono::Utc::now().to_rfc3339();
                        let _ = client.db.upsert_setting("last_sync_at", &now).await;
                        info!("Sync cycle complete: pushed={}, pulled={}", pushed, pulled);
                    }
                    Err(e) => {
                        error!("Sync cycle failed: {}", e);
                    }
                }
            }
        });
    }
}
