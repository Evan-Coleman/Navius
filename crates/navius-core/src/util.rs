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

/// Parse a duration string into milliseconds
/// Format: 1s, 100ms, 5m, 1h
pub fn parse_duration(duration_str: &str) -> Result<u64> {
    if duration_str.is_empty() {
        return Err(Error::validation("Empty duration string"));
    }

    let duration_str = duration_str.trim();

    // Extract the unit and value
    let (value_str, unit) = if duration_str.ends_with("ms") {
        (&duration_str[0..duration_str.len() - 2], "ms")
    } else if duration_str.ends_with('s') {
        (&duration_str[0..duration_str.len() - 1], "s")
    } else if duration_str.ends_with('m') {
        (&duration_str[0..duration_str.len() - 1], "m")
    } else if duration_str.ends_with('h') {
        (&duration_str[0..duration_str.len() - 1], "h")
    } else {
        return Err(Error::validation(&format!(
            "Unknown duration unit in: {}",
            duration_str
        )));
    };

    // Parse the value
    let value = value_str
        .parse::<u64>()
        .map_err(|_| Error::validation(&format!("Invalid duration value: {}", value_str)))?;

    // Convert to milliseconds
    match unit {
        "ms" => Ok(value),
        "s" => Ok(value * 1000),
        "m" => Ok(value * 1000 * 60),
        "h" => Ok(value * 1000 * 60 * 60),
        _ => Err(Error::validation(&format!(
            "Unknown duration unit: {}",
            unit
        ))),
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
        assert_eq!(parse_duration("100ms").unwrap(), 100);
        assert_eq!(parse_duration("5s").unwrap(), 5000);
        assert_eq!(parse_duration("2m").unwrap(), 120000);
        assert_eq!(parse_duration("1h").unwrap(), 3600000);

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
