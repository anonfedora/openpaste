#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use arboard;
use clipboard_ipc::{AppSettings, IpcClient, IpcMessage, PluginInfoItem, TagItem};
use notify_rust;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{
    CustomMenuItem, GlobalShortcutManager, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu,
    SystemTrayMenuItem,
};

#[derive(Serialize, Deserialize)]
struct ClipboardItem {
    id: String,
    content_type: String,
    content: String,
    hash: String,
    created_at: String,
    accessed_at: Option<String>,
    pinned: bool,
    favorite: bool,
}

/// Convert bytes to string, using base64 for binary content
fn bytes_to_string(content_type: &str, bytes: &[u8]) -> String {
    let type_lower = content_type.to_lowercase();
    if type_lower.contains("text")
        || type_lower.contains("plain")
        || type_lower.contains("url")
        || type_lower.contains("code")
        || type_lower.contains("html")
        || type_lower.contains("json")
    {
        String::from_utf8_lossy(bytes).to_string()
    } else {
        use base64::{engine::general_purpose::STANDARD, Engine as _};
        STANDARD.encode(bytes)
    }
}

fn get_ipc_socket_path() -> PathBuf {
    // On macOS, dirs::data_local_dir() returns ~/Library/Application Support
    // which matches what the daemon uses.  Fall back to ~/.local/share on Linux.
    let data_dir = dirs::data_local_dir()
        .or_else(|| dirs::home_dir().map(|h| h.join(".local").join("share")))
        .unwrap_or_else(|| PathBuf::from("."))
        .join("openpaste");

    #[cfg(unix)]
    let socket_path = data_dir.join("openpaste.sock");

    // Windows named pipes must use the \\.\pipe\ prefix
    #[cfg(windows)]
    let socket_path = PathBuf::from(r"\\.\pipe\openpaste");

    socket_path
}

#[tauri::command]
async fn get_clipboard_history() -> Result<Vec<ClipboardItem>, String> {
    let socket_path = get_ipc_socket_path();
    let client = IpcClient::new(socket_path.to_string_lossy().to_string());

    match client.send(IpcMessage::GetHistory).await {
        Ok(IpcMessage::ClipboardHistory { items }) => {
            let clipboard_items = items
                .into_iter()
                .map(|item| ClipboardItem {
                    id: item.id,
                    content_type: item.content_type.clone(),
                    content: bytes_to_string(&item.content_type, &item.content),
                    hash: item.hash,
                    created_at: item.created_at,
                    accessed_at: item.accessed_at,
                    pinned: item.pinned,
                    favorite: item.favorite,
                })
                .collect();
            Ok(clipboard_items)
        }
        Ok(IpcMessage::Error { message }) => Err(format!("Daemon error: {}", message)),
        Err(e) => Err(format!("IPC error: {}", e)),
        _ => Err("Unexpected response from daemon".to_string()),
    }
}

#[tauri::command]
async fn search_clipboard_items(query: String) -> Result<Vec<ClipboardItem>, String> {
    let socket_path = get_ipc_socket_path();
    let client = IpcClient::new(socket_path.to_string_lossy().to_string());

    match client.send(IpcMessage::SearchItems { query }).await {
        Ok(IpcMessage::ClipboardHistory { items }) => {
            let clipboard_items = items
                .into_iter()
                .map(|item| ClipboardItem {
                    id: item.id,
                    content_type: item.content_type.clone(),
                    content: bytes_to_string(&item.content_type, &item.content),
                    hash: item.hash,
                    created_at: item.created_at,
                    accessed_at: item.accessed_at,
                    pinned: item.pinned,
                    favorite: item.favorite,
                })
                .collect();
            Ok(clipboard_items)
        }
        Ok(IpcMessage::Error { message }) => Err(format!("Daemon error: {}", message)),
        Err(e) => Err(format!("IPC error: {}", e)),
        _ => Err("Unexpected response from daemon".to_string()),
    }
}

#[tauri::command]
async fn set_clipboard_content(content: String) -> Result<(), String> {
    let socket_path = get_ipc_socket_path();
    let client = IpcClient::new(socket_path.to_string_lossy().to_string());

    match client
        .send(IpcMessage::SetClipboard {
            content: content.into_bytes(),
        })
        .await
    {
        Ok(IpcMessage::ClipboardContent { .. }) => Ok(()),
        Ok(IpcMessage::Error { message }) => Err(format!("Daemon error: {}", message)),
        Err(e) => Err(format!("IPC error: {}", e)),
        _ => Err("Unexpected response from daemon".to_string()),
    }
}

