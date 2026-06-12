use crate::config::binding::Binding;
use crate::config::Config;
use crate::cli::BindCommands;
use anyhow::Result;

pub fn execute(command: BindCommands) -> Result<()> {
    let config_path = Config::default_path();
    let mut config = if config_path.exists() {
        Config::load(&config_path)?
    } else {
        Config {
            settings: Default::default(),
            providers: vec![],
            bindings: vec![],
        }
    };

    match command {
        BindCommands::Add { name, hotkey, provider, item_id } => {
            let binding = Binding { name, hotkey, provider, item_id };
            config.bindings.push(binding);
            config.save(&config_path)?;
            println!("Binding added: {} ({})", config.bindings.last().unwrap().name, config.bindings.last().unwrap().hotkey);
        }
        BindCommands::Remove { name } => {
            let before = config.bindings.len();
            config.bindings.retain(|b| b.name != name);
            if config.bindings.len() < before {
                config.save(&config_path)?;
                println!("Binding removed: {name}");
            } else {
                println!("Binding not found: {name}");
            }
        }
        BindCommands::List => {
            if config.bindings.is_empty() {
                println!("No bindings configured.");
            } else {
                println!("{:<20} {:<10} {:<15} {}", "NAME", "HOTKEY", "PROVIDER", "ITEM_ID");
                println!("{}", "-".repeat(70));
                for b in &config.bindings {
                    println!(
                        "{:<20} {:<10} {:<15} {}",
                        b.name,
                        b.hotkey,
                        b.provider,
                        b.item_id.as_deref().unwrap_or("-")
                    );
                }
            }
        }
    }

    Ok(())
}
