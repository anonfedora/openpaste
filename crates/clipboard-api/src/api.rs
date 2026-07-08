//! REST API server implementation

use crate::ApiError;
use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use tokio::net::TcpListener;

/// REST API server
pub struct ApiServer {
    addr: SocketAddr,
}

impl ApiServer {
    /// Create a new API server
    pub fn new(addr: SocketAddr) -> Self {
        Self { addr }
    }

    /// Build the router
    fn router() -> Router {
        Router::new()
            .route("/api/v1/status", get(handlers::status))
            .route(
                "/api/v1/clipboard",
                get(handlers::get_clipboard).post(handlers::set_clipboard),
            )
            .route("/api/v1/search", post(handlers::search))
    }

    /// Start the API server
    pub async fn start(&self) -> Result<(), ApiError> {
        let app = Self::router();
        let listener = TcpListener::bind(self.addr)
            .await
            .map_err(|e: std::io::Error| ApiError::RequestFailed(e.to_string()))?;

        axum::serve(listener, app)
            .await
            .map_err(|e: std::io::Error| ApiError::RequestFailed(e.to_string()))
    }
}

mod handlers {
    use axum::{response::IntoResponse, Json};
    use serde_json::json;

    pub async fn status() -> impl IntoResponse {
        Json(json!({
            "status": "ok",
            "version": "0.1.0"
        }))
    }

    pub async fn get_clipboard() -> impl IntoResponse {
        Json(json!({
            "success": false,
            "error": "Not implemented"
        }))
    }

    pub async fn set_clipboard() -> impl IntoResponse {
        Json(json!({
            "success": false,
            "error": "Not implemented"
        }))
    }

    pub async fn search() -> impl IntoResponse {
        Json(json!({
            "success": false,
            "error": "Not implemented"
        }))
    }
}
