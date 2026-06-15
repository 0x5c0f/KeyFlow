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
    /// Input mode — how text is delivered to the target field.
    #[serde(default)]
    pub input_mode: InputMode,
}
