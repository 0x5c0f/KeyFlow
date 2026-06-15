//! Hotkey string parser.
//!
//! Parses hotkey strings like "Ctrl+Shift+F7" into (keysym, modifiers).

use crate::error::KeyflowError;

/// Parsed hotkey combination.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HotkeyCombo {
    pub keysym: u32,
    pub modifiers: u16,
}

// X11 modifier masks (from X.h)
pub const SHIFT_MASK: u16 = 1 << 0; // ShiftMask = 1
pub const CONTROL_MASK: u16 = 1 << 2; // ControlMask = 4
pub const MOD1_MASK: u16 = 1 << 3; // Mod1Mask = 8 (usually Alt)
pub const MOD4_MASK: u16 = 1 << 6; // Mod4Mask = 64 (usually Super)

/// Parse a modifier name to its X11 mask value.
fn parse_modifier(name: &str) -> Option<u16> {
    match name.to_lowercase().as_str() {
        "ctrl" | "control" => Some(CONTROL_MASK),
        "shift" => Some(SHIFT_MASK),
        "alt" => Some(MOD1_MASK),
        "super" | "win" | "meta" => Some(MOD4_MASK),
        _ => None,
    }
}

/// Map key name to X11 keysym value.
pub fn keysym_from_name(name: &str) -> Option<u32> {
    let upper = name.to_uppercase();

    // Function keys F1-F24
    if let Some(rest) = upper.strip_prefix('F') {
        if let Ok(n) = rest.parse::<u32>() {
            if (1..=24).contains(&n) {
                return Some(0xFFBE + n - 1); // XK_F1 = 0xFFBE
            }
        }
    }

    match upper.as_str() {
        // Letters A-Z (XK_a = 0x61)
        s if s.len() == 1 && s.chars().next().unwrap().is_ascii_alphabetic() => {
            let c = s.chars().next().unwrap().to_ascii_lowercase();
            Some(c as u32)
        }
        // Digits 0-9
        s if s.len() == 1 && s.chars().next().unwrap().is_ascii_digit() => {
            let c = s.chars().next().unwrap();
            Some(c as u32)
        }
        // Special keys
        "SPACE" => Some(0x0020),
        "TAB" => Some(0xFF09),
        "ESC" | "ESCAPE" => Some(0xFF1B),
        "ENTER" | "RETURN" => Some(0xFF0D),
        "BACKSPACE" => Some(0xFF08),
        "DELETE" | "DEL" => Some(0xFFFF),
        "INSERT" | "INS" => Some(0xFF63),
        "HOME" => Some(0xFF50),
        "END" => Some(0xFF57),
        "PAGEUP" | "PGUP" => Some(0xFF55),
        "PAGEDOWN" | "PGDN" => Some(0xFF56),
        "UP" => Some(0xFF52),
        "DOWN" => Some(0xFF54),
        "LEFT" => Some(0xFF51),
        "RIGHT" => Some(0xFF53),
        // Punctuation
        "MINUS" | "-" => Some(0x002d),
        "EQUAL" | "=" => Some(0x003d),
        "BRACKETLEFT" | "[" => Some(0x005b),
        "BRACKETRIGHT" | "]" => Some(0x005d),
        "BACKSLASH" | "\\" => Some(0x005c),
        "SEMICOLON" | ";" => Some(0x003b),
        "APOSTROPHE" | "'" => Some(0x0027),
        "GRAVE" | "`" => Some(0x0060),
        "COMMA" | "," => Some(0x002c),
        "PERIOD" | "." => Some(0x002e),
        "SLASH" | "/" => Some(0x002f),
        _ => None,
    }
}

/// Parse a hotkey string like "Ctrl+Shift+F7" into a HotkeyCombo.
pub fn parse_hotkey(hotkey: &str) -> Result<HotkeyCombo, KeyflowError> {
    let hotkey = hotkey.trim();
    if hotkey.is_empty() {
        return Err(KeyflowError::HotkeyParse {
            input: hotkey.to_string(),
            reason: "empty hotkey string".to_string(),
        });
    }

    let parts: Vec<&str> = hotkey.split('+').map(|s| s.trim()).collect();
    let mut modifiers: u16 = 0;
    let mut keysym: Option<u32> = None;

    for part in &parts {
        if let Some(mod_mask) = parse_modifier(part) {
            modifiers |= mod_mask;
        } else if let Some(ks) = keysym_from_name(part) {
            if keysym.is_some() {
                return Err(KeyflowError::HotkeyParse {
                    input: hotkey.to_string(),
                    reason: "multiple key names (only one non-modifier key allowed)".to_string(),
                });
            }
            keysym = Some(ks);
        } else {
            return Err(KeyflowError::HotkeyParse {
                input: hotkey.to_string(),
                reason: format!("unknown key name: '{part}'"),
            });
        }
    }

    let keysym = keysym.ok_or_else(|| KeyflowError::HotkeyParse {
        input: hotkey.to_string(),
        reason: "no key specified (only modifiers)".to_string(),
    })?;

    Ok(HotkeyCombo { keysym, modifiers })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_modifier_ctrl() {
        assert_eq!(parse_modifier("Ctrl"), Some(CONTROL_MASK));
        assert_eq!(parse_modifier("ctrl"), Some(CONTROL_MASK));
        assert_eq!(parse_modifier("Control"), Some(CONTROL_MASK));
    }

    #[test]
    fn test_parse_modifier_shift() {
        assert_eq!(parse_modifier("Shift"), Some(SHIFT_MASK));
    }

    #[test]
    fn test_parse_modifier_alt() {
        assert_eq!(parse_modifier("Alt"), Some(MOD1_MASK));
    }

    #[test]
    fn test_parse_modifier_super() {
        assert_eq!(parse_modifier("Super"), Some(MOD4_MASK));
        assert_eq!(parse_modifier("Win"), Some(MOD4_MASK));
        assert_eq!(parse_modifier("Meta"), Some(MOD4_MASK));
    }

    #[test]
    fn test_parse_modifier_unknown() {
        assert_eq!(parse_modifier("Unknown"), None);
        assert_eq!(parse_modifier(""), None);
    }
}
