# Configuration

This directory contains configuration files for the Navius application.

## Environment Configuration Files

- `default.yaml` - Default configuration values used as fallbacks
- `development.yaml` - Development environment overrides
- `production.yaml` - Production environment overrides
- `reliability.yaml` - Circuit breaker, retry, and reliability configurations
- `api_registry.json` - Registry of API resources and endpoints

## Usage

The application uses a hierarchical configuration system where:

1. `default.yaml` provides base configuration values
2. Environment-specific files (`development.yaml`, `production.yaml`) override default values
3. Environment variables override file-based configuration

## Configuration Structure

All configuration files follow a standardized structure:

```yaml
server:
  port: 3000
  host: "127.0.0.1"
  
logging:
  level: "info"
  format: "json"
  
auth:
  enabled: true
  provider: "entra"
  
database:
  url: "postgres://user:password@localhost:5432/navius"
  pool_size: 5
  
cache:
  enabled: true
  provider: "redis"
  ttl: 3600
  
api:
  base_url: "/api"
  version: "v1"
```

## Adding New Configuration

When adding new configuration values:

1. Add the default value to `default.yaml`
2. Add environment-specific overrides to the relevant environment files
3. Document the configuration option in this README
4. Update the `AppConfig` struct in `src/core/config/app_config.rs`

## Naming Conventions

- Use snake_case for configuration keys
- Group related settings under namespaces
- Use descriptive, self-documenting names
- Follow the existing structure when adding new options 