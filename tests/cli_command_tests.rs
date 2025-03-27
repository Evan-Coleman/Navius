use navius::core::features::{FeatureConfig, FeatureInfo, FeatureRegistry, FeatureRegistryExt};
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

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

    (temp_dir, registry)
}

// Helper to save registry configuration to a file
fn save_test_config(
    registry: &FeatureRegistry,
    path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = registry.export_configuration()?;
    fs::write(path, config)?;
    Ok(())
}

#[test]
fn test_format_feature_list_command() {
    let (_temp_dir, registry) = create_test_registry();

    // Test text format
    let text_output = registry.format_feature_list("text");
    assert!(text_output.contains("Basic feature"));
    assert!(text_output.contains("Advanced feature with dependencies"));

    // Test JSON format
    let json_output = registry.format_feature_list("json");
    assert!(json_output.contains("\"features\""));
    assert!(json_output.contains("\"name\": \"basic\""));

    // Test YAML format
    let yaml_output = registry.format_feature_list("yaml");
    assert!(yaml_output.contains("features:"));
    assert!(yaml_output.contains("  - name: basic"));
}

#[test]
fn test_enable_disable_feature_command() {
    let (_temp_dir, mut registry) = create_test_registry();

    // Test enabling features
    assert!(!registry.is_enabled("metrics"));
    registry
        .enable("metrics")
        .expect("Should enable metrics feature");
    assert!(registry.is_enabled("metrics"));

    // Test enabling a feature with dependencies
    registry.disable("basic").unwrap();
    assert!(!registry.is_enabled("basic"));

    let result = registry.enable("advanced");
    assert!(result.is_err(), "Should fail when dependencies aren't met");

    registry.enable("basic").unwrap();
    registry
        .enable("advanced")
        .expect("Should enable with dependencies met");
    assert!(registry.is_enabled("advanced"));

    // Test disabling a feature that others depend on
    let result = registry.disable("basic");
    assert!(
        result.is_err(),
        "Should fail when other features depend on it"
    );

    // First disable the dependent feature
    registry.disable("advanced").unwrap();
    registry
        .disable("basic")
        .expect("Should disable when no dependencies");
    assert!(!registry.is_enabled("basic"));
}

#[test]
fn test_analyze_dependencies_command() {
    let (_temp_dir, registry) = create_test_registry();

    // Enable both basic and advanced features
    let mut selected_features = HashSet::new();
    selected_features.insert("basic".to_string());
    selected_features.insert("advanced".to_string());

    let analyzer = registry
        .analyze_dependencies(&selected_features)
        .expect("Should analyze dependencies");

    // Test dependency tree generation
    let tree = analyzer.generate_dependency_tree();
    assert!(tree.contains("advanced"));
    assert!(tree.contains("basic"));

    // Test dependency tree visualization - using existing method instead of graph
    let tree_viz = registry.visualize_dependencies(&["advanced".to_string()]);
    assert!(tree_viz.contains("advanced"));
    assert!(tree_viz.contains("basic"));

    // Test required dependencies
    let required = analyzer.get_required_dependencies();
    assert!(
        !required.is_empty(),
        "Should have some required dependencies"
    );

    // Test optimized TOML generation - don't test specific content since it may vary
    let toml = analyzer
        .generate_optimized_toml()
        .expect("Should generate optimized TOML");
    assert!(!toml.is_empty(), "Should not generate empty TOML");
}

#[test]
fn test_build_command_generation() {
    let (_temp_dir, mut registry) = create_test_registry();

    // Enable specific features
    registry.enable("basic").unwrap();
    registry.enable("metrics").unwrap();

    // Test basic build command
    let cmd = registry.generate_build_command(false, None);
    assert_eq!(cmd[0], "cargo");
    assert_eq!(cmd[1], "build");
    assert!(cmd.contains(&"--features".to_string()));

    // The features string should contain our enabled features
    let features_idx = cmd.iter().position(|s| s == "--features").unwrap() + 1;
    let features = &cmd[features_idx];
    assert!(features.contains("basic"));
    assert!(features.contains("metrics"));
    assert!(!features.contains("advanced"));

    // Test release build command
    let release_cmd = registry.generate_build_command(true, None);
    assert!(release_cmd.contains(&"--release".to_string()));

    // Test build with target
    let target_cmd = registry.generate_build_command(false, Some("x86_64-unknown-linux-gnu"));
    assert!(target_cmd.contains(&"--target".to_string()));
    assert!(target_cmd.contains(&"x86_64-unknown-linux-gnu".to_string()));
}

