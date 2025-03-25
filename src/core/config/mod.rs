pub mod app_config;
pub mod constants;
#[cfg(test)]
mod tests;

pub use app_config::AppConfig;
pub use app_config::load_config;
use app_config::{
    ApiConfig, AuthConfig, CacheConfig, LoggingConfig, ReliabilityConfig, ServerConfig,
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
            api: ApiConfig::default(),
            logging: LoggingConfig::default(),
            cache: CacheConfig::default(),
            auth: AuthConfig::default(),
            reliability: ReliabilityConfig::default(),
            openapi: app_config::OpenApiConfig::default(),
            endpoint_security: app_config::EndpointSecurityConfig::default(),
        }
    }
}
