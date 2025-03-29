# Configuration

This guide explains the configuration system in Navius, focusing on how to set up, customize, and access application settings.

## Overview

Navius uses a layered configuration system that allows:

1. **Default settings**: Base configuration values
2. **Environment-specific overrides**: Development, testing, production settings
3. **External configuration files**: YAML configuration files
4. **Environment variable overrides**: Runtime configuration
5. **Command-line arguments**: CLI overrides

## Configuration Files

The standard location for configuration files is the `config/` directory:

```
config/
├── default.yaml        # Base settings
├── development.yaml    # Development overrides
├── test.yaml           # Test overrides
└── production.yaml     # Production overrides
```

### Example Configuration File

```yaml
# config/default.yaml
app:
  name: "My Navius App"
  version: "1.0.0"
  
server:
  host: "127.0.0.1"
  port: 3000
  
logging:
  level: "info"
  format: "json"
  
database:
  url: "postgres://localhost:5432/myapp"
  max_connections: 10
  timeout_seconds: 30
  
cache:
  enabled: true
  ttl_seconds: 300
  provider: "memory"
```

## Loading Configuration

Configuration is loaded at application startup:

```rust
use navius::core::config::{Config, ConfigBuilder};

fn main() {
    // Load configuration
    let config = ConfigBuilder::new()
        .add_default()                   // Load default.yaml
        .add_environment()               // Load environment-specific YAML
        .add_environment_variables()     // Override with env vars
        .add_command_line_args()         // Override with CLI args
        .build()
        .expect("Failed to load configuration");
        
    // Use configuration
    let server_port = config.get_int("server.port").unwrap_or(8080);
    println!("Server will start on port {}", server_port);
}
```

## Environment-Specific Configuration

The environment is determined by the `ENVIRONMENT` environment variable (defaulting to "development").

For example, with `ENVIRONMENT=production`:
1. The base `default.yaml` is loaded
2. Values from `production.yaml` override matching keys
3. Environment variables can further override these values

## Accessing Configuration

Configuration values can be accessed in several ways:

### Direct Value Access

```rust
// Get values with expected type
let port = config.get_int("server.port").unwrap_or(8080);
let host = config.get_string("server.host").unwrap_or_else(|| "localhost".to_string());
let cache_enabled = config.get_bool("cache.enabled").unwrap_or(false);
```

### Structured Configuration

You can map configuration sections to structs:

```rust
use serde::Deserialize;

#[derive(Deserialize)]
struct ServerConfig {
    host: String,
    port: u16,
}

let server_config: ServerConfig = config.get_section("server")
    .expect("Server configuration not found");
    
println!("Server: {}:{}", server_config.host, server_config.port);
```

### Config Service

In a typical Navius application, configuration is accessed through the `ConfigService`:

```rust
use std::sync::Arc;
use navius::core::config::{Config, ConfigService};

pub struct AppState {
    pub config_service: Arc<ConfigService>,
}

impl AppState {
    pub fn new() -> Self {
        let config = ConfigBuilder::new()
            .add_default()
            .add_environment()
            .add_environment_variables()
            .build()
            .expect("Failed to load configuration");
            
        let config_service = Arc::new(ConfigService::new(config));
        
        Self {
            config_service,
        }
    }
}

// Usage in a service
struct UserService {
    config: Arc<ConfigService>,
}

impl UserService {
    pub fn new(config: Arc<ConfigService>) -> Self {
        Self { config }
    }
    
    pub fn is_feature_enabled(&self, feature: &str) -> bool {
        self.config.get_bool(&format!("features.{}", feature)).unwrap_or(false)
    }
}
```

## Environment Variable Overrides

Configuration values can be overridden with environment variables using this pattern:

```
APP__NAME="My Overridden App Name"
SERVER__PORT=8080
DATABASE__URL="postgres://user:pass@remote-host:5432/myapp"
```

Notes:
- Double underscores (`__`) separate nested keys
- Case is preserved (though keys in YAML are typically lowercase)

## Command-Line Arguments

Command-line arguments can also override configuration:

```bash
./my-app --server.port=8080 --logging.level=debug
```

## Feature Flags

Feature flags can be managed through configuration:

```yaml
features:
  new_user_flow: true
  experimental_api: false
  rate_limiting: true
```

## Secrets Management

For sensitive configuration:

1. **Environment Variables**: Use environment variables for secrets
2. **External Services**: Support for secrets managers like AWS Secrets Manager or HashiCorp Vault
3. **File-Based Secrets**: Load secrets from secure files

Example secrets configuration:

```yaml
# config/default.yaml
secrets:
  source: "environment"  # or "vault", "aws", "file"
  
  # If using file source
  file:
    path: "/path/to/secrets.json"
    
  # If using vault
  vault:
    url: "https://vault.example.com"
    token_path: "/path/to/vault/token"
```

## Dynamic Configuration

Navius supports dynamic configuration updates:

```rust
// Subscribe to configuration changes
config_service.subscribe(|updated_config| {
    println!("Configuration updated!");
    
    // React to changes
    let new_log_level = updated_config.get_string("logging.level")
        .unwrap_or_else(|| "info".to_string());
        
    update_log_level(&new_log_level);
});

// Trigger update (e.g., from admin API)
config_service.reload().await?;
```

## Best Practices

1. **Use Nesting**: Organize configuration hierarchically
2. **Default Values**: Always provide sensible defaults for optional settings
3. **Environment Separation**: Keep environment-specific settings in their own files
4. **Type Safety**: Use strongly-typed configuration with structs
5. **Documentation**: Document all configuration options
6. **Validation**: Validate configuration at startup

## Complete Example

Here's a complete configuration setup:

```rust
use navius::core::config::{Config, ConfigBuilder, ConfigService};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
struct AppConfig {
    name: String,
    version: String,
}

#[derive(Deserialize)]
struct ServerConfig {
    host: String,
    port: u16,
    timeout_seconds: u32,
}

#[derive(Deserialize)]
struct DatabaseConfig {
    url: String,
    max_connections: u32,
}

#[derive(Deserialize)]
struct LoggingConfig {
    level: String,
    format: String,
}

#[derive(Deserialize)]
struct ServiceConfig {
    app: AppConfig,
    server: ServerConfig,
    database: DatabaseConfig,
    logging: LoggingConfig,
}

fn main() {
    // Load configuration
    let config = ConfigBuilder::new()
        .add_default()
        .add_environment()
        .add_environment_variables()
        .add_command_line_args()
        .build()
        .expect("Failed to load configuration");
    
    // Map to structured config
    let service_config: ServiceConfig = config.get_full()
        .expect("Failed to parse configuration");
        
    println!("Starting {} v{}", 
        service_config.app.name, 
        service_config.app.version);
        
    println!("Server will listen on {}:{}", 
        service_config.server.host, 
        service_config.server.port);
        
    // Create config service
    let config_service = Arc::new(ConfigService::new(config));
    
    // Build application with config
    let app = build_application(config_service);
    
    // Start application...
}
```

## Related Guides

- [Service Registration](service-registration.md) for using configuration with services
- [Environment Variables](environment-variables.md) for managing runtime settings
- [Feature Selection](feature-selection.md) for enabling/disabling features 