use navius::core::features::{
    FeatureConfig, FeatureInfo, FeatureRegistry, FeatureRegistryExt, documentation::DocConfig,
    packaging::BuildConfig,
};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use tempfile::{TempDir, tempdir};

// Helper function to create a test registry with sample features
fn create_test_registry() -> (TempDir, FeatureRegistry) {
    let temp_dir = TempDir::new().unwrap();
    let mut registry = FeatureRegistry::new_empty();

    // Add some test features
    let basic_feature = FeatureInfo {
        name: "basic".to_string(),
        description: "Basic feature".to_string(),
        dependencies: vec![],
        default_enabled: true,
        category: "Core".to_string(),
        tags: vec!["core".to_string()],
        size_impact: 100,
    };

    let advanced_feature = FeatureInfo {
        name: "advanced".to_string(),
        description: "Advanced feature with dependencies".to_string(),
        dependencies: vec!["basic".to_string()],
        default_enabled: false,
        category: "Advanced".to_string(),
        tags: vec!["advanced".to_string()],
        size_impact: 250,
    };

    let metrics_feature = FeatureInfo {
        name: "metrics".to_string(),
        description: "Metrics collection and reporting".to_string(),
        dependencies: vec![],
        default_enabled: false,
        category: "Observability".to_string(),
        tags: vec!["monitoring".to_string()],
        size_impact: 150,
    };

    registry.register(basic_feature);
    registry.register(advanced_feature);
    registry.register(metrics_feature);

    // Enable the basic feature since it's default_enabled=true
    let _ = registry.select("basic");

    (temp_dir, registry)
}

#[test]
fn test_feature_listing() {
    let (_temp_dir, registry) = create_test_registry();

    // Get feature listing output
    let output = get_feature_listing(&registry);

    // Check that it includes our test features
    assert!(output.contains("Available features:"));
    assert!(output.contains("[Core]"));
    assert!(output.contains("basic - Basic feature"));
    assert!(output.contains("[Advanced]"));
    assert!(output.contains("advanced - Advanced feature with dependencies"));
    assert!(output.contains("[Observability]"));
    assert!(output.contains("metrics - Metrics collection and reporting"));
}

// Helper function that reproduces the functionality of the feature-builder list command
fn get_feature_listing(registry: &FeatureRegistry) -> String {
    let mut output = String::new();

    output.push_str("Available features:\n");
    output.push_str("------------------\n");

    let categories = registry.get_categories();

    for category in categories {
        output.push_str(&format!("\n[{}]\n", category));

        let features = registry.get_features_by_category(&category);
        for feature in features {
            let status = if registry.is_selected(&feature.name) {
                "[✓]"
            } else {
                "[ ]"
            };

            output.push_str(&format!(
                "{} {} - {}\n",
                status, feature.name, feature.description
            ));

            if !feature.dependencies.is_empty() {
                output.push_str(&format!(
                    "    Dependencies: {}\n",
                    feature.dependencies.join(", ")
                ));
            }

            if !feature.tags.is_empty() {
                output.push_str(&format!("    Tags: {}\n", feature.tags.join(", ")));
            }
        }
    }

    output
}

#[test]
fn test_enable_disable_feature() {
    let (_temp_dir, mut registry) = create_test_registry();

    // Initially basic is enabled by default
    assert!(registry.is_selected("basic"));
    assert!(!registry.is_selected("advanced"));

    // Test enabling advanced feature (should also keep basic enabled as dependency)
    let result = handle_enable_feature(&mut registry, "advanced");
    assert!(result.contains("Successfully enabled feature: advanced"));
    assert!(result.contains("Dependencies also enabled:"));
    assert!(result.contains("basic"));

    // Verify both features are now selected
    assert!(registry.is_selected("basic"));
    assert!(registry.is_selected("advanced"));

    // Test disabling basic feature (should fail as it's required by advanced)
    let result = handle_disable_feature(&mut registry, "basic");
    assert!(result.contains("Error disabling feature:"));
    assert!(result.contains("required by"));

    // Test disabling advanced feature (should succeed)
    let result = handle_disable_feature(&mut registry, "advanced");
    assert!(result.contains("Successfully disabled feature: advanced"));

    // Verify advanced is now deselected but basic is still selected
    assert!(registry.is_selected("basic"));
    assert!(!registry.is_selected("advanced"));
}

