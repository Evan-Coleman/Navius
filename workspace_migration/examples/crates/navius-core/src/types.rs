//! Common types used throughout the Navius framework.
//!
//! This module defines common types, type aliases, and utility types that are used
//! across different parts of the Navius framework.

use std::collections::HashMap;
use std::fmt;
use std::time::{Duration, SystemTime};

/// A map of string keys to string values
pub type StringMap = HashMap<String, String>;

/// Represents metadata for various objects in the system
pub type Metadata = HashMap<String, String>;

/// ID type for unique identifiers
pub type Id = String;

/// Time representation used throughout the framework
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Timestamp(pub SystemTime);

impl Timestamp {
    /// Create a new timestamp with the current time
    pub fn now() -> Self {
        Self(SystemTime::now())
    }

    /// Create a new timestamp with the specified time
    pub fn new(time: SystemTime) -> Self {
        Self(time)
    }

    /// Get the inner system time
    pub fn inner(&self) -> SystemTime {
        self.0
    }

    /// Calculate the duration since this timestamp
    pub fn elapsed(&self) -> Duration {
        SystemTime::now()
            .duration_since(self.0)
            .unwrap_or(Duration::from_secs(0))
    }
}

impl Default for Timestamp {
    fn default() -> Self {
        Self::now()
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Format as ISO 8601 with RFC 3339 (YYYY-MM-DD'T'HH:MM:SS'Z')
        match self.0.duration_since(SystemTime::UNIX_EPOCH) {
            Ok(duration) => {
                let secs = duration.as_secs();
                let nanos = duration.subsec_nanos();

                let secs_f64 = secs as f64 + (nanos as f64 / 1_000_000_000.0);

                // This is a simplified implementation; a real one would use chrono or another
                // time library to properly format the timestamp
                write!(f, "{:.6}s since epoch", secs_f64)
            }
            Err(_) => write!(f, "invalid time"),
        }
    }
}

/// A version number following semantic versioning
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    /// Major version (incompatible API changes)
    pub major: u16,
    /// Minor version (backwards-compatible functionality)
    pub minor: u16,
    /// Patch version (backwards-compatible bug fixes)
    pub patch: u16,
}

impl Version {
    /// Create a new version number
    pub fn new(major: u16, minor: u16, patch: u16) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl Default for Version {
    fn default() -> Self {
        Self {
            major: 0,
            minor: 1,
            patch: 0,
        }
    }
}

/// HTTP methods used in the framework
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    /// GET method
    Get,
    /// POST method
    Post,
    /// PUT method
    Put,
    /// DELETE method
    Delete,
    /// PATCH method
    Patch,
    /// HEAD method
    Head,
    /// OPTIONS method
    Options,
}

impl fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Get => write!(f, "GET"),
            Self::Post => write!(f, "POST"),
            Self::Put => write!(f, "PUT"),
            Self::Delete => write!(f, "DELETE"),
            Self::Patch => write!(f, "PATCH"),
            Self::Head => write!(f, "HEAD"),
            Self::Options => write!(f, "OPTIONS"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    #[test]
    fn string_map() {
        let mut map = StringMap::new();
        map.insert("key".to_string(), "value".to_string());
        assert_eq!(map.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn timestamp() {
        let ts = Timestamp::now();
        sleep(Duration::from_millis(10));
        assert!(ts.elapsed() >= Duration::from_millis(10));
    }

    #[test]
    fn version() {
        let v = Version::new(1, 2, 3);
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
        assert_eq!(v.to_string(), "1.2.3");
    }

    #[test]
    fn http_method() {
        let method = HttpMethod::Get;
        assert_eq!(method.to_string(), "GET");
    }
}
