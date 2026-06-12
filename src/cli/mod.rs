//! CLI command definitions using clap.

pub mod run;
pub mod stop;
pub mod status;
pub mod bind;
pub mod config_cmd;
pub mod unlock;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "keyflow", version, about = "Non-paste password input assistant")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start the KeyFlow daemon (listens for hotkeys)
    Run {
        /// Run as background daemon
        #[arg(long)]
        daemon: bool,
    },
    /// Stop the running daemon
    Stop,
    /// Show daemon and Bitwarden status
    Status,
    /// Manage hotkey bindings
    #[command(subcommand)]
    Bind(BindCommands),
    /// Show configuration
    #[command(subcommand)]
    Config(ConfigCommands),
    /// Unlock Bitwarden vault
    Unlock,
}

#[derive(Subcommand)]
pub enum BindCommands {
    /// Add a new hotkey binding
    Add {
        /// Human-readable name
        #[arg(long)]
        name: String,
        /// Hotkey to bind (e.g., F7, F8)
        #[arg(long)]
        hotkey: String,
        /// Provider type (clipboard or bitwarden)
        #[arg(long)]
        provider: String,
        /// Item ID for the provider (required for bitwarden)
        #[arg(long)]
        item_id: Option<String>,
    },
    /// Remove a binding by name
    Remove {
        /// Name of the binding to remove
        #[arg(long)]
        name: String,
    },
    /// List all bindings
    List,
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Show current configuration
    Show,
    /// Show config file path
    Path,
}