#[tauri::command]
async fn get_current_clipboard() -> Result<String, String> {
    let socket_path = get_ipc_socket_path();
    let client = IpcClient::new(socket_path.to_string_lossy().to_string());

    match client.send(IpcMessage::GetClipboard).await {
        Ok(IpcMessage::ClipboardContent { content }) => Ok(bytes_to_string("text/plain", &content)),
        Ok(IpcMessage::Error { message }) => Err(format!("Daemon error: {}", message)),
        Err(e) => Err(format!("IPC error: {}", e)),
        _ => Err("Unexpected response from daemon".to_string()),
    }
}

#[tauri::command]
async fn delete_clipboard_item(id: String) -> Result<(), String> {
    let id_i64 = id.parse::<i64>().map_err(|e| format!("Invalid ID: {}", e))?;
    let socket_path = get_ipc_socket_path();
    let client = IpcClient::new(socket_path.to_string_lossy().to_string());

    match client.send(IpcMessage::DeleteItem { id: id_i64 }).await {
        Ok(IpcMessage::Success) => Ok(()),
        Ok(IpcMessage::Error { message }) => Err(format!("Daemon error: {}", message)),
        Err(e) => Err(format!("IPC error: {}", e)),
        _ => Err("Unexpected response from daemon".to_string()),
    }
}

#[tauri::command]
async fn toggle_pin_item(id: String) -> Result<(), String> {
    let id_i64 = id.parse::<i64>().map_err(|e| format!("Invalid ID: {}", e))?;
    let socket_path = get_ipc_socket_path();
    let client = IpcClient::new(socket_path.to_string_lossy().to_string());

    match client.send(IpcMessage::TogglePin { id: id_i64 }).await {
        Ok(IpcMessage::Success) => Ok(()),
        Ok(IpcMessage::Error { message }) => Err(format!("Daemon error: {}", message)),
        Err(e) => Err(format!("IPC error: {}", e)),
        _ => Err("Unexpected response from daemon".to_string()),
    }
}

#[tauri::command]
async fn toggle_favorite_item(id: String) -> Result<(), String> {
    let id_i64 = id.parse::<i64>().map_err(|e| format!("Invalid ID: {}", e))?;
    let socket_path = get_ipc_socket_path();
    let client = IpcClient::new(socket_path.to_string_lossy().to_string());

    match client.send(IpcMessage::ToggleFavorite { id: id_i64 }).await {
        Ok(IpcMessage::Success) => Ok(()),
        Ok(IpcMessage::Error { message }) => Err(format!("Daemon error: {}", message)),
        Err(e) => Err(format!("IPC error: {}", e)),
        _ => Err("Unexpected response from daemon".to_string()),
    }
}

#[tauri::command]
async fn get_settings() -> Result<AppSettings, String> {
    let socket_path = get_ipc_socket_path();
    let client = IpcClient::new(socket_path.to_string_lossy().to_string());

    match client.send(IpcMessage::GetSettings).await {
        Ok(IpcMessage::Settings { settings }) => Ok(settings),
        Ok(IpcMessage::Error { message }) => Err(format!("Daemon error: {}", message)),
        Err(e) => Err(format!("IPC error: {}", e)),
        _ => Err("Unexpected response from daemon".to_string()),
    }
}

#[tauri::command]
async fn save_settings(settings: AppSettings) -> Result<(), String> {
    let socket_path = get_ipc_socket_path();
    let client = IpcClient::new(socket_path.to_string_lossy().to_string());

    match client.send(IpcMessage::SaveSettings { settings }).await {
        Ok(IpcMessage::Success) => Ok(()),
        Ok(IpcMessage::Error { message }) => Err(format!("Daemon error: {}", message)),
        Err(e) => Err(format!("IPC error: {}", e)),
        _ => Err("Unexpected response from daemon".to_string()),
    }
}

#[tauri::command]
async fn set_master_password(password: String) -> Result<(), String> {
    let socket_path = get_ipc_socket_path();
    let client = IpcClient::new(socket_path.to_string_lossy().to_string());

    match client.send(IpcMessage::SetMasterPassword { password }).await {
        Ok(IpcMessage::Success) => Ok(()),
        Ok(IpcMessage::Error { message }) => Err(format!("Daemon error: {}", message)),
        Err(e) => Err(format!("IPC error: {}", e)),
        _ => Err("Unexpected response from daemon".to_string()),
    }
}

