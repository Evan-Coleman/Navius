---
title: "Configuration Standards"
description: ""
category: "Documentation"
tags: []
last_updated: "March 28, 2025"
version: "1.0"
---

# Configuration Standards

This document outlines the configuration standards and patterns used throughout the Navius framework, providing a reference for consistent configuration implementation.

## Configuration File Structure

### File Locations

Navius applications use a standardized configuration file structure:

```
config/
├── default.yaml        # Base configuration (required)
├── development.yaml    # Development environment overrides
├── test.yaml           # Testing environment overrides
└── production.yaml     # Production environment overrides
```

### File Naming Convention

- `default.yaml`: Base configuration that applies to all environments
- `{environment}.yaml`: Environment-specific overrides (development, test, production)
- Custom environments can be defined with `{custom-name}.yaml`

## YAML Format Standards

### Nesting and Organization

Configuration should be organized in a hierarchical structure:

```yaml
# Top-level application configuration
app:
  name: "Navius Application"
  version: "1.0.0"
  description: "A Navius framework application"

# Server configuration
server:
  host: "127.0.0.1"
  port: 3000
  timeout_seconds: 30
  
# Feature flags
features:
  advanced_metrics: true
  experimental_api: false
  
# Subsystem configurations
database:
  url: "postgres://localhost:5432/navius"
  max_connections: 10
  timeout_seconds: 5
  
logging:
  level: "info"
  format: "json"
  file: "/var/log/navius.log"
```

### Naming Conventions

- Use snake_case for all configuration keys
- Group related settings under common prefixes
- Use descriptive, clear names
- Avoid abbreviations unless widely understood

### Value Types

- Strings: Use quotes (`"value"`)
- Numbers: No quotes (`42`, `3.14`)
- Booleans: Use `true` or `false` (lowercase)
- Arrays: Use `[item1, item2]` or multiline list format
- Maps: Use nested format with indentation

## Environment Variables

### Environment Variable Mapping

Configuration values can be overridden via environment variables using this pattern:

```
APP__NAME="Overridden App Name"
SERVER__PORT=8080
FEATURES__ADVANCED_METRICS=false
```

Rules:
- Double underscore (`__`) separates configuration keys
- Keys are case-insensitive
- Environment variables take precedence over file configuration

### Variable Types

- Strings: Use as-is
- Numbers: Parsed from string representation
- Booleans: `true`, `false`, `1`, `0`, `yes`, `no`, `y`, `n` (case-insensitive)
- Arrays: Comma-separated values (`val1,val2,val3`)
- Objects: JSON format (`{"key": "value"}`)

## Secrets Management

### Sensitive Data

Never store secrets in configuration files. Use these approaches instead:

1. **Environment Variables**: For most secrets
   ```
   DATABASE__PASSWORD="secure-password"
   JWT__SECRET_KEY="jwt-signing-key"
   ```

2. **External Secret Managers**: For advanced scenarios
   ```yaml
   secrets:
     provider: "vault"  # or "aws-secrets", "azure-keyvault"
     url: "https://vault.example.com"
     path: "secret/navius"
   ```

3. **File References**: For certificate files
   ```yaml
   tls:
     cert_file: "/path/to/cert.pem"
     key_file: "/path/to/key.pem"
   ```

### Secret Configuration Patterns

Use this format for secret references:

```yaml
database:
  username: "db_user"
  password: "${DB_PASSWORD}"  # Resolved from environment
  
jwt:
  secret: "${JWT_SECRET}"     # Resolved from environment
```

## Configuration Loading

### Load Order

Configuration is loaded in this order, with later steps overriding earlier ones:

1. Default configuration file (`default.yaml`)
2. Environment-specific file (based on `ENVIRONMENT` variable)
3. Environment variables
4. Command-line arguments

### Command-Line Arguments

Command-line arguments follow this format:

```
./my-app --server.port=8080 --logging.level=debug
```

## Validation

### Schema Validation

All configuration should be validated against a schema:

```rust
#[derive(Debug, Deserialize, Validate)]
pub struct ServerConfig {
    #[validate(required)]
    pub host: String,
    
    #[validate(range(min = 1, max = 65535))]
    pub port: u16,
    
    #[validate(range(min = 1, max = 300))]
    pub timeout_seconds: u32,
}
```

### Required vs Optional Settings

Always provide clear documentation about which settings are required vs optional:

```yaml
# Required settings (no defaults provided by application)
database:
  url: "postgres://localhost:5432/navius"  # REQUIRED
  
# Optional settings (defaults provided by application)
server:
  port: 3000  # Optional, defaults to 3000 if not specified
  host: "127.0.0.1"  # Optional, defaults to 127.0.0.1 if not specified
```

