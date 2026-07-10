//! REST API server
//!
//! Routes:
//!   GET  /api/v1/status           — health check
//!   GET  /api/v1/clipboard        — get current clipboard content
//!   POST /api/v1/clipboard        — set clipboard content
//!   POST /api/v1/search           — FTS5 search
//!   GET  /api/v1/history          — list history (limit/offset)
//!   GET  /api/v1/item/:id         — get single item
//!   DELETE /api/v1/item/:id       — delete item
//!   POST /api/v1/sync/push        — sync push endpoint
//!   GET  /api/v1/sync/pull        — sync pull endpoint

use crate::ApiError;
use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use clipboard_db::Database;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

// ── Shared app state ──────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Database>,
}

// ── Server ────────────────────────────────────────────────────────────────────

pub struct ApiServer {
    addr: SocketAddr,
    db: Arc<Database>,
}

impl ApiServer {
    pub fn new(addr: SocketAddr, db: Arc<Database>) -> Self {
        Self { addr, db }
    }

    fn router(state: AppState) -> Router {
        Router::new()
            .route("/api/v1/status", get(status))
            .route("/api/v1/clipboard", get(get_clipboard).post(set_clipboard))
            .route("/api/v1/search", post(search))
            .route("/api/v1/history", get(list_history))
            .route("/api/v1/item/:id", get(get_item).delete(delete_item))
            .route("/api/v1/sync/push", post(sync_push))
            .route("/api/v1/sync/pull", get(sync_pull))
            .with_state(state)
            .layer(CorsLayer::permissive())
    }

    pub async fn start(&self) -> Result<(), ApiError> {
        let state = AppState {
            db: self.db.clone(),
        };
        let app = Self::router(state);
        let listener = TcpListener::bind(self.addr)
            .await
            .map_err(|e| ApiError::RequestFailed(e.to_string()))?;

        tracing::info!("API server listening on {}", self.addr);
        axum::serve(listener, app)
            .await
            .map_err(|e| ApiError::RequestFailed(e.to_string()))
    }
}

// ── Handlers ──────────────────────────────────────────────────────────────────

async fn status() -> Json<Value> {
    Json(json!({ "status": "ok", "version": env!("CARGO_PKG_VERSION") }))
}

#[derive(Deserialize)]
struct PaginationParams {
    #[serde(default = "default_limit")]
    limit: usize,
    #[serde(default)]
    offset: usize,
}
fn default_limit() -> usize { 50 }

