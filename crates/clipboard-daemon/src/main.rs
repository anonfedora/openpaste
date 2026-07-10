//! OpenPaste daemon - Background service for clipboard management

use anyhow::Result;
use clipboard_core::ClipboardItem;
use clipboard_db::Database;
use clipboard_events::{Event, EventBus};
use clipboard_ipc::{IpcMessage, IpcServer};
use clipboard_platform::{get_provider, ClipboardProvider};
use clipboard_plugin::PluginManager;
use clipboard_sync::{SyncClient, SyncConfig};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::signal;
use tokio::sync::{Mutex, RwLock};
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
    let _key = [0u8; 32]; // TODO: Get from user password
    let encryption = Arc::new(RwLock::new(None::<clipboard_encryption::Encryption>));

    // Initialize plugin manager
    let plugin_manager = Arc::new(PluginManager::new());

    // Load plugins from data_dir/plugins/*.wasm if any exist
    let plugins_dir = data_dir.join("plugins");
    std::fs::create_dir_all(&plugins_dir)?;
    if let Ok(entries) = std::fs::read_dir(&plugins_dir) {
        for entry in entries.flatten() {
            if entry.path().extension().and_then(|e| e.to_str()) == Some("wasm") {
                match plugin_manager.load(&entry.path()) {
                    Ok(name) => info!("Auto-loaded plugin: {}", name),
                    Err(e) => error!("Failed to load plugin {:?}: {}", entry.path(), e),
                }
            }
        }
    }

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
        let encryption = encryption.clone();
        let plugin_manager = plugin_manager.clone();

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
                    let encryption_clone = encryption.clone();
                    Box::pin(async move {
                        let limit = database_clone.get_setting("max_items").await
                            .ok().flatten()
                            .and_then(|v| v.parse::<usize>().ok())
                            .unwrap_or(10_000)
                            .min(10_000);
                        match database_clone.list_items(limit, 0).await {
                            Ok(items) => {
                                let encryption_guard = encryption_clone.read().await;
                                let history_items: Vec<clipboard_ipc::ClipboardHistoryItem> = items.into_iter().map(|item| {
                                    let (content, content_type) = if item.encrypted {
                                        if let Some(ref enc) = *encryption_guard {
                                            match enc.decrypt(&clipboard_encryption::EncryptedData {
                                                nonce: item.nonce.clone().unwrap_or_default(),
                                                ciphertext: item.content.clone(),
                                            }) {
                                                Ok(decrypted) => (decrypted, item.content_type.clone()),
                                                Err(_) => (b"[decryption failed]".to_vec(), "encrypted".to_string()),
                                            }
                                        } else {
                                            // Vault locked — return placeholder, not raw ciphertext
                                            (b"\xf0\x9f\x94\x92 Encrypted".to_vec(), "encrypted".to_string())
                                        }
                                    } else {
                                        (item.content, item.content_type.clone())
                                    };

                                    clipboard_ipc::ClipboardHistoryItem {
                                        id: item.id.to_string(),
                                        content_type,
                                        content,
                                        hash: item.hash,
                                        created_at: item.created_at.to_rfc3339(),
                                        accessed_at: item.accessed_at.map(|t| t.to_rfc3339()),
                                        pinned: item.pinned,
                                        favorite: item.favorite,
                                        nonce: item.nonce,
                                        encrypted: item.encrypted,
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
                    let encryption_clone = encryption.clone();
                    Box::pin(async move {
                        // Cap search results at 500 — more than enough for any query
                        match database_clone.search_items(&query, 500, 0).await {
                            Ok(items) => {
                                let encryption_guard = encryption_clone.read().await;
                                let history_items: Vec<clipboard_ipc::ClipboardHistoryItem> = items.into_iter().map(|item| {
                                    let (content, content_type) = if item.encrypted {
                                        if let Some(ref enc) = *encryption_guard {
                                            match enc.decrypt(&clipboard_encryption::EncryptedData {
                                                nonce: item.nonce.clone().unwrap_or_default(),
                                                ciphertext: item.content.clone(),
                                            }) {
                                                Ok(decrypted) => (decrypted, item.content_type.clone()),
                                                Err(_) => (b"[decryption failed]".to_vec(), "encrypted".to_string()),
                                            }
                                        } else {
                                            (b"\xf0\x9f\x94\x92 Encrypted".to_vec(), "encrypted".to_string())
                                        }
                                    } else {
                                        (item.content, item.content_type.clone())
                                    };

                                    clipboard_ipc::ClipboardHistoryItem {
                                        id: item.id.to_string(),
                                        content_type,
                                        content,
                                        hash: item.hash,
                                        created_at: item.created_at.to_rfc3339(),
                                        accessed_at: item.accessed_at.map(|t| t.to_rfc3339()),
                                        pinned: item.pinned,
                                        favorite: item.favorite,
                                        nonce: item.nonce,
                                        encrypted: item.encrypted,
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
                    let encryption_clone = encryption.clone();
                    let item = ClipboardItem::new(clipboard_core::ContentType::Text, content.clone());
                    Box::pin(async move {
                        let provider = provider_clone.lock().await;
                        match provider.set_content(&item).await {
                            Ok(_) => {
                                // Encrypt if encryption is enabled
                                let (content_to_store, nonce, encrypted) = {
                                    let encryption_guard = encryption_clone.read().await;
                                    if let Some(ref enc) = *encryption_guard {
                                        match enc.encrypt(&item.content) {
                                            Ok(encrypted_data) => (encrypted_data.ciphertext, Some(encrypted_data.nonce), true),
                                            Err(e) => {
                                                error!("Encryption failed: {}, storing unencrypted", e);
                                                (item.content.clone(), None, false)
                                            }
                                        }
                                    } else {
                                        (item.content.clone(), None, false)
                                    }
                                };
                                drop(encryption_clone);

                                // Convert to DB model and save
                                let db_item = clipboard_db::models::ClipboardItem {
                                    id: 0, // Will be auto-assigned
                                    content_type: item.content_type.to_string(),
                                    content: content_to_store,
                                    hash: item.hash,
                                    created_at: item.created_at,
                                    accessed_at: item.accessed_at,
                                    pinned: item.pinned,
                                    favorite: item.favorite,
                                    nonce,
                                    encrypted,
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
                IpcMessage::GetSettings => {
                    let database_clone = database.clone();
                    Box::pin(async move {
                        // Load settings from database
                        let settings = match database_clone.get_setting("max_items").await {
                            Ok(Some(val)) => val.parse::<i32>().unwrap_or(10000),
                            _ => 10000,
                        };
                        let retention = match database_clone.get_setting("retention_days").await {
                            Ok(Some(val)) => val.parse::<i32>().unwrap_or(90),
                            _ => 90,
                        };
                        let encryption = match database_clone.get_setting("encryption_enabled").await {
                            Ok(Some(val)) => val.parse::<bool>().unwrap_or(false),
                            _ => false,
                        };
                        let auto_lock = match database_clone.get_setting("auto_lock_minutes").await {
                            Ok(Some(val)) => val.parse::<i32>().unwrap_or(5),
                            _ => 5,
                        };
                        let refresh = match database_clone.get_setting("refresh_interval").await {
                            Ok(Some(val)) => val.parse::<i32>().unwrap_or(2),
                            _ => 2,
                        };
                        let notifications = match database_clone.get_setting("show_notifications").await {
                            Ok(Some(val)) => val.parse::<bool>().unwrap_or(true),
                            _ => true,
                        };

                        IpcMessage::Settings {
                            settings: clipboard_ipc::AppSettings {
                                encryption_enabled: encryption,
                                auto_lock_minutes: auto_lock,
                                max_items: settings,
                                retention_days: retention,
                                refresh_interval: refresh,
                                show_notifications: notifications,
                            }
                        }
                    })
                }
                IpcMessage::SaveSettings { settings } => {
                    let database_clone = database.clone();
                    Box::pin(async move {
                        // Save settings to database
                        let results = vec![
                            database_clone.upsert_setting("max_items", &settings.max_items.to_string()).await,
                            database_clone.upsert_setting("retention_days", &settings.retention_days.to_string()).await,
                            database_clone.upsert_setting("encryption_enabled", &settings.encryption_enabled.to_string()).await,
                            database_clone.upsert_setting("auto_lock_minutes", &settings.auto_lock_minutes.to_string()).await,
                            database_clone.upsert_setting("refresh_interval", &settings.refresh_interval.to_string()).await,
                            database_clone.upsert_setting("show_notifications", &settings.show_notifications.to_string()).await,
                        ];

                        if results.iter().all(|r| r.is_ok()) {
                            IpcMessage::Success
                        } else {
                            IpcMessage::Error { message: "Failed to save some settings".to_string() }
                        }
                    })
                }
                IpcMessage::SetMasterPassword { password } => {
                    let database_clone = database.clone();
                    Box::pin(async move {
                        use clipboard_encryption::KeyDerivation;
                        let key_derivation = KeyDerivation::new();
                        
                        // Hash the password and store it
                        match key_derivation.hash_password(&password) {
                            Ok(hash) => {
                                // Generate and store salt for key derivation
                                let salt = KeyDerivation::generate_salt();
                                match database_clone.upsert_setting("encryption_salt", &salt).await {
                                    Ok(_) => {
                                        match database_clone.upsert_setting("master_password_hash", &hash).await {
                                            Ok(_) => IpcMessage::Success,
                                            Err(e) => IpcMessage::Error { message: format!("Failed to store password: {}", e) },
                                        }
                                    }
                                    Err(e) => IpcMessage::Error { message: format!("Failed to store salt: {}", e) },
                                }
                            }
                            Err(e) => IpcMessage::Error { message: format!("Failed to hash password: {}", e) },
                        }
                    })
                }
                IpcMessage::UnlockVault { password } => {
                    let database_clone = database.clone();
                    let encryption_clone = encryption.clone();
                    Box::pin(async move {
                        use clipboard_encryption::KeyDerivation;
                        let key_derivation = KeyDerivation::new();
                        
                        // Get stored password hash and salt
                        let stored_hash = match database_clone.get_setting("master_password_hash").await {
                            Ok(Some(hash)) => hash,
                            Ok(None) => return IpcMessage::Error { message: "No master password set".to_string() },
                            Err(e) => return IpcMessage::Error { message: format!("Failed to get password: {}", e) },
                        };
                        
                        let salt = match database_clone.get_setting("encryption_salt").await {
                            Ok(Some(s)) => s,
                            Ok(None) => return IpcMessage::Error { message: "No encryption salt found".to_string() },
                            Err(e) => return IpcMessage::Error { message: format!("Failed to get salt: {}", e) },
                        };

                        match key_derivation.verify(&password, &stored_hash) {
                            Ok(true) => {
                                // Derive encryption key from password using stored salt
                                match key_derivation.derive_key(&password, &salt) {
                                    Ok(key) => {
                                        use clipboard_encryption::Encryption;
                                        let enc = Encryption::new(key);
                                        *encryption_clone.write().await = Some(enc);
                                        IpcMessage::Success
                                    }
                                    Err(e) => IpcMessage::Error { message: format!("Failed to derive key: {}", e) },
                                }
                            }
                            Ok(false) => IpcMessage::Error { message: "Invalid password".to_string() },
                            Err(e) => IpcMessage::Error { message: format!("Verification failed: {}", e) },
                        }
                    })
                }
                IpcMessage::LockVault => {
                    let encryption_clone = encryption.clone();
                    Box::pin(async move {
                        *encryption_clone.write().await = None;
                        IpcMessage::Success
                    })
                }
                IpcMessage::CheckVaultStatus => {
                    let database_clone = database.clone();
                    let encryption_clone = encryption.clone();
                    Box::pin(async move {
                        // Check if master password is configured
                        let has_password = matches!(
                            database_clone.get_setting("master_password_hash").await,
                            Ok(Some(_))
                        );

                        if !has_password {
                            // No password configured — vault feature not set up
                            return IpcMessage::VaultStatus { is_locked: false };
                        }

                        // Password is configured: locked = encryption key not loaded in memory
                        let encryption_guard = encryption_clone.read().await;
                        let is_locked = encryption_guard.is_none();
                        IpcMessage::VaultStatus { is_locked }
                    })
                }
                IpcMessage::HasMasterPassword => {
                    let database_clone = database.clone();
                    Box::pin(async move {
                        let has_password = matches!(
                            database_clone.get_setting("master_password_hash").await,
                            Ok(Some(_))
                        );
                        IpcMessage::PasswordConfigured { has_password }
                    })
                }
                // ── Tag handlers ──────────────────────────────────────────────
                IpcMessage::ListTags => {
                    let database_clone = database.clone();
                    Box::pin(async move {
                        match database_clone.list_tags().await {
                            Ok(tags) => IpcMessage::TagsList {
                                tags: tags.into_iter().map(|t| clipboard_ipc::TagItem {
                                    id: t.id, name: t.name, color: t.color,
                                }).collect()
                            },
                            Err(e) => IpcMessage::Error { message: e.to_string() },
                        }
                    })
                }
                IpcMessage::CreateTag { name, color } => {
                    let database_clone = database.clone();
                    Box::pin(async move {
                        match database_clone.create_tag(&name, color.as_deref()).await {
                            Ok(_) => IpcMessage::Success,
                            Err(e) => IpcMessage::Error { message: e.to_string() },
                        }
                    })
                }
                IpcMessage::DeleteTag { id } => {
                    let database_clone = database.clone();
                    Box::pin(async move {
                        match database_clone.delete_tag(id).await {
                            Ok(_) => IpcMessage::Success,
                            Err(e) => IpcMessage::Error { message: e.to_string() },
                        }
                    })
                }
                IpcMessage::GetItemTags { item_id } => {
                    let database_clone = database.clone();
                    Box::pin(async move {
                        match database_clone.get_item_tags(item_id).await {
                            Ok(tags) => IpcMessage::ItemTags {
                                tags: tags.into_iter().map(|t| clipboard_ipc::TagItem {
                                    id: t.id, name: t.name, color: t.color,
                                }).collect()
                            },
                            Err(e) => IpcMessage::Error { message: e.to_string() },
                        }
                    })
                }
                IpcMessage::AddTagToItem { item_id, tag_name, color } => {
                    let database_clone = database.clone();
                    Box::pin(async move {
                        // Create tag if needed, then assign
                        match database_clone.create_tag(&tag_name, color.as_deref()).await {
                            Ok(tag_id) => {
                                // create_tag returns last_insert_rowid which is 0 on conflict update
                                // fetch the actual id by name
                                let actual_id = match database_clone.list_tags().await {
                                    Ok(tags) => tags.into_iter()
                                        .find(|t| t.name == tag_name)
                                        .map(|t| t.id)
                                        .unwrap_or(tag_id),
                                    Err(_) => tag_id,
                                };
                                match database_clone.add_tag_to_item(item_id, actual_id).await {
                                    Ok(_) => IpcMessage::Success,
                                    Err(e) => IpcMessage::Error { message: e.to_string() },
                                }
                            }
                            Err(e) => IpcMessage::Error { message: e.to_string() },
                        }
                    })
                }
                IpcMessage::RemoveTagFromItem { item_id, tag_id } => {
                    let database_clone = database.clone();
                    Box::pin(async move {
                        match database_clone.remove_tag_from_item(item_id, tag_id).await {
                            Ok(_) => IpcMessage::Success,
                            Err(e) => IpcMessage::Error { message: e.to_string() },
                        }
                    })
                }
                IpcMessage::ListItemsByTag { tag_id } => {
                    let database_clone = database.clone();
                    let encryption_clone = encryption.clone();
                    Box::pin(async move {
                        let limit = database_clone.get_setting("max_items").await
                            .ok().flatten()
                            .and_then(|v| v.parse::<usize>().ok())
                            .unwrap_or(10_000)
                            .min(10_000);
                        match database_clone.list_items_by_tag(tag_id, limit, 0).await {
                            Ok(items) => {
                                let encryption_guard = encryption_clone.read().await;
                                let history_items = items.into_iter().map(|item| {
                                    let (content, content_type) = if item.encrypted {
                                        if let Some(ref enc) = *encryption_guard {
                                            match enc.decrypt(&clipboard_encryption::EncryptedData {
                                                nonce: item.nonce.clone().unwrap_or_default(),
                                                ciphertext: item.content.clone(),
                                            }) {
                                                Ok(d) => (d, item.content_type.clone()),
                                                Err(_) => (b"[decryption failed]".to_vec(), "encrypted".to_string()),
                                            }
                                        } else {
                                            (b"\xf0\x9f\x94\x92 Encrypted".to_vec(), "encrypted".to_string())
                                        }
                                    } else {
                                        (item.content, item.content_type.clone())
                                    };
                                    clipboard_ipc::ClipboardHistoryItem {
                                        id: item.id.to_string(),
                                        content_type,
                                        content,
                                        hash: item.hash,
                                        created_at: item.created_at.to_rfc3339(),
                                        accessed_at: item.accessed_at.map(|t| t.to_rfc3339()),
                                        pinned: item.pinned,
                                        favorite: item.favorite,
                                        nonce: item.nonce,
                                        encrypted: item.encrypted,
                                    }
                                }).collect();
                                IpcMessage::ClipboardHistory { items: history_items }
                            }
                            Err(e) => IpcMessage::Error { message: e.to_string() },
                        }
                    })
                }
                // ── Sync handlers ─────────────────────────────────────────────
                IpcMessage::GetSyncConfig => {
                    let database_clone = database.clone();
                    Box::pin(async move {
                        let server_url = database_clone.get_setting("sync_server_url").await
                            .ok().flatten().unwrap_or_default();
                        let api_token = database_clone.get_setting("sync_api_token").await
                            .ok().flatten();
                        let enabled = database_clone.get_setting("sync_enabled").await
                            .ok().flatten().and_then(|v| v.parse::<bool>().ok()).unwrap_or(false);
                        let last_sync_at = database_clone.get_setting("last_sync_at").await
                            .ok().flatten();
                        IpcMessage::SyncConfig { server_url, api_token, enabled, last_sync_at }
                    })
                }
                IpcMessage::SetSyncConfig { server_url, api_token, enabled } => {
                    let database_clone = database.clone();
                    Box::pin(async move {
                        let results = vec![
                            database_clone.upsert_setting("sync_server_url", &server_url).await,
                            database_clone.upsert_setting("sync_enabled", &enabled.to_string()).await,
                        ];
                        if let Some(token) = &api_token {
                            let _ = database_clone.upsert_setting("sync_api_token", token).await;
                        }
                        if results.iter().all(|r| r.is_ok()) {
                            IpcMessage::Success
                        } else {
                            IpcMessage::Error { message: "Failed to save sync config".to_string() }
                        }
                    })
                }
                IpcMessage::SyncNow => {
                    let database_clone = database.clone();
                    Box::pin(async move {
                        let server_url = match database_clone.get_setting("sync_server_url").await {
                            Ok(Some(url)) if !url.is_empty() => url,
                            _ => return IpcMessage::Error { message: "Sync server not configured".to_string() },
                        };
                        let api_token = database_clone.get_setting("sync_api_token").await.ok().flatten();
                        let device_id = database_clone.get_setting("device_id").await.ok().flatten()
                            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
                        // Persist device_id if new
                        let _ = database_clone.upsert_setting("device_id", &device_id).await;

                        let config = SyncConfig { server_url, device_id, api_token };
                        let client = SyncClient::new(config, database_clone.clone());
                        let last_sync = database_clone.get_setting("last_sync_at").await.ok().flatten();
                        match client.sync_once(last_sync.as_deref()).await {
                            Ok((pushed, pulled)) => {
                                let now = chrono::Utc::now().to_rfc3339();
                                let _ = database_clone.upsert_setting("last_sync_at", &now).await;
                                IpcMessage::ClipboardContent {
                                    content: format!("pushed={}, pulled={}", pushed, pulled).into_bytes()
                                }
                            }
                            Err(e) => IpcMessage::Error { message: e.to_string() },
                        }
                    })
                }
                // ── Plugin handlers ────────────────────────────────────────────
                IpcMessage::ListPlugins => {
                    let pm = plugin_manager.clone();
                    Box::pin(async move {
                        let plugins = pm.list().into_iter().map(|p| clipboard_ipc::PluginInfoItem {
                            name: p.name,
                            path: p.path.to_string_lossy().to_string(),
                            enabled: p.enabled,
                        }).collect();
                        IpcMessage::PluginList { plugins }
                    })
                }
                IpcMessage::LoadPlugin { path } => {
                    let pm = plugin_manager.clone();
                    Box::pin(async move {
                        match pm.load(std::path::Path::new(&path)) {
                            Ok(name) => IpcMessage::ClipboardContent { content: name.into_bytes() },
                            Err(e) => IpcMessage::Error { message: e.to_string() },
                        }
                    })
                }
                IpcMessage::UnloadPlugin { name } => {
                    let pm = plugin_manager.clone();
                    Box::pin(async move {
                        match pm.unload(&name) {
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
    let encryption_watch = encryption.clone();
    tokio::spawn(async move {
        let mut last_hash = String::new();
        let mut is_startup = true; // suppress notification for the initial clipboard snapshot
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(500));

        loop {
            interval.tick().await;

            let provider = provider_watch.lock().await;
            match provider.get_content().await {
                Ok(item) => {
                    if item.hash != last_hash {
                        let is_new_capture = !is_startup;
                        last_hash = item.hash.clone();
                        is_startup = false;

                        if !is_new_capture {
                            // First tick — just record current hash, no save, no notification
                            continue;
                        }

                        info!("Clipboard changed: hash={}", item.hash);

                        // Publish clipboard change event
                        let event = Event::ClipboardAdded { id: item.id };
                        let _ = event_bus_watch.publish(event);

                        // Encrypt if encryption is enabled
                        let (content_to_store, nonce, encrypted) = {
                            let encryption_guard = encryption_watch.read().await;
                            if let Some(ref enc) = *encryption_guard {
                                match enc.encrypt(&item.content) {
                                    Ok(encrypted_data) => (encrypted_data.ciphertext, Some(encrypted_data.nonce), true),
                                    Err(e) => {
                                        error!("Encryption failed: {}, storing unencrypted", e);
                                        (item.content.clone(), None, false)
                                    }
                                }
                            } else {
                                (item.content.clone(), None, false)
                            }
                        };

                        // Save to database — silently skip duplicates (UNIQUE constraint on hash)
                        let db_item = clipboard_db::models::ClipboardItem {
                            id: 0,
                            content_type: item.content_type.to_string(),
                            content: content_to_store,
                            hash: item.hash,
                            created_at: item.created_at,
                            accessed_at: item.accessed_at,
                            pinned: item.pinned,
                            favorite: item.favorite,
                            nonce,
                            encrypted,
                        };
                        // Send notification if enabled
                        let show_notifications = match database_watch.get_setting("show_notifications").await {
                            Ok(Some(val)) => val.parse::<bool>().unwrap_or(true),
                            _ => true,
                        };
                        if show_notifications {
                            let preview: String = if item.content_type.to_string() == "image" {
                                "📷 Image captured".to_string()
                            } else {
                                let text = String::from_utf8_lossy(&item.content);
                                let truncated = text.chars().take(60).collect::<String>();
                                if text.len() > 60 { format!("{}…", truncated) } else { truncated }
                            };
                            let _ = notify_rust::Notification::new()
                                .summary("OpenPaste")
                                .body(&preview)
                                .timeout(notify_rust::Timeout::Milliseconds(2500))
                                .show();
                        }

                        match database_watch.insert_item(&db_item).await {
                            Ok(_) => {}
                            Err(e) => {
                                let msg = e.to_string();
                                // UNIQUE constraint = duplicate, not a real error
                                if !msg.contains("UNIQUE") {
                                    error!("Failed to save clipboard item: {}", msg);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to get clipboard content: {}", e);
                }
            }
        }
    });

    // Start periodic cleanup task
    let database_cleanup = database.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(3600)); // Run every hour

        loop {
            interval.tick().await;

            // Load retention settings
            let max_items = match database_cleanup.get_setting("max_items").await {
                Ok(Some(val)) => val.parse::<i32>().unwrap_or(10000),
                _ => 10000,
            };
            let retention_days = match database_cleanup.get_setting("retention_days").await {
                Ok(Some(val)) => val.parse::<i32>().unwrap_or(90),
                _ => 90,
            };

            // Run cleanup
            match database_cleanup.cleanup_old_items(retention_days).await {
                Ok(deleted) => {
                    if deleted > 0 {
                        info!("Cleaned up {} items older than {} days", deleted, retention_days);
                    }
                }
                Err(e) => error!("Failed to cleanup old items: {}", e),
            }

            match database_cleanup.enforce_max_items(max_items).await {
                Ok(deleted) => {
                    if deleted > 0 {
                        info!("Deleted {} items to enforce max items limit of {}", deleted, max_items);
                    }
                }
                Err(e) => error!("Failed to enforce max items: {}", e),
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
