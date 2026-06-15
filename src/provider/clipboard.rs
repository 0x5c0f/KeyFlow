//! Clipboard-based password provider.

use crate::error::ProviderError;
use crate::provider::PasswordProvider;

/// Reads the password from the system clipboard.
pub struct ClipboardProvider;

impl ClipboardProvider {
    pub fn new() -> Self {
        Self
    }
}

impl PasswordProvider for ClipboardProvider {
    fn get_password(&self) -> Result<String, ProviderError> {
        let mut clipboard = arboard::Clipboard::new().map_err(|e| {
            ProviderError::ClipboardError(format!("Failed to access clipboard: {e}"))
        })?;

        let text = clipboard.get_text().map_err(|e| {
            ProviderError::ClipboardError(format!("Failed to read clipboard: {e}"))
        })?;

        let text = text.trim().to_string();
        if text.is_empty() {
            return Err(ProviderError::ClipboardEmpty);
        }

        Ok(text)
    }

    fn name(&self) -> &str {
        "clipboard"
    }
}
