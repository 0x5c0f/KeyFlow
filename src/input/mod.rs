//! Input simulation — keyboard and mouse control.

pub mod keyboard;
pub mod mouse;

use crate::error::InputError;

/// Trait for input simulation (keyboard + mouse).
pub trait InputEngine: Send + Sync {
    /// Get the current mouse cursor position.
    fn get_mouse_position(&self) -> Result<(i32, i32), InputError>;

    /// Click at the specified screen coordinates.
    fn click_at(&self, x: i32, y: i32) -> Result<(), InputError>;

    /// Type text by simulating keystrokes.
    fn type_text(&self, text: &str) -> Result<(), InputError>;
}

/// Create the platform-appropriate InputEngine.
pub fn create_engine() -> Box<dyn InputEngine> {
    Box::new(EnigoEngine::new())
}

/// InputEngine implementation using the `enigo` crate.
struct EnigoEngine;

impl EnigoEngine {
    fn new() -> Self {
        Self
    }
}

impl InputEngine for EnigoEngine {
    fn get_mouse_position(&self) -> Result<(i32, i32), InputError> {
        mouse::get_mouse_position()
    }

    fn click_at(&self, x: i32, y: i32) -> Result<(), InputError> {
        mouse::click_at(x, y)
    }

    fn type_text(&self, text: &str) -> Result<(), InputError> {
        keyboard::type_text(text)
    }
}
