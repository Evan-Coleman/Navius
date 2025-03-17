use config::{Config, ConfigError, Environment, File};
use dotenvy::dotenv;
use serde::{Deserialize, Serialize};
use std::env;
use std::path::Path;
use std::time::Duration;
use tracing::info;

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub timeout_seconds: u64,
    pub max_retries: u32,
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub enabled: bool,
    pub ttl_seconds: u64,
    pub max_capacity: u64,
}

/// API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub cat_fact_url: String,
    pub petstore_url: String,
    pub api_key: Option<String>,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub response_fields: ResponseLoggingConfig,
}

/// Response logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseLoggingConfig {
    pub cat_fact_fields: Vec<String>,
    pub health_fields: Vec<String>,
    pub api_response_fields: Vec<String>,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            response_fields: ResponseLoggingConfig {
                cat_fact_fields: vec!["fact".to_string(), "length".to_string()],
                health_fields: vec!["status".to_string(), "uptime_seconds".to_string()],
                api_response_fields: vec!["code".to_string(), "message".to_string()],
            },
        }
    }
}

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub api: ApiConfig,
    pub app: ApplicationConfig,
    pub cache: CacheConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
}

/// Application metadata configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationConfig {
    pub name: String,
    pub version: String,
    pub log_level: String,
}

impl AppConfig {
    pub fn new() -> Result<Self, ConfigError> {
        // Use the load_config function which implements the layered configuration approach
        load_config()
    }

    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }

    pub fn cache_ttl(&self) -> Duration {
        Duration::from_secs(self.cache.ttl_seconds)
    }

    /// Get the full Petstore API URL
    pub fn petstore_api_url(&self) -> String {
        self.api.petstore_url.trim_end_matches('/').to_string()
    }
}

/// Load configuration from files and environment variables
pub fn load_config() -> Result<AppConfig, ConfigError> {
    // Load .env file for secrets and overrides
    let _ = dotenv();

    // Determine the configuration directory
    let config_dir = env::var("CONFIG_DIR").unwrap_or_else(|_| "./config".to_string());

    // Determine the environment (development, production, etc.)
    let environment = env::var("RUN_ENV").unwrap_or_else(|_| "development".to_string());

    info!("Loading configuration for environment: {}", environment);

    // Build configuration with the following priority (highest to lowest):
    // 1. Environment variables (for secrets and CI/CD overrides)
    // 2. Environment-specific local overrides (local-{env}.yaml - not in version control)
    // 3. Environment-specific config ({env}.yaml)
    // 4. Local overrides (local.yaml - not in version control)
    // 5. Default config (default.yaml)
    let config = Config::builder()
        // 5. Start with default settings
        .add_source(File::from(Path::new(&config_dir).join("default.yaml")).required(false))
        // 4. Add local settings (not in version control)
        .add_source(File::from(Path::new(&config_dir).join("local.yaml")).required(false))
        // 3. Add environment-specific settings
        .add_source(
            File::from(Path::new(&config_dir).join(format!("{}.yaml", environment)))
                .required(false),
        )
        // 2. Add environment-specific local overrides (not in version control)
        .add_source(
            File::from(Path::new(&config_dir).join(format!("local-{}.yaml", environment)))
                .required(false),
        )
        // 1. Add environment variables (highest priority, for secrets and CI/CD)
        // Only use specific prefixes for environment variables to avoid conflicts
        .add_source(Environment::with_prefix("SERVER").separator("_"))
        .add_source(Environment::with_prefix("API").separator("_"))
        .add_source(Environment::with_prefix("APP").separator("_"))
        .add_source(Environment::with_prefix("CACHE").separator("_"))
        // Build the config
        .build()?;

    // Deserialize the config into our settings struct
    let app_config: AppConfig = config.try_deserialize()?;

    info!("Configuration loaded successfully");

    Ok(app_config)
}
