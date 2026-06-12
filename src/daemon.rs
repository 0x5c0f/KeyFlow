//! Daemon lifecycle management.
//!
//! The daemon loads config, registers hotkeys, and enters the event loop.
//! Each hotkey triggers: mouse click at cursor -> get password -> type password.

use crate::config::Config;
use crate::error::KeyflowError;
use crate::hotkey;
use crate::input::{self, InputEngine};
use crate::provider::{self, PasswordProvider};
use std::sync::Arc;

/// Run the daemon with the given config.
pub fn run(config: Config) -> Result<(), KeyflowError> {
    let input_engine: Arc<dyn InputEngine> = Arc::from(input::create_engine());
    let mut hotkey_mgr = hotkey::create_hotkey_manager();

    // Register each binding as a hotkey
    for binding in &config.bindings {
        let provider_config = config
            .providers
            .iter()
            .find(|p| p.provider_type == binding.provider);

        let provider: Option<Box<dyn PasswordProvider>> =
            provider_config.and_then(|pc| provider::create_provider(pc));

        let provider = match provider {
            Some(p) => p,
            None => {
                log::warn!(
                    "Skipping binding '{}': unknown provider '{}'",
                    binding.name,
                    binding.provider
                );
                continue;
            }
        };

        let input = Arc::clone(&input_engine);
        let binding_name = binding.name.clone();
        let binding_hotkey = binding.hotkey.clone();
        let item_id = binding.item_id.clone();
        let clear_secs = config.settings.clipboard_clear_after_secs;

        let callback: hotkey::HotkeyCallback = Box::new(move || {
            log::info!("Hotkey triggered: {binding_hotkey} ({binding_name})");

            // 1. Get mouse position
            let (x, y) = match input.get_mouse_position() {
                Ok(pos) => pos,
                Err(e) => {
                    log::error!("Failed to get mouse position: {e}");
                    return;
                }
            };

            // 2. Click at mouse position to focus the target field
            if let Err(e) = input.click_at(x, y) {
                log::error!("Failed to click: {e}");
                return;
            }

            // 3. Get password from provider
            let password = if let Some(ref id) = item_id {
                provider.get_password_for(id)
            } else {
                provider.get_password()
            };

            let password = match password {
                Ok(p) => p,
                Err(e) => {
                    log::error!("Failed to get password: {e}");
                    return;
                }
            };

            // 4. Type the password
            if let Err(e) = input.type_text(&password) {
                log::error!("Failed to type password: {e}");
                return;
            }

            log::info!("Password typed successfully for: {binding_name}");

            // 5. Clear clipboard after delay
            if clear_secs > 0 {
                let secs = clear_secs;
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_secs(secs));
                    if let Ok(mut cb) = arboard::Clipboard::new() {
                        let _ = cb.set_text("");
                        log::debug!("Clipboard cleared after {secs}s");
                    }
                });
            }
        });

        hotkey_mgr.register(&binding.hotkey, callback)?;
    }

    log::info!("KeyFlow daemon running. Press Ctrl+C to stop.");

    // Handle Ctrl+C
    let running = Arc::new(std::sync::atomic::AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, std::sync::atomic::Ordering::SeqCst);
    })
    .map_err(|e| {
        KeyflowError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
    })?;

    // Run the event loop
    hotkey_mgr.run()?;

    Ok(())
}