## Feature Flags

### Feature Configuration

Organize feature flags under a dedicated section:

```yaml
features:
  advanced_metrics: true
  experimental_api: false
  beta_endpoints: false
  cache_enabled: true
```

### Feature-Specific Configuration

Group feature-specific settings under the feature name:

```yaml
features:
  advanced_metrics:
    enabled: true
    sampling_rate: 0.1
    export_interval_seconds: 60
    
  cache:
    enabled: true
    ttl_seconds: 300
    max_entries: 10000
```

## Documentation

### Configuration Comments

Include comments in YAML files to document configuration options:

```yaml
server:
  # The host IP address to bind the server to
  # Use "0.0.0.0" to bind to all interfaces
  host: "127.0.0.1"
  
  # The port number to listen on (1-65535)
  port: 3000
  
  # Request timeout in seconds
  timeout_seconds: 30
```

### Configuration Reference

Maintain a comprehensive configuration reference:

```rust
/// Server configuration options
/// 
/// # Examples
/// 
/// ```yaml
/// server:
///   host: "127.0.0.1"
///   port: 3000
///   timeout_seconds: 30
/// ```
#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    /// The host address to bind to
    pub host: String,
    
    /// The port to listen on (1-65535)
    pub port: u16,
    
    /// Request timeout in seconds
    pub timeout_seconds: u32,
}
```

## Default Values

### Sensible Defaults

Provide sensible defaults for all optional configuration:

```rust
impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 3000,
            timeout_seconds: 30,
        }
    }
}
```

### Overriding Defaults

Document how defaults can be overridden:

```yaml
# Override defaults in environment-specific files
# development.yaml
server:
  port: 8080  # Override default port
```

## Configuration Update

### Dynamic Configuration

For settings that can be updated at runtime:

```yaml
# Settings that support runtime updates
dynamic:
  logging:
    level: "info"  # Can be changed at runtime
  
  cache:
    ttl_seconds: 300  # Can be changed at runtime
```

### Reload Mechanism

Support configuration reload where appropriate:

```rust
// Reload configuration from disk
config_service.reload().await?;

// Subscribe to configuration changes
config_service.subscribe(|updated_config| {
    // React to changes
});
```

## Integration with Services

### Dependency Injection

Inject configuration into services:

```rust
// Service that uses configuration
pub struct UserService {
    config: Arc<ConfigService>,
    repository: Arc<UserRepository>,
}

impl UserService {
    pub fn new(
        config: Arc<ConfigService>,
        repository: Arc<UserRepository>,
    ) -> Self {
        Self { config, repository }
    }
    
    pub async fn get_user(&self, id: &str) -> Result<User, Error> {
        let timeout = self.config.get::<u64>("user_service.timeout_seconds")
            .unwrap_or(5);
            
        self.repository.get_user_with_timeout(id, timeout).await
    }
}
```

## Best Practices

1. **Centralized Configuration**: Keep configuration in one central location
2. **Environment Separation**: Use separate files for each environment
3. **Validation**: Always validate configuration at startup
4. **Documentation**: Document all configuration options
5. **Defaults**: Provide sensible defaults for all optional settings
6. **Type Safety**: Use strongly-typed configuration objects
7. **Secrets Management**: Never store secrets in configuration files
8. **Configuration Testing**: Test configuration loading and validation

## Example Configuration

### Complete Example

```yaml
# Application configuration
app:
  name: "Navius Example App"
  version: "1.0.0"
  description: "An example Navius application"

# Server configuration
server:
  host: "127.0.0.1"
  port: 3000
  timeout_seconds: 30
  
# Feature flags
features:
  advanced_metrics: true
  experimental_api: false
  
# Database configuration
database:
  driver: "postgres"
  host: "localhost"
  port: 5432
  name: "navius"
  username: "navius_user"
  password: "${DB_PASSWORD}"  # Set via environment variable
  pool:
    max_connections: 10
    timeout_seconds: 5
    idle_timeout_seconds: 300
  
# Logging configuration
logging:
  level: "info"
  format: "json"
  output:
    console: true
    file: "/var/log/navius.log"
  
# Cache configuration
cache:
  enabled: true
  provider: "redis"
  url: "redis://localhost:6379"
  ttl_seconds: 300
  
# Metrics configuration
metrics:
  enabled: true
  exporter: "prometheus"
  endpoint: "/metrics"
  interval_seconds: 15
  
# Health check configuration
health:
  enabled: true
  endpoint: "/health"
  include_details: true
  
# API configuration
api:
  base_path: "/api/v1"
  rate_limit:
    enabled: true
    requests_per_minute: 60
  cors:
    enabled: true
    allowed_origins: ["https://example.com"]
``` 
