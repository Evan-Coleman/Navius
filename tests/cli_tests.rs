use navius::core::features::{FeatureConfig, FeatureInfo, FeatureRegistry, FeatureRegistryExt};
use std::collections::{HashMap, HashSet};
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

#[test]
fn test_dependency_tree_visualization() {
    let (_temp_dir, registry) = create_test_registry();

    // Enable the advanced feature which requires basic
    let mut selected_features = HashSet::new();
    selected_features.insert("advanced".to_string());

    let analyzer = registry.analyze_dependencies(&selected_features).unwrap();
    let tree = analyzer.generate_dependency_tree();

    // Verify tree structure
    assert!(tree.contains("# Dependency Tree"));
    assert!(tree.contains("## Selected Features"));
    assert!(tree.contains("advanced"));
    assert!(tree.contains("basic")); // Should be included as a dependency
}

#[test]
fn test_feature_status_display() {
    let (_temp_dir, registry) = create_test_registry();

    // Get feature status
    let status = registry.get_feature_status();

    // Verify status information
    assert!(status.contains("basic"));
    assert!(status.contains("advanced"));
    assert!(status.contains("metrics"));

    // Basic feature should be enabled by default
    assert!(registry.is_enabled("basic"));
    assert!(!registry.is_enabled("advanced"));
}

#[test]
fn test_feature_selection_validation() {
    let (_temp_dir, mut registry) = create_test_registry();

    // Disable basic feature first
    registry.disable("basic").unwrap();

    // Try to enable advanced without its dependency
    let result = registry.enable("advanced");
    assert!(
        result.is_err(),
        "Should fail when enabling feature without dependencies"
    );

    // Enable basic first, then advanced should work
    registry.enable("basic").unwrap();
    let result = registry.enable("advanced");
    assert!(result.is_ok(), "Should succeed when dependencies are met");
}

#[test]
fn test_feature_size_impact_calculation() {
    let (_temp_dir, registry) = create_test_registry();

    // Calculate total size impact of enabled features
    let total_size = registry.calculate_size_impact();

    // Only basic feature is enabled by default (100 KB)
    assert_eq!(total_size, 100, "Initial size impact should be 100 KB");

    // Enable advanced feature and recalculate
    let mut registry = registry;
    registry.enable("basic").unwrap();
    registry.enable("advanced").unwrap();

    let new_total_size = registry.calculate_size_impact();
    assert_eq!(
        new_total_size, 350,
        "Size impact should be 350 KB with both features"
    );
}

#[test]
fn test_feature_configuration_serialization() {
    let (_temp_dir, mut registry) = create_test_registry();

    // Enable basic feature
    registry.enable("basic").unwrap();

    let config = registry.export_configuration().unwrap();

    // Verify configuration content
    assert!(config.contains("basic"));
    assert!(config.contains("advanced"));
    assert!(config.contains("metrics"));

    // Create a new registry and register features
    let mut new_registry = FeatureRegistry::new_empty();
    new_registry.register(FeatureInfo {
        name: "basic".to_string(),
        description: "Basic feature".to_string(),
        dependencies: vec![],
        default_enabled: true,
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
        size_impact: 200,
    });
    new_registry.register(FeatureInfo {
        name: "metrics".to_string(),
        description: "Metrics feature".to_string(),
        dependencies: vec![],
        default_enabled: false,
        category: "Observability".to_string(),
        tags: vec!["metrics".to_string()],
        size_impact: 150,
    });

    // Import configuration
    new_registry.import_configuration(&config).unwrap();

    // Verify features were properly imported
    assert!(new_registry.is_enabled("basic"));
    assert!(!new_registry.is_enabled("advanced"));
    assert!(!new_registry.is_enabled("metrics"));
}

#[test]
fn test_dependency_optimization() {
    let (_temp_dir, registry) = create_test_registry();

    // Enable both basic and advanced features
    let mut selected_features = HashSet::new();
    selected_features.insert("basic".to_string());
    selected_features.insert("advanced".to_string());

    let analyzer = registry.analyze_dependencies(&selected_features).unwrap();

    // Verify required dependencies
    let required = analyzer.get_required_dependencies();
    assert!(required.contains("tokio"), "Should require tokio");
    assert!(required.contains("axum"), "Should require axum");

    // Generate optimized configuration
    let optimized = analyzer.generate_optimized_toml().unwrap();
    assert!(optimized.contains("tokio"));
    assert!(optimized.contains("axum"));
}