// Helper function that reproduces the functionality of the feature-builder enable command
fn handle_enable_feature(registry: &mut FeatureRegistry, name: &str) -> String {
    let mut output = String::new();

    // Enable the new feature
    match registry.select(name) {
        Ok(_) => {
            output.push_str(&format!("Successfully enabled feature: {}\n", name));

            if let Some(feature) = registry.get_feature_info(name) {
                output.push_str("Dependencies also enabled:\n");
                for dep in &feature.dependencies {
                    output.push_str(&format!("  - {}\n", dep));
                }
            }
        }
        Err(e) => {
            output.push_str(&format!("Error enabling feature: {}\n", e));
        }
    }

    output
}

// Helper function that reproduces the functionality of the feature-builder disable command
fn handle_disable_feature(registry: &mut FeatureRegistry, name: &str) -> String {
    let mut output = String::new();

    // Disable the feature
    match registry.deselect(name) {
        Ok(_) => {
            output.push_str(&format!("Successfully disabled feature: {}\n", name));
        }
        Err(e) => {
            output.push_str(&format!("Error disabling feature: {}\n", e));

            if e.to_string().contains("required by") {
                // Parse the error to extract which features depend on this
                if let Some(other_feature) = e.to_string().split("required by").nth(1) {
                    output.push_str(&format!(
                        "The feature is required by: {}\n",
                        other_feature.trim()
                    ));
                    output.push_str("You must disable those features first.\n");
                }
            }
        }
    }

    output
}

#[test]
fn test_status_display() {
    let (_temp_dir, registry) = create_test_registry();

    // Get status output
    let output = get_status_output(&registry);

    // Check that it shows the correct enabled features
    assert!(output.contains("Currently selected features:"));
    assert!(output.contains("- basic"));
    assert!(output.contains("Total features enabled: 1"));
}

// Helper function that reproduces the functionality of the feature-builder status command
fn get_status_output(registry: &FeatureRegistry) -> String {
    let mut output = String::new();
    let selected = registry.get_selected();

    output.push_str("Currently selected features:\n");
    output.push_str("--------------------------\n");

    for feature in &selected {
        output.push_str(&format!("- {}\n", feature));
    }

    output.push_str(&format!("\nTotal features enabled: {}\n", selected.len()));

    output
}

#[test]
fn test_build_flags_generation() {
    let (_temp_dir, registry) = create_test_registry();

    // Create a FeatureConfig from the registry
    let config = FeatureConfig::from_registry(&registry);

    // Generate build flags
    let flags = config.generate_build_flags();

    // Basic is enabled by default, so should be in features
    assert!(flags.iter().any(|f| f.contains("--features")));
    assert!(flags.iter().any(|f| f.contains("basic")));
    assert!(!flags.iter().any(|f| f.contains("advanced")));
}

#[test]
fn test_save_load_config() {
    let (temp_dir, mut registry) = create_test_registry();

    // Enable the advanced feature
    registry.select("advanced").unwrap();

    // Create a config path in the temp directory
    let config_path = temp_dir.path().join("features.json");
    let config_path_str = config_path.to_string_lossy().to_string();

    // Save the configuration
    let save_result = handle_save_config(&registry, &config_path_str);
    assert!(save_result.contains("Feature configuration saved to:"));
    assert!(config_path.exists());

    // Create a new registry with default settings (only basic enabled)
    let (_, mut new_registry) = create_test_registry();
    assert!(new_registry.is_selected("basic"));
    assert!(!new_registry.is_selected("advanced"));

    // Load the saved configuration
    let load_result = handle_load_config(&mut new_registry, &config_path_str);
    assert!(load_result.contains("Feature configuration loaded from:"));

    // Now both basic and advanced should be enabled
    assert!(new_registry.is_selected("basic"));
    assert!(new_registry.is_selected("advanced"));
}

// Helper function that reproduces the functionality of the feature-builder save command
fn handle_save_config(registry: &FeatureRegistry, path: &str) -> String {
    let config = FeatureConfig::from_registry(registry);

    match config.save(&PathBuf::from(path)) {
        Ok(_) => format!("Feature configuration saved to: {}\n", path),
        Err(e) => format!("Error saving feature configuration: {}\n", e),
    }
}

