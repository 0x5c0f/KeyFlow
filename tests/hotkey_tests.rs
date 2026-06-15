use keyflow::hotkey::keys::{keysym_from_name, parse_hotkey, CONTROL_MASK, MOD4_MASK, SHIFT_MASK};

// Modifier parsing tests
#[test]
fn test_parse_modifier_ctrl() {
    let combo = parse_hotkey("Ctrl+F7").unwrap();
    assert_eq!(combo.modifiers, CONTROL_MASK);
}

#[test]
fn test_parse_modifier_shift() {
    let combo = parse_hotkey("Shift+F7").unwrap();
    assert_eq!(combo.modifiers, SHIFT_MASK);
}

#[test]
fn test_parse_modifier_alt() {
    let combo = parse_hotkey("Alt+F7").unwrap();
    assert_eq!(combo.modifiers, 1 << 3); // MOD1_MASK
}

#[test]
fn test_parse_modifier_super() {
    let combo = parse_hotkey("Super+A").unwrap();
    assert_eq!(combo.modifiers, MOD4_MASK);
}

// Keysym mapping tests
#[test]
fn test_keysym_function_keys() {
    assert_eq!(keysym_from_name("F1"), Some(0xFFBE));
    assert_eq!(keysym_from_name("F7"), Some(0xFFC4));
    assert_eq!(keysym_from_name("F12"), Some(0xFFC9));
    assert_eq!(keysym_from_name("F24"), Some(0xFFD5));
}

#[test]
fn test_keysym_letters() {
    assert_eq!(keysym_from_name("A"), Some(0x61));
    assert_eq!(keysym_from_name("a"), Some(0x61));
    assert_eq!(keysym_from_name("Z"), Some(0x7a));
}

#[test]
fn test_keysym_digits() {
    assert_eq!(keysym_from_name("0"), Some(0x30));
    assert_eq!(keysym_from_name("9"), Some(0x39));
}

#[test]
fn test_keysym_special() {
    assert_eq!(keysym_from_name("Space"), Some(0x0020));
    assert_eq!(keysym_from_name("Enter"), Some(0xFF0D));
    assert_eq!(keysym_from_name("Esc"), Some(0xFF1B));
    assert_eq!(keysym_from_name("Up"), Some(0xFF52));
}

#[test]
fn test_keysym_unknown() {
    assert_eq!(keysym_from_name("Unknown"), None);
    assert_eq!(keysym_from_name(""), None);
}

// Parse hotkey integration tests
#[test]
fn test_parse_hotkey_single_key() {
    let combo = parse_hotkey("F7").unwrap();
    assert_eq!(combo.keysym, 0xFFC4);
    assert_eq!(combo.modifiers, 0);
}

#[test]
fn test_parse_hotkey_ctrl_f7() {
    let combo = parse_hotkey("Ctrl+F7").unwrap();
    assert_eq!(combo.keysym, 0xFFC4);
    assert_eq!(combo.modifiers, CONTROL_MASK);
}

#[test]
fn test_parse_hotkey_ctrl_shift_f9() {
    let combo = parse_hotkey("Ctrl+Shift+F9").unwrap();
    assert_eq!(combo.keysym, 0xFFC6);
    assert_eq!(combo.modifiers, CONTROL_MASK | SHIFT_MASK);
}

#[test]
fn test_parse_hotkey_super_a() {
    let combo = parse_hotkey("Super+A").unwrap();
    assert_eq!(combo.keysym, 0x61);
    assert_eq!(combo.modifiers, MOD4_MASK);
}

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
fn test_keysym_case_insensitive() {
    assert_eq!(keysym_from_name("space"), keysym_from_name("Space"));
    assert_eq!(keysym_from_name("ENTER"), keysym_from_name("Enter"));
    assert_eq!(keysym_from_name("f7"), keysym_from_name("F7"));
    assert_eq!(keysym_from_name("a"), keysym_from_name("A"));
}

#[test]
fn test_parse_hotkey_lowercase() {
    let combo = parse_hotkey("ctrl+shift+f7").unwrap();
    assert_eq!(combo.keysym, 0xFFC4);
    assert_eq!(combo.modifiers, CONTROL_MASK | SHIFT_MASK);
}

// Punctuation tests
#[test]
fn test_keysym_punctuation() {
    assert_eq!(keysym_from_name("-"), Some(0x002d));
    assert_eq!(keysym_from_name("Minus"), Some(0x002d));
    assert_eq!(keysym_from_name("="), Some(0x003d));
    assert_eq!(keysym_from_name("["), Some(0x005b));
    assert_eq!(keysym_from_name("]"), Some(0x005d));
}