#[test]
fn test_feature_group_organization() {
    let (_temp_dir, registry) = create_test_registry();

    // Get features organized by groups
    let groups = registry.get_feature_groups();

    // Verify group structure
    assert!(groups.contains_key("Core"));
    assert!(groups.contains_key("Advanced"));

    // Basic feature should be in core group
    let core_features = groups.get("Core").unwrap();
    assert!(core_features.contains(&"basic".to_string()));

    // Advanced and metrics should be in their respective groups
    let advanced_features = groups.get("Advanced").unwrap();
    assert!(advanced_features.contains(&"advanced".to_string()));

    let observability_features = groups.get("Observability").unwrap();
    assert!(observability_features.contains(&"metrics".to_string()));
}

#[test]
fn test_cli_feature_list_display() {
    let (_temp_dir, registry) = create_test_registry();

    // Test text format output
    let text_output = registry.format_feature_list("text");
    assert!(text_output.contains("[Core]"));
    assert!(text_output.contains("basic - Basic feature"));
    assert!(text_output.contains("Dependencies:"));
    assert!(text_output.contains("Tags: core"));

    // Test JSON format output
    let json_output = registry.format_feature_list("json");
    assert!(json_output.contains("\"features\": ["));
    assert!(json_output.contains("\"name\": \"basic\""));
    assert!(json_output.contains("\"description\": \"Basic feature\""));

    // Test YAML format output
    let yaml_output = registry.format_feature_list("yaml");
    assert!(yaml_output.contains("features:"));
    assert!(yaml_output.contains("  - name: basic"));
    assert!(yaml_output.contains("    description: Basic feature"));
}

#[test]
fn test_cli_interactive_menu() {
    let (_temp_dir, registry) = create_test_registry();

    // Test menu item generation
    let menu_items = registry.build_interactive_menu();
    assert!(menu_items.contains(&"Select Features (Interactive)".to_string()));
    assert!(menu_items.contains(&"Show Feature Status".to_string()));
    assert!(menu_items.contains(&"Apply Configuration".to_string()));
    assert!(menu_items.contains(&"Exit".to_string()));
}

#[test]
fn test_cli_feature_selection_display() {
    let (_temp_dir, mut registry) = create_test_registry();

    // Enable some features
    registry.enable("basic").unwrap();
    registry.enable("metrics").unwrap();

    // Test feature status display
    let status_display = registry.format_feature_status();
    assert!(status_display.contains("✓ basic"));
    assert!(status_display.contains("✗ advanced"));
    assert!(status_display.contains("✓ metrics"));

    // Test feature selection display
    let selection_display = registry.format_feature_selection();
    assert!(selection_display.contains("[✓] basic"));
    assert!(selection_display.contains("[ ] advanced"));
    assert!(selection_display.contains("[✓] metrics"));

    // Verify required features are marked
    assert!(selection_display.contains("(required)"));
}

#[test]
fn test_cli_dependency_visualization() {
    let (_temp_dir, registry) = create_test_registry();

    // Test dependency tree visualization
    let tree = registry.visualize_dependencies(&["advanced".to_string()]);
    assert!(tree.contains("advanced"));
    assert!(tree.contains("└── basic"));

    // Test dependency graph visualization
    let graph = registry.visualize_dependency_graph();
    assert!(graph.contains("digraph {"));
    assert!(graph.contains("\"advanced\" -> \"basic\""));

    // Test impact visualization
    let impact = registry.visualize_size_impact();
    assert!(impact.contains("Size Impact:"));
    assert!(impact.contains("basic: 100 KB"));
    assert!(impact.contains("advanced: 250 KB"));
    assert!(impact.contains("metrics: 150 KB"));
}

#[test]
fn test_cli_build_command_generation() {
    let (_temp_dir, registry) = create_test_registry();

    // Test basic build command
    let basic_cmd = registry.generate_build_command(false, None);
    assert_eq!(basic_cmd[0], "cargo");
    assert_eq!(basic_cmd[1], "build");

    // Test release build command
    let release_cmd = registry.generate_build_command(true, None);
    assert!(release_cmd.contains(&"--release".to_string()));

    // Test build command with target
    let target_cmd = registry.generate_build_command(false, Some("x86_64-unknown-linux-gnu"));
    assert!(target_cmd.contains(&"--target".to_string()));
    assert!(target_cmd.contains(&"x86_64-unknown-linux-gnu".to_string()));
}
