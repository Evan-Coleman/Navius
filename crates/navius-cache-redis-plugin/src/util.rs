use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Serialize a value to JSON string
pub fn serialize<T>(value: &T) -> Result<String, serde_json::Error>
where
    T: Serialize + ?Sized,
{
    serde_json::to_string(value)
}

/// Deserialize a value from JSON string
pub fn deserialize<T>(data: &str) -> Result<T, serde_json::Error>
where
    T: for<'de> Deserialize<'de>,
{
    serde_json::from_str(data)
}

/// Format a key with the prefix
pub fn format_key(prefix: &str, key: &str) -> String {
    format!("{}{}", prefix, key)
}

/// Format a TTL for Redis (converts Duration to seconds)
pub fn format_ttl(ttl: Duration) -> u64 {
    // Convert to seconds, ensuring at least 1 second
    std::cmp::max(ttl.as_secs(), 1)
}

/// Parse a Redis TTL response to a Duration
pub fn parse_ttl(ttl: i64) -> Option<Duration> {
    if ttl > 0 {
        Some(Duration::from_secs(ttl as u64))
    } else {
        None
    }
}

/// Convert Redis bool response (1/0) to Rust bool
pub fn redis_bool_to_bool(value: i64) -> bool {
    value != 0
}

/// Create a script sha for Lua scripts
pub fn script_sha(script: &str) -> String {
    use sha1_smol::Sha1;
    let mut sha1 = Sha1::new();
    sha1.update(script.as_bytes());
    sha1.digest().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_deserialize() {
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct TestStruct {
            name: String,
            age: u32,
        }

        let test = TestStruct {
            name: "Test".to_string(),
            age: 42,
        };

        let serialized = serialize(&test).unwrap();
        let deserialized: TestStruct = deserialize(&serialized).unwrap();

        assert_eq!(test, deserialized);
    }

    #[test]
    fn test_format_key() {
        assert_eq!(format_key("prefix:", "key"), "prefix:key");
        assert_eq!(format_key("", "key"), "key");
    }

    #[test]
    fn test_format_ttl() {
        assert_eq!(format_ttl(Duration::from_secs(0)), 1); // Minimum 1 second
        assert_eq!(format_ttl(Duration::from_secs(60)), 60);
        assert_eq!(format_ttl(Duration::from_millis(500)), 1); // Rounds up to 1 second
    }

    #[test]
    fn test_parse_ttl() {
        assert_eq!(parse_ttl(60), Some(Duration::from_secs(60)));
        assert_eq!(parse_ttl(0), None); // Key exists but no expiry
        assert_eq!(parse_ttl(-1), None); // Key doesn't exist
        assert_eq!(parse_ttl(-2), None); // Key exists but no TTL information available
    }

    #[test]
    fn test_redis_bool_to_bool() {
        assert_eq!(redis_bool_to_bool(1), true);
        assert_eq!(redis_bool_to_bool(0), false);
    }

    #[test]
    fn test_script_sha() {
        let script = "return redis.call('GET', KEYS[1])";
        // SHA1 of the script should be deterministic
        assert!(!script_sha(script).is_empty());
        assert_eq!(script_sha(script), script_sha(script));
    }
}
