//! Keyboard simulation via enigo.

use crate::error::InputError;
use enigo::{Enigo, Keyboard, Settings};

/// Type text by simulating keystrokes, character by character.
///
/// Uses enigo's built-in pacing (1ms between different keycodes, configurable
/// delay for same-keycode repeats). No manual batching needed.
pub fn type_text(text: &str) -> Result<(), InputError> {
    let mut enigo = Enigo::new(&Settings::default())
        .map_err(|e| InputError::KeystrokeFailed(e.to_string()))?;

    enigo
        .text(text)
        .map_err(|e| InputError::KeystrokeFailed(e.to_string()))?;

    Ok(())
}

/// Paste text from clipboard by simulating Ctrl+V.
/// This is faster and preserves formatting, but requires the text
/// to already be in the clipboard.
pub fn paste_from_clipboard() -> Result<(), InputError> {
    use enigo::{Direction, Key};

    let mut enigo = Enigo::new(&Settings::default())
        .map_err(|e| InputError::KeystrokeFailed(e.to_string()))?;

    // Simulate Ctrl+V
    enigo
        .key(Key::Control, Direction::Press)
        .map_err(|e| InputError::KeystrokeFailed(e.to_string()))?;
    enigo
        .key(Key::Unicode('v'), Direction::Click)
        .map_err(|e| InputError::KeystrokeFailed(e.to_string()))?;
    enigo
        .key(Key::Control, Direction::Release)
        .map_err(|e| InputError::KeystrokeFailed(e.to_string()))?;

    Ok(())
}
