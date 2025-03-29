---
title: "Configuration Example"
description: ""
category: "Documentation"
tags: []
last_updated: "March 28, 2025"
version: "1.0"
---

---
title: "Configuration Example"
description: "Working with configuration in Navius applications"
category: examples
tags:
  - examples
  - configuration
  - settings
related:
  - examples/basic-application-example.md
  - guides/configuration.md
last_updated: March 27, 2025
version: 1.0
---

# Configuration Example

This example demonstrates how to work with configuration in Navius applications. Configuration is a critical aspect of any application, and Navius provides flexible ways to manage settings across different environments.

## Project Structure

```
configuration-example/
├── Cargo.toml
├── config/
│   ├── default.yaml           # Default configuration (all environments)
│   ├── development.yaml       # Development-specific overrides
│   ├── production.yaml        # Production-specific overrides
│   └── test.yaml              # Test-specific overrides
└── src/
    ├── main.rs
    ├── app/
    │   ├── mod.rs
    │   └── services/
    │       ├── mod.rs
    │       └── config_service.rs
    └── core/
        ├── mod.rs
        ├── config.rs
        └── error.rs
```

## Implementation

### Core Configuration Module

#### `core/config.rs`

```
use config::{Config, ConfigBuilder, ConfigError, Environment, File};
use serde::Deserialize;
use std::env;
use std::sync::Arc;

// Database configuration
#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub username: String,
    pub password: String,
    pub max_connections: u32,
    pub connection_timeout_ms: u64,
}

// Server configuration
#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub timeout_seconds: u64,
    pub cors_allowed_origins: Vec<String>,
}

// Logging configuration
#[derive(Debug, Clone, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
    pub output_file: Option<String>,
}

// Feature flags
#[derive(Debug, Clone, Deserialize)]
pub struct FeatureFlags {
    pub enable_metrics: bool,
    pub enable_tracing: bool,
    pub enable_cache: bool,
    pub enable_auth: bool,
}

// Complete application configuration
#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub app_name: String,
    pub environment: String,
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub logging: LoggingConfig,
    pub features: FeatureFlags,
    pub api_keys: std::collections::HashMap<String, String>,
}

// Default implementation
impl Default for AppConfig {
    fn default() -> Self {
        Self {
            app_name: "Default App".to_string(),
            environment: "development".to_string(),
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 3000,
                timeout_seconds: 30,
                cors_allowed_origins: vec!["http://localhost:3000".to_string()],
            },
            database: DatabaseConfig {
                url: "postgres://localhost/app".to_string(),
                username: "app_user".to_string(),
                password: "password".to_string(),
                max_connections: 10,
                connection_timeout_ms: 5000,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                format: "json".to_string(),
                output_file: None,
            },
            features: FeatureFlags {
                enable_metrics: true,
                enable_tracing: true,
                enable_cache: true,
                enable_auth: true,
            },
            api_keys: std::collections::HashMap::new(),
        }
    }
}

// Configuration loader
pub fn load_config() -> Result<Arc<AppConfig>, ConfigError> {
    // Determine which environment we're running in
    let environment = env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());
    
    // Start with default configuration
    let mut builder = Config::builder()
        .add_source(File::with_name("config/default"));
    
    // Add environment-specific configuration
    builder = builder.add_source(File::with_name(&format!("config/{}", environment)).required(false));
    
    // Override with environment variables (e.g., APP_SERVER__PORT=8080)
    builder = builder.add_source(Environment::with_prefix("APP").separator("__"));
    
    // Build the configuration
    let config = builder.build()?;
    
    // Deserialize into our config struct
    let app_config: AppConfig = config.try_deserialize()?;
    
    // Wrap in Arc for sharing
    Ok(Arc::new(app_config))
}

// Test-specific configuration loader
#[cfg(test)]
pub fn load_test_config() -> Arc<AppConfig> {
    let mut builder = Config::builder()
        .add_source(File::with_name("config/default"))
        .add_source(File::with_name("config/test"));
    
    match builder.build() {
        Ok(config) => {
            match config.try_deserialize::<AppConfig>() {
                Ok(app_config) => Arc::new(app_config),
                Err(_) => Arc::new(AppConfig::default()),
            }
        },
        Err(_) => Arc::new(AppConfig::default()),
    }
}
```

### Application Configuration Service

A service to access configuration values in a type-safe way.

#### `app/services/config_service.rs`

```
use crate::core::config::AppConfig;
use std::sync::Arc;

// Configuration service for accessing config values
pub struct ConfigService {
    config: Arc<AppConfig>,
}

impl ConfigService {
    pub fn new(config: Arc<AppConfig>) -> Self {
        Self { config }
    }
    
    // Get the entire configuration
    pub fn get_config(&self) -> Arc<AppConfig> {
        self.config.clone()
    }
    
    // Get application name
    pub fn get_app_name(&self) -> &str {
        &self.config.app_name
    }
    
    // Get current environment
    pub fn get_environment(&self) -> &str {
        &self.config.environment
    }
    
    // Check if running in production
    pub fn is_production(&self) -> bool {
        self.config.environment == "production"
    }
    
    // Get server host and port as a formatted string
    pub fn get_server_address(&self) -> String {
        format!("{}:{}", self.config.server.host, self.config.server.port)
    }
    
    // Check if a feature is enabled
    pub fn is_feature_enabled(&self, feature: &str) -> bool {
        match feature {
            "metrics" => self.config.features.enable_metrics,
            "tracing" => self.config.features.enable_tracing,
            "cache" => self.config.features.enable_cache,
            "auth" => self.config.features.enable_auth,
            _ => false,
        }
    }
    
    // Get API key for a service
    pub fn get_api_key(&self, service: &str) -> Option<String> {
        self.config.api_keys.get(service).cloned()
    }
    
    // Get database connection string
    pub fn get_database_url(&self) -> String {
        let db = &self.config.database;
        format!("{}?user={}&password={}", db.url, db.username, db.password)
    }
}
```

