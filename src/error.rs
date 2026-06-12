//! Unified error types for KeyFlow.

use std::path::PathBuf;
use thiserror::Error;

/// Top-level KeyFlow error.
#[derive(Debug, Error)]
pub enum KeyflowError {
    #[error("Config file not found: {}", .0.display())]
    ConfigNotFound(PathBuf),

    #[error("Config parse error: {0}")]
    ConfigParse(#[from] toml::de::Error),

    #[error("Config write error: {0}")]
    ConfigWrite(#[from] toml::ser::Error),

    #[error("Hotkey registration failed: {hotkey} — {reason}")]
    HotkeyRegistration { hotkey: String, reason: String },

    #[error("Provider error: {0}")]
    Provider(#[from] ProviderError),

    #[error("Input error: {0}")]
    Input(#[from] InputError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Errors from password providers.
#[derive(Debug, Error)]
pub enum ProviderError {
    #[error("Bitwarden is not unlocked. Set BW_PASSWORD env var or run: bw unlock")]
    BitwardenLocked,

    #[error("Bitwarden item not found: {item_id}")]
    BitwardenItemNotFound { item_id: String },

    #[error("Clipboard is empty")]
    ClipboardEmpty,

    #[error("bw command failed: {stderr}")]
    BitwardenCliError { stderr: String },

    #[error("Unknown provider type: {provider_type}")]
    UnknownProvider { provider_type: String },
}

/// Errors from input simulation.
#[derive(Debug, Error)]
pub enum InputError {
    #[error("Failed to simulate keystroke: {0}")]
    KeystrokeFailed(String),

    #[error("Failed to get mouse position: {0}")]
    MousePositionFailed(String),

    #[error("Failed to click: {0}")]
    ClickFailed(String),
}
