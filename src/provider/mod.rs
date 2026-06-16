//! Password provider abstraction.

pub mod bitwarden;
pub mod cached;
pub mod clipboard;
pub mod static_provider;

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

/// Create a provider by type name and optional CLI path.
/// Returns None for unknown provider types.
pub fn create_provider(
    provider_type: &str,
    cli_path: Option<String>,
) -> Option<Box<dyn PasswordProvider>> {
    match provider_type {
        "clipboard" => Some(Box::new(clipboard::ClipboardProvider::new())),
        "bitwarden" => Some(Box::new(bitwarden::BitwardenProvider::new(cli_path))),
        _ => None,
    }
}

/// Create a static provider with content and encryption settings.
pub fn create_static_provider(
    content: String,
    encrypted: bool,
    encryption_key: Option<String>,
) -> Box<dyn PasswordProvider> {
    Box::new(static_provider::StaticProvider::new(
        content,
        encrypted,
        encryption_key,
    ))
}