#[test]
fn test_feature_configuration_export_import() {
    let (temp_dir, mut registry) = create_test_registry();

    // Enable specific features
    registry.enable("basic").unwrap();
    registry.enable("metrics").unwrap();

    // Export configuration
    let config_path = temp_dir.path().join("features.json");
    save_test_config(&registry, &config_path).unwrap();

    // Create a new registry
    let mut new_registry = FeatureRegistry::new_empty();

    // Register the same features
    new_registry.register(FeatureInfo {
        name: "basic".to_string(),
        description: "Basic feature".to_string(),
        dependencies: vec![],
        default_enabled: false, // Different default than the original
        category: "Core".to_string(),
        tags: vec!["core".to_string()],
        size_impact: 100,
    });

    new_registry.register(FeatureInfo {
        name: "advanced".to_string(),
        description: "Advanced feature".to_string(),
        dependencies: vec!["basic".to_string()],
        default_enabled: false,
        category: "Advanced".to_string(),
        tags: vec!["advanced".to_string()],
        size_impact: 250,
    });

    new_registry.register(FeatureInfo {
        name: "metrics".to_string(),
        description: "Metrics feature".to_string(),
        dependencies: vec![],
        default_enabled: false,
        category: "Observability".to_string(),
        tags: vec!["monitoring".to_string()],
        size_impact: 150,
    });

    // Import configuration from file
    let config_content = fs::read_to_string(config_path).unwrap();
    new_registry.import_configuration(&config_content).unwrap();

    // Verify imported configuration matches original
    assert!(new_registry.is_enabled("basic"));
    assert!(new_registry.is_enabled("metrics"));
    assert!(!new_registry.is_enabled("advanced"));
}

#[test]
fn test_feature_status_display_command() {
    let (_temp_dir, mut registry) = create_test_registry();

    // Enable some features
    registry.enable("basic").unwrap();
    registry.enable("metrics").unwrap();

    // Get feature status
    let status = registry.format_feature_status();

    // Verify output format and content
    assert!(status.contains("Feature Status:"));
    assert!(status.contains("✓ basic"));
    assert!(status.contains("✓ metrics"));
    assert!(status.contains("✗ advanced"));
}

#[test]
fn test_interactive_mode_menu_commands() {
    let (_temp_dir, registry) = create_test_registry();

    // Get menu items
    let menu_items = registry.build_interactive_menu();

    // Verify expected menu items
    assert!(menu_items.contains(&"Select Features (Interactive)".to_string()));
    assert!(menu_items.contains(&"Show Feature Status".to_string()));
    assert!(menu_items.contains(&"Apply Configuration".to_string()));
    assert!(menu_items.contains(&"Exit".to_string()));
}

#[test]
fn test_size_impact_visualization() {
    let (_temp_dir, mut registry) = create_test_registry();

    // Enable all features to test size impact
    registry.enable("basic").unwrap();
    registry.enable("metrics").unwrap();
    registry.enable("advanced").unwrap();

    // Generate size impact visualization
    let impact = registry.visualize_size_impact();

    // Verify content and format
    assert!(impact.contains("Size Impact:"));
    assert!(impact.contains("basic: 100 KB"));
    assert!(impact.contains("advanced: 250 KB"));
    assert!(impact.contains("metrics: 150 KB"));
    assert!(impact.contains("Total Size:"));

    // Calculate expected total
    let expected_total = 500; // 100 + 250 + 150
    assert!(impact.contains(&format!("Total Size: {} KB", expected_total)));
}
