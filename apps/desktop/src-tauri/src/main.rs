#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use clipboard_ipc::{IpcClient, IpcMessage};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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

fn get_ipc_socket_path() -> PathBuf {
    let data_dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("openpaste");

    #[cfg(unix)]
    let socket_path = data_dir.join("openpaste.sock");

    #[cfg(windows)]
    let socket_path = data_dir.join("openpaste.pipe");

    socket_path
}

#[tauri::command]
async fn get_clipboard_history() -> Result<Vec<ClipboardItem>, String> {
    // Connect to daemon via IPC
    let socket_path = get_ipc_socket_path();
    let client = IpcClient::new(socket_path.to_string_lossy().to_string());

    match client.send(IpcMessage::GetHistory).await {
        Ok(IpcMessage::ClipboardHistory { items }) => {
            let clipboard_items: Vec<ClipboardItem> = items
                .into_iter()
                .map(|item| ClipboardItem {
                    id: item.id,
                    content_type: item.content_type,
                    content: String::from_utf8_lossy(&item.content).to_string(),
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
            let clipboard_items: Vec<ClipboardItem> = items
                .into_iter()
                .map(|item| ClipboardItem {
                    id: item.id,
                    content_type: item.content_type,
                    content: String::from_utf8_lossy(&item.content).to_string(),
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
        Ok(IpcMessage::ClipboardContent { content }) => {
            Ok(String::from_utf8_lossy(&content).to_string())
        }
        Ok(IpcMessage::Error { message }) => Err(format!("Daemon error: {}", message)),
        Err(e) => Err(format!("IPC error: {}", e)),
        _ => Err("Unexpected response from daemon".to_string()),
    }
}

#[tauri::command]
async fn delete_clipboard_item(id: i64) -> Result<(), String> {
    let socket_path = get_ipc_socket_path();
    let client = IpcClient::new(socket_path.to_string_lossy().to_string());

    match client.send(IpcMessage::DeleteItem { id }).await {
        Ok(IpcMessage::Success) => Ok(()),
        Ok(IpcMessage::Error { message }) => Err(format!("Daemon error: {}", message)),
        Err(e) => Err(format!("IPC error: {}", e)),
        _ => Err("Unexpected response from daemon".to_string()),
    }
}

#[tauri::command]
async fn toggle_pin_item(id: i64) -> Result<(), String> {
    let socket_path = get_ipc_socket_path();
    let client = IpcClient::new(socket_path.to_string_lossy().to_string());

    match client.send(IpcMessage::TogglePin { id }).await {
        Ok(IpcMessage::Success) => Ok(()),
        Ok(IpcMessage::Error { message }) => Err(format!("Daemon error: {}", message)),
        Err(e) => Err(format!("IPC error: {}", e)),
        _ => Err("Unexpected response from daemon".to_string()),
    }
}

#[tauri::command]
async fn toggle_favorite_item(id: i64) -> Result<(), String> {
    let socket_path = get_ipc_socket_path();
    let client = IpcClient::new(socket_path.to_string_lossy().to_string());

    match client.send(IpcMessage::ToggleFavorite { id }).await {
        Ok(IpcMessage::Success) => Ok(()),
        Ok(IpcMessage::Error { message }) => Err(format!("Daemon error: {}", message)),
        Err(e) => Err(format!("IPC error: {}", e)),
        _ => Err("Unexpected response from daemon".to_string()),
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_clipboard_history,
            search_clipboard_items,
            set_clipboard_content,
            get_current_clipboard,
            delete_clipboard_item,
            toggle_pin_item,
            toggle_favorite_item
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
