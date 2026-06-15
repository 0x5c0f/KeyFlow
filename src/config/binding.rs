//! Hotkey binding definition.

use serde::{Deserialize, Serialize};

/// Input mode for a binding — controls how text is delivered to the target field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum InputMode {
    /// Default — currently equivalent to `type`. Reserved for future heuristics.
    Auto,
    /// Character-by-character XTEST input. Works in paste-disabled fields.
    Type,
    /// Ctrl+V clipboard paste. Preserves formatting, fast for CJK.
    Paste,
}

impl Default for InputMode {
    fn default() -> Self {
        Self::Auto
    }
}

/// A binding maps a hotkey to a password provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Binding {
    /// Human-readable name for this binding.
    pub name: String,
    /// Hotkey to trigger this binding (e.g., "F7", "F8").
    pub hotkey: String,
    /// Provider type to use ("clipboard" or "bitwarden").
    pub provider: String,
    /// Item ID for provider-specific lookup (e.g., Bitwarden item ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_id: Option<String>,
    /// Optional path to the provider CLI binary (e.g. bw CLI path).
    /// If not set, auto-searches common locations for known providers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cli_path: Option<String>,
    /// Input mode — how text is delivered to the target field.
    #[serde(default)]
    pub input_mode: InputMode,
    /// Seconds to wait before clearing clipboard after input.
    /// `Some(0)` = don't clear. `None` = fall back to global setting.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clipboard_clear_after_secs: Option<u64>,
    /// Password cache duration in seconds. `0` = no cache. Only effective for providers
    /// that support caching (e.g. bitwarden). Reduces repeated CLI invocations.
    #[serde(default)]
    pub cache_secs: Option<u64>,
}
