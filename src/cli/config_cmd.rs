use crate::config::Config;
use crate::cli::ConfigCommands;
use anyhow::Result;

pub fn execute(command: ConfigCommands) -> Result<()> {
    match command {
        ConfigCommands::Show => {
            let config_path = Config::default_path();
            if config_path.exists() {
                let content = std::fs::read_to_string(&config_path)?;
                println!("{content}");
            } else {
                println!("No config file found at: {}", config_path.display());
            }
        }
        ConfigCommands::Path => {
            println!("{}", Config::default_path().display());
        }
    }
    Ok(())
}
