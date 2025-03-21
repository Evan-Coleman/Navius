# Core Configuration System

This directory contains the core configuration system used by the application. It provides functionality for loading and accessing configuration from various sources, including YAML files and environment variables.

## Components

- `app_config.rs`: Main configuration structures and loading logic
- `constants.rs`: Constants used throughout the configuration system
- `mod.rs`: Module definitions and exports
- `tests.rs`: Tests for the configuration system

## Design

The configuration system loads settings from multiple sources with the following priority (highest to lowest):
1. Environment variables
2. Environment-specific local overrides (`local-{env}.yaml`)
3. Environment-specific config (`{env}.yaml`)
4. Local overrides (`local.yaml`)
5. Default config (`default.yaml`)

## Key Features

- Environment-specific configuration
- Local overrides (not in version control)
- Environment variable overrides
- Typed configuration with defaults
- Validation of critical settings

## Usage

The core configuration system is not meant to be used directly by application code. Instead, use the application-level configuration module in `src/config`, which provides a more user-friendly interface.

If you need to interact with the core configuration system directly, use the following approach:

```rust
use crate::core::config::app_config::{AppConfig, load_config};

// Load the configuration
let config = load_config().expect("Failed to load config");

// Access configuration values
let server_addr = config.server_addr();
let cache_ttl = config.cache_ttl();
```

## Adding New Configuration Settings

To add new configuration settings:

1. Add the setting to the appropriate struct in `app_config.rs`
2. Add a default value if necessary
3. Add the setting to the appropriate YAML configuration file
4. Add environment variable mapping if needed 