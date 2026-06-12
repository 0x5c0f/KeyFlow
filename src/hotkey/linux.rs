//! Linux global hotkey implementation.
//!
//! Uses X11 XGrabKey for global hotkey registration.
//! Wayland support will be added later via libei.

use crate::error::KeyflowError;
use crate::hotkey::{HotkeyCallback, HotkeyManager};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct LinuxHotkeyManager {
    callbacks: HashMap<String, HotkeyCallback>,
    running: Arc<AtomicBool>,
}

impl LinuxHotkeyManager {
    pub fn new() -> Self {
        Self {
            callbacks: HashMap::new(),
            running: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl HotkeyManager for LinuxHotkeyManager {
    fn register(&mut self, hotkey: &str, callback: HotkeyCallback) -> Result<(), KeyflowError> {
        // Parse hotkey string (e.g., "F7", "F8") to X11 keysym
        // Register with XGrabKey
        // For now, just store the callback
        self.callbacks.insert(hotkey.to_string(), callback);
        log::info!("Registered hotkey: {hotkey}");
        Ok(())
    }

    fn run(&self) -> Result<(), KeyflowError> {
        self.running.store(true, Ordering::SeqCst);
        log::info!("Hotkey manager started (Linux X11)");

        // X11 event loop — poll for KeyPress events
        // This is a simplified version; real implementation would use X11 bindings
        while self.running.load(Ordering::SeqCst) {
            std::thread::sleep(std::time::Duration::from_millis(100));
            // In real implementation: XNextEvent -> match keysym -> invoke callback
        }

        log::info!("Hotkey manager stopped");
        Ok(())
    }

    fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    fn running_flag(&self) -> Arc<AtomicBool> {
        self.running.clone()
    }
}
