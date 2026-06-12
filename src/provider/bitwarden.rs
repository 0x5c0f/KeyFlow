//! Bitwarden CLI password provider.

use crate::error::ProviderError;
use crate::provider::PasswordProvider;

/// Retrieves passwords from Bitwarden via the `bw` CLI.
pub struct BitwardenProvider {
    #[allow(dead_code)] // Will be used in Task 6
    cli_path: String,
}

impl BitwardenProvider {
    pub fn new(cli_path: Option<String>) -> Self {
        Self {
            cli_path: cli_path.unwrap_or_else(|| "bw".to_string()),
        }
    }
}

impl PasswordProvider for BitwardenProvider {
    fn get_password(&self) -> Result<String, ProviderError> {
        // Stub — will be fully implemented in Task 6
        Err(ProviderError::BitwardenLocked)
    }

    fn name(&self) -> &str {
        "bitwarden"
    }
}
