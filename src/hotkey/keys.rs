//! Platform-agnostic hotkey definitions.
//!
//! Parses hotkey strings like "Ctrl+Shift+F7" into platform-independent
//! representations that each platform backend can map to native keycodes.

use crate::error::KeyflowError;

/// Platform-agnostic key identifiers.
///
/// Each variant represents a physical key, independent of platform-specific
/// keycodes (X11 keysym, Windows VK, macOS CGKeyCode).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    // Letters
    A, B, C, D, E, F, G, H, I, J, K, L, M,
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z,

    // Digits
    Digit0, Digit1, Digit2, Digit3, Digit4,
    Digit5, Digit6, Digit7, Digit8, Digit9,

    // Function keys
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10,
    F11, F12, F13, F14, F15, F16, F17, F18, F19, F20,
    F21, F22, F23, F24,

    // Navigation
    Home, End, PageUp, PageDown,
    Up, Down, Left, Right,
    Insert, Delete,
    Tab, Enter, Escape, Backspace, Space,

    // Punctuation
    Minus, Equal, BracketLeft, BracketRight,
    Backslash, Semicolon, Apostrophe, Grave,
    Comma, Period, Slash,
}

/// Modifier key flags (bitmask).
///
/// Uses a platform-agnostic representation. Each platform backend maps
/// these to native modifier values.
pub mod modifiers {
    pub const SHIFT: u16 = 1 << 0;
    pub const CONTROL: u16 = 1 << 1;
    pub const ALT: u16 = 1 << 2;
    pub const SUPER: u16 = 1 << 3;

    /// All modifier masks with their string aliases.
    pub const ALL: &[(&str, u16)] = &[
        ("ctrl", CONTROL),
        ("control", CONTROL),
        ("shift", SHIFT),
        ("alt", ALT),
        ("super", SUPER),
        ("win", SUPER),
        ("meta", SUPER),
    ];
}

/// Parsed hotkey combination.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HotkeyCombo {
    pub key: Key,
    pub modifiers: u16,
}

/// Parse a modifier name to its flag value.
fn parse_modifier(name: &str) -> Option<u16> {
    let lower = name.to_lowercase();
    modifiers::ALL
        .iter()
        .find(|(n, _)| *n == lower.as_str())
        .map(|(_, m)| *m)
}

