use config::{Config, ConfigBuilder, Environment, File};
use navius_core::error::{Error, Result};
use std::path::Path;
use tracing::{error, info};

/// WebConfigurator handles loading configuration for web applications
pub struct WebConfigurator {
    config_prefix: String,
    config_dir: String,
}

impl Default for WebConfigurator {
    fn default() -> Self {
        Self::new()
    }
}

impl WebConfigurator {
    /// Create a new WebConfigurator with default settings
    pub fn new() -> Self {
        Self {
            config_prefix: "NAVIUS".to_string(),
            config_dir: "config".to_string(),
        }
    }

    /// Set the environment variable prefix for configuration
    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.config_prefix = prefix.into();
        self
    }

    /// Set the configuration directory
    pub fn with_config_dir(mut self, dir: impl Into<String>) -> Self {
        self.config_dir = dir.into();
        self
    }

    /// Load configuration from files and environment variables
    pub fn load_config(&self) -> Result<Config> {
        info!("Loading configuration from {} directory", self.config_dir);

        // Start with an empty Config builder
        let mut builder = Config::builder();

        // Add default configuration file (required=false means it's optional)
        let default_path = Path::new(&self.config_dir).join("default");
        builder =
            builder.add_source(File::with_name(default_path.to_str().unwrap()).required(false));

        // Add environment-specific configuration if available
        if let Ok(env) = std::env::var("NAVIUS_ENV") {
            let env_path = Path::new(&self.config_dir).join(&env);
            builder =
                builder.add_source(File::with_name(env_path.to_str().unwrap()).required(false));
            info!("Added environment-specific config for: {}", env);
        }

        // Add local configuration file (useful for development, gitignored)
        let local_path = Path::new(&self.config_dir).join("local");
        builder = builder.add_source(File::with_name(local_path.to_str().unwrap()).required(false));

        // Add environment variables with the specified prefix
        builder = builder.add_source(Environment::with_prefix(&self.config_prefix).separator("__"));

        // Build the Config
        match builder.build() {
            Ok(config) => {
                info!("Configuration loaded successfully");
                Ok(config)
            }
            Err(e) => {
                error!("Failed to load configuration: {}", e);
                Err(Error::configuration(format!("Config build error: {}", e)))
            }
        }
    }
}
