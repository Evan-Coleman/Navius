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