// Helper function that reproduces the functionality of the feature-builder load command
fn handle_load_config(registry: &mut FeatureRegistry, path: &str) -> String {
    match FeatureConfig::load(&PathBuf::from(path)) {
        Ok(config) => {
            // Clear current selections
            for feature in registry.get_selected() {
                let _ = registry.deselect(&feature);
            }

            // Apply loaded selections
            for feature in &config.selected_features {
                let _ = registry.select(feature);
            }

            format!("Feature configuration loaded from: {}\n", path)
        }
        Err(e) => format!("Error loading feature configuration: {}\n", e),
    }
}

#[test]
fn test_documentation_generation() {
    let (_temp_dir, registry) = create_test_registry();

    // Create temporary directory for output
    let output_dir = tempdir().unwrap();
    let templates_dir = tempdir().unwrap();

    // Create template files
    fs::write(
        templates_dir.path().join("feature-generic.md"),
        "# {{feature.name}}\n\n{{feature.description}}\n",
    )
    .unwrap();

    // Create a README template which is required for index generation
    fs::write(
        templates_dir.path().join("README.md"),
        "# Documentation\n\nEnabled features:\n{{feature_list}}\n",
    )
    .unwrap();

    // Configure documentation generator
    let doc_config = DocConfig {
        output_dir: output_dir.path().to_path_buf(),
        template_dir: templates_dir.path().to_path_buf(),
        version: "1.0.0-test".to_string(),
        generate_api_reference: true,
        generate_config_examples: true,
        generate_feature_docs: true,
    };

    // Generate documentation
    let result = handle_generate_docs(&registry, doc_config);

    // Verify output contains success message
    assert!(result.contains("Documentation generated successfully"));

    // Verify files were created
    let index_file = output_dir.path().join("README.md");
    assert!(
        index_file.exists(),
        "Index file was not created at {:?}",
        index_file
    );

    let features_dir = output_dir.path().join("features");
    assert!(features_dir.exists());
}

// Helper function that reproduces the functionality of the feature-builder docs command
fn handle_generate_docs(registry: &FeatureRegistry, config: DocConfig) -> String {
    use navius::core::features::documentation::DocGenerator;

    match DocGenerator::new(registry.clone(), config) {
        Ok(generator) => match generator.generate() {
            Ok(_) => format!(
                "Documentation generated successfully at: {:?}\n",
                generator.config.output_dir
            ),
            Err(e) => format!("Error generating documentation: {}\n", e),
        },
        Err(e) => format!("Error creating documentation generator: {}\n", e),
    }
}

#[test]
fn test_reset_to_defaults() {
    let (_temp_dir, mut registry) = create_test_registry();

    // Enable all features
    registry.select("basic").unwrap();
    registry.select("advanced").unwrap();
    registry.select("metrics").unwrap();

    // Verify all features are selected
    assert!(registry.is_selected("basic"));
    assert!(registry.is_selected("advanced"));
    assert!(registry.is_selected("metrics"));

    // Reset to defaults
    let result = handle_reset_to_defaults(&mut registry);
    assert!(result.contains("Feature configuration reset to defaults"));

    // Only basic should be selected (it's the default)
    assert!(registry.is_selected("basic"));
    assert!(!registry.is_selected("advanced"));
    assert!(!registry.is_selected("metrics"));
}

// Helper function that reproduces the functionality of the feature-builder reset command
fn handle_reset_to_defaults(registry: &mut FeatureRegistry) -> String {
    // Get all currently selected features
    let selected = registry.get_selected();

    // Deselect all features
    for feature in &selected {
        let _ = registry.deselect(feature);
    }

    // Collect the names of default-enabled features first
    let mut default_features = Vec::new();
    let categories = registry.get_categories();
    for category in &categories {
        let features = registry.get_features_by_category(category);
        for feature in features {
            if feature.default_enabled {
                default_features.push(feature.name.clone());
            }
        }
    }

    // Now re-enable the default features
    for feature_name in default_features {
        let _ = registry.select(&feature_name);
    }

    "Feature configuration reset to defaults".to_string()
}
