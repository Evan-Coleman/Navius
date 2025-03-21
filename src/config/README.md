# Application Configuration

This directory provides a user-friendly interface to the application's configuration system. It serves as a wrapper around the core configuration system located in `src/core/config`.

## Usage

To access configuration in your application code:

```rust
use crate::config::{get_config, get_cache_config, get_server_config};

// Get the full application configuration
let config = get_config();
println!("Server address: {}", config.server_addr());

// Get specific configuration sections
let cache_config = get_cache_config();
let server_config = get_server_config();

// Use specific configuration values
let ttl = cache_config.ttl_seconds;
let host = server_config.host;
```

## Adding Custom Configuration

If you need to add custom configuration for your application:

1. Create helper functions in `mod.rs` to access your configuration
2. Use the existing `AppConfig` structure, or extend it as needed

Example:

```rust
// In src/config/mod.rs
pub fn get_my_custom_config() -> String {
    get_config().my_custom.value
}
```

## Configuration Files

Configuration is loaded from YAML files in the `config/` directory at the project root:

- `default.yaml`: Default settings
- `development.yaml`: Development environment settings
- `production.yaml`: Production environment settings
- `testing.yaml`: Test environment settings

You can also create local override files that aren't committed to version control:
- `local.yaml`: Local settings that override default
- `local-development.yaml`: Local development settings

## Environment Variables

Most configuration values can be overridden with environment variables following these patterns:
- `SERVER_*` for server settings
- `API_*` for API settings
- `APP_*` for app settings
- `AUTH_*` for auth settings
- `CACHE_*` for cache settings

Example: `SERVER_PORT=4000` to override the default server port. 