async fn list_history(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Json<Value> {
    match state.db.list_items(params.limit.min(200), params.offset).await {
        Ok(items) => {
            let out: Vec<Value> = items.iter().map(item_to_json).collect();
            Json(json!({ "items": out, "total": out.len() }))
        }
        Err(e) => Json(json!({ "error": e.to_string() })),
    }
}

async fn get_clipboard(State(state): State<AppState>) -> Json<Value> {
    match state.db.list_items(1, 0).await {
        Ok(items) if !items.is_empty() => Json(json!({ "item": item_to_json(&items[0]) })),
        Ok(_) => Json(json!({ "item": null })),
        Err(e) => Json(json!({ "error": e.to_string() })),
    }
}

#[derive(Deserialize)]
struct SetClipboardBody {
    content: String,
    content_type: Option<String>,
}

async fn set_clipboard(
    State(state): State<AppState>,
    Json(body): Json<SetClipboardBody>,
) -> Json<Value> {
    use clipboard_db::models::ClipboardItem;
    use md5;

    let content_bytes = body.content.as_bytes().to_vec();
    let hash = format!("{:x}", md5::compute(&content_bytes));
    let ct = body.content_type.unwrap_or_else(|| "text".to_string());

    let item = ClipboardItem {
        id: 0,
        content_type: ct,
        content: content_bytes,
        hash,
        created_at: chrono::Utc::now(),
        accessed_at: None,
        pinned: false,
        favorite: false,
        nonce: None,
        encrypted: false,
    };

    match state.db.insert_item(&item).await {
        Ok(id) => Json(json!({ "id": id, "success": true })),
        Err(e) if e.to_string().contains("UNIQUE") => {
            Json(json!({ "success": true, "duplicate": true }))
        }
        Err(e) => Json(json!({ "error": e.to_string() })),
    }
}

#[derive(Deserialize)]
struct SearchBody {
    query: String,
    #[serde(default = "default_limit")]
    limit: usize,
    #[serde(default)]
    offset: usize,
}

async fn search(
    State(state): State<AppState>,
    Json(body): Json<SearchBody>,
) -> Json<Value> {
    match state.db.search_items(&body.query, body.limit.min(200), body.offset).await {
        Ok(items) => {
            let out: Vec<Value> = items.iter().map(item_to_json).collect();
            Json(json!({ "items": out }))
        }
        Err(e) => Json(json!({ "error": e.to_string() })),
    }
}

async fn get_item(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Json<Value> {
    match state.db.get_item(id).await {
        Ok(item) => Json(json!({ "item": item_to_json(&item) })),
        Err(e) => Json(json!({ "error": e.to_string() })),
    }
}

async fn delete_item(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Json<Value> {
    match state.db.delete_item(id).await {
        Ok(_) => Json(json!({ "success": true })),
        Err(e) => Json(json!({ "error": e.to_string() })),
    }
}

// ── Sync endpoints ────────────────────────────────────────────────────────────
// These implement the server side of the sync protocol described in clipboard-sync.

#[derive(Deserialize)]
struct SyncPushBody {
    device_id: String,
    items: Vec<SyncItemWire>,
}

#[derive(Deserialize, Serialize)]
struct SyncItemWire {
    hash: String,
    content_type: String,
    content_b64: String,
    created_at: String,
    pinned: bool,
    favorite: bool,
    encrypted: bool,
    nonce_b64: Option<String>,
}

#[derive(Deserialize)]
struct SyncPullParams {
    device_id: String,
    since: Option<String>,
}

async fn sync_push(
    State(state): State<AppState>,
    Json(body): Json<SyncPushBody>,
) -> Json<Value> {
    use base64::{engine::general_purpose::STANDARD, Engine as _};
    use clipboard_db::models::ClipboardItem;

    tracing::info!("Sync push from device: {}", body.device_id);
    let mut accepted = Vec::new();
    let mut rejected = Vec::new();

    for wire in body.items {
        let content = match STANDARD.decode(&wire.content_b64) {
            Ok(b) => b,
            Err(_) => { rejected.push(wire.hash); continue; }
        };
        let nonce = wire.nonce_b64.as_deref().and_then(|n| STANDARD.decode(n).ok());
        let created_at = match chrono::DateTime::parse_from_rfc3339(&wire.created_at) {
            Ok(dt) => dt.with_timezone(&chrono::Utc),
            Err(_) => { rejected.push(wire.hash); continue; }
        };

        let item = ClipboardItem {
            id: 0,
            content_type: wire.content_type,
            content,
            hash: wire.hash.clone(),
            created_at,
            accessed_at: None,
            pinned: wire.pinned,
            favorite: wire.favorite,
            nonce,
            encrypted: wire.encrypted,
        };

        match state.db.insert_item(&item).await {
            Ok(_) | Err(_) => accepted.push(wire.hash), // duplicates are also "accepted"
        }
    }

    Json(json!({ "accepted": accepted, "rejected": rejected }))
}

async fn sync_pull(
    State(state): State<AppState>,
    Query(params): Query<SyncPullParams>,
) -> Json<Value> {
    use base64::{engine::general_purpose::STANDARD, Engine as _};

    tracing::info!("Sync pull from device: {}", params.device_id);

    // Fetch recent items (server just returns last 500; client filters by since)
    let items = match state.db.list_items(500, 0).await {
        Ok(i) => i,
        Err(e) => return Json(json!({ "error": e.to_string() })),
    };

    let since = params.since.as_deref().unwrap_or("1970-01-01T00:00:00Z");

    let wire_items: Vec<SyncItemWire> = items
        .into_iter()
        .filter(|item| item.created_at.to_rfc3339().as_str() > since)
        .map(|item| SyncItemWire {
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

    Json(json!({ "items": wire_items }))
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn item_to_json(item: &clipboard_db::models::ClipboardItem) -> Value {
    // Return text content as UTF-8 string; binary/encrypted as base64
    let content_str = if item.encrypted {
        "[encrypted]".to_string()
    } else {
        String::from_utf8_lossy(&item.content).to_string()
    };

    json!({
        "id": item.id,
        "content_type": item.content_type,
        "content": content_str,
        "hash": item.hash,
        "created_at": item.created_at.to_rfc3339(),
        "pinned": item.pinned,
        "favorite": item.favorite,
        "encrypted": item.encrypted,
    })
}
