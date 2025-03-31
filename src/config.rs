use navius_core::config::{ConfigError, ConfigLoader};
use serde::{Deserialize, Serialize};

/// Application configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    /// HTTP server configuration
    pub http: navius_http::config::HttpConfig,
    /// Log level
    pub log_level: String,
    // Add other configuration sections as needed
}

/// Load application configuration
pub async fn load_config() -> Result<AppConfig, ConfigError> {
    // For now, return a minimal config for development
    Ok(AppConfig {
        http: navius_http::config::HttpConfig {
            host: "127.0.0.1".to_string(),
            port: 8080,
            cors_allowed_origins: vec!["*".to_string()],
            request_timeout_secs: 30,
        },
        log_level: "debug".to_string(),
    })

    // In a real implementation, we would use the ConfigLoader
    // let loader = ConfigLoader::new()
    //     .with_env_prefix("NAVIUS")
    //     .with_default_path("config/default.yaml");
    //
    // loader.load::<AppConfig>().await
}
