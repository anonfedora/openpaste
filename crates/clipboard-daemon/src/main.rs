//! OpenPaste daemon - Background service for clipboard management

use anyhow::Result;
use clipboard_core::ClipboardItem;
use clipboard_db::Database;
use clipboard_encryption::Encryption;
use clipboard_events::{Event, EventBus};
use clipboard_ipc::{IpcMessage, IpcServer};
use clipboard_platform::{get_provider, ClipboardProvider};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::signal;
use tokio::sync::Mutex;
use tracing::{error, info};
use tracing_subscriber::fmt;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .init();

    info!("Starting OpenPaste daemon");

    // Initialize components
    let data_dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("openpaste");

    std::fs::create_dir_all(&data_dir)?;

    let db_path = data_dir.join("clipboard.db");
    let storage_path = data_dir.join("storage");
    let ipc_socket_path = data_dir.join("openpaste.sock");

    std::fs::create_dir_all(&storage_path)?;

    // Initialize database
    let database = Arc::new(
        Database::new(&db_path)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to initialize database: {}", e))?,
    );

    // Initialize storage
    let _storage = Arc::new(clipboard_storage::Storage::new(&storage_path));

    // Initialize encryption (placeholder key - should come from user password)
    let key = [0u8; 32]; // TODO: Get from user password
    let _encryption = Arc::new(Encryption::new(key));

    // Initialize event bus
    let event_bus = Arc::new(EventBus::new(100));

    // Initialize clipboard provider
    let provider = Arc::new(Mutex::new(get_provider()?));

    // Start IPC server
    let ipc_server = IpcServer::new(ipc_socket_path.to_string_lossy().to_string());
    let ipc_handler = {
        let database = database.clone();
        let event_bus = event_bus.clone();
        let provider = provider.clone();

        move |message: IpcMessage| -> std::pin::Pin<Box<dyn std::future::Future<Output = IpcMessage> + Send>> {
            match message {
                IpcMessage::GetClipboard => {
                    let provider_clone = provider.clone();
                    Box::pin(async move {
                        let provider = provider_clone.lock().await;
                        match provider.get_content().await {
                            Ok(item) => IpcMessage::ClipboardContent { content: item.content },
                            Err(e) => IpcMessage::Error { message: e.to_string() },
                        }
                    })
                }
                IpcMessage::GetHistory => {
                    let database_clone = database.clone();
                    Box::pin(async move {
                        match database_clone.list_items(100, 0).await {
                            Ok(items) => {
                                let history_items: Vec<clipboard_ipc::ClipboardHistoryItem> = items.into_iter().map(|item| {
                                    clipboard_ipc::ClipboardHistoryItem {
                                        id: item.id.to_string(),
                                        content_type: item.content_type,
                                        content: item.content,
                                        hash: item.hash,
                                        created_at: item.created_at.to_rfc3339(),
                                        accessed_at: item.accessed_at.map(|t| t.to_rfc3339()),
                                        pinned: item.pinned,
                                        favorite: item.favorite,
                                    }
                                }).collect();
                                IpcMessage::ClipboardHistory { items: history_items }
                            }
                            Err(e) => IpcMessage::Error { message: e.to_string() },
                        }
                    })
                }
                IpcMessage::SearchItems { query } => {
                    let database_clone = database.clone();
                    Box::pin(async move {
                        match database_clone.search_items(&query, 100, 0).await {
                            Ok(items) => {
                                let history_items: Vec<clipboard_ipc::ClipboardHistoryItem> = items.into_iter().map(|item| {
                                    clipboard_ipc::ClipboardHistoryItem {
                                        id: item.id.to_string(),
                                        content_type: item.content_type,
                                        content: item.content,
                                        hash: item.hash,
                                        created_at: item.created_at.to_rfc3339(),
                                        accessed_at: item.accessed_at.map(|t| t.to_rfc3339()),
                                        pinned: item.pinned,
                                        favorite: item.favorite,
                                    }
                                }).collect();
                                IpcMessage::ClipboardHistory { items: history_items }
                            }
                            Err(e) => IpcMessage::Error { message: e.to_string() },
                        }
                    })
                }
                IpcMessage::SetClipboard { content } => {
                    let provider_clone = provider.clone();
                    let database_clone = database.clone();
                    let event_bus_clone = event_bus.clone();
                    let item = ClipboardItem::new(clipboard_core::ContentType::Text, content.clone());
                    Box::pin(async move {
                        let provider = provider_clone.lock().await;
                        match provider.set_content(&item).await {
                            Ok(_) => {
                                // Convert to DB model and save
                                let db_item = clipboard_db::models::ClipboardItem {
                                    id: 0, // Will be auto-assigned
                                    content_type: item.content_type.to_string(),
                                    content: item.content.clone(),
                                    hash: item.hash,
                                    created_at: item.created_at,
                                    accessed_at: item.accessed_at,
                                    pinned: item.pinned,
                                    favorite: item.favorite,
                                };
                                let _ = database_clone.insert_item(&db_item).await;

                                // Publish event
                                let event = Event::ClipboardAdded { id: item.id };
                                let _ = event_bus_clone.publish(event);

                                IpcMessage::ClipboardContent { content }
                            }
                            Err(e) => IpcMessage::Error { message: e.to_string() },
                        }
                    })
                }
                IpcMessage::DeleteItem { id } => {
                    let database_clone = database.clone();
                    Box::pin(async move {
                        match database_clone.delete_item(id).await {
                            Ok(_) => IpcMessage::Success,
                            Err(e) => IpcMessage::Error { message: e.to_string() },
                        }
                    })
                }
                IpcMessage::TogglePin { id } => {
                    let database_clone = database.clone();
                    Box::pin(async move {
                        match database_clone.toggle_pin(id).await {
                            Ok(_) => IpcMessage::Success,
                            Err(e) => IpcMessage::Error { message: e.to_string() },
                        }
                    })
                }
                IpcMessage::ToggleFavorite { id } => {
                    let database_clone = database.clone();
                    Box::pin(async move {
                        match database_clone.toggle_favorite(id).await {
                            Ok(_) => IpcMessage::Success,
                            Err(e) => IpcMessage::Error { message: e.to_string() },
                        }
                    })
                }
                _ => Box::pin(async move { IpcMessage::Error { message: "Invalid message".to_string() } }),
            }
        }
    };

    tokio::spawn(async move {
        if let Err(e) = ipc_server.start(ipc_handler).await {
            error!("IPC server error: {}", e);
        }
    });

    // Start clipboard watching with polling
    let provider_watch = provider.clone();
    let event_bus_watch = event_bus.clone();
    let database_watch = database.clone();
    tokio::spawn(async move {
        let mut last_hash = String::new();
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(500));

        loop {
            interval.tick().await;

            let provider = provider_watch.lock().await;
            match provider.get_content().await {
                Ok(item) => {
                    if item.hash != last_hash {
                        last_hash = item.hash.clone();
                        info!("Clipboard changed: hash={}", item.hash);

                        // Publish clipboard change event
                        let event = Event::ClipboardAdded { id: item.id };
                        let _ = event_bus_watch.publish(event);

                        // Save to database
                        let db_item = clipboard_db::models::ClipboardItem {
                            id: 0, // Will be auto-assigned
                            content_type: item.content_type.to_string(),
                            content: item.content.clone(),
                            hash: item.hash,
                            created_at: item.created_at,
                            accessed_at: item.accessed_at,
                            pinned: item.pinned,
                            favorite: item.favorite,
                        };
                        let _ = database_watch.insert_item(&db_item).await;
                    }
                }
                Err(e) => {
                    error!("Failed to get clipboard content: {}", e);
                }
            }
        }
    });

    // Setup graceful shutdown
    let shutdown_signal = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install CTRL+C handler");
        info!("Received shutdown signal");
    };

    #[cfg(unix)]
    {
        use signal::unix::{signal, SignalKind};

        let mut sigterm = signal(SignalKind::terminate())?;
        tokio::select! {
            _ = shutdown_signal => {},
            _ = sigterm.recv() => {
                info!("Received SIGTERM");
            },
        }
    }

    #[cfg(not(unix))]
    {
        shutdown_signal.await;
    }

    info!("Shutting down OpenPaste daemon");

    Ok(())
}
