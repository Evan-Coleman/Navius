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

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub api: ApiConfig,
    pub app: ApplicationConfig,
    pub cache: CacheConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationConfig {
    pub name: String,
    pub version: String,
    pub log_level: String,
}

impl AppConfig {
    pub fn new() -> Result<Self, ConfigError> {
        // Load .env file if it exists
        let _ = dotenv();

        let config = Config::builder()
            // Start with default settings
            .set_default("server.host", "127.0.0.1")?
            .set_default("server.port", 3000)?
            .set_default("server.timeout_seconds", 10)?
            .set_default("server.max_retries", 3)?
            .set_default("api.cat_fact_url", "https://catfact.ninja/fact")?
            .set_default("api.petstore_url", "https://petstore3.swagger.io/api/v3")?
            .set_default("app.name", "Petstore API Server")?
            .set_default("app.version", "1.0.0")?
            .set_default("app.log_level", "info")?
            .set_default("cache.enabled", true)?
            .set_default("cache.ttl_seconds", 300)? // 5 minutes
            .set_default("cache.max_capacity", 1000)?
            // Add config file
            .add_source(File::with_name("config").required(false))
            // Add environment variables with prefix
            .add_source(
                Environment::with_prefix("APP")
                    .separator("_")
                    .prefix_separator("_")
                    .keep_prefix(false),
            )
            .add_source(
                Environment::with_prefix("SERVER")
                    .separator("_")
                    .prefix_separator("_")
                    .keep_prefix(false),
            )
            .add_source(
                Environment::with_prefix("API")
                    .separator("_")
                    .prefix_separator("_")
                    .keep_prefix(false),
            )
            .add_source(
                Environment::with_prefix("CACHE")
                    .separator("_")
                    .prefix_separator("_")
                    .keep_prefix(false),
            )
            .build()?;

        // Deserialize the configuration
        config.try_deserialize()
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
    // Determine the configuration directory
    let config_dir = env::var("CONFIG_DIR").unwrap_or_else(|_| "./config".to_string());

    // Build configuration
    let config = Config::builder()
        // Start with default settings
        .add_source(File::from(Path::new(&config_dir).join("default.yaml")).required(false))
        // Add environment-specific settings
        .add_source(
            File::from(Path::new(&config_dir).join(format!(
                "{}.yaml",
                env::var("RUN_ENV").unwrap_or_else(|_| "development".to_string())
            )))
            .required(false),
        )
        // Add local settings (not in version control)
        .add_source(File::from(Path::new(&config_dir).join("local.yaml")).required(false))
        // Add environment variables with prefix APP_
        .add_source(Environment::with_prefix("APP").separator("__"))
        // Build the config
        .build()?;

    // Deserialize the config into our settings struct
    let app_config: AppConfig = config.try_deserialize()?;

    info!("Configuration loaded successfully");

    Ok(app_config)
}