### Using Configuration in Main

#### `main.rs`

```
use std::net::SocketAddr;
use std::sync::Arc;
use axum::{Router, routing::get};
use crate::app::services::config_service::ConfigService;
use crate::core::config::load_config;
use crate::core::error::AppError;

mod app;
mod core;

// Handler to show current configuration
async fn show_config(config_service: Arc<ConfigService>) -> String {
    let mut config_info = String::new();
    
    config_info.push_str(&format!("App Name: {}\n", config_service.get_app_name()));
    config_info.push_str(&format!("Environment: {}\n", config_service.get_environment()));
    config_info.push_str(&format!("Server Address: {}\n", config_service.get_server_address()));
    
    config_info.push_str("\nFeature Flags:\n");
    for feature in &["metrics", "tracing", "cache", "auth"] {
        config_info.push_str(&format!("  - {}: {}\n", feature, config_service.is_feature_enabled(feature)));
    }
    
    config_info
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    // Load configuration
    let config = load_config()
        .map_err(|e| AppError::configuration_error(format!("Failed to load configuration: {}", e)))?;
    
    // Create configuration service
    let config_service = Arc::new(ConfigService::new(config.clone()));
    
    // Set up logging based on configuration
    let log_level = &config.logging.level;
    println!("Setting up logging with level: {}", log_level);
    
    // Configure tracing if enabled
    if config_service.is_feature_enabled("tracing") {
        println!("Initializing tracing...");
        tracing_subscriber::fmt::init();
    }
    
    // Print startup information
    println!("Starting {} in {} mode", config_service.get_app_name(), config_service.get_environment());
    
    // Create router
    let app = Router::new()
        .route("/config", get(|_| show_config(config_service.clone())))
        .with_state(config.clone());
    
    // Extract server address
    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port).parse()
        .map_err(|_| AppError::configuration_error("Invalid server address"))?;
    
    // Start server
    println!("Listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .map_err(|e| AppError::internal_server_error(format!("Server error: {}", e)))?;
    
    Ok(())
}
```

## Configuration Files

### `config/default.yaml`

```
app_name: "Configuration Example"
environment: "development"

server:
  host: "127.0.0.1"
  port: 3000
  timeout_seconds: 30
  cors_allowed_origins:
    - "http://localhost:3000"
    - "http://localhost:8080"

database:
  url: "postgres://localhost/app_dev"
  username: "dev_user"
  password: "dev_password"
  max_connections: 10
  connection_timeout_ms: 5000

logging:
  level: "debug"
  format: "json"
  output_file: null

features:
  enable_metrics: true
  enable_tracing: true
  enable_cache: true
  enable_auth: false

api_keys:
  weather_service: "dev_weather_api_key"
  payment_gateway: "dev_payment_api_key"
```

### `config/production.yaml`

```
environment: "production"

server:
  host: "0.0.0.0"
  port: 8080
  timeout_seconds: 60
  cors_allowed_origins:
    - "https://app.example.com"

database:
  url: "postgres://db.example.com/app_prod"
  username: "prod_user"
  password: "prod_password"
  max_connections: 50
  connection_timeout_ms: 3000

logging:
  level: "info"
  format: "json"
  output_file: "/var/log/app.log"

features:
  enable_metrics: true
  enable_tracing: true
  enable_cache: true
  enable_auth: true

api_keys:
  weather_service: "${WEATHER_API_KEY}"
  payment_gateway: "${PAYMENT_API_KEY}"
```

### `config/test.yaml`

```
environment: "test"

server:
  port: 0  # Use random port for tests

database:
  url: "postgres://localhost/app_test"
  username: "test_user"
  password: "test_password"

logging:
  level: "error"
  output_file: null

features:
  enable_metrics: false
  enable_tracing: false
```

## Running the Example

1. Clone the Navius repository
2. Navigate to the `examples/configuration-example` directory
3. Run with default configuration:

```
cargo run
```

4. Run with production configuration:

```
APP_ENV=production cargo run
```

5. Override specific values with environment variables:

```
APP_SERVER__PORT=9000 cargo run
```

6. Access the configuration endpoint:

```
curl http://localhost:3000/config
```

## Key Concepts Demonstrated

1. **Hierarchical Configuration**: Default settings with environment-specific overrides
2. **Configuration Abstraction**: Type-safe access to configuration values
3. **Environment Variables**: Overriding configuration via environment variables
4. **Feature Flags**: Enabling/disabling features based on configuration
5. **Sensitive Information**: Handling API keys and credentials

## Best Practices

1. **Default Values**: Always provide reasonable defaults in `default.yaml`
2. **Environment Separation**: Use separate files for different environments
3. **Environmental Security**: Keep sensitive information in environment variables
4. **Configuration Service**: Use a service to provide clean access to configuration
5. **Feature Flags**: Use configuration to toggle features rather than compilation flags
6. **Validation**: Validate configuration at startup to fail fast

## Next Steps

- [Dependency Injection Example](dependency-injection-example.md): Using configuration with dependency injection
- [Error Handling Example](error-handling-example.md): Error handling strategies
- [Database Service Example](database-service-example.md): Configuring database connections 
