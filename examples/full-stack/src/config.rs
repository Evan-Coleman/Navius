use navius_core::config::{ConfigError, ConfigManager, Environment};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub idle_timeout_seconds: u64,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "postgres://navius:navius_password@localhost:5433/navius_fullstack".to_string(),
            max_connections: 10,
            idle_timeout_seconds: 300,
        }
    }
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub url: String,
    pub ttl_seconds: u64,
    pub pool_size: u32,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            url: "redis://localhost:6379".to_string(),
            ttl_seconds: 3600,
            pool_size: 5,
        }
    }
}

/// HTTP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpConfig {
    pub host: String,
    pub port: u16,
    pub request_timeout_ms: u64,
    pub graceful_shutdown_timeout_ms: u64,
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            request_timeout_ms: 30000,
            graceful_shutdown_timeout_ms: 5000,
        }
    }
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub token_expiry_seconds: u64,
    pub refresh_token_expiry_seconds: u64,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            jwt_secret: "navius-development-secret-key".to_string(),
            token_expiry_seconds: 3600,
            refresh_token_expiry_seconds: 86400,
        }
    }
}

/// Email configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub from_address: String,
    pub username: String,
    pub password: String,
}

impl Default for EmailConfig {
    fn default() -> Self {
        Self {
            smtp_host: "localhost".to_string(),
            smtp_port: 1025, // Mailhog default
            from_address: "noreply@example.com".to_string(),
            username: "".to_string(),
            password: "".to_string(),
        }
    }
}

/// Metrics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    pub enabled: bool,
    pub prometheus_endpoint: String,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            prometheus_endpoint: "/metrics".to_string(),
        }
    }
}

/// Main application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub environment: String,
    pub log_level: String,
    pub database: DatabaseConfig,
    pub cache: CacheConfig,
    pub http: HttpConfig,
    pub auth: AuthConfig,
    pub email: EmailConfig,
    pub metrics: MetricsConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            environment: "development".to_string(),
            log_level: "debug".to_string(),
            database: DatabaseConfig::default(),
            cache: CacheConfig::default(),
            http: HttpConfig::default(),
            auth: AuthConfig::default(),
            email: EmailConfig::default(),
            metrics: MetricsConfig::default(),
        }
    }
}

/// Load configuration from file and environment variables
pub fn load_config() -> Result<Arc<AppConfig>, ConfigError> {
    let config_path = std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config".to_string());

    // Determine environment
    let env_string = std::env::var("NAVIUS_ENV").unwrap_or_else(|_| "development".to_string());
    let env = match env_string.as_str() {
        "production" => Environment::Production,
        "staging" => Environment::Staging,
        _ => Environment::Development,
    };

    // Initialize config manager
    let config_manager = ConfigManager::new(Path::new(&config_path), env);

    // Load configuration
    let config = config_manager.load::<AppConfig>()?;

    Ok(Arc::new(config))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.environment, "development");
        assert_eq!(config.http.port, 8080);
        assert!(config.metrics.enabled);
    }
}
