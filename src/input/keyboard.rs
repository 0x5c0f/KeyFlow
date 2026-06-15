//! Keyboard simulation via enigo.

use crate::error::InputError;
use enigo::{Enigo, Keyboard, Settings};
use std::thread;
use std::time::Duration;

/// Number of characters per batch. Keeps the X11 event queue manageable
/// without noticeable visual delay for the user.
const BATCH_SIZE: usize = 10;

/// Delay between batches in milliseconds. Gives the target application
/// time to process each batch before the next arrives.
const BATCH_DELAY_MS: u64 = 20;

/// Type text by simulating keystrokes, character by character.
///
/// Text is sent in small batches with brief pauses between them to prevent
/// X11 event queue saturation (especially for CJK characters that require
/// keycode remapping per character).
pub fn type_text(text: &str) -> Result<(), InputError> {
    let mut enigo = Enigo::new(&Settings::default())
        .map_err(|e| InputError::KeystrokeFailed(e.to_string()))?;

    let chars: Vec<char> = text.chars().collect();
    if chars.is_empty() {
        return Ok(());
    }

    // Send in batches with delays to prevent event queue saturation
    for chunk in chars.chunks(BATCH_SIZE) {
        let batch: String = chunk.iter().collect();
        enigo
            .text(&batch)
            .map_err(|e| InputError::KeystrokeFailed(e.to_string()))?;

        // Pause between batches (skip delay after the last batch)
        if chunk.len() == BATCH_SIZE {
            thread::sleep(Duration::from_millis(BATCH_DELAY_MS));
        }
    }

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
