use anyhow::Result;
use clap::Parser;
use keyflow::cli::{Cli, Commands};

fn main() -> Result<()> {
    // Filter logs: keyflow uses user-specified level, third-party crates only show warnings.
    // This prevents sensitive data (e.g. passwords) from appearing in third-party debug logs.
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .filter_module("enigo", log::LevelFilter::Warn)
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Run { daemon } => keyflow::cli::run::execute(daemon)?,
        Commands::Stop => keyflow::cli::stop::execute()?,
        Commands::Status => keyflow::cli::status::execute()?,
        Commands::Bind(cmd) => keyflow::cli::bind::execute(cmd)?,
        Commands::Config(cmd) => keyflow::cli::config_cmd::execute(cmd)?,
        Commands::Unlock => keyflow::cli::unlock::execute()?,
        Commands::Encrypt { plaintext } => keyflow::cli::encrypt::execute(plaintext)?,
    }

    Ok(())
}