#[tauri::command]
async fn unlock_vault(password: String) -> Result<(), String> {
    let socket_path = get_ipc_socket_path();
    let client = IpcClient::new(socket_path.to_string_lossy().to_string());

    match client.send(IpcMessage::UnlockVault { password }).await {
        Ok(IpcMessage::Success) => Ok(()),
        Ok(IpcMessage::Error { message }) => Err(format!("Daemon error: {}", message)),
        Err(e) => Err(format!("IPC error: {}", e)),
        _ => Err("Unexpected response from daemon".to_string()),
    }
}

#[tauri::command]
async fn check_vault_status() -> Result<bool, String> {
    let socket_path = get_ipc_socket_path();
    let client = IpcClient::new(socket_path.to_string_lossy().to_string());

    match client.send(IpcMessage::CheckVaultStatus).await {
        Ok(IpcMessage::VaultStatus { is_locked }) => Ok(is_locked),
        Ok(IpcMessage::Error { message }) => Err(format!("Daemon error: {}", message)),
        Err(e) => Err(format!("IPC error: {}", e)),
        _ => Err("Unexpected response from daemon".to_string()),
    }
}

#[tauri::command]
async fn has_master_password() -> Result<bool, String> {
    let socket_path = get_ipc_socket_path();
    let client = IpcClient::new(socket_path.to_string_lossy().to_string());

    match client.send(IpcMessage::HasMasterPassword).await {
        Ok(IpcMessage::PasswordConfigured { has_password }) => Ok(has_password),
        Ok(IpcMessage::Error { message }) => Err(format!("Daemon error: {}", message)),
        Err(e) => Err(format!("IPC error: {}", e)),
        _ => Err("Unexpected response from daemon".to_string()),
    }
}

#[tauri::command]
async fn lock_vault() -> Result<(), String> {
    let socket_path = get_ipc_socket_path();
    let client = IpcClient::new(socket_path.to_string_lossy().to_string());

    match client.send(IpcMessage::LockVault).await {
        Ok(IpcMessage::Success) => Ok(()),
        Ok(IpcMessage::Error { message }) => Err(format!("Daemon error: {}", message)),
        Err(e) => Err(format!("IPC error: {}", e)),
        _ => Err("Unexpected response from daemon".to_string()),
    }
}

/// Returns the path to the macOS LaunchAgent plist for the Tauri app.
#[cfg(target_os = "macos")]
fn launch_agent_plist_path() -> Option<std::path::PathBuf> {
    let home = dirs::home_dir()?;
    Some(home.join("Library/LaunchAgents/com.openpaste.desktop.plist"))
}

#[tauri::command]
async fn get_launch_at_login() -> Result<bool, String> {
    #[cfg(target_os = "macos")]
    {
        if let Some(plist) = launch_agent_plist_path() {
            return Ok(plist.exists());
        }
        return Ok(false);
    }
    #[cfg(not(target_os = "macos"))]
    Ok(false)
}

#[tauri::command]
async fn set_launch_at_login(enabled: bool) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let plist_path = launch_agent_plist_path()
            .ok_or("Could not determine home directory".to_string())?;

        if !enabled {
            if plist_path.exists() {
                std::fs::remove_file(&plist_path)
                    .map_err(|e| format!("Failed to remove launch agent: {}", e))?;
            }
            return Ok(());
        }

        // Get path to the current executable
        let exe = std::env::current_exe()
            .map_err(|e| format!("Could not find executable path: {}", e))?;

        let plist_content = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.openpaste.desktop</string>
    <key>ProgramArguments</key>
    <array>
        <string>{}</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <false/>
    <key>StandardErrorPath</key>
    <string>/tmp/com.openpaste.desktop.err</string>
    <key>StandardOutPath</key>
    <string>/tmp/com.openpaste.desktop.out</string>
</dict>
</plist>"#,
            exe.to_string_lossy()
        );

        // Ensure LaunchAgents directory exists
        if let Some(parent) = plist_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create LaunchAgents dir: {}", e))?;
        }

        std::fs::write(&plist_path, plist_content)
            .map_err(|e| format!("Failed to write launch agent plist: {}", e))?;

        return Ok(());
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = enabled;
        Err("Launch at login is only supported on macOS currently".to_string())
    }
}

