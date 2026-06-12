//! Hotkey binding definition.

use serde::{Deserialize, Serialize};

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
}
