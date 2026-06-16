//! Configuration management for KeyFlow.

pub mod binding;

use crate::config::binding::Binding;
use crate::error::KeyflowError;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Top-level configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub settings: Settings,
    #[serde(default)]
    pub bindings: Vec<Binding>,
}

/// Global settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// Seconds to wait before clearing clipboard after input. 0 = don't clear.
    #[serde(default = "default_clipboard_clear")]
    pub clipboard_clear_after_secs: u64,
    /// Bitwarden session key for vault access.
    /// Set via `keyflow unlock` or manually from `bw unlock --raw`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bw_session: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            clipboard_clear_after_secs: default_clipboard_clear(),
            bw_session: None,
        }
    }
}

fn default_clipboard_clear() -> u64 {
    5
}

impl Config {
    /// Get the default config file path for the current platform.
    pub fn default_path() -> PathBuf {
        let config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        config_dir.join("keyflow").join("keyflow.toml")
    }

    /// Load config from a TOML file.
    pub fn load(path: &std::path::Path) -> Result<Self, KeyflowError> {
        let content = std::fs::read_to_string(path)
            .map_err(|_| KeyflowError::ConfigNotFound(path.to_path_buf()))?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Save config to a TOML file.
    pub fn save(&self, path: &std::path::Path) -> Result<(), KeyflowError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Find a binding by hotkey name.
    pub fn find_binding(&self, hotkey: &str) -> Option<&Binding> {
        self.bindings.iter().find(|b| b.hotkey == hotkey)
    }
}