#[tauri::command]
async fn list_tags() -> Result<Vec<TagItem>, String> {
    let client = IpcClient::new(get_ipc_socket_path().to_string_lossy().to_string());
    match client.send(IpcMessage::ListTags).await {
        Ok(IpcMessage::TagsList { tags }) => Ok(tags),
        Ok(IpcMessage::Error { message }) => Err(format!("Daemon error: {}", message)),
        Err(e) => Err(format!("IPC error: {}", e)),
        _ => Err("Unexpected response".to_string()),
    }
}

#[tauri::command]
async fn create_tag(name: String, color: Option<String>) -> Result<(), String> {
    let client = IpcClient::new(get_ipc_socket_path().to_string_lossy().to_string());
    match client.send(IpcMessage::CreateTag { name, color }).await {
        Ok(IpcMessage::Success) => Ok(()),
        Ok(IpcMessage::Error { message }) => Err(format!("Daemon error: {}", message)),
        Err(e) => Err(format!("IPC error: {}", e)),
        _ => Err("Unexpected response".to_string()),
    }
}

#[tauri::command]
async fn delete_tag(id: i64) -> Result<(), String> {
    let client = IpcClient::new(get_ipc_socket_path().to_string_lossy().to_string());
    match client.send(IpcMessage::DeleteTag { id }).await {
        Ok(IpcMessage::Success) => Ok(()),
        Ok(IpcMessage::Error { message }) => Err(format!("Daemon error: {}", message)),
        Err(e) => Err(format!("IPC error: {}", e)),
        _ => Err("Unexpected response".to_string()),
    }
}

#[tauri::command]
async fn get_item_tags(item_id: i64) -> Result<Vec<TagItem>, String> {
    let client = IpcClient::new(get_ipc_socket_path().to_string_lossy().to_string());
    match client.send(IpcMessage::GetItemTags { item_id }).await {
        Ok(IpcMessage::ItemTags { tags }) => Ok(tags),
        Ok(IpcMessage::Error { message }) => Err(format!("Daemon error: {}", message)),
        Err(e) => Err(format!("IPC error: {}", e)),
        _ => Err("Unexpected response".to_string()),
    }
}

#[tauri::command]
async fn add_tag_to_item(item_id: i64, tag_name: String, color: Option<String>) -> Result<(), String> {
    let client = IpcClient::new(get_ipc_socket_path().to_string_lossy().to_string());
    match client.send(IpcMessage::AddTagToItem { item_id, tag_name, color }).await {
        Ok(IpcMessage::Success) => Ok(()),
        Ok(IpcMessage::Error { message }) => Err(format!("Daemon error: {}", message)),
        Err(e) => Err(format!("IPC error: {}", e)),
        _ => Err("Unexpected response".to_string()),
    }
}

#[tauri::command]
async fn remove_tag_from_item(item_id: i64, tag_id: i64) -> Result<(), String> {
    let client = IpcClient::new(get_ipc_socket_path().to_string_lossy().to_string());
    match client.send(IpcMessage::RemoveTagFromItem { item_id, tag_id }).await {
        Ok(IpcMessage::Success) => Ok(()),
        Ok(IpcMessage::Error { message }) => Err(format!("Daemon error: {}", message)),
        Err(e) => Err(format!("IPC error: {}", e)),
        _ => Err("Unexpected response".to_string()),
    }
}

#[tauri::command]
async fn list_items_by_tag(tag_id: i64) -> Result<Vec<ClipboardItem>, String> {
    let client = IpcClient::new(get_ipc_socket_path().to_string_lossy().to_string());
    match client.send(IpcMessage::ListItemsByTag { tag_id }).await {
        Ok(IpcMessage::ClipboardHistory { items }) => {
            Ok(items.into_iter().map(|item| ClipboardItem {
                id: item.id,
                content_type: item.content_type.clone(),
                content: bytes_to_string(&item.content_type, &item.content),
                hash: item.hash,
                created_at: item.created_at,
                accessed_at: item.accessed_at,
                pinned: item.pinned,
                favorite: item.favorite,
            }).collect())
        }
        Ok(IpcMessage::Error { message }) => Err(format!("Daemon error: {}", message)),
        Err(e) => Err(format!("IPC error: {}", e)),
        _ => Err("Unexpected response".to_string()),
    }
}

// ── Plugin commands ───────────────────────────────────────────────────────────

