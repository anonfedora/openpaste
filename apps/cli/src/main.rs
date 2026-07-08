//! OpenPaste CLI binary

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "openpaste")]
#[command(about = "OpenPaste clipboard manager CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Search clipboard history
    Search { query: String },
    /// Get clipboard item by ID
    Get { id: String },
    /// Copy item to clipboard
    Copy { id: String },
    /// List clipboard items
    List {
        #[arg(short, long, default_value_t = 20)]
        limit: usize,
    },
    /// Pin an item
    Pin { id: String },
    /// Favorite an item
    Favorite { id: String },
    /// Delete an item
    Delete { id: String },
    /// Show daemon status
    Status,
    /// Start daemon
    DaemonStart,
    /// Stop daemon
    DaemonStop,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Search { query } => {
            println!("Searching for: {}", query);
            // TODO: Implement search
        }
        Commands::Get { id } => {
            println!("Getting item: {}", id);
            // TODO: Implement get
        }
        Commands::Copy { id } => {
            println!("Copying item: {}", id);
            // TODO: Implement copy
        }
        Commands::List { limit } => {
            println!("Listing {} items", limit);
            // TODO: Implement list
        }
        Commands::Pin { id } => {
            println!("Pinning item: {}", id);
            // TODO: Implement pin
        }
        Commands::Favorite { id } => {
            println!("Favoriting item: {}", id);
            // TODO: Implement favorite
        }
        Commands::Delete { id } => {
            println!("Deleting item: {}", id);
            // TODO: Implement delete
        }
        Commands::Status => {
            println!("Daemon status");
            // TODO: Implement status
        }
        Commands::DaemonStart => {
            println!("Starting daemon");
            // TODO: Implement daemon start
        }
        Commands::DaemonStop => {
            println!("Stopping daemon");
            // TODO: Implement daemon stop
        }
    }

    Ok(())
}
