use keyflow::config::binding::{Binding, InputMode};

#[test]
fn test_binding_creation() {
    let binding = Binding {
        name: "test".to_string(),
        hotkey: "F7".to_string(),
        provider: "clipboard".to_string(),
        item_id: None,
        input_mode: InputMode::default(),
        clipboard_clear_after_secs: None,
    };
    assert_eq!(binding.name, "test");
    assert_eq!(binding.hotkey, "F7");
    assert_eq!(binding.provider, "clipboard");
    assert!(binding.item_id.is_none());
    assert_eq!(binding.input_mode, InputMode::Auto);
    assert!(binding.clipboard_clear_after_secs.is_none());
}

#[test]
fn test_binding_with_item_id() {
    let binding = Binding {
        name: "bw-entry".to_string(),
        hotkey: "F8".to_string(),
        provider: "bitwarden".to_string(),
        item_id: Some("abc-123".to_string()),
        input_mode: InputMode::Type,
        clipboard_clear_after_secs: Some(0),
    };
    assert_eq!(binding.item_id.as_deref(), Some("abc-123"));
    assert_eq!(binding.input_mode, InputMode::Type);
    assert_eq!(binding.clipboard_clear_after_secs, Some(0));
}

use keyflow::config::{Config, Settings};

#[test]
fn test_default_settings() {
    let settings = Settings::default();
    assert_eq!(settings.clipboard_clear_after_secs, 5);
}

#[test]
fn test_config_from_toml() {
    let toml_str = r#"
[settings]
clipboard_clear_after_secs = 10

[[providers]]
type = "clipboard"

[[providers]]
type = "bitwarden"

[[bindings]]
name = "test"
hotkey = "F7"
provider = "clipboard"
"#;
    let config: Config = toml::from_str(toml_str).unwrap();
    assert_eq!(config.settings.clipboard_clear_after_secs, 10);
    assert_eq!(config.providers.len(), 2);
    assert_eq!(config.bindings.len(), 1);
    assert_eq!(config.bindings[0].name, "test");
}

#[test]
fn test_config_find_binding() {
    let toml_str = r#"
[[bindings]]
name = "first"
hotkey = "F7"
provider = "clipboard"

[[bindings]]
name = "second"
hotkey = "F8"
provider = "bitwarden"
item_id = "abc"
"#;
    let config: Config = toml::from_str(toml_str).unwrap();
    assert!(config.find_binding("F7").is_some());
    assert_eq!(config.find_binding("F7").unwrap().name, "first");
    assert!(config.find_binding("F8").unwrap().item_id.is_some());
    assert!(config.find_binding("F9").is_none());
}

#[test]
fn test_config_load_save_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.toml");

    let toml_str = r#"
[settings]
clipboard_clear_after_secs = 3

[[bindings]]
name = "roundtrip"
hotkey = "F7"
provider = "clipboard"
"#;
    let config: Config = toml::from_str(toml_str).unwrap();
    config.save(&path).unwrap();

    let loaded = Config::load(&path).unwrap();
    assert_eq!(loaded.settings.clipboard_clear_after_secs, 3);
    assert_eq!(loaded.bindings[0].name, "roundtrip");
}

#[test]
fn test_config_default_path() {
    let path = Config::default_path();
    assert!(path.ends_with("keyflow/keyflow.toml"));
}
