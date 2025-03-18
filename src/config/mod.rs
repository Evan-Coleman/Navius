pub mod app_config;
pub mod constants;
#[cfg(test)]
mod tests;

pub use app_config::AppConfig;
pub use app_config::load_config;

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
            env: String::from("development"),
            server: ServerConfig {
                port: 3000,
                timeout_seconds: 30,
                max_retries: 3,
            },
            api: ApiConfig {
                petstore_url: String::from("https://petstore3.swagger.io/api/v3"),
                cat_fact_url: String::from("https://catfact.ninja/fact"),
            },
            logging: LoggingConfig {
                level: String::from("info"),
                format: String::from("json"),
            },
            cache: CacheConfig {
                enabled: true,
                max_capacity: 100,
                ttl_seconds: 3600,
            },
            auth: AuthConfig::default(),
            reliability: ReliabilityConfig::default(),
        }
    }
}
