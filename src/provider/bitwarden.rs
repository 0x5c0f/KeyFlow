//! Bitwarden CLI password provider.

use crate::error::ProviderError;
use crate::provider::PasswordProvider;
use std::process::Command;

/// Retrieves passwords from Bitwarden via the `bw` CLI.
///
/// Requires:
/// - `bw` CLI installed and in PATH (or custom cli_path)
/// - BW_PASSWORD env var set for auto-unlock
/// - BW_SESSION env var (auto-managed by this provider)
pub struct BitwardenProvider {
    cli_path: String,
}

impl BitwardenProvider {
    pub fn new(cli_path: Option<String>) -> Self {
        Self {
            cli_path: cli_path.unwrap_or_else(|| "bw".to_string()),
        }
    }

    /// Check if Bitwarden is unlocked by running `bw status`.
    fn is_unlocked(&self) -> bool {
        Command::new(&self.cli_path)
            .args(["status"])
            .output()
            .ok()
            .and_then(|output| {
                let stdout = String::from_utf8_lossy(&output.stdout);
                Some(stdout.contains("\"unlocked\""))
            })
            .unwrap_or(false)
    }

    /// Attempt to unlock Bitwarden using BW_PASSWORD env var.
    fn unlock(&self) -> Result<(), ProviderError> {
        let output = Command::new(&self.cli_path)
            .args(["unlock", "--passwordenv", "BW_PASSWORD", "--raw"])
            .output()
            .map_err(|e| ProviderError::BitwardenCliError {
                stderr: format!("Failed to run bw unlock: {e}"),
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ProviderError::BitwardenCliError {
                stderr: stderr.to_string(),
            });
        }

        let session = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !session.is_empty() {
            std::env::set_var("BW_SESSION", &session);
        }

        Ok(())
    }

    /// Get password for a specific item ID.
    fn get_password_for_item(&self, item_id: &str) -> Result<String, ProviderError> {
        let output = Command::new(&self.cli_path)
            .args(["get", "password", item_id])
            .output()
            .map_err(|e| ProviderError::BitwardenCliError {
                stderr: format!("Failed to run bw get password: {e}"),
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("not found") || stderr.contains("Could not find") {
                return Err(ProviderError::BitwardenItemNotFound {
                    item_id: item_id.to_string(),
                });
            }
            return Err(ProviderError::BitwardenCliError {
                stderr: stderr.to_string(),
            });
        }

        let password = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if password.is_empty() {
            return Err(ProviderError::BitwardenCliError {
                stderr: "bw returned empty password".to_string(),
            });
        }

        Ok(password)
    }
}

impl PasswordProvider for BitwardenProvider {
    fn get_password(&self) -> Result<String, ProviderError> {
        // Bitwarden requires item_id, so get_password() alone is not sufficient
        Err(ProviderError::BitwardenLocked)
    }

    fn get_password_for(&self, item_id: &str) -> Result<String, ProviderError> {
        if !self.is_unlocked() {
            self.unlock()?;
        }
        self.get_password_for_item(item_id)
    }

    fn name(&self) -> &str {
        "bitwarden"
    }
}
