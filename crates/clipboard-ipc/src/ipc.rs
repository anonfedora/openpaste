//! IPC server and client implementation

use crate::IpcError;
use serde::{Deserialize, Serialize};

// Unix-only imports
#[cfg(unix)]
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

// Windows needs AsyncReadExt + AsyncWriteExt
#[cfg(windows)]
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// IPC message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IpcMessage {
    /// Request clipboard content
    GetClipboard,
    /// Request clipboard history
    GetHistory,
    /// Search clipboard items
    SearchItems { query: String },
    /// Set clipboard content
    SetClipboard { content: Vec<u8> },
    /// Delete clipboard item
    DeleteItem { id: i64 },
    /// Toggle pin status
    TogglePin { id: i64 },
    /// Toggle favorite status
    ToggleFavorite { id: i64 },
    /// Get settings
    GetSettings,
    /// Save settings
    SaveSettings { settings: AppSettings },
    /// Set master password
    SetMasterPassword { password: String },
    /// Unlock vault with password
    UnlockVault { password: String },
    /// Lock vault
    LockVault,
    /// Check if vault is locked
    CheckVaultStatus,
    /// Check if a master password has been configured
    HasMasterPassword,
    // ── Tag operations ────────────────────────────────────────────────────────
    /// List all tags
    ListTags,
    /// Create or upsert a tag
    CreateTag { name: String, color: Option<String> },
    /// Delete a tag by id
    DeleteTag { id: i64 },
    /// Get tags for a clipboard item
    GetItemTags { item_id: i64 },
    /// Add a tag to a clipboard item (creates tag if needed)
    AddTagToItem { item_id: i64, tag_name: String, color: Option<String> },
    /// Remove a tag from a clipboard item
    RemoveTagFromItem { item_id: i64, tag_id: i64 },
    /// List items with a given tag
    ListItemsByTag { tag_id: i64 },
    // ── Sync operations ───────────────────────────────────────────────────────
    /// Get sync configuration (server_url, device_id, enabled)
    GetSyncConfig,
    /// Save sync configuration
    SetSyncConfig { server_url: String, api_token: Option<String>, enabled: bool },
    /// Trigger an immediate sync cycle
    SyncNow,
    // ── Plugin operations ─────────────────────────────────────────────────────
    /// List loaded plugins
    ListPlugins,
    /// Load a plugin from an absolute path
    LoadPlugin { path: String },
    /// Unload a plugin by name
    UnloadPlugin { name: String },
    /// Clipboard content response
    ClipboardContent { content: Vec<u8> },
    /// Clipboard history response
    ClipboardHistory { items: Vec<ClipboardHistoryItem> },
    /// Settings response
    Settings { settings: AppSettings },
    /// Vault status response
    VaultStatus { is_locked: bool },
    /// Whether a master password has been configured
    PasswordConfigured { has_password: bool },
    /// Tags list response
    TagsList { tags: Vec<TagItem> },
    /// Tags for a single item
    ItemTags { tags: Vec<TagItem> },
    /// Sync configuration response
    SyncConfig { server_url: String, api_token: Option<String>, enabled: bool, last_sync_at: Option<String> },
    /// Plugin list response
    PluginList { plugins: Vec<PluginInfoItem> },
    /// Success response
    Success,
    /// Error response
    Error { message: String },
}

/// Application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub encryption_enabled: bool,
    pub auto_lock_minutes: i32,
    pub max_items: i32,
    pub retention_days: i32,
    pub refresh_interval: i32,
    pub show_notifications: bool,
}

/// Tag item for IPC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagItem {
    pub id: i64,
    pub name: String,
    pub color: Option<String>,
}

/// Plugin info for IPC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfoItem {
    pub name: String,
    pub path: String,
    pub enabled: bool,
}

/// Clipboard history item for IPC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardHistoryItem {    pub id: String,
    pub content_type: String,
    pub content: Vec<u8>,
    pub hash: String,
    pub created_at: String,
    pub accessed_at: Option<String>,
    pub pinned: bool,
    pub favorite: bool,
    pub nonce: Option<Vec<u8>>,
    pub encrypted: bool,
}

/// IPC server
pub struct IpcServer {
    path: String,
}

impl IpcServer {
    /// Create a new IPC server
    pub fn new(path: String) -> Self {
        Self { path }
    }

    /// Start the IPC server
    pub async fn start<F, Fut>(&self, handler: F) -> Result<(), IpcError>
    where
        F: Fn(IpcMessage) -> Fut + Send + Sync + Clone + 'static,
        Fut: std::future::Future<Output = IpcMessage> + Send + 'static,
    {
        #[cfg(unix)]
        {
            use tokio::net::UnixListener;

            // Remove existing socket if present
            let _ = std::fs::remove_file(&self.path);

            let listener = UnixListener::bind(&self.path)
                .map_err(|e| IpcError::ConnectionFailed(e.to_string()))?;

            tracing::info!("IPC server listening on {}", self.path);

            loop {
                match listener.accept().await {
                    Ok((stream, _)) => {
                        let handler = handler.clone();
                        tokio::spawn(async move {
                            if let Err(e) = Self::handle_connection(stream, handler).await {
                                tracing::error!("Connection error: {}", e);
                            }
                        });
                    }
                    Err(e) => {
                        tracing::error!("Accept error: {}", e);
                    }
                }
            }
        }

        #[cfg(windows)]
        {
            use tokio::net::windows::named_pipe::ServerOptions;

            tracing::info!("IPC server listening on {}", self.path);

            let mut first = true;
            loop {
                // The first instance must claim ownership with first_pipe_instance(true)
                let mut server = ServerOptions::new()
                    .first_pipe_instance(first)
                    .create(&self.path)
                    .map_err(|e| IpcError::ConnectionFailed(e.to_string()))?;
                first = false;

                // Wait for a client to connect
                server
                    .connect()
                    .await
                    .map_err(|e| IpcError::ConnectionFailed(e.to_string()))?;

                let handler = handler.clone();
                tokio::spawn(async move {
                    if let Err(e) = Self::handle_connection_windows(server, handler).await {
                        tracing::error!("Connection error: {}", e);
                    }
                });
            }
        }
    }

