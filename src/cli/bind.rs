use crate::config::binding::{Binding, InputMode};
use crate::config::Config;
use crate::cli::BindCommands;
use anyhow::Result;

/// Calculate the display width of a string, accounting for CJK characters (2 columns each).
fn display_width(s: &str) -> usize {
    s.chars().map(|c| {
        if c.is_ascii() { 1 } else { 2 }
    }).sum()
}

/// Pad a string to a target display width with spaces.
fn pad_to_width(s: &str, width: usize) -> String {
    let current = display_width(s);
    if current >= width {
        s.to_string()
    } else {
        format!("{}{}", s, " ".repeat(width - current))
    }
}

pub fn execute(command: BindCommands) -> Result<()> {
    let config_path = Config::default_path();
    let mut config = if config_path.exists() {
        Config::load(&config_path)?
    } else {
        Config {
            settings: Default::default(),
            bindings: vec![],
        }
    };

    match command {
        BindCommands::Add { name, hotkey, provider, item_id } => {
            let binding = Binding {
                name,
                hotkey,
                provider,
                item_id,
                content: None,
                encrypted: false,
                cli_path: None,
                input_mode: InputMode::default(),
                clipboard_clear_after_secs: None,
                cache_secs: None,
            };
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
                // Column widths (display width)
                let w_name = 20;
                let w_hotkey = 18;
                let w_provider = 12;
                let w_mode = 8;
                let w_clear = 8;

                // Header
                println!(
                    "{} {} {} {} {}",
                    pad_to_width("NAME", w_name),
                    pad_to_width("HOTKEY", w_hotkey),
                    pad_to_width("PROVIDER", w_provider),
                    pad_to_width("MODE", w_mode),
                    pad_to_width("CLEAR", w_clear),
                );
                println!("{}", "-".repeat(w_name + w_hotkey + w_provider + w_mode + w_clear + 4));

                // Rows
                for b in &config.bindings {
                    let mode_str = format!("{:?}", b.input_mode).to_lowercase();
                    let clear_str = match b.clipboard_clear_after_secs {
                        Some(secs) => format!("{}s", secs),
                        None => format!("{}s*", config.settings.clipboard_clear_after_secs),
                    };
                    println!(
                        "{} {} {} {} {}",
                        pad_to_width(&b.name, w_name),
                        pad_to_width(&b.hotkey, w_hotkey),
                        pad_to_width(&b.provider, w_provider),
                        pad_to_width(&mode_str, w_mode),
                        pad_to_width(&clear_str, w_clear),
                    );
                }
            }
        }
    }

    Ok(())
}
