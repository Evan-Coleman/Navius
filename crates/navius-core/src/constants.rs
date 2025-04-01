//! Constants for the Navius framework.
//!
//! This module defines constants used throughout the Navius framework.

/// Default application name
pub const DEFAULT_APP_NAME: &str = "navius-app";

/// Default application port
pub const DEFAULT_PORT: u16 = 8080;

/// Default log level
pub const DEFAULT_LOG_LEVEL: &str = "info";

/// Default configuration file name
pub const DEFAULT_CONFIG_FILE: &str = "config.yaml";

/// Environment variable prefix for configuration
pub const CONFIG_ENV_PREFIX: &str = "NAVIUS_";

/// Constants related to timeouts
pub mod timeouts {
    /// Default HTTP request timeout in seconds
    pub const DEFAULT_HTTP_TIMEOUT_SECS: u64 = 30;

    /// Default database connection timeout in seconds
    pub const DEFAULT_DB_TIMEOUT_SECS: u64 = 5;

    /// Default cache operation timeout in seconds
    pub const DEFAULT_CACHE_TIMEOUT_SECS: u64 = 2;
}

/// Constants related to retry policies
pub mod retry {
    /// Default maximum number of retries
    pub const DEFAULT_MAX_RETRIES: u32 = 3;

    /// Default initial retry delay in milliseconds
    pub const DEFAULT_INITIAL_DELAY_MS: u64 = 100;

    /// Default backoff factor for exponential backoff
    pub const DEFAULT_BACKOFF_FACTOR: f64 = 2.0;
}

/// Constants related to health checks
pub mod health {
    /// Default health check interval in seconds
    pub const DEFAULT_CHECK_INTERVAL_SECS: u64 = 15;

    /// Default health check timeout in seconds
    pub const DEFAULT_CHECK_TIMEOUT_SECS: u64 = 5;
}

/// Constants related to HTTP headers
pub mod headers {
    /// Request ID header
    pub const REQUEST_ID: &str = "X-Request-ID";

    /// Correlation ID header
    pub const CORRELATION_ID: &str = "X-Correlation-ID";

    /// Client ID header
    pub const CLIENT_ID: &str = "X-Client-ID";

    /// API Version header
    pub const API_VERSION: &str = "X-API-Version";
}

/// Default values for various settings
pub mod defaults {
    /// Default server port
    pub const SERVER_PORT: u16 = 8080;

    /// Default server host
    pub const SERVER_HOST: &str = "127.0.0.1";

    /// Default request timeout in seconds
    pub const REQUEST_TIMEOUT_SECS: u64 = 30;

    /// Default connection timeout in seconds
    pub const CONNECTION_TIMEOUT_SECS: u64 = 5;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(DEFAULT_APP_NAME, "navius-app");
        assert_eq!(DEFAULT_PORT, 8080);
        assert_eq!(DEFAULT_LOG_LEVEL, "info");
        assert_eq!(DEFAULT_CONFIG_FILE, "config.yaml");
        assert_eq!(CONFIG_ENV_PREFIX, "NAVIUS_");
    }

    #[test]
    fn test_timeout_constants() {
        assert_eq!(timeouts::DEFAULT_HTTP_TIMEOUT_SECS, 30);
        assert_eq!(timeouts::DEFAULT_DB_TIMEOUT_SECS, 5);
        assert_eq!(timeouts::DEFAULT_CACHE_TIMEOUT_SECS, 2);
    }

    #[test]
    fn test_retry_constants() {
        assert_eq!(retry::DEFAULT_MAX_RETRIES, 3);
        assert_eq!(retry::DEFAULT_INITIAL_DELAY_MS, 100);
        assert_eq!(retry::DEFAULT_BACKOFF_FACTOR, 2.0);
    }

    #[test]
    fn test_health_constants() {
        assert_eq!(health::DEFAULT_CHECK_INTERVAL_SECS, 15);
        assert_eq!(health::DEFAULT_CHECK_TIMEOUT_SECS, 5);
    }

    #[test]
    fn test_header_constants() {
        assert_eq!(headers::REQUEST_ID, "X-Request-ID");
        assert_eq!(headers::CORRELATION_ID, "X-Correlation-ID");
        assert_eq!(headers::CLIENT_ID, "X-Client-ID");
        assert_eq!(headers::API_VERSION, "X-API-Version");
    }

    #[test]
    fn test_default_constants() {
        assert_eq!(defaults::SERVER_PORT, 8080);
        assert_eq!(defaults::SERVER_HOST, "127.0.0.1");
        assert_eq!(defaults::REQUEST_TIMEOUT_SECS, 30);
        assert_eq!(defaults::CONNECTION_TIMEOUT_SECS, 5);
    }
}
