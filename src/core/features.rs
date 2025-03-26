//! Feature selection and customization system
//!
//! Provides a framework for creating customized server deployments with tailored feature sets.

// Declare submodules
pub mod config;
pub mod dependency_analyzer;
pub mod documentation;
pub mod features;
pub mod macros;
pub mod packaging;
pub mod runtime;

// Re-export the feature registry and related types
pub use self::config::FeatureConfig;
pub use self::dependency_analyzer::DependencyAnalyzer;
pub use self::documentation::{DocConfig, DocGenerator, DocTemplate};
pub use self::features::{FeatureError, FeatureInfo, FeatureRegistry, FeatureRegistryExt};
pub use self::packaging::{BuildConfig, ContainerConfig, PackageManager, VersionInfo};
pub use self::runtime::RuntimeFeatures;

// Re-export test utilities for integration tests
#[cfg(test)]
pub use self::features::test_utils;

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
