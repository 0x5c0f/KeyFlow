//! Mouse position and click simulation.

use crate::error::InputError;
use enigo::{Coordinate, Enigo, Mouse, Settings, Button, Direction};

/// Get the current mouse cursor position.
pub fn get_mouse_position() -> Result<(i32, i32), InputError> {
    let enigo = Enigo::new(&Settings::default())
        .map_err(|e| InputError::MousePositionFailed(e.to_string()))?;

    let (x, y) = enigo
        .location()
        .map_err(|e| InputError::MousePositionFailed(e.to_string()))?;

    Ok((x, y))
}

/// Click at the specified screen coordinates.
pub fn click_at(x: i32, y: i32) -> Result<(), InputError> {
    let mut enigo = Enigo::new(&Settings::default())
        .map_err(|e| InputError::ClickFailed(e.to_string()))?;

    // Move mouse to target position
    enigo
        .move_mouse(x, y, Coordinate::Abs)
        .map_err(|e| InputError::ClickFailed(e.to_string()))?;

    // Small delay for focus to settle
    std::thread::sleep(std::time::Duration::from_millis(50));

    // Click
    enigo
        .button(Button::Left, Direction::Click)
        .map_err(|e| InputError::ClickFailed(e.to_string()))?;

    // Small delay after click for focus to take effect
    std::thread::sleep(std::time::Duration::from_millis(100));

    Ok(())
}
