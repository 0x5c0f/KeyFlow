use crate::config::Config;
use crate::daemon;
use anyhow::Result;

pub fn execute(daemon_mode: bool) -> Result<()> {
    let config_path = Config::default_path();
    let config = Config::load(&config_path)?;

    if daemon_mode {
        log::info!("Starting in daemon mode...");
    }

    daemon::run(config)?;
    Ok(())
}
