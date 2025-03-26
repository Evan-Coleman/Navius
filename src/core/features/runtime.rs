use crate::core::features::FeatureConfig;
use std::collections::{HashMap, HashSet};

/// Runtime feature flags for dynamic behavior
pub struct RuntimeFeatures {
    /// Currently enabled features
    enabled_features: HashSet<String>,

    /// Feature default status
    default_status: HashMap<String, bool>,
}

impl RuntimeFeatures {
    /// Create new runtime features
    pub fn new() -> Self {
        let mut default_status = HashMap::new();

        // Set up default status based on Cargo features
        #[cfg(feature = "metrics")]
        default_status.insert("metrics".to_string(), true);

        #[cfg(not(feature = "metrics"))]
        default_status.insert("metrics".to_string(), false);

        #[cfg(feature = "caching")]
        default_status.insert("caching".to_string(), true);

        #[cfg(not(feature = "caching"))]
        default_status.insert("caching".to_string(), false);

        #[cfg(feature = "auth")]
        default_status.insert("auth".to_string(), true);

        #[cfg(not(feature = "auth"))]
        default_status.insert("auth".to_string(), false);

        #[cfg(feature = "reliability")]
        default_status.insert("reliability".to_string(), true);

        #[cfg(not(feature = "reliability"))]
        default_status.insert("reliability".to_string(), false);

        #[cfg(feature = "advanced_metrics")]
        default_status.insert("advanced_metrics".to_string(), true);

        #[cfg(not(feature = "advanced_metrics"))]
        default_status.insert("advanced_metrics".to_string(), false);

        // Core features are always enabled
        default_status.insert("core".to_string(), true);
        default_status.insert("error_handling".to_string(), true);
        default_status.insert("config".to_string(), true);

        // Try to load from config file first
        if let Ok(config) = FeatureConfig::load_default() {
            // Initialize enabled features from config
            return Self {
                enabled_features: config.selected_features,
                default_status,
            };
        }

        // Initialize enabled features from defaults
        let enabled_features = default_status
            .iter()
            .filter_map(|(k, v)| if *v { Some(k.clone()) } else { None })
            .collect();

        Self {
            enabled_features,
            default_status,
        }
    }

    /// Check if a feature is enabled
    pub fn is_enabled(&self, feature: &str) -> bool {
        self.enabled_features.contains(feature)
    }

    /// Get a list of all enabled features
    pub fn get_enabled_features(&self) -> &HashSet<String> {
        &self.enabled_features
    }

    /// Enable a feature at runtime
    pub fn enable(&mut self, feature: &str) {
        self.enabled_features.insert(feature.to_string());
    }

    /// Disable a feature at runtime
    pub fn disable(&mut self, feature: &str) {
        self.enabled_features.remove(feature);
    }

    /// Reset a feature to its compile-time default
    pub fn reset_feature(&mut self, feature: &str) {
        if let Some(default) = self.default_status.get(feature) {
            if *default {
                self.enable(feature);
            } else {
                self.disable(feature);
            }
        }
    }

    /// Reset all features to their compile-time defaults
    pub fn reset_all(&mut self) {
        self.enabled_features = self
            .default_status
            .iter()
            .filter_map(|(k, v)| if *v { Some(k.clone()) } else { None })
            .collect();
    }

    /// Get all enabled features
    pub fn get_enabled(&self) -> HashSet<String> {
        self.enabled_features.clone()
    }

    /// Get status of a specific feature
    pub fn get_feature_status(&self, feature: &str) -> Option<bool> {
        if self.enabled_features.contains(feature) {
            Some(true)
        } else if self.default_status.contains_key(feature) {
            Some(false)
        } else {
            None
        }
    }
}