#[tauri::command]
async fn list_plugins() -> Result<Vec<PluginInfoItem>, String> {
    let client = IpcClient::new(get_ipc_socket_path().to_string_lossy().to_string());
    match client.send(IpcMessage::ListPlugins).await {
        Ok(IpcMessage::PluginList { plugins }) => Ok(plugins),
        Ok(IpcMessage::Error { message }) => Err(format!("Daemon error: {}", message)),
        Err(e) => Err(format!("IPC error: {}", e)),
        _ => Err("Unexpected response".to_string()),
    }
}

#[tauri::command]
async fn load_plugin(path: String) -> Result<String, String> {
    let client = IpcClient::new(get_ipc_socket_path().to_string_lossy().to_string());
    match client.send(IpcMessage::LoadPlugin { path }).await {
        // Daemon returns ClipboardContent { content: name.into_bytes() } on success
        Ok(IpcMessage::ClipboardContent { content }) => {
            String::from_utf8(content).map_err(|e| format!("UTF-8 error: {}", e))
        }
        Ok(IpcMessage::Error { message }) => Err(format!("Daemon error: {}", message)),
        Err(e) => Err(format!("IPC error: {}", e)),
        _ => Err("Unexpected response".to_string()),
    }
}

#[tauri::command]
async fn unload_plugin(name: String) -> Result<(), String> {
    let client = IpcClient::new(get_ipc_socket_path().to_string_lossy().to_string());
    match client.send(IpcMessage::UnloadPlugin { name }).await {
        Ok(IpcMessage::Success) => Ok(()),
        Ok(IpcMessage::Error { message }) => Err(format!("Daemon error: {}", message)),
        Err(e) => Err(format!("IPC error: {}", e)),
        _ => Err("Unexpected response".to_string()),
    }
}

// ── Sync commands ─────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize)]
struct SyncConfig {
    server_url: String,
    api_token: Option<String>,
    enabled: bool,
    last_sync_at: Option<String>,
}

#[tauri::command]
async fn get_sync_config() -> Result<SyncConfig, String> {
    let client = IpcClient::new(get_ipc_socket_path().to_string_lossy().to_string());
    match client.send(IpcMessage::GetSyncConfig).await {
        Ok(IpcMessage::SyncConfig { server_url, api_token, enabled, last_sync_at }) => {
            Ok(SyncConfig { server_url, api_token, enabled, last_sync_at })
        }
        Ok(IpcMessage::Error { message }) => Err(format!("Daemon error: {}", message)),
        Err(e) => Err(format!("IPC error: {}", e)),
        _ => Err("Unexpected response".to_string()),
    }
}

#[tauri::command]
async fn set_sync_config(server_url: String, api_token: Option<String>, enabled: bool) -> Result<(), String> {
    let client = IpcClient::new(get_ipc_socket_path().to_string_lossy().to_string());
    match client.send(IpcMessage::SetSyncConfig { server_url, api_token, enabled }).await {
        Ok(IpcMessage::Success) => Ok(()),
        Ok(IpcMessage::Error { message }) => Err(format!("Daemon error: {}", message)),
        Err(e) => Err(format!("IPC error: {}", e)),
        _ => Err("Unexpected response".to_string()),
    }
}

#[tauri::command]
async fn sync_now() -> Result<String, String> {
    let client = IpcClient::new(get_ipc_socket_path().to_string_lossy().to_string());
    match client.send(IpcMessage::SyncNow).await {
        // Daemon returns ClipboardContent with "pushed=N, pulled=M" on success
        Ok(IpcMessage::ClipboardContent { content }) => {
            String::from_utf8(content).map_err(|e| format!("UTF-8 error: {}", e))
        }
        Ok(IpcMessage::Error { message }) => Err(format!("Daemon error: {}", message)),
        Err(e) => Err(format!("IPC error: {}", e)),
        _ => Err("Unexpected response".to_string()),
    }
}

fn build_tray() -> SystemTray {
    let show = CustomMenuItem::new("show".to_string(), "Show OpenPaste  ⌘⇧V");
    let quick_paste = CustomMenuItem::new("quick_paste".to_string(), "Quick Paste  ⌘⇧C");
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");

    let menu = SystemTrayMenu::new()
        .add_item(show)
        .add_item(quick_paste)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(quit);

    SystemTray::new().with_menu(menu)
}

fn show_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
        let _ = window.center();
        let _ = window.unminimize();
    }
}

