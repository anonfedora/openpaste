//! OpenPaste CLI binary

use anyhow::{anyhow, Result};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use clap::{Parser, Subcommand};
use clipboard_ipc::{IpcClient, IpcMessage};
use colored::Colorize;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "openpaste")]
#[command(about = "OpenPaste clipboard manager CLI", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List clipboard history
    List {
        #[arg(short, long, default_value_t = 20)]
        limit: usize,
    },
    /// Search clipboard history
    Search {
        /// Search query
        query: String,
    },
    /// Copy a clipboard item back to the clipboard by ID
    Copy {
        /// Item ID
        id: String,
    },
    /// Print item content to stdout by ID
    Get {
        /// Item ID
        id: String,
    },
    /// Pin or unpin an item
    Pin {
        /// Item ID
        id: String,
    },
    /// Favorite or unfavorite an item
    Favorite {
        /// Item ID
        id: String,
    },
    /// Delete an item
    Delete {
        /// Item ID
        id: String,
    },
    /// Show daemon connection status
    Status,
}

fn get_socket_path() -> String {
    let data_dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("openpaste");

    data_dir
        .join("openpaste.sock")
        .to_string_lossy()
        .to_string()
}

async fn client() -> IpcClient {
    IpcClient::new(get_socket_path())
}

fn truncate(s: &str, max: usize) -> String {
    let s = s.trim();
    if s.chars().count() > max {
        format!("{}…", s.chars().take(max).collect::<String>())
    } else {
        s.to_string()
    }
}

