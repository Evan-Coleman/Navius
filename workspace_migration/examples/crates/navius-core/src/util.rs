//! Utility functions for the Navius framework.
//!
//! This module provides various utility functions that are used throughout the framework.

use std::collections::HashMap;
use std::time::{Duration, Instant, SystemTime};

use crate::error::{Error, Result};
use crate::types::StringMap;

/// Generate a random ID
pub fn random_id(prefix: &str) -> String {
    use std::sync::atomic::{AtomicUsize, Ordering};

    static COUNTER: AtomicUsize = AtomicUsize::new(1);
    let counter = COUNTER.fetch_add(1, Ordering::SeqCst);

    // For a real implementation, we would use UUID v4 or similar
    format!("{}-{}-{}", prefix, current_time_millis(), counter)
}

/// Get the current time in milliseconds since the Unix epoch
pub fn current_time_millis() -> u128 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_millis()
}

/// Measure the execution time of a function
pub fn measure_time<F, T>(f: F) -> (T, Duration)
where
    F: FnOnce() -> T,
{
    let start = Instant::now();
    let result = f();
    let duration = start.elapsed();
    (result, duration)
}

/// Parse a string into a duration
///
/// Supports formats like:
/// - "1s" (1 second)
/// - "500ms" (500 milliseconds)
/// - "5m" (5 minutes)
/// - "2h" (2 hours)
pub fn parse_duration(s: &str) -> Result<Duration> {
    let s = s.trim();

    if s.is_empty() {
        return Err(Error::new("Empty duration string"));
    }

    let (value_str, unit) = if s.ends_with("ms") {
        (&s[..s.len() - 2], "ms")
    } else if s.ends_with('s') {
        (&s[..s.len() - 1], "s")
    } else if s.ends_with('m') {
        (&s[..s.len() - 1], "m")
    } else if s.ends_with('h') {
        (&s[..s.len() - 1], "h")
    } else {
        (s, "ms") // Default to milliseconds
    };

    let value: u64 = value_str
        .parse()
        .map_err(|_| Error::new(&format!("Invalid duration value: {}", value_str)))?;

    match unit {
        "ms" => Ok(Duration::from_millis(value)),
        "s" => Ok(Duration::from_secs(value)),
        "m" => Ok(Duration::from_secs(value * 60)),
        "h" => Ok(Duration::from_secs(value * 3600)),
        _ => Err(Error::new(&format!("Unknown duration unit: {}", unit))),
    }
}

/// Encode a StringMap as a query string
pub fn encode_query(params: &StringMap) -> String {
    if params.is_empty() {
        return String::new();
    }

    let mut pairs = Vec::with_capacity(params.len());

    for (key, value) in params {
        // In a real implementation, we would URL-encode these values
        pairs.push(format!("{}={}", key, value));
    }

    pairs.join("&")
}

/// Parse a query string into a StringMap
pub fn parse_query(query: &str) -> StringMap {
    let mut map = StringMap::new();

    if query.is_empty() {
        return map;
    }

    for pair in query.split('&') {
        if let Some(index) = pair.find('=') {
            let key = &pair[..index];
            let value = &pair[index + 1..];
            // In a real implementation, we would URL-decode these values
            map.insert(key.to_string(), value.to_string());
        }
    }

    map
}

/// Merge two HashMaps
pub fn merge_maps<K, V>(mut map1: HashMap<K, V>, map2: HashMap<K, V>) -> HashMap<K, V>
where
    K: Eq + std::hash::Hash,
{
    for (k, v) in map2 {
        map1.insert(k, v);
    }
    map1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_id() {
        let id1 = random_id("test");
        let id2 = random_id("test");

        assert!(id1.starts_with("test-"));
        assert!(id2.starts_with("test-"));
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_measure_time() {
        let (result, duration) = measure_time(|| {
            std::thread::sleep(Duration::from_millis(10));
            42
        });

        assert_eq!(result, 42);
        assert!(duration >= Duration::from_millis(10));
    }

    #[test]
    fn test_parse_duration() {
        assert_eq!(parse_duration("100ms").unwrap(), Duration::from_millis(100));
        assert_eq!(parse_duration("5s").unwrap(), Duration::from_secs(5));
        assert_eq!(parse_duration("2m").unwrap(), Duration::from_secs(120));
        assert_eq!(parse_duration("1h").unwrap(), Duration::from_secs(3600));

        assert!(parse_duration("invalid").is_err());
    }

    #[test]
    fn test_encode_query() {
        let mut params = StringMap::new();
        params.insert("key1".to_string(), "value1".to_string());
        params.insert("key2".to_string(), "value2".to_string());

        let query = encode_query(&params);
        assert!(query.contains("key1=value1"));
        assert!(query.contains("key2=value2"));
        assert!(query.contains("&"));
    }

    #[test]
    fn test_parse_query() {
        let query = "key1=value1&key2=value2";
        let params = parse_query(query);

        assert_eq!(params.get("key1"), Some(&"value1".to_string()));
        assert_eq!(params.get("key2"), Some(&"value2".to_string()));
    }

    #[test]
    fn test_merge_maps() {
        let mut map1 = HashMap::new();
        map1.insert("key1", "value1");

        let mut map2 = HashMap::new();
        map2.insert("key2", "value2");
        map2.insert("key1", "new_value"); // This will override map1's value

        let merged = merge_maps(map1, map2);

        assert_eq!(merged.get("key1"), Some(&"new_value"));
        assert_eq!(merged.get("key2"), Some(&"value2"));
    }
}
