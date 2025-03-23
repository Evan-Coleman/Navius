# Application Configuration

This directory contains user-facing configuration functionality for the Navius application. Use this module to access and extend the core configuration system for your specific application needs.

## Usage

To use configuration in your application code:

```rust
use crate::app::config;

fn example() {
    // Get the entire application config
    let app_config = config::get_config();
    
    // Get specific config sections
    let server_config = config::get_server_config();
    let cache_config = config::get_cache_config();
    
    // Access configuration values
    let server_port = server_config.port;
    let cache_ttl = cache_config.ttl_seconds;
    
    println!("Server running on port {}", server_port);
}
```

## Extending Configuration

### Adding Custom Configuration Sections

Create a new configuration section by extending the AppConfig struct:

```rust
// src/app/config/app_settings.rs
use serde::Deserialize;
use config::{Config, ConfigError};

#[derive(Debug, Deserialize, Clone)]
pub struct FeatureFlagConfig {
    pub enable_new_ui: bool,
    pub enable_beta_features: bool,
    pub max_items_per_page: i32,
}

impl Default for FeatureFlagConfig {
    fn default() -> Self {
        Self {
            enable_new_ui: false,
            enable_beta_features: false,
            max_items_per_page: 50,
        }
    }
}

pub fn load_feature_flags(config: &Config) -> Result<FeatureFlagConfig, ConfigError> {
    let feature_flags: FeatureFlagConfig = config
        .get("feature_flags")
        .unwrap_or_else(|_| FeatureFlagConfig::default());
    
    Ok(feature_flags)
}

// Then extend the main app config getter
pub fn get_feature_flags() -> FeatureFlagConfig {
    let config = crate::core::config::app_config::get_raw_config()
        .expect("Failed to load raw configuration");
    
    load_feature_flags(&config)
        .expect("Failed to load feature flags configuration")
}
```

### Adding Configuration Files

Add custom configuration files in the `config` directory:

```yaml
# config/default.yaml (add to existing file)
feature_flags:
  enable_new_ui: false
  enable_beta_features: false
  max_items_per_page: 50

# config/development.yaml (environment-specific overrides)
feature_flags:
  enable_new_ui: true
  enable_beta_features: true
```

## Best Practices

1. Use strongly typed configuration with struct validation
2. Provide sensible defaults for all configuration values
3. Document each configuration option with examples and descriptions
4. Keep sensitive configuration separate (use environment variables)
5. Use environment-specific configuration files for different settings
6. Validate configuration at startup to fail fast
7. Follow the design patterns established in the core configuration

## Core Configuration System

The core configuration system is provided by `crate::core::config` and includes:

- Configuration loading from files and environment variables
- Typed configuration structures
- Environment-specific configuration
- Configuration validation

Do not modify the core configuration system directly. Instead, use this directory to extend and customize configuration for your specific application needs. 