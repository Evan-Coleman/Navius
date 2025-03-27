use crate::core::features::features::{FeatureError, FeatureRegistry};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::env;
use std::path::Path;

/// Feature configuration for conditional compilation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureConfig {
    /// Selected features
    pub selected_features: HashSet<String>,

    /// Build-time configuration
    pub build_config: HashMap<String, String>,
}

impl FeatureConfig {
    /// Create a new feature configuration from registry
    pub fn from_registry(registry: &FeatureRegistry) -> Self {
        Self {
            selected_features: registry.get_selected(),
            build_config: HashMap::new(),
        }
    }

    /// Check if a feature is enabled
    pub fn is_enabled(&self, feature: &str) -> bool {
        self.selected_features.contains(feature)
    }

    /// Save configuration to file
    pub fn save(&self, path: &Path) -> Result<(), FeatureError> {
        let config = serde_json::to_string_pretty(self)
            .map_err(|e| FeatureError::SerializationError(e.to_string()))?;

        std::fs::write(path, config).map_err(|e| FeatureError::IoError(e.to_string()))?;

        Ok(())
    }

    /// Load configuration from file
    pub fn load(path: &Path) -> Result<Self, FeatureError> {
        let config =
            std::fs::read_to_string(path).map_err(|e| FeatureError::IoError(e.to_string()))?;

        serde_json::from_str(&config).map_err(|e| FeatureError::DeserializationError(e.to_string()))
    }

    /// Save configuration to the default config path
    pub fn save_default(&self) -> Result<(), FeatureError> {
        let path = Self::default_config_path()?;
        self.save(&path)
    }

    /// Load configuration from the default config path
    pub fn load_default() -> Result<Self, FeatureError> {
        let path = Self::default_config_path()?;

        if !path.exists() {
            // Create default configuration
            let registry = FeatureRegistry::new();
            let config = Self::from_registry(&registry);
            config.save(&path)?;
        }

        Self::load(&path)
    }

    /// Get the default configuration path
    pub fn default_config_path() -> Result<std::path::PathBuf, FeatureError> {
        // Use the standard config directory structure instead of ~/.navius
        let config_dir = env::var("CONFIG_DIR").unwrap_or_else(|_| "./config".to_string());
        let config_path = std::path::PathBuf::from(&config_dir);

        // Make sure the directory exists
        if !config_path.exists() {
            std::fs::create_dir_all(&config_path).map_err(|e| {
                FeatureError::IoError(format!("Failed to create config directory: {}", e))
            })?;
        }

        // Use features.json in the config directory
        Ok(config_path.join("features.json"))
    }

    /// Generate Cargo build flags based on the selected features
    pub fn generate_build_flags(&self) -> Vec<String> {
        let mut flags = Vec::new();

        for feature in &self.selected_features {
            flags.push(format!("--features={}", feature));
        }

        flags
    }
}
