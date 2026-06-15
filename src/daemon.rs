//! Daemon lifecycle management.
//!
//! The daemon loads config, registers hotkeys, and enters the event loop.
//! Each hotkey triggers: mouse click at cursor -> get password -> type password.

use crate::config::binding::InputMode;
use crate::config::{Config, ProviderConfig};
use crate::error::KeyflowError;
use crate::hotkey;
use crate::input::{self, InputEngine};
use crate::provider::{self, PasswordProvider};
use std::sync::Arc;

/// Run the daemon with the given config.
pub fn run(config: Config) -> Result<(), KeyflowError> {
    let input_engine: Arc<dyn InputEngine> = Arc::from(input::create_engine());
    let mut hotkey_mgr = hotkey::create_hotkey_manager()?;

    // Register each binding as a hotkey
    for binding in &config.bindings {
        let provider_config = config
            .providers
            .iter()
            .find(|p| p.provider_type == binding.provider);

        // If no explicit config found, create a default one (clipboard doesn't need config)
        let default_config = ProviderConfig {
            provider_type: binding.provider.clone(),
            cli_path: Some(String::new()),
        };
        let config_ref = provider_config.unwrap_or(&default_config);

        let provider: Option<Box<dyn PasswordProvider>> =
            provider::create_provider(config_ref);

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
        let input_mode = binding.input_mode;

        let callback: hotkey::HotkeyCallback = Box::new(move || {
            log::info!("=== Hotkey triggered: {binding_hotkey} ({binding_name}) ===");

            // 1. Get mouse position
            log::debug!("[{binding_name}] Step 1: Getting mouse position...");
            let (x, y) = match input.get_mouse_position() {
                Ok((x, y)) => {
                    log::debug!("[{binding_name}] Mouse position: ({x}, {y})");
                    (x, y)
                }
                Err(e) => {
                    log::error!("[{binding_name}] Failed to get mouse position: {e}");
                    return;
                }
            };

            // 2. Wait for hotkey modifier keys to be released before proceeding
            // This prevents Ctrl/Shift/Alt from being "stuck" when typing
            log::debug!("[{binding_name}] Step 2: Waiting for modifier keys to release...");
            std::thread::sleep(std::time::Duration::from_millis(200));

            // 3. Click at mouse position to focus the target field
            log::debug!("[{binding_name}] Step 3: Clicking at ({x}, {y}) to focus target...");
            if let Err(e) = input.click_at(x, y) {
                log::error!("[{binding_name}] Failed to click: {e}");
                return;
            }
            log::debug!("[{binding_name}] Click successful, focus settled");

            // 4. Get password from provider
            log::debug!("[{binding_name}] Step 4: Getting password from provider '{}'...", provider.name());
            let password = if let Some(ref id) = item_id {
                log::debug!("[{binding_name}] Using item_id: {id}");
                provider.get_password_for(id)
            } else {
                provider.get_password()
            };

            let password = match password {
                Ok(p) => {
                    log::debug!("[{binding_name}] Password retrieved ({} chars)", p.len());
                    p
                }
                Err(e) => {
                    log::error!("[{binding_name}] Failed to get password: {e}");
                    return;
                }
            };

            // 5. Input text using the configured mode
            log::debug!("[{binding_name}] Step 5: Input mode={input_mode:?}, text_len={}", password.len());
            let input_result = match input_mode {
                InputMode::Type | InputMode::Auto => {
                    log::debug!("[{binding_name}] Typing character by character...");
                    input.type_text(&password)
                }
                InputMode::Paste => {
                    log::debug!("[{binding_name}] Pasting via clipboard + Ctrl+V...");
                    // Write to clipboard, then simulate Ctrl+V
                    match arboard::Clipboard::new().and_then(|mut cb| cb.set_text(&password)) {
                        Ok(_) => {
                            std::thread::sleep(std::time::Duration::from_millis(50));
                            input.paste_from_clipboard()
                        }
                        Err(e) => Err(crate::error::InputError::KeystrokeFailed(e.to_string())),
                    }
                }
            };
            if let Err(e) = input_result {
                log::error!("[{binding_name}] Failed to input text: {e}");
                return;
            }

            log::info!("[{binding_name}] ✓ Password typed successfully ({} chars)", password.len());

            // 6. Clear clipboard after delay
            if clear_secs > 0 {
                let secs = clear_secs;
                let name_for_clear = binding_name.clone();
                log::debug!("[{binding_name}] Step 6: Clipboard will be cleared in {secs}s");
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_secs(secs));
                    if let Ok(mut cb) = arboard::Clipboard::new() {
                        let _ = cb.set_text("");
                        log::debug!("[{name_for_clear}] Clipboard cleared after {secs}s");
                    }
                });
            } else {
                log::debug!("[{binding_name}] Step 6: Clipboard clear disabled");
            }
        });

        hotkey_mgr.register(&binding.hotkey, callback)?;
    }

    log::info!("KeyFlow daemon running. Press Ctrl+C to stop.");

    // Handle Ctrl+C — call hotkey_mgr.stop() to break the event loop
    let stop_flag = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let stop_flag_clone = stop_flag.clone();
    ctrlc::set_handler(move || {
        stop_flag_clone.store(true, std::sync::atomic::Ordering::SeqCst);
    })
    .map_err(|e| {
        KeyflowError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
    })?;

    // Run the event loop in a thread so we can monitor the stop flag
    let running = hotkey_mgr.running_flag();
    std::thread::spawn(move || {
        while !stop_flag.load(std::sync::atomic::Ordering::SeqCst) {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        running.store(false, std::sync::atomic::Ordering::SeqCst);
    });

    hotkey_mgr.run()?;

    Ok(())
}