fn main() {
    let tray = build_tray();

    tauri::Builder::default()
        .system_tray(tray)
        .on_system_tray_event(|app, event| match event {
            // Left-click on tray icon — toggle window
            SystemTrayEvent::LeftClick { .. } => {
                if let Some(window) = app.get_window("main") {
                    if window.is_visible().unwrap_or(false) {
                        let _ = window.hide();
                    } else {
                        show_window(app);
                    }
                }
            }
            // Tray menu items
            SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                "show" => show_window(app),
                "quick_paste" => {
                    let socket_path = get_ipc_socket_path().to_string_lossy().to_string();
                    tauri::async_runtime::spawn(async move {
                        let client = IpcClient::new(socket_path);
                        if let Ok(IpcMessage::ClipboardHistory { items }) =
                            client.send(IpcMessage::GetHistory).await
                        {
                            if let Some(item) = items.into_iter().next() {
                                if item.content_type != "encrypted" {
                                    let text = String::from_utf8_lossy(&item.content).to_string();
                                    if let Ok(mut ctx) = arboard::Clipboard::new() {
                                        let _ = ctx.set_text(&text);
                                        let preview: String = text.chars().take(40).collect();
                                        let preview = if text.len() > 40 { format!("{}…", preview) } else { preview };
                                        let _ = notify_rust::Notification::new()
                                            .summary("OpenPaste — Quick Paste")
                                            .body(&preview)
                                            .timeout(notify_rust::Timeout::Milliseconds(1800))
                                            .show();
                                    }
                                }
                            }
                        }
                    });
                }
                "quit" => {
                    let _ = app.global_shortcut_manager().unregister_all();
                    // Kill the sidecar daemon if we spawned it
                    if let Some(state) = app.try_state::<Arc<Mutex<Option<std::process::Child>>>>() {
                        if let Ok(mut guard) = state.lock() {
                            if let Some(child) = guard.as_mut() {
                                let _ = child.kill();
                            }
                        }
                    }
                    std::process::exit(0);
                }
                _ => {}
            },
            _ => {}
        })
        // Intercept close — hide to tray instead of quitting
        .on_window_event(|event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event.event() {
                api.prevent_close();
                let _ = event.window().hide();
            }
        })
        .setup(|app| {
            let app_handle = app.handle();

            // ── Daemon sidecar ───────────────────────────────────────────────
            // Probe the IPC socket. If the daemon is already running (e.g. the
            // developer started it manually) we skip spawning.  Otherwise we
            // locate and launch the daemon binary.
            let daemon_child: Arc<Mutex<Option<std::process::Child>>> =
                Arc::new(Mutex::new(None));

            let socket_path = get_ipc_socket_path();
            let daemon_already_running = {
                #[cfg(unix)]
                {
                    use std::os::unix::net::UnixStream;
                    UnixStream::connect(&socket_path).is_ok()
                }
                #[cfg(windows)]
                {
                    // Try opening the named pipe in non-blocking mode to probe
                    std::fs::OpenOptions::new()
                        .read(true)
                        .write(true)
                        .open(r"\\.\pipe\openpaste")
                        .is_ok()
                }
                #[cfg(not(any(unix, windows)))]
                { false }
            };

            if !daemon_already_running {
                // Find the daemon binary next to the current exe, or fall back
                // to the workspace target directory for dev builds.
                // On Windows the binary has a .exe extension.
                #[cfg(windows)]
                let daemon_bin_name = "openpaste-daemon.exe";
                #[cfg(not(windows))]
                let daemon_bin_name = "openpaste-daemon";

                let daemon_path = std::env::current_exe()
                    .ok()
                    .and_then(|p| p.parent().map(|d| d.join(daemon_bin_name)))
                    .filter(|p| p.exists())
                    .or_else(|| {
                        // dev: workspace root / target / debug / openpaste-daemon[.exe]
                        std::env::current_exe().ok().and_then(|p| {
                            let mut dir = p.parent()?.to_path_buf();
                            for _ in 0..8 {
                                let candidate = dir.join("target").join("debug").join(daemon_bin_name);
                                if candidate.exists() {
                                    return Some(candidate);
                                }
                                // also check release
                                let candidate = dir.join("target").join("release").join(daemon_bin_name);
                                if candidate.exists() {
                                    return Some(candidate);
                                }
                                dir = dir.parent()?.to_path_buf();
                            }
                            None
                        })
                    });

                if let Some(bin) = daemon_path {
                    eprintln!("[openpaste] spawning daemon: {:?}", bin);
                    match std::process::Command::new(&bin).spawn() {
                        Ok(child) => {
                            *daemon_child.lock().unwrap() = Some(child);
                            // Give daemon time to bind the socket
                            std::thread::sleep(std::time::Duration::from_millis(400));
                        }
                        Err(e) => eprintln!("[openpaste] failed to spawn daemon: {}", e),
                    }
                } else {
                    eprintln!("[openpaste] daemon binary not found — please start it manually");
                }
            } else {
                eprintln!("[openpaste] daemon already running, skipping spawn");
            }

            // Store child handle so the quit handler can kill it
            app_handle.manage(daemon_child.clone());

            // ── Shortcut 1: Show/hide window (Cmd+Shift+V) ──────────────────
            #[cfg(target_os = "macos")]
            let show_shortcut = "Super+Shift+V";
            #[cfg(not(target_os = "macos"))]
            let show_shortcut = "Ctrl+Shift+V";

            let app_handle_show = app_handle.clone();
            app_handle
                .global_shortcut_manager()
                .register(show_shortcut, move || {
                    show_window(&app_handle_show);
                })
                .unwrap_or_else(|e| eprintln!("Failed to register show shortcut: {}", e));

            // ── Shortcut 2: Quick-paste most recent item (Cmd+Shift+C) ──────
            // Fetches the top history item from the daemon and writes it
            // directly to the system clipboard — no window shown.
            #[cfg(target_os = "macos")]
            let paste_shortcut = "Super+Shift+C";
            #[cfg(not(target_os = "macos"))]
            let paste_shortcut = "Ctrl+Shift+C";

            app_handle
                .global_shortcut_manager()
                .register(paste_shortcut, move || {
                    let socket_path = get_ipc_socket_path()
                        .to_string_lossy()
                        .to_string();

                    // Spawn async work on Tauri's runtime
                    tauri::async_runtime::spawn(async move {
                        let client = IpcClient::new(socket_path);
                        match client.send(IpcMessage::GetHistory).await {
                            Ok(IpcMessage::ClipboardHistory { items }) => {
                                if let Some(item) = items.into_iter().next() {
                                    // Skip encrypted items — can't paste ciphertext
                                    if item.content_type == "encrypted" {
                                        let _ = notify_rust::Notification::new()
                                            .summary("OpenPaste")
                                            .body("Most recent item is encrypted. Unlock the vault first.")
                                            .timeout(notify_rust::Timeout::Milliseconds(2500))
                                            .show();
                                        return;
                                    }

                                    let text = String::from_utf8_lossy(&item.content).to_string();

                                    // Write to system clipboard via arboard
                                    match arboard::Clipboard::new() {
                                        Ok(mut ctx) => {
                                            let preview: String = text.chars().take(40).collect();
                                            let preview = if text.len() > 40 {
                                                format!("{}…", preview)
                                            } else {
                                                preview
                                            };

                                            if ctx.set_text(&text).is_ok() {
                                                let _ = notify_rust::Notification::new()
                                                    .summary("OpenPaste — Quick Paste")
                                                    .body(&preview)
                                                    .timeout(notify_rust::Timeout::Milliseconds(1800))
                                                    .show();
                                            }
                                        }
                                        Err(e) => eprintln!("Quick-paste clipboard error: {}", e),
                                    }
                                }
                            }
                            Err(e) => eprintln!("Quick-paste IPC error: {}", e),
                            _ => {}
                        }
                    });
                })
                .unwrap_or_else(|e| eprintln!("Failed to register quick-paste shortcut: {}", e));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_clipboard_history,
            search_clipboard_items,
            set_clipboard_content,
            get_current_clipboard,
            delete_clipboard_item,
            toggle_pin_item,
            toggle_favorite_item,
            get_settings,
            save_settings,
            set_master_password,
            unlock_vault,
            check_vault_status,
            lock_vault,
            has_master_password,
            get_launch_at_login,
            set_launch_at_login,
            list_tags,
            create_tag,
            delete_tag,
            get_item_tags,
            add_tag_to_item,
            remove_tag_from_item,
            list_items_by_tag,
            list_plugins,
            load_plugin,
            unload_plugin,
            get_sync_config,
            set_sync_config,
            sync_now
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
