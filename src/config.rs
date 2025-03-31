use dotenvy::dotenv;
use navius_core::error::ConfigError;
use navius_http::config::HttpConfig;
use serde::{Deserialize, Serialize};
use std::env;
use std::path::Path;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// HTTP server configuration
    pub http: HttpConfig,
    /// Log level
    pub log_level: String,
    /// App name
    pub app_name: String,
    /// Authentication configuration
    #[serde(default)]
    pub auth: Option<AuthConfig>,
    /// Database configuration
    #[serde(default)]
    pub database: Option<DatabaseConfig>,
    /// Cache configuration
    #[serde(default)]
    pub cache: Option<CacheConfig>,
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Enabled
    pub enabled: bool,
    /// Provider
    pub provider: String,
    /// Entra configuration
    #[serde(default)]
    pub entra: Option<EntraConfig>,
}

/// Entra configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntraConfig {
    /// Tenant ID
    pub tenant_id: String,
    /// Client ID
    pub client_id: String,
    /// Client secret
    pub client_secret: String,
    /// Issuer
    pub issuer: String,
    /// JWKS URI
    pub jwks_uri: String,
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Enabled
    pub enabled: bool,
    /// URL
    pub url: String,
    /// Maximum connections
    pub max_connections: u32,
    /// Minimum connections
    pub min_connections: u32,
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Enabled
    pub enabled: bool,
    /// URL
    pub url: String,
    /// TTL in seconds
    pub ttl_seconds: u64,
}

/// Load application configuration
pub async fn load_config() -> Result<AppConfig, ConfigError> {
    // Load .env file if it exists
    let _ = dotenv();
    
    // Determine the configuration directory
    let config_dir = env::var("CONFIG_DIR").unwrap_or_else(|_| "config".to_string());
    
    // Determine the runtime environment
    let environment = env::var("NAVIUS_ENV").unwrap_or_else(|_| "development".to_string());
    
    tracing::info!("Loading configuration for environment: {}", environment);
    
    // Build configuration from multiple sources
    let config_builder = config::Config::builder()
        // Start with default configuration
        .add_source(config::File::from(Path::new(&config_dir).join("default")))
        // Add environment-specific configuration
        .add_source(
            config::File::from(Path::new(&config_dir).join(&environment))
                .required(false),
        )
        // Add local overrides (not checked into version control)
        .add_source(
            config::File::from(Path::new(&config_dir).join("local"))
                .required(false),
        )
        // Add environment variables with prefix NAVIUS_
        .add_source(config::Environment::with_prefix("NAVIUS").separator("__"));
    
    // Try to build the configuration
    let settings = config_builder
        .build()
        .map_err(|e| ConfigError::LoadError(e.to_string()))?;
    
    // Deserialize the configuration
    let app_config: AppConfig = settings
        .try_deserialize()
        .map_err(|e| ConfigError::ParseError(e.to_string()))?;
    
    // Validate the configuration
    validate_config(&app_config)?;
    
    Ok(app_config)
}

fn validate_config(config: &AppConfig) -> Result<(), ConfigError> {
    // Validate HTTP configuration
    if config.http.port == 0 {
        return Err(ConfigError::ValidationError("HTTP port cannot be 0".to_string()));
    }
    
    // Validate database configuration if enabled
    if let Some(db_config) = &config.database {
        if db_config.enabled && db_config.url.is_empty() {
            return Err(ConfigError::ValidationError("Database URL cannot be empty when database is enabled".to_string()));
        }
    }
    
    // Validate cache configuration if enabled
    if let Some(cache_config) = &config.cache {
        if cache_config.enabled && cache_config.url.is_empty() {
            return Err(ConfigError::ValidationError("Cache URL cannot be empty when cache is enabled".to_string()));
        }
    }
    
    // Validate auth configuration if enabled
    if let Some(auth_config) = &config.auth {
        if auth_config.enabled {
            match auth_config.provider.as_str() {
                "entra" => {
                    if let Some(entra_config) = &auth_config.entra {
                        if entra_config.tenant_id.is_empty() || entra_config.client_id.is_empty() {
                            return Err(ConfigError::ValidationError("Entra configuration is incomplete".to_string()));
                        }
                    } else {
                        return Err(ConfigError::ValidationError("Entra configuration is missing".to_string()));
                    }
                },
                _ => return Err(ConfigError::ValidationError(format!("Unsupported auth provider: {}", auth_config.provider))),
            }
        }
    }
    
    Ok(())
}
