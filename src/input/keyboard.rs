//! Keyboard simulation via enigo.

use crate::error::InputError;
use enigo::{Enigo, Keyboard, Settings};

/// Type text by simulating keystrokes.
pub fn type_text(text: &str) -> Result<(), InputError> {
    let mut enigo = Enigo::new(&Settings::default())
        .map_err(|e| InputError::KeystrokeFailed(e.to_string()))?;

    enigo
        .text(text)
        .map_err(|e| InputError::KeystrokeFailed(e.to_string()))?;

    Ok(())
}
