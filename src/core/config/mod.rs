pub mod app_config;
pub mod constants;
#[cfg(test)]
mod tests;

pub use app_config::AppConfig;
pub use app_config::load_config;
use app_config::{
    ApiConfig, AuthConfig, CacheConfig, DatabaseConfig, LoggingConfig, ReliabilityConfig,
    ServerConfig,
};

use lazy_static::lazy_static;
use std::default::Default;
use std::sync::Arc;

lazy_static! {
    pub static ref CONFIG: Arc<AppConfig> =
        Arc::new(load_config().expect("Failed to load configuration"));
}

/// Default implementation for tests
impl Default for AppConfig {
    fn default() -> Self {
        Self {
            environment: app_config::EnvironmentType::Development,
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 3000,
                timeout_seconds: 30,
                max_retries: 3,
                protocol: "http".to_string(),
            },
            api: ApiConfig {
                petstore_url: String::from("https://petstore3.swagger.io/api/v3"),
                api_key: None,
            },
            logging: LoggingConfig {
                level: String::from("info"),
                format: String::from("json"),
            },
            cache: CacheConfig {
                enabled: true,
                max_capacity: 100,
                ttl_seconds: 3600,
                reconnect_interval_seconds: 30,
            },
            database: DatabaseConfig::default(),
            auth: AuthConfig::default(),
            reliability: ReliabilityConfig::default(),
            openapi: app_config::OpenApiConfig::default(),
            endpoint_security: app_config::EndpointSecurityConfig::default(),
        }
    }
}
