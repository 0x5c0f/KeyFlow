use anyhow::Result;
use std::io::Write;
use std::process::Command;

use crate::config::Config;

pub fn execute() -> Result<()> {
    let config_path = Config::default_path();

    // Load existing config or create default
    let mut config = if config_path.exists() {
        Config::load(&config_path)?
    } else {
        Config {
            settings: Default::default(),
            bindings: vec![],
        }
    };

    // Check current status
    let status = Command::new("bw")
        .args(["status"])
        .output()?;

    let stdout = String::from_utf8_lossy(&status.stdout);
    if stdout.contains("\"unlocked\"") {
        println!("Bitwarden is already unlocked.");
        // Still ask if they want to refresh the session
        print!("Refresh session? [y/N] ");
        std::io::stdout().flush()?;
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            return Ok(());
        }
    }

    // Prompt for password (hidden input)
    print!("Enter Bitwarden master password: ");
    std::io::stdout().flush()?;

    let password = rpassword::read_password()?;

    if password.is_empty() {
        println!("Password cannot be empty.");
        return Ok(());
    }

    // Unlock using password via stdin
    let output = Command::new("bw")
        .args(["unlock", "--raw", "--passwordenv", "BW_PASSWORD"])
        .env("BW_PASSWORD", &password)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("Unlock failed: {stderr}");
        return Ok(());
    }

    let session = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if session.is_empty() {
        println!("Unlock failed: empty session returned.");
        return Ok(());
    }

    // Update config with new session
    config.settings.bw_session = Some(session);
    config.save(&config_path)?;

    // Set for current process
    if let Some(ref s) = config.settings.bw_session {
        std::env::set_var("BW_SESSION", s);
    }

    println!("Bitwarden unlocked successfully.");
    println!("Session saved to: {}", config_path.display());
    println!("Restart the daemon to use the new session: systemctl --user restart keyflow");

    Ok(())
}
