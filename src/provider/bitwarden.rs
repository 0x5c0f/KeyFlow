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

/// Common locations where the `bw` CLI might be installed.
const BW_SEARCH_PATHS: &[&str] = &[
    "/usr/bin/bw",
    "/usr/local/bin/bw",
    "/opt/homebrew/bin/bw",
    "/snap/bin/bw",
];

impl BitwardenProvider {
    pub fn new(cli_path: Option<String>) -> Self {
        let resolved = cli_path
            .filter(|p| !p.is_empty())
            .unwrap_or_else(Self::find_bw_cli);
        Self { cli_path: resolved }
    }

    /// Search for `bw` CLI in common locations, then fall back to `bw` (rely on PATH).
    fn find_bw_cli() -> String {
        // 1. Check common absolute paths
        for path in BW_SEARCH_PATHS {
            if std::path::Path::new(path).exists() {
                log::debug!("Found bw CLI at: {path}");
                return path.to_string();
            }
        }
        // 2. Check ~/.local/bin/bw
        if let Some(home) = dirs::home_dir() {
            let local_bin = home.join(".local/bin/bw");
            if local_bin.exists() {
                let p = local_bin.to_string_lossy().to_string();
                log::debug!("Found bw CLI at: {p}");
                return p;
            }
        }
        // 3. Fall back to bare "bw" (rely on PATH lookup)
        log::debug!("bw CLI not found in common locations, falling back to PATH");
        "bw".to_string()
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
        // Skip `bw status` check — directly attempt to get password.
        // If it fails (locked or empty), unlock and retry. This saves ~1s per invocation.
        match self.get_password_for_item(item_id) {
            Ok(password) => Ok(password),
            Err(ProviderError::BitwardenCliError { ref stderr })
                if stderr.contains("not logged in")
                    || stderr.contains("locked")
                    || stderr.contains("You are not logged in")
                    || stderr.contains("empty password") =>
            {
                log::debug!("Bitwarden locked/not logged in/empty, attempting unlock...");
                self.unlock()?;
                self.get_password_for_item(item_id)
            }
            Err(e) => {
                log::error!("Bitwarden error: {e}. Run 'keyflow unlock' to refresh session.");
                Err(e)
            }
        }
    }

    fn name(&self) -> &str {
        "bitwarden"
    }
}
