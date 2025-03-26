//! Feature selection and customization system
//!
//! Provides a framework for creating customized server deployments with tailored feature sets.

// Declare submodules
pub mod config;
pub mod documentation;
pub mod features;
pub mod macros;
pub mod packaging;
pub mod runtime;

// Re-export the feature registry and related types
pub use self::config::FeatureConfig;
pub use self::documentation::{DocConfig, DocGenerator, DocTemplate};
pub use self::features::{FeatureError, FeatureInfo, FeatureRegistry};
pub use self::packaging::{BuildConfig, ContainerConfig, PackageManager, VersionInfo};
pub use self::runtime::RuntimeFeatures;

/// Feature registry extension methods for easier access
pub trait FeatureRegistryExt {
    /// Get all available features
    fn features(&self) -> Vec<&FeatureInfo>;

    /// Get the total number of features
    fn feature_count(&self) -> usize;

    /// Get set of enabled features
    fn enabled_features(&self) -> &std::collections::HashSet<String>;

    /// Get feature info by name
    fn get_feature_info(&self, name: &str) -> Option<&FeatureInfo>;

    /// Check if a feature is enabled
    fn is_enabled(&self, name: &str) -> bool;

    /// Enable a feature and its dependencies
    fn enable(&mut self, name: &str) -> Result<(), FeatureError>;

    /// Disable a feature if no other features depend on it
    fn disable(&mut self, name: &str) -> Result<(), FeatureError>;
}

// Testing utilities
#[cfg(test)]
pub mod test_utils {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    /// Create a test feature registry with sample features
    pub fn create_test_registry() -> FeatureRegistry {
        FeatureRegistry::new()
    }

    /// Create a temporary directory for test output
    pub fn create_temp_dir() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_path_buf();
        (temp_dir, path)
    }
}
