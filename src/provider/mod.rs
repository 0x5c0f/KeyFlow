//! Password provider abstraction.

pub mod bitwarden;
pub mod clipboard;

use crate::config::ProviderConfig;
use crate::error::ProviderError;

/// Trait for password providers.
///
/// Each provider knows how to retrieve a password from its source
/// (clipboard, Bitwarden CLI, etc.).
pub trait PasswordProvider: Send + Sync {
    /// Retrieve the password.
    fn get_password(&self) -> Result<String, ProviderError>;

    /// Retrieve password for a specific item (for providers like Bitwarden).
    /// Default implementation calls get_password().
    fn get_password_for(&self, _item_id: &str) -> Result<String, ProviderError> {
        self.get_password()
    }

    /// Human-readable name of this provider.
    fn name(&self) -> &str;
}

/// Create a provider from config. Returns None for unknown types.
pub fn create_provider(config: &ProviderConfig) -> Option<Box<dyn PasswordProvider>> {
    match config.provider_type.as_str() {
        "clipboard" => Some(Box::new(clipboard::ClipboardProvider::new())),
        "bitwarden" => Some(Box::new(bitwarden::BitwardenProvider::new(
            config.cli_path.clone(),
        ))),
        _ => None,
    }
}
