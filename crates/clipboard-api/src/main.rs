//! OpenPaste sync relay server
//!
//! Starts the HTTP API server so other devices can push/pull clipboard history.
//!
//! Usage:
//!   cargo run --bin clipboard-api -- --addr 0.0.0.0:8080
//!   OPENPASTE_API_ADDR=0.0.0.0:8080 cargo run --bin clipboard-api

use anyhow::Result;
use clipboard_api::ApiServer;
use clipboard_db::Database;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // Basic logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    // Resolve bind address — flag > env > default
    let addr_str = std::env::args()
        .skip_while(|a| a != "--addr")
        .nth(1)
        .or_else(|| std::env::var("OPENPASTE_API_ADDR").ok())
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());

    let addr: SocketAddr = addr_str
        .parse()
        .map_err(|e| anyhow::anyhow!("Invalid address '{}': {}", addr_str, e))?;

    // Use the same data directory as the daemon so they share the same DB
    let data_dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("openpaste");

    std::fs::create_dir_all(&data_dir)?;
    let db_path = data_dir.join("clipboard.db");

    tracing::info!("Opening database at {:?}", db_path);
    let db = Arc::new(
        Database::new(&db_path)
            .await
            .map_err(|e| anyhow::anyhow!("Database error: {}", e))?,
    );

    tracing::info!("OpenPaste API server listening on {}", addr);
    ApiServer::new(addr, db).start().await?;

    Ok(())
}