/// Map a key name to a Key variant.
pub fn key_from_name(name: &str) -> Option<Key> {
    let upper = name.to_uppercase();

    // Function keys F1-F24
    if let Some(rest) = upper.strip_prefix('F') {
        if let Ok(n) = rest.parse::<u32>() {
            if (1..=24).contains(&n) {
                return Some(match n {
                    1 => Key::F1, 2 => Key::F2, 3 => Key::F3, 4 => Key::F4,
                    5 => Key::F5, 6 => Key::F6, 7 => Key::F7, 8 => Key::F8,
                    9 => Key::F9, 10 => Key::F10, 11 => Key::F11, 12 => Key::F12,
                    13 => Key::F13, 14 => Key::F14, 15 => Key::F15, 16 => Key::F16,
                    17 => Key::F17, 18 => Key::F18, 19 => Key::F19, 20 => Key::F20,
                    21 => Key::F21, 22 => Key::F22, 23 => Key::F23, 24 => Key::F24,
                    _ => unreachable!(),
                });
            }
        }
    }

    match upper.as_str() {
        // Letters
        "A" => Some(Key::A), "B" => Some(Key::B), "C" => Some(Key::C),
        "D" => Some(Key::D), "E" => Some(Key::E), "F" => Some(Key::F),
        "G" => Some(Key::G), "H" => Some(Key::H), "I" => Some(Key::I),
        "J" => Some(Key::J), "K" => Some(Key::K), "L" => Some(Key::L),
        "M" => Some(Key::M), "N" => Some(Key::N), "O" => Some(Key::O),
        "P" => Some(Key::P), "Q" => Some(Key::Q), "R" => Some(Key::R),
        "S" => Some(Key::S), "T" => Some(Key::T), "U" => Some(Key::U),
        "V" => Some(Key::V), "W" => Some(Key::W), "X" => Some(Key::X),
        "Y" => Some(Key::Y), "Z" => Some(Key::Z),

        // Digits
        "0" => Some(Key::Digit0), "1" => Some(Key::Digit1), "2" => Some(Key::Digit2),
        "3" => Some(Key::Digit3), "4" => Some(Key::Digit4), "5" => Some(Key::Digit5),
        "6" => Some(Key::Digit6), "7" => Some(Key::Digit7), "8" => Some(Key::Digit8),
        "9" => Some(Key::Digit9),

        // Special keys
        "SPACE" => Some(Key::Space),
        "TAB" => Some(Key::Tab),
        "ESC" | "ESCAPE" => Some(Key::Escape),
        "ENTER" | "RETURN" => Some(Key::Enter),
        "BACKSPACE" => Some(Key::Backspace),
        "DELETE" | "DEL" => Some(Key::Delete),
        "INSERT" | "INS" => Some(Key::Insert),
        "HOME" => Some(Key::Home),
        "END" => Some(Key::End),
        "PAGEUP" | "PGUP" => Some(Key::PageUp),
        "PAGEDOWN" | "PGDN" => Some(Key::PageDown),
        "UP" => Some(Key::Up),
        "DOWN" => Some(Key::Down),
        "LEFT" => Some(Key::Left),
        "RIGHT" => Some(Key::Right),

        // Punctuation
        "MINUS" | "-" => Some(Key::Minus),
        "EQUAL" | "=" => Some(Key::Equal),
        "BRACKETLEFT" | "[" => Some(Key::BracketLeft),
        "BRACKETRIGHT" | "]" => Some(Key::BracketRight),
        "BACKSLASH" | "\\" => Some(Key::Backslash),
        "SEMICOLON" | ";" => Some(Key::Semicolon),
        "APOSTROPHE" | "'" => Some(Key::Apostrophe),
        "GRAVE" | "`" => Some(Key::Grave),
        "COMMA" | "," => Some(Key::Comma),
        "PERIOD" | "." => Some(Key::Period),
        "SLASH" | "/" => Some(Key::Slash),

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
    let mut mod_flags: u16 = 0;
    let mut key: Option<Key> = None;

    for part in &parts {
        if let Some(m) = parse_modifier(part) {
            mod_flags |= m;
        } else if let Some(k) = key_from_name(part) {
            if key.is_some() {
                return Err(KeyflowError::HotkeyParse {
                    input: hotkey.to_string(),
                    reason: "multiple key names (only one non-modifier key allowed)".to_string(),
                });
            }
            key = Some(k);
        } else {
            return Err(KeyflowError::HotkeyParse {
                input: hotkey.to_string(),
                reason: format!("unknown key name: '{part}'"),
            });
        }
    }

    let key = key.ok_or_else(|| KeyflowError::HotkeyParse {
        input: hotkey.to_string(),
        reason: "no key specified (only modifiers)".to_string(),
    })?;

    Ok(HotkeyCombo {
        key,
        modifiers: mod_flags,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_modifier_names() {
        assert_eq!(parse_modifier("Ctrl"), Some(modifiers::CONTROL));
        assert_eq!(parse_modifier("ctrl"), Some(modifiers::CONTROL));
        assert_eq!(parse_modifier("Control"), Some(modifiers::CONTROL));
        assert_eq!(parse_modifier("Shift"), Some(modifiers::SHIFT));
        assert_eq!(parse_modifier("Alt"), Some(modifiers::ALT));
        assert_eq!(parse_modifier("Super"), Some(modifiers::SUPER));
        assert_eq!(parse_modifier("Win"), Some(modifiers::SUPER));
        assert_eq!(parse_modifier("Meta"), Some(modifiers::SUPER));
        assert_eq!(parse_modifier("Unknown"), None);
    }

    #[test]
    fn test_parse_simple_hotkey() {
        let combo = parse_hotkey("F7").unwrap();
        assert_eq!(combo.key, Key::F7);
        assert_eq!(combo.modifiers, 0);
    }

    #[test]
    fn test_parse_modified_hotkey() {
        let combo = parse_hotkey("Ctrl+Shift+F7").unwrap();
        assert_eq!(combo.key, Key::F7);
        assert_eq!(combo.modifiers, modifiers::CONTROL | modifiers::SHIFT);
    }

    #[test]
    fn test_parse_with_super() {
        let combo = parse_hotkey("Super+Space").unwrap();
        assert_eq!(combo.key, Key::Space);
        assert_eq!(combo.modifiers, modifiers::SUPER);
    }

    #[test]
    fn test_parse_with_punctuation() {
        let combo = parse_hotkey("Ctrl+[").unwrap();
        assert_eq!(combo.key, Key::BracketLeft);
    }

    #[test]
    fn test_parse_empty() {
        assert!(parse_hotkey("").is_err());
    }

    #[test]
    fn test_parse_only_modifiers() {
        assert!(parse_hotkey("Ctrl+Shift").is_err());
    }

    #[test]
    fn test_parse_duplicate_keys() {
        assert!(parse_hotkey("A+B").is_err());
    }
}
