use crate::error::RedisCacheError;
use std::borrow::Cow;
use std::fmt::Debug;
use tracing::warn;

/// Maximum allowed length for Redis keys (in bytes)
const MAX_KEY_LENGTH: usize = 1024;

/// Redis key validation options
#[derive(Debug, Clone)]
pub struct KeyValidationOptions {
    /// Maximum key length in bytes
    pub max_length: usize,
    /// Whether to allow empty keys
    pub allow_empty: bool,
    /// Whether to allow spaces in keys
    pub allow_spaces: bool,
    /// Whether to allow control characters in keys
    pub allow_control_chars: bool,
}

impl Default for KeyValidationOptions {
    fn default() -> Self {
        Self {
            max_length: MAX_KEY_LENGTH,
            allow_empty: false,
            allow_spaces: true,
            allow_control_chars: false,
        }
    }
}

/// Validate a Redis key based on the given options
///
/// # Errors
///
/// Returns `RedisCacheError::InvalidKey` if the key is invalid based on the options
pub fn validate_key<K>(key: &K, options: &KeyValidationOptions) -> Result<(), RedisCacheError>
where
    K: AsRef<str> + std::fmt::Display + ?Sized,
{
    let key_str = key.as_ref();

    // Check for empty key
    if !options.allow_empty && key_str.is_empty() {
        return Err(RedisCacheError::InvalidKey(
            "Empty keys are not allowed".to_string(),
        ));
    }

    // Check key length
    if key_str.len() > options.max_length {
        return Err(RedisCacheError::InvalidKey(format!(
            "Key is too long: {} bytes (maximum is {} bytes)",
            key_str.len(),
            options.max_length
        )));
    }

    // Check for control characters
    if !options.allow_control_chars && key_str.chars().any(|c| c.is_control()) {
        return Err(RedisCacheError::InvalidKey(
            "Key contains control characters".to_string(),
        ));
    }

    // Check for spaces
    if !options.allow_spaces && key_str.chars().any(|c| c.is_whitespace()) {
        return Err(RedisCacheError::InvalidKey(
            "Key contains spaces".to_string(),
        ));
    }

    Ok(())
}

/// Apply a prefix to a key if the prefix is not empty
///
/// If the prefix is empty, the original key is returned without modification.
/// Otherwise, the prefix and key are joined with a colon separator.
pub fn prefix_key<'a, K>(prefix: &str, key: &'a K) -> Cow<'a, str>
where
    K: AsRef<str> + ?Sized,
{
    let key_str = key.as_ref();

    if prefix.is_empty() {
        Cow::Borrowed(key_str)
    } else {
        Cow::Owned(format!("{}:{}", prefix, key_str))
    }
}

/// Generate cache keys with consistent naming
///
/// # Example
///
/// ```rust
/// use navius_cache_redis::key::{generate_key, KeyType};
///
/// let user_id = 123;
/// let user_key = generate_key(KeyType::Entity, "user", &user_id);
/// assert_eq!(user_key, "entity:user:123");
///
/// let session_key = generate_key(KeyType::Session, "auth", "abc123");
/// assert_eq!(session_key, "session:auth:abc123");
/// ```
#[derive(Debug, Clone, Copy)]
pub enum KeyType {
    /// Entity cache (e.g., user:123)
    Entity,
    /// Session cache (e.g., session:abc123)
    Session,
    /// Configuration cache (e.g., config:app_settings)
    Config,
    /// Statistics cache (e.g., stats:daily:visits)
    Stats,
    /// Temporary values (e.g., temp:job:456)
    Temp,
    /// Locks (e.g., lock:resource:789)
    Lock,
}

impl KeyType {
    /// Get the prefix for this key type
    pub fn prefix(&self) -> &'static str {
        match self {
            KeyType::Entity => "entity",
            KeyType::Session => "session",
            KeyType::Config => "config",
            KeyType::Stats => "stats",
            KeyType::Temp => "temp",
            KeyType::Lock => "lock",
        }
    }
}

/// Generate a cache key with the given key type, entity, and identifier
pub fn generate_key<T, I>(key_type: KeyType, entity: T, id: I) -> String
where
    T: AsRef<str>,
    I: Debug,
{
    format!("{}:{}:{:?}", key_type.prefix(), entity.as_ref(), id)
}

/// Utility function to sanitize a key by replacing invalid characters
pub fn sanitize_key(key: &str) -> String {
    key.chars()
        .map(|c| {
            if c.is_control() || c.is_whitespace() {
                '_'
            } else {
                c
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_key_default_options() {
        let options = KeyValidationOptions::default();

        // Valid keys
        assert!(validate_key(&"valid_key", &options).is_ok());
        assert!(validate_key(&"valid-key", &options).is_ok());
        assert!(validate_key(&"valid:key", &options).is_ok());
        assert!(validate_key(&"valid key", &options).is_ok());

        // Invalid keys
        assert!(validate_key(&"", &options).is_err());
        assert!(validate_key(&"a".repeat(options.max_length + 1), &options).is_err());
        assert!(validate_key(&"invalid\tkey", &options).is_err());
        assert!(validate_key(&"invalid\nkey", &options).is_err());
    }

    #[test]
    fn test_validate_key_custom_options() {
        let options = KeyValidationOptions {
            max_length: 10,
            allow_empty: true,
            allow_spaces: false,
            allow_control_chars: false,
        };

        // Valid keys
        assert!(validate_key(&"", &options).is_ok());
        assert!(validate_key(&"valid_key", &options).is_ok());

        // Invalid keys
        assert!(validate_key(&"a".repeat(options.max_length + 1), &options).is_err());
        assert!(validate_key(&"invalid key", &options).is_err());
        assert!(validate_key(&"invalid\tkey", &options).is_err());
    }

    #[test]
    fn test_prefix_key() {
        // With prefix
        let prefixed = prefix_key("prefix", &"key");
        assert_eq!(prefixed, "prefix:key");

        // Empty prefix
        let unprefixed = prefix_key("", &"key");
        assert_eq!(unprefixed, "key");

        // Empty key
        let empty_key = prefix_key("prefix", &"");
        assert_eq!(empty_key, "prefix:");
    }

    #[test]
    fn test_generate_key() {
        let user_id = 123;
        let user_key = generate_key(KeyType::Entity, "user", user_id);
        assert_eq!(user_key, "entity:user:123");

        let session_id = "abc123";
        let session_key = generate_key(KeyType::Session, "auth", session_id);
        assert_eq!(session_key, "session:auth:\"abc123\"");
    }

    #[test]
    fn test_sanitize_key() {
        let key = "invalid\tkey with\ncontrol characters";
        let sanitized = sanitize_key(key);
        assert_eq!(sanitized, "invalid_key_with_control_characters");
    }
}