    #[cfg(unix)]
    async fn handle_connection<F, Fut>(
        mut stream: tokio::net::UnixStream,
        handler: F,
    ) -> Result<(), IpcError>
    where
        F: Fn(IpcMessage) -> Fut + Clone,
        Fut: std::future::Future<Output = IpcMessage>,
    {
        let (reader, mut writer) = stream.split();
        let mut reader = BufReader::new(reader);
        let mut line = String::new();

        reader
            .read_line(&mut line)
            .await
            .map_err(|e| IpcError::ReceiveFailed(e.to_string()))?;

        let request: IpcMessage = serde_json::from_str(&line)
            .map_err(|e| IpcError::DeserializationFailed(e.to_string()))?;

        let response = handler(request).await;
        let response_json = serde_json::to_string(&response)
            .map_err(|e| IpcError::SerializationFailed(e.to_string()))?;

        writer
            .write_all(response_json.as_bytes())
            .await
            .map_err(|e| IpcError::SendFailed(e.to_string()))?;

        writer
            .write_all(b"\n")
            .await
            .map_err(|e| IpcError::SendFailed(e.to_string()))?;

        Ok(())
    }

    #[cfg(windows)]
    async fn handle_connection_windows<F, Fut>(
        mut server: tokio::net::windows::named_pipe::NamedPipeServer,
        handler: F,
    ) -> Result<(), IpcError>
    where
        F: Fn(IpcMessage) -> Fut + Clone,
        Fut: std::future::Future<Output = IpcMessage>,
    {
        let mut buf = vec![0u8; 65536];
        let n = server
            .read(&mut buf)
            .await
            .map_err(|e| IpcError::ReceiveFailed(e.to_string()))?;

        let request: IpcMessage = serde_json::from_slice(&buf[..n])
            .map_err(|e| IpcError::DeserializationFailed(e.to_string()))?;

        let response = handler(request).await;
        let response_json = serde_json::to_vec(&response)
            .map_err(|e| IpcError::SerializationFailed(e.to_string()))?;

        server
            .write_all(&response_json)
            .await
            .map_err(|e| IpcError::SendFailed(e.to_string()))?;

        Ok(())
    }
}

/// IPC client
pub struct IpcClient {
    path: String,
}

impl IpcClient {
    /// Create a new IPC client
    pub fn new(path: String) -> Self {
        Self { path }
    }

    /// Send a message and wait for response
    pub async fn send(&self, message: IpcMessage) -> Result<IpcMessage, IpcError> {
        #[cfg(unix)]
        {
            use tokio::net::UnixStream;

            let mut stream = UnixStream::connect(&self.path)
                .await
                .map_err(|e| IpcError::ConnectionFailed(e.to_string()))?;

            let message_json = serde_json::to_string(&message)
                .map_err(|e| IpcError::SerializationFailed(e.to_string()))?;

            stream
                .write_all(message_json.as_bytes())
                .await
                .map_err(|e| IpcError::SendFailed(e.to_string()))?;

            stream
                .write_all(b"\n")
                .await
                .map_err(|e| IpcError::SendFailed(e.to_string()))?;

            let (reader, _) = stream.split();
            let mut reader = BufReader::new(reader);
            let mut line = String::new();

            reader
                .read_line(&mut line)
                .await
                .map_err(|e| IpcError::ReceiveFailed(e.to_string()))?;

            let response: IpcMessage = serde_json::from_str(&line)
                .map_err(|e| IpcError::DeserializationFailed(e.to_string()))?;

            Ok(response)
        }

        #[cfg(windows)]
        {
            use tokio::net::windows::named_pipe::ClientOptions;

            let mut client = ClientOptions::new()
                .open(&self.path)
                .map_err(|e| IpcError::ConnectionFailed(e.to_string()))?;

            let message_json = serde_json::to_vec(&message)
                .map_err(|e| IpcError::SerializationFailed(e.to_string()))?;

            client
                .write_all(&message_json)
                .await
                .map_err(|e| IpcError::SendFailed(e.to_string()))?;

            let mut buf = vec![0u8; 65536];
            let n = client
                .read(&mut buf)
                .await
                .map_err(|e| IpcError::ReceiveFailed(e.to_string()))?;

            let response: IpcMessage = serde_json::from_slice(&buf[..n])
                .map_err(|e| IpcError::DeserializationFailed(e.to_string()))?;

            Ok(response)
        }
    }
}