fn type_badge(content_type: &str) -> colored::ColoredString {
    match content_type {
        "image" => "IMAGE".magenta(),
        "code" => "CODE".yellow(),
        "html" => "HTML".cyan(),
        _ => {
            // check URL heuristic from content_type string alone
            "TEXT".blue()
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Status => {
            let c = client().await;
            match c.send(IpcMessage::GetHistory).await {
                Ok(IpcMessage::ClipboardHistory { items }) => {
                    println!("{} daemon is running — {} items in history",
                        "✓".green().bold(),
                        items.len().to_string().bold()
                    );
                }
                Err(e) => {
                    eprintln!("{} daemon is not reachable: {}", "✗".red().bold(), e);
                    std::process::exit(1);
                }
                _ => {
                    eprintln!("{} unexpected response from daemon", "✗".red().bold());
                    std::process::exit(1);
                }
            }
        }

        Commands::List { limit } => {
            let c = client().await;
            match c.send(IpcMessage::GetHistory).await {
                Ok(IpcMessage::ClipboardHistory { items }) => {
                    if items.is_empty() {
                        println!("{}", "No clipboard history.".dimmed());
                        return Ok(());
                    }
                    println!("{}", format!("{:<6}  {:<8}  {:<12}  {}", "ID", "TYPE", "WHEN", "CONTENT").dimmed());
                    println!("{}", "─".repeat(72).dimmed());
                    for item in items.iter().take(limit) {
                        let content_str = if item.content_type == "image" {
                            "(image)".to_string()
                        } else {
                            truncate(&String::from_utf8_lossy(&item.content), 50)
                        };
                        let when = item.created_at.get(..16).unwrap_or(&item.created_at);
                        let pin = if item.pinned { "📌 " } else { "" };
                        let fav = if item.favorite { "⭐ " } else { "" };
                        println!(
                            "{:<6}  {:<8}  {:<12}  {}{}{}",
                            item.id.bold(),
                            type_badge(&item.content_type),
                            when.dimmed(),
                            pin, fav,
                            content_str
                        );
                    }
                }
                Ok(IpcMessage::Error { message }) => return Err(anyhow!("Daemon error: {}", message)),
                Err(e) => return Err(anyhow!("IPC error: {}", e)),
                _ => return Err(anyhow!("Unexpected response")),
            }
        }

        Commands::Search { query } => {
            let c = client().await;
            match c.send(IpcMessage::SearchItems { query: query.clone() }).await {
                Ok(IpcMessage::ClipboardHistory { items }) => {
                    if items.is_empty() {
                        println!("{}", format!("No results for '{}'.", query).dimmed());
                        return Ok(());
                    }
                    println!("{}", format!("{:<6}  {:<8}  {:<12}  {}", "ID", "TYPE", "WHEN", "CONTENT").dimmed());
                    println!("{}", "─".repeat(72).dimmed());
                    for item in &items {
                        let content_str = if item.content_type == "image" {
                            "(image)".to_string()
                        } else {
                            truncate(&String::from_utf8_lossy(&item.content), 50)
                        };
                        let when = item.created_at.get(..16).unwrap_or(&item.created_at);
                        println!(
                            "{:<6}  {:<8}  {:<12}  {}",
                            item.id.bold(),
                            type_badge(&item.content_type),
                            when.dimmed(),
                            content_str
                        );
                    }
                }
                Ok(IpcMessage::Error { message }) => return Err(anyhow!("Daemon error: {}", message)),
                Err(e) => return Err(anyhow!("IPC error: {}", e)),
                _ => return Err(anyhow!("Unexpected response")),
            }
        }

        Commands::Get { id } => {
            // Fetch full list and find by ID (no single-item IPC yet)
            let c = client().await;
            match c.send(IpcMessage::GetHistory).await {
                Ok(IpcMessage::ClipboardHistory { items }) => {
                    let item = items.iter().find(|i| i.id == id)
                        .ok_or_else(|| anyhow!("Item {} not found", id))?;
                    if item.content_type == "image" {
                        // Write PNG to stdout (pipe to file if needed)
                        let bytes = STANDARD.decode(&item.content)
                            .unwrap_or_else(|_| item.content.clone());
                        use std::io::Write;
                        std::io::stdout().write_all(&bytes)?;
                    } else {
                        print!("{}", String::from_utf8_lossy(&item.content));
                    }
                }
                Ok(IpcMessage::Error { message }) => return Err(anyhow!("Daemon error: {}", message)),
                Err(e) => return Err(anyhow!("IPC error: {}", e)),
                _ => return Err(anyhow!("Unexpected response")),
            }
        }

        Commands::Copy { id } => {
            // Fetch item content then push it back to clipboard via SetClipboard
            let c = client().await;
            let content = match c.send(IpcMessage::GetHistory).await {
                Ok(IpcMessage::ClipboardHistory { items }) => {
                    items.into_iter().find(|i| i.id == id)
                        .map(|i| i.content)
                        .ok_or_else(|| anyhow!("Item {} not found", id))?
                }
                Ok(IpcMessage::Error { message }) => return Err(anyhow!("Daemon error: {}", message)),
                Err(e) => return Err(anyhow!("IPC error: {}", e)),
                _ => return Err(anyhow!("Unexpected response")),
            };

            let c2 = client().await;
            match c2.send(IpcMessage::SetClipboard { content }).await {
                Ok(IpcMessage::ClipboardContent { .. }) => {
                    println!("{} Copied item {} to clipboard", "✓".green().bold(), id.bold());
                }
                Ok(IpcMessage::Error { message }) => return Err(anyhow!("Daemon error: {}", message)),
                Err(e) => return Err(anyhow!("IPC error: {}", e)),
                _ => return Err(anyhow!("Unexpected response")),
            }
        }

        Commands::Pin { id } => {
            let id_i64: i64 = id.parse().map_err(|_| anyhow!("Invalid ID: {}", id))?;
            let c = client().await;
            match c.send(IpcMessage::TogglePin { id: id_i64 }).await {
                Ok(IpcMessage::Success) => {
                    println!("{} Toggled pin on item {}", "✓".green().bold(), id.bold());
                }
                Ok(IpcMessage::Error { message }) => return Err(anyhow!("Daemon error: {}", message)),
                Err(e) => return Err(anyhow!("IPC error: {}", e)),
                _ => return Err(anyhow!("Unexpected response")),
            }
        }

        Commands::Favorite { id } => {
            let id_i64: i64 = id.parse().map_err(|_| anyhow!("Invalid ID: {}", id))?;
            let c = client().await;
            match c.send(IpcMessage::ToggleFavorite { id: id_i64 }).await {
                Ok(IpcMessage::Success) => {
                    println!("{} Toggled favorite on item {}", "✓".green().bold(), id.bold());
                }
                Ok(IpcMessage::Error { message }) => return Err(anyhow!("Daemon error: {}", message)),
                Err(e) => return Err(anyhow!("IPC error: {}", e)),
                _ => return Err(anyhow!("Unexpected response")),
            }
        }

        Commands::Delete { id } => {
            let id_i64: i64 = id.parse().map_err(|_| anyhow!("Invalid ID: {}", id))?;
            let c = client().await;
            match c.send(IpcMessage::DeleteItem { id: id_i64 }).await {
                Ok(IpcMessage::Success) => {
                    println!("{} Deleted item {}", "✓".green().bold(), id.bold());
                }
                Ok(IpcMessage::Error { message }) => return Err(anyhow!("Daemon error: {}", message)),
                Err(e) => return Err(anyhow!("IPC error: {}", e)),
                _ => return Err(anyhow!("Unexpected response")),
            }
        }
    }

    Ok(())
}
