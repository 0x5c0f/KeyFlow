use anyhow::Result;
use std::process::Command;

pub fn execute() -> Result<()> {
    // Check Bitwarden status
    let bw_status = Command::new("bw")
        .args(["status"])
        .output();

    match bw_status {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            println!("Bitwarden: {}", if stdout.contains("\"unlocked\"") {
                "unlocked"
            } else if stdout.contains("\"locked\"") {
                "locked (run: keyflow unlock)"
            } else {
                "not logged in"
            });
        }
        Err(_) => {
            println!("Bitwarden: bw CLI not found");
        }
    }

    // Check config
    let config_path = crate::config::Config::default_path();
    if config_path.exists() {
        println!("Config:    {}", config_path.display());
    } else {
        println!("Config:    not found (run: keyflow config show)");
    }

    Ok(())
}
