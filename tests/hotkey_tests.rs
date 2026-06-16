use keyflow::hotkey::keys::{key_from_name, modifiers, parse_hotkey, Key};

// Modifier parsing tests
#[test]
fn test_parse_modifier_ctrl() {
    let combo = parse_hotkey("Ctrl+F7").unwrap();
    assert_eq!(combo.modifiers, modifiers::CONTROL);
}

#[test]
fn test_parse_modifier_shift() {
    let combo = parse_hotkey("Shift+F7").unwrap();
    assert_eq!(combo.modifiers, modifiers::SHIFT);
}

#[test]
fn test_parse_modifier_alt() {
    let combo = parse_hotkey("Alt+F7").unwrap();
    assert_eq!(combo.modifiers, modifiers::ALT);
}

#[test]
fn test_parse_modifier_super() {
    let combo = parse_hotkey("Super+A").unwrap();
    assert_eq!(combo.modifiers, modifiers::SUPER);
}

// Key name mapping tests
#[test]
fn test_key_from_name_function_keys() {
    assert_eq!(key_from_name("F1"), Some(Key::F1));
    assert_eq!(key_from_name("F7"), Some(Key::F7));
    assert_eq!(key_from_name("F12"), Some(Key::F12));
    assert_eq!(key_from_name("F24"), Some(Key::F24));
}

#[test]
fn test_key_from_name_letters() {
    assert_eq!(key_from_name("A"), Some(Key::A));
    assert_eq!(key_from_name("a"), Some(Key::A));
    assert_eq!(key_from_name("Z"), Some(Key::Z));
}

#[test]
fn test_key_from_name_digits() {
    assert_eq!(key_from_name("0"), Some(Key::Digit0));
    assert_eq!(key_from_name("9"), Some(Key::Digit9));
}

#[test]
fn test_key_from_name_special() {
    assert_eq!(key_from_name("Space"), Some(Key::Space));
    assert_eq!(key_from_name("Enter"), Some(Key::Enter));
    assert_eq!(key_from_name("Esc"), Some(Key::Escape));
    assert_eq!(key_from_name("Up"), Some(Key::Up));
}

#[test]
fn test_key_from_name_unknown() {
    assert_eq!(key_from_name("Unknown"), None);
    assert_eq!(key_from_name(""), None);
}

// Parse hotkey integration tests
#[test]
fn test_parse_hotkey_single_key() {
    let combo = parse_hotkey("F7").unwrap();
    assert_eq!(combo.key, Key::F7);
    assert_eq!(combo.modifiers, 0);
}

#[test]
fn test_parse_hotkey_ctrl_f7() {
    let combo = parse_hotkey("Ctrl+F7").unwrap();
    assert_eq!(combo.key, Key::F7);
    assert_eq!(combo.modifiers, modifiers::CONTROL);
}

#[test]
fn test_parse_hotkey_ctrl_shift_f9() {
    let combo = parse_hotkey("Ctrl+Shift+F9").unwrap();
    assert_eq!(combo.key, Key::F9);
    assert_eq!(combo.modifiers, modifiers::CONTROL | modifiers::SHIFT);
}

#[test]
fn test_parse_hotkey_super_a() {
    let combo = parse_hotkey("Super+A").unwrap();
    assert_eq!(combo.key, Key::A);
    assert_eq!(combo.modifiers, modifiers::SUPER);
}

// Error cases
#[test]
fn test_parse_hotkey_empty() {
    assert!(parse_hotkey("").is_err());
    assert!(parse_hotkey("  ").is_err());
}

#[test]
fn test_parse_hotkey_only_modifiers() {
    assert!(parse_hotkey("Ctrl").is_err());
    assert!(parse_hotkey("Ctrl+Shift").is_err());
}

#[test]
fn test_parse_hotkey_unknown_key() {
    assert!(parse_hotkey("Ctrl+UnknownKey").is_err());
}

#[test]
fn test_parse_hotkey_multiple_keys() {
    assert!(parse_hotkey("A+B").is_err());
}

// Case insensitivity tests
#[test]
fn test_key_from_name_case_insensitive() {
    assert_eq!(key_from_name("space"), key_from_name("Space"));
    assert_eq!(key_from_name("ENTER"), key_from_name("Enter"));
    assert_eq!(key_from_name("f7"), key_from_name("F7"));
    assert_eq!(key_from_name("a"), key_from_name("A"));
}

#[test]
fn test_parse_hotkey_lowercase() {
    let combo = parse_hotkey("ctrl+shift+f7").unwrap();
    assert_eq!(combo.key, Key::F7);
    assert_eq!(combo.modifiers, modifiers::CONTROL | modifiers::SHIFT);
}

// Punctuation tests
#[test]
fn test_key_from_name_punctuation() {
    assert_eq!(key_from_name("-"), Some(Key::Minus));
    assert_eq!(key_from_name("Minus"), Some(Key::Minus));
    assert_eq!(key_from_name("="), Some(Key::Equal));
    assert_eq!(key_from_name("["), Some(Key::BracketLeft));
    assert_eq!(key_from_name("]"), Some(Key::BracketRight));
}
