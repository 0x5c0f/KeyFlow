use anyhow::Result;
use clap::Parser;
use keyflow::cli::{Cli, Commands};

fn main() -> Result<()> {
    env_logger::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Run { daemon } => keyflow::cli::run::execute(daemon)?,
        Commands::Stop => keyflow::cli::stop::execute()?,
        Commands::Status => keyflow::cli::status::execute()?,
        Commands::Bind(cmd) => keyflow::cli::bind::execute(cmd)?,
        Commands::Config(cmd) => keyflow::cli::config_cmd::execute(cmd)?,
        Commands::Unlock => keyflow::cli::unlock::execute()?,
    }

    Ok(())
}
