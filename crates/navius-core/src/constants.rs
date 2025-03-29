//! Common constants used throughout the Navius framework.
//!
//! This module provides constant values that are used across different parts of the framework.

/// Default values for configuration and initialization.
pub mod defaults {
    /// Default server port.
    pub const SERVER_PORT: u16 = 3000;

    /// Default server host.
    pub const SERVER_HOST: &str = "127.0.0.1";

    /// Default application name.
    pub const APP_NAME: &str = "navius-app";

    /// Default log level.
    pub const LOG_LEVEL: &str = "info";

    /// Default environment.
    pub const ENVIRONMENT: &str = "development";

    /// Default request timeout in seconds.
    pub const REQUEST_TIMEOUT_SECS: u64 = 30;

    /// Default connection pool size.
    pub const DB_POOL_SIZE: u32 = 5;

    /// Default connection timeout in seconds.
    pub const DB_CONNECTION_TIMEOUT_SECS: u64 = 5;

    /// Default maximum connections in pool.
    pub const DB_MAX_CONNECTIONS: u32 = 20;

    /// Default health check interval in seconds.
    pub const HEALTH_CHECK_INTERVAL_SECS: u64 = 60;
}

/// HTTP header names.
pub mod headers {
    /// Request ID header.
    pub const REQUEST_ID: &str = "X-Request-Id";

    /// Correlation ID header.
    pub const CORRELATION_ID: &str = "X-Correlation-Id";

    /// API key header.
    pub const API_KEY: &str = "X-Api-Key";

    /// Authorization header.
    pub const AUTHORIZATION: &str = "Authorization";

    /// Content type header.
    pub const CONTENT_TYPE: &str = "Content-Type";

    /// Accept header.
    pub const ACCEPT: &str = "Accept";

    /// User agent header.
    pub const USER_AGENT: &str = "User-Agent";
}

/// Common MIME types.
pub mod mime_types {
    /// JSON MIME type.
    pub const JSON: &str = "application/json";

    /// XML MIME type.
    pub const XML: &str = "application/xml";

    /// Text MIME type.
    pub const TEXT: &str = "text/plain";

    /// HTML MIME type.
    pub const HTML: &str = "text/html";

    /// Form URL encoded MIME type.
    pub const FORM: &str = "application/x-www-form-urlencoded";

    /// Multipart form data MIME type.
    pub const MULTIPART: &str = "multipart/form-data";
}

/// Environment variable names.
pub mod env_vars {
    /// Application environment.
    pub const APP_ENV: &str = "APP_ENV";

    /// Application port.
    pub const PORT: &str = "PORT";

    /// Application host.
    pub const HOST: &str = "HOST";

    /// Log level.
    pub const LOG_LEVEL: &str = "LOG_LEVEL";

    /// Database URL.
    pub const DATABASE_URL: &str = "DATABASE_URL";

    /// Redis URL.
    pub const REDIS_URL: &str = "REDIS_URL";

    /// JWT secret.
    pub const JWT_SECRET: &str = "JWT_SECRET";

    /// Config path.
    pub const CONFIG_PATH: &str = "CONFIG_PATH";
}

/// Path constants.
pub mod paths {
    /// Health check path.
    pub const HEALTH: &str = "/actuator/health";

    /// Metrics path.
    pub const METRICS: &str = "/actuator/metrics";

    /// Info path.
    pub const INFO: &str = "/actuator/info";

    /// API base path.
    pub const API_BASE: &str = "/api";

    /// API version 1 path.
    pub const API_V1: &str = "/api/v1";

    /// Documentation path.
    pub const DOCS: &str = "/docs";

    /// OpenAPI path.
    pub const OPENAPI: &str = "/openapi.json";
}

/// Time constants.
pub mod time {
    /// One second in milliseconds.
    pub const SECOND_MS: i64 = 1_000;

    /// One minute in milliseconds.
    pub const MINUTE_MS: i64 = 60 * SECOND_MS;

    /// One hour in milliseconds.
    pub const HOUR_MS: i64 = 60 * MINUTE_MS;

    /// One day in milliseconds.
    pub const DAY_MS: i64 = 24 * HOUR_MS;

    /// One week in milliseconds.
    pub const WEEK_MS: i64 = 7 * DAY_MS;

    /// Default token expiration time in seconds.
    pub const DEFAULT_TOKEN_EXPIRY_SECS: i64 = 3600;
}

/// Validation constants.
pub mod validation {
    /// Minimum username length.
    pub const MIN_USERNAME_LENGTH: usize = 3;

    /// Maximum username length.
    pub const MAX_USERNAME_LENGTH: usize = 50;

    /// Minimum password length.
    pub const MIN_PASSWORD_LENGTH: usize = 8;

    /// Maximum password length.
    pub const MAX_PASSWORD_LENGTH: usize = 100;

    /// Email regex pattern.
    pub const EMAIL_REGEX: &str = r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$";

    /// Username regex pattern (alphanumeric and underscore).
    pub const USERNAME_REGEX: &str = r"^[a-zA-Z0-9_]+$";
}

/// Feature flag constants.
pub mod features {
    /// Authentication feature flag.
    pub const AUTH: &str = "auth";

    /// Metrics feature flag.
    pub const METRICS: &str = "metrics";

    /// Tracing feature flag.
    pub const TRACING: &str = "tracing";

    /// Database feature flag.
    pub const DATABASE: &str = "database";

    /// Redis feature flag.
    pub const REDIS: &str = "redis";

    /// Caching feature flag.
    pub const CACHING: &str = "caching";

    /// Development feature flag.
    pub const DEVELOPMENT: &str = "development";

    /// Production feature flag.
    pub const PRODUCTION: &str = "production";
}
