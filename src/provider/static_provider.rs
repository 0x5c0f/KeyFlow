//! Static text provider with optional encryption support.

use crate::crypto;
use crate::error::ProviderError;
use crate::provider::PasswordProvider;

/// Provides static text content configured in the config file.
/// Supports optional decryption if `encrypted` is true.
pub struct StaticProvider {
    content: String,
    encrypted: bool,
    encryption_key: Option<String>,
}

impl StaticProvider {
    /// Create a new static provider.
    ///
    /// - `content`: The text content (plaintext or encrypted).
    /// - `encrypted`: Whether the content is encrypted.
    /// - `encryption_key`: The decryption key (required if encrypted).
    pub fn new(content: String, encrypted: bool, encryption_key: Option<String>) -> Self {
        Self {
            content,
            encrypted,
            encryption_key,
        }
    }
}

impl PasswordProvider for StaticProvider {
    fn get_password(&self) -> Result<String, ProviderError> {
        if !self.encrypted {
            // Plaintext — return as-is
            return Ok(self.content.clone());
        }

        // Encrypted — need to decrypt
        let key = self
            .encryption_key
            .as_ref()
            .ok_or(ProviderError::EncryptionKeyMissing)?;

        if self.content.is_empty() {
            return Err(ProviderError::InvalidConfig(
                "content is empty for encrypted static binding".to_string(),
            ));
        }

        crypto::decrypt(&self.content, key).map_err(|e| {
            ProviderError::InvalidConfig(format!("Failed to decrypt content: {e}"))
        })
    }

    fn name(&self) -> &str {
        "static"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plaintext() {
        let provider = StaticProvider::new("hello".to_string(), false, None);
        assert_eq!(provider.get_password().unwrap(), "hello");
    }

    #[test]
    fn test_encrypted_with_key() {
        let key = "test-key";
        let encrypted = crypto::encrypt("secret", key).unwrap();
        let provider = StaticProvider::new(encrypted, true, Some(key.to_string()));
        assert_eq!(provider.get_password().unwrap(), "secret");
    }

    #[test]
    fn test_encrypted_without_key() {
        let provider = StaticProvider::new("enc:v1:xxx".to_string(), true, None);
        assert!(matches!(
            provider.get_password(),
            Err(ProviderError::EncryptionKeyMissing)
        ));
    }

    #[test]
    fn test_encrypted_wrong_key() {
        let encrypted = crypto::encrypt("secret", "correct-key").unwrap();
        let provider =
            StaticProvider::new(encrypted, true, Some("wrong-key".to_string()));
        assert!(matches!(
            provider.get_password(),
            Err(ProviderError::InvalidConfig(_))
        ));
    }

    #[test]
    fn test_empty_content() {
        let provider = StaticProvider::new("".to_string(), false, None);
        assert_eq!(provider.get_password().unwrap(), "");
    }

    #[test]
    fn test_empty_encrypted_content() {
        let provider = StaticProvider::new("".to_string(), true, Some("key".to_string()));
        assert!(matches!(
            provider.get_password(),
            Err(ProviderError::InvalidConfig(_))
        ));
    }
}
