use super::*;
use std::collections::HashSet;
use std::path::Path;
use tempfile::TempDir;

#[test]
fn test_feature_registry_creation() {
    let registry = FeatureRegistry::new();
    assert!(
        registry.is_selected("core"),
        "Core feature should be selected by default"
    );
    assert!(
        registry.is_selected("error_handling"),
        "Error handling should be selected by default"
    );
    assert!(
        !registry.is_selected("advanced_metrics"),
        "Advanced metrics should not be selected by default"
    );
}

#[test]
fn test_feature_selection() {
    let mut registry = FeatureRegistry::new();
    assert!(registry.select("advanced_metrics").is_ok());
    assert!(registry.is_selected("advanced_metrics"));

    // Should also select dependencies
    assert!(
        registry.is_selected("metrics"),
        "Dependency 'metrics' should be selected automatically"
    );
}

#[test]
fn test_feature_deselection() {
    let mut registry = FeatureRegistry::new();
    registry.select("advanced_metrics").unwrap();

    // Try to deselect dependency
    let result = registry.deselect("metrics");
    assert!(
        result.is_err(),
        "Should not be able to deselect a dependency"
    );

    // Deselect feature first
    registry.deselect("advanced_metrics").unwrap();

    // Now can deselect dependency if no other features depend on it
    assert!(registry.deselect("metrics").is_ok());
}

#[test]
fn test_validation() {
    let mut registry = FeatureRegistry::new();

    // Manually create invalid state for testing
    registry.selected.insert("advanced_metrics".to_string());
    registry.selected.remove("metrics");

    // Validation should fail because dependency 'metrics' is missing
    assert!(registry.validate().is_err());

    // Add dependency
    registry.selected.insert("metrics".to_string());

    // Now validation should pass
    assert!(registry.validate().is_ok());
}

#[test]
fn test_feature_config_serialization() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.json");

    let mut registry = FeatureRegistry::new();
    registry.select("advanced_metrics").unwrap();

    let config = FeatureConfig::from_registry(&registry);

    // Save config
    config.save(Path::new(&config_path)).unwrap();

    // Load config
    let loaded = FeatureConfig::load(Path::new(&config_path)).unwrap();

    assert!(loaded.is_enabled("core"));
    assert!(loaded.is_enabled("metrics"));
    assert!(loaded.is_enabled("advanced_metrics"));
}

#[test]
fn test_runtime_features() {
    let runtime = RuntimeFeatures::new();

    // Core features should always be enabled
    assert!(runtime.is_enabled("core"));
    assert!(runtime.is_enabled("error_handling"));

    // Test enabling/disabling features
    let mut runtime = RuntimeFeatures::new();

    // These may be enabled or disabled depending on how the tests are run,
    // so we'll explicitly set them for the test
    runtime.disable_feature("advanced_metrics");
    assert!(!runtime.is_enabled("advanced_metrics"));

    runtime.enable_feature("advanced_metrics");
    assert!(runtime.is_enabled("advanced_metrics"));

    // Test reset
    runtime.reset_feature("advanced_metrics");
    // The reset state depends on compile-time features,
    // so we can't assert a specific value

    // Get status
    let status = runtime.get_feature_status("core");
    assert_eq!(status, Some(true));

    let status = runtime.get_feature_status("unknown_feature");
    assert_eq!(status, None);
}

#[cfg(test)]
mod feature_selection_tests {
    // ... existing tests ...
}

#[cfg(test)]
mod packaging_tests {
    use super::super::features::FeatureRegistry;
    use super::super::packaging::{BuildConfig, ContainerConfig, PackageManager, VersionInfo};
    use std::collections::{HashMap, HashSet};
    use std::path::PathBuf;
    use tempfile::TempDir;

    /// Setup function to create test resources
    fn setup() -> (TempDir, FeatureRegistry, BuildConfig) {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().to_path_buf();
        let source_path = std::env::current_dir().unwrap();

        let mut registry = FeatureRegistry::new();

        // Enable core features for the test
        registry.select("core").unwrap();
        registry.select("metrics").unwrap();

        let build_config = BuildConfig::new(source_path, output_path)
            .with_optimization("debug") // Use debug for faster tests
            .with_features(registry.get_selected())
            .with_version(VersionInfo::default());

        (temp_dir, registry, build_config)
    }

    #[test]
    fn test_build_config_creation() {
        let (_, _, config) = setup();

        assert_eq!(config.optimization_level, "debug");
        assert!(config.features.contains("core"));
        assert!(config.features.contains("metrics"));
        assert!(config.container.is_none());
    }

    #[test]
    fn test_build_command_generation() {
        let (_, _, config) = setup();

        let cmd = config.generate_build_command();

        assert_eq!(cmd[0], "cargo");
        assert_eq!(cmd[1], "build");

        // Check for features parameter
        let features_index = cmd.iter().position(|arg| arg == "--features").unwrap();
        let features = &cmd[features_index + 1];
        assert!(features.contains("core"));
        assert!(features.contains("metrics"));
    }

    #[test]
    fn test_container_config() {
        let (_, _, config) = setup();

        let container_config = ContainerConfig {
            base_image: "rust:slim".to_string(),
            tags: vec!["test:latest".to_string()],
            env_vars: vec![],
            ports: vec![8080],
            labels: HashMap::new(),
        };

        let config_with_container = config.with_container(container_config);

        assert!(config_with_container.container.is_some());
        if let Some(container) = &config_with_container.container {
            assert_eq!(container.base_image, "rust:slim");
            assert_eq!(container.tags[0], "test:latest");
            assert_eq!(container.ports[0], 8080);
        }
    }

    #[test]
    fn test_version_info() {
        let version = VersionInfo {
            major: 1,
            minor: 2,
            patch: 3,
            build: Some("test".to_string()),
            commit: Some("abc123".to_string()),
        };

        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 3);
        assert_eq!(version.build, Some("test".to_string()));
        assert_eq!(version.commit, Some("abc123".to_string()));
    }

    #[test]
    fn test_package_manager_creation() {
        let (_, registry, config) = setup();

        let package_manager = PackageManager::new(registry, config);

        // This is mostly a compilation test, just to ensure the types match up
        assert!(true);
    }

    // NOTE: More comprehensive tests for package_manager.build_package(),
    // create_container(), and create_update_package() would typically be
    // implemented as integration tests with mocks for the build commands
    // since we don't want to actually run cargo build in unit tests
}
