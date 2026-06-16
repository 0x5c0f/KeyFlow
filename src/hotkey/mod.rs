//! Global hotkey management.

pub mod keys;

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "macos")]
pub mod macos;

use crate::error::KeyflowError;
use std::sync::Arc;

/// Callback type for hotkey events.
pub type HotkeyCallback = Box<dyn Fn() + Send + Sync>;

use std::sync::atomic::AtomicBool;

/// Trait for global hotkey managers.
pub trait HotkeyManager: Send {
    /// Register a global hotkey with a callback.
    fn register(&mut self, hotkey: &str, callback: HotkeyCallback) -> Result<(), KeyflowError>;

    /// Start the event loop (blocks until stopped).
    fn run(&self) -> Result<(), KeyflowError>;

    /// Signal the event loop to stop.
    fn stop(&self);

    /// Get a reference to the running flag (for external stop control).
    fn running_flag(&self) -> Arc<AtomicBool>;
}

/// Create the platform-appropriate HotkeyManager.
pub fn create_hotkey_manager() -> Result<Box<dyn HotkeyManager>, KeyflowError> {
    #[cfg(target_os = "linux")]
    {
        Ok(Box::new(linux::LinuxHotkeyManager::new()?))
    }

    #[cfg(target_os = "windows")]
    {
        Ok(Box::new(windows::WindowsHotkeyManager::new()?))
    }

    #[cfg(target_os = "macos")]
    {
        Ok(Box::new(macos::MacosHotkeyManager::new()?))
    }
}
