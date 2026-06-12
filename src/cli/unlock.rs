use anyhow::Result;
use std::process::Command;

pub fn execute() -> Result<()> {
    // Check if already unlocked
    let status = Command::new("bw")
        .args(["status"])
        .output()?;

    let stdout = String::from_utf8_lossy(&status.stdout);
    if stdout.contains("\"unlocked\"") {
        println!("Bitwarden is already unlocked.");
        return Ok(());
    }

    // Try to unlock using BW_PASSWORD env var
    if std::env::var("BW_PASSWORD").is_ok() {
        let output = Command::new("bw")
            .args(["unlock", "--passwordenv", "BW_PASSWORD", "--raw"])
            .output()?;

        if output.status.success() {
            let session = String::from_utf8_lossy(&output.stdout).trim().to_string();
            println!("Bitwarden unlocked successfully.");
            println!("Run: export BW_SESSION={session}");
            return Ok(());
        }
    }

    // Fallback: interactive unlock
    println!("BW_PASSWORD not set. Running interactive unlock...");
    let status = Command::new("bw")
        .args(["unlock"])
        .status()?;

    if status.success() {
        println!("Bitwarden unlocked. Set BW_SESSION from the output above.");
    } else {
        println!("Unlock failed.");
    }

    Ok(())
}
