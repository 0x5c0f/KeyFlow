//! Encrypt command — encrypt text for use in static bindings.

use crate::config::Config;
use crate::crypto;
use anyhow::{bail, Result};
use std::io::{self, Read};

/// Execute the encrypt command.
///
/// Reads plaintext from argument or stdin, encrypts with the configured key,
/// and outputs the encrypted string.
pub fn execute(plaintext: Option<String>) -> Result<()> {
    // Load config to get encryption_key
    let config_path = Config::default_path();
    let config = if config_path.exists() {
        Config::load(&config_path)?
    } else {
        bail!(
            "Config file not found at: {}\nRun 'keyflow config init' first.",
            config_path.display()
        );
    };

    let key = match config.settings.encryption_key {
        Some(ref k) if !k.is_empty() => k.clone(),
        _ => bail!(
            "encryption_key not configured in [settings].\n\
             Add to {}:\n\n\
             [settings]\n\
             encryption_key = \"your-secret-key\"",
            config_path.display()
        ),
    };

    // Get plaintext from argument or stdin
    let text = match plaintext {
        Some(t) => t,
        None => {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf)?;
            buf.trim().to_string()
        }
    };

    if text.is_empty() {
        bail!("Plaintext is empty. Provide text as argument or via stdin.");
    }

    // Encrypt
    let encrypted = crypto::encrypt(&text, &key)?;

    // Output
    println!("{encrypted}");

    Ok(())
}
