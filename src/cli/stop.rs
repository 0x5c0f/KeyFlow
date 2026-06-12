use anyhow::Result;

pub fn execute() -> Result<()> {
    println!("To stop the daemon, press Ctrl+C or send SIGTERM.");
    Ok(())
}
