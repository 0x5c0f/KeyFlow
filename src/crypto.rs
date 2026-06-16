//! Encryption/decryption for static content.
//!
//! Uses AES-256-GCM with Argon2id key derivation.
//! Format: `enc:v1:<base64(salt + nonce + ciphertext)>`

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use argon2::Argon2;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use thiserror::Error;

const PREFIX: &str = "enc:v1:";
const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;
const KEY_LEN: usize = 32; // AES-256

#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("Invalid encrypted content format")]
    InvalidFormat,
    #[error("Unsupported encryption version: {0}")]
    UnsupportedVersion(String),
    #[error("Decryption failed — wrong key or corrupted data")]
    DecryptionFailed,
    #[error("Encryption failed")]
    EncryptionFailed,
    #[error("Invalid base64: {0}")]
    InvalidBase64(#[from] base64::DecodeError),
    #[error("encryption_key not configured in [settings]")]
    KeyNotConfigured,
}

/// Derive a 256-bit key from passphrase + salt using Argon2id.
fn derive_key(passphrase: &str, salt: &[u8]) -> Result<[u8; KEY_LEN], CryptoError> {
    let mut key = [0u8; KEY_LEN];
    Argon2::default()
        .hash_password_into(passphrase.as_bytes(), salt, &mut key)
        .map_err(|_| CryptoError::EncryptionFailed)?;
    Ok(key)
}

/// Encrypt plaintext with the given passphrase.
///
/// Returns a string in format `enc:v1:<base64(salt + nonce + ciphertext)>`.
pub fn encrypt(plaintext: &str, passphrase: &str) -> Result<String, CryptoError> {
    use rand::RngCore;

    // Generate random salt and nonce
    let mut salt = [0u8; SALT_LEN];
    let mut nonce_bytes = [0u8; NONCE_LEN];
    rand::rng().fill_bytes(&mut salt);
    rand::rng().fill_bytes(&mut nonce_bytes);

    // Derive key
    let key = derive_key(passphrase, &salt)?;
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|_| CryptoError::EncryptionFailed)?;
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Encrypt
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|_| CryptoError::EncryptionFailed)?;

    // Pack: salt + nonce + ciphertext
    let mut packed = Vec::with_capacity(SALT_LEN + NONCE_LEN + ciphertext.len());
    packed.extend_from_slice(&salt);
    packed.extend_from_slice(&nonce_bytes);
    packed.extend_from_slice(&ciphertext);

    Ok(format!("{}{}", PREFIX, BASE64.encode(&packed)))
}

/// Decrypt an `enc:v1:...` string with the given passphrase.
pub fn decrypt(encrypted: &str, passphrase: &str) -> Result<String, CryptoError> {
    // Parse prefix
    let b64 = encrypted
        .strip_prefix(PREFIX)
        .ok_or(CryptoError::InvalidFormat)?;

    // Decode base64
    let packed = BASE64.decode(b64)?;

    // Minimum size: salt + nonce + GCM tag (16 bytes, even for empty plaintext)
    if packed.len() < SALT_LEN + NONCE_LEN + 16 {
        return Err(CryptoError::InvalidFormat);
    }

    let salt = &packed[..SALT_LEN];
    let nonce_bytes = &packed[SALT_LEN..SALT_LEN + NONCE_LEN];
    let ciphertext = &packed[SALT_LEN + NONCE_LEN..];

    // Derive key
    let key = derive_key(passphrase, salt)?;
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|_| CryptoError::DecryptionFailed)?;
    let nonce = Nonce::from_slice(nonce_bytes);

    // Decrypt
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| CryptoError::DecryptionFailed)?;

    String::from_utf8(plaintext).map_err(|_| CryptoError::DecryptionFailed)
}

/// Check if a string looks like encrypted content.
pub fn is_encrypted(s: &str) -> bool {
    s.starts_with(PREFIX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip() {
        let passphrase = "test-password-123";
        let plaintext = "my-secret-api-key";

        let encrypted = encrypt(plaintext, passphrase).unwrap();
        assert!(is_encrypted(&encrypted));

        let decrypted = decrypt(&encrypted, passphrase).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_wrong_password() {
        let encrypted = encrypt("secret", "correct-password").unwrap();
        let result = decrypt(&encrypted, "wrong-password");
        assert!(result.is_err());
    }

    #[test]
    fn test_not_encrypted() {
        assert!(!is_encrypted("plain-text"));
        assert!(!is_encrypted(""));
    }

    #[test]
    fn test_empty_plaintext() {
        let passphrase = "test-key";
        let encrypted = encrypt("", passphrase).unwrap();
        let decrypted = decrypt(&encrypted, passphrase).unwrap();
        assert_eq!(decrypted, "");
    }

    #[test]
    fn test_unicode() {
        let passphrase = "测试密码";
        let plaintext = "中文内容🔑";

        let encrypted = encrypt(plaintext, passphrase).unwrap();
        let decrypted = decrypt(&encrypted, passphrase).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_invalid_format() {
        let result = decrypt("not-encrypted", "key");
        assert!(matches!(result, Err(CryptoError::InvalidFormat)));
    }

    #[test]
    fn test_corrupted_base64() {
        let result = decrypt("enc:v1:!!!invalid!!!", "key");
        assert!(matches!(result, Err(CryptoError::InvalidBase64(_))));
    }
}
