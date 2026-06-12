use keyflow::config::ProviderConfig;
use keyflow::error::ProviderError;
use keyflow::provider::{create_provider, PasswordProvider};

/// A mock provider for testing.
struct MockProvider {
    password: Option<String>,
}

impl PasswordProvider for MockProvider {
    fn get_password(&self) -> Result<String, ProviderError> {
        self.password.clone().ok_or(ProviderError::ClipboardEmpty)
    }

    fn name(&self) -> &str {
        "mock"
    }
}

#[test]
fn test_mock_provider_returns_password() {
    let provider = MockProvider {
        password: Some("secret123".to_string()),
    };
    assert_eq!(provider.get_password().unwrap(), "secret123");
}

#[test]
fn test_mock_provider_empty_returns_error() {
    let provider = MockProvider { password: None };
    assert!(matches!(
        provider.get_password(),
        Err(ProviderError::ClipboardEmpty)
    ));
}

#[test]
fn test_create_clipboard_provider() {
    let config = ProviderConfig {
        provider_type: "clipboard".to_string(),
        cli_path: None,
    };
    let provider = create_provider(&config);
    assert!(provider.is_some());
    assert_eq!(provider.unwrap().name(), "clipboard");
}

#[test]
fn test_create_unknown_provider() {
    let config = ProviderConfig {
        provider_type: "unknown".to_string(),
        cli_path: None,
    };
    let provider = create_provider(&config);
    assert!(provider.is_none());
}
