use navius::core::features::{FeatureConfig, FeatureRegistry, FeatureRegistryExt};
use std::env;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

// A helper struct to simulate the main function environment for testing
struct MainFunctionTestEnv {
    registry: FeatureRegistry,
    temp_dir: TempDir,
    config_path: PathBuf,
}

impl MainFunctionTestEnv {
    fn new() -> Self {
        let temp_dir = TempDir::new().unwrap();
        let mut registry = FeatureRegistry::new_empty();

        // Add some test features
        let basic_feature = navius::core::features::FeatureInfo {
            name: "basic".to_string(),
            description: "Basic feature".to_string(),
            dependencies: vec![],
            default_enabled: true,
            category: "Core".to_string(),
            tags: vec!["core".to_string()],
            size_impact: 100,
        };

        let advanced_feature = navius::core::features::FeatureInfo {
            name: "advanced".to_string(),
            description: "Advanced feature with dependencies".to_string(),
            dependencies: vec!["basic".to_string()],
            default_enabled: false,
            category: "Advanced".to_string(),
            tags: vec!["advanced".to_string()],
            size_impact: 250,
        };

        registry.register(basic_feature);
        registry.register(advanced_feature);

        // Explicitly enable the basic feature since it's default_enabled
        registry.select("basic").unwrap();

        // Set up a config path
        let config_path = temp_dir.path().join("features.json");

        Self {
            registry,
            temp_dir,
            config_path,
        }
    }

    // This function simulates saving and loading operations that would be done in the main function
    fn save_and_load_config(&mut self) -> Result<(), String> {
        // Save configuration
        let config = FeatureConfig::from_registry(&self.registry);
        config
            .save(&self.config_path)
            .map_err(|e| format!("Failed to save config: {}", e))?;

        // Load configuration
        let config = FeatureConfig::load(&self.config_path)
            .map_err(|e| format!("Failed to load config: {}", e))?;

        // Update registry with loaded config
        for feature in &config.selected_features {
            let _ = self.registry.select(feature);
        }

        Ok(())
    }

    // Simulate the feature enabling from command line
    fn enable_feature(&mut self, feature_name: &str) -> Result<(), String> {
        match self.registry.select(feature_name) {
            Ok(_) => {
                // Save config after enabling
                let config = FeatureConfig::from_registry(&self.registry);
                config
                    .save(&self.config_path)
                    .map_err(|e| format!("Failed to save config: {}", e))?;
                Ok(())
            }
            Err(e) => Err(format!("Failed to enable feature: {}", e)),
        }
    }

    // Simulate the feature disabling from command line
    fn disable_feature(&mut self, feature_name: &str) -> Result<(), String> {
        match self.registry.deselect(feature_name) {
            Ok(_) => {
                // Save config after disabling
                let config = FeatureConfig::from_registry(&self.registry);
                config
                    .save(&self.config_path)
                    .map_err(|e| format!("Failed to save config: {}", e))?;
                Ok(())
            }
            Err(e) => Err(format!("Failed to disable feature: {}", e)),
        }
    }
}

#[test]
fn test_main_function_save_load_workflow() {
    let mut env = MainFunctionTestEnv::new();

    // Enable a feature
    env.registry.select("advanced").unwrap();

    // Save and reload
    let result = env.save_and_load_config();
    assert!(result.is_ok());

    // Verify the feature is still enabled after loading
    assert!(env.registry.is_selected("advanced"));
}

#[test]
fn test_main_function_enable_disable_workflow() {
    let mut env = MainFunctionTestEnv::new();

    // Initially only basic should be enabled by default
    assert!(env.registry.is_selected("basic"));
    assert!(!env.registry.is_selected("advanced"));

    // Enable advanced
    let result = env.enable_feature("advanced");
    assert!(result.is_ok());
    assert!(env.registry.is_selected("advanced"));

    // Save and reload
    let result = env.save_and_load_config();
    assert!(result.is_ok());

    // Feature should still be enabled
    assert!(env.registry.is_selected("advanced"));
    assert!(env.registry.is_selected("basic")); // Basic should also be enabled

    // Disable basic (should fail due to dependency)
    let result = env.disable_feature("basic");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("required by"));

    // Basic should still be enabled
    assert!(env.registry.is_selected("basic"));

    // Disable advanced
    let result = env.disable_feature("advanced");
    assert!(result.is_ok());
    assert!(!env.registry.is_selected("advanced"));
}

#[test]
fn test_main_function_error_handling() {
    let mut env = MainFunctionTestEnv::new();

    // Test enabling non-existent feature (should fail)
    let result = env.enable_feature("non_existent");
    assert!(result.is_err());

    // Test disabling non-existent feature (should fail)
    let result = env.disable_feature("non_existent");
    assert!(result.is_err());

    // Modify the env to have a non-existent file path
    let non_existent_path = env
        .temp_dir
        .path()
        .join("non_existent_dir")
        .join("non_existent_file.json");
    env.config_path = non_existent_path;

    // Loading from a non-existent path should fail
    let result = env.save_and_load_config();
    assert!(result.is_err());
}

#[test]
fn test_main_function_feature_dependency_resolution() {
    let mut env = MainFunctionTestEnv::new();

    // Initially only basic is enabled
    assert!(env.registry.is_selected("basic"));
    assert!(!env.registry.is_selected("advanced"));

    // Enable advanced (should also keep basic enabled as dependency)
    let result = env.enable_feature("advanced");
    assert!(result.is_ok());

    // Both features should be enabled
    assert!(env.registry.is_selected("basic"));
    assert!(env.registry.is_selected("advanced"));

    // Save and reload
    let result = env.save_and_load_config();
    assert!(result.is_ok());

    // Both features should still be enabled
    assert!(env.registry.is_selected("basic"));
    assert!(env.registry.is_selected("advanced"));
}
