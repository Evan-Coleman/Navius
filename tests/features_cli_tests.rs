use navius::core::features::{
    DependencyAnalyzer, FeatureConfig, FeatureInfo, FeatureRegistry, FeatureRegistryExt,
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

    let logging_feature = FeatureInfo {
        name: "logging".to_string(),
        description: "Logging and diagnostics".to_string(),
        dependencies: vec![],
        default_enabled: true,
        category: "Observability".to_string(),
        tags: vec!["monitoring".to_string()],
        size_impact: 120,
    };

    let metrics_feature = FeatureInfo {
        name: "metrics".to_string(),
        description: "Metrics collection and reporting".to_string(),
        dependencies: vec!["logging".to_string()],
        default_enabled: false,
        category: "Observability".to_string(),
        tags: vec!["monitoring".to_string()],
        size_impact: 150,
    };

    registry.register(basic_feature);
    registry.register(advanced_feature);
    registry.register(logging_feature);
    registry.register(metrics_feature);

    // Enable the default features
    let _ = registry.select("basic");
    let _ = registry.select("logging");

    (temp_dir, registry)
}

// Test feature listing functionality with different output formats
#[test]
fn test_features_cli_list_formats() {
    let (_temp_dir, registry) = create_test_registry();

    // Test plain text format (default)
    let text_output = format_features_list(&registry, "text");
    assert!(text_output.contains("Available features:"));
    assert!(text_output.contains("[Core]"));
    assert!(text_output.contains("basic"));
    assert!(text_output.contains("[Advanced]"));
    assert!(text_output.contains("advanced"));

    // Test JSON format
    let json_output = format_features_list(&registry, "json");
    assert!(json_output.contains("\"features\":"));
    assert!(json_output.contains("\"name\": \"basic\""));
    assert!(json_output.contains("\"category\": \"Core\""));

    // Test YAML format
    let yaml_output = format_features_list(&registry, "yaml");
    assert!(yaml_output.contains("features:"));
    assert!(yaml_output.contains("  - name: basic"));
    assert!(yaml_output.contains("    category: Core"));
}

// Helper function to simulate the features_cli list command
fn format_features_list(registry: &FeatureRegistry, format: &str) -> String {
    match format {
        "json" => {
            // Create a simplified JSON representation since we can't directly serialize FeatureInfo
            let mut output = String::from("{\n  \"features\": [\n");

            for feature in registry.feature_list() {
                output.push_str(&format!("    {{\n"));
                output.push_str(&format!("      \"name\": \"{}\",\n", feature.name));
                output.push_str(&format!(
                    "      \"description\": \"{}\",\n",
                    feature.description
                ));
                output.push_str(&format!("      \"category\": \"{}\",\n", feature.category));
                output.push_str(&format!(
                    "      \"size_impact\": {},\n",
                    feature.size_impact
                ));
                output.push_str(&format!(
                    "      \"default_enabled\": {}\n",
                    feature.default_enabled
                ));
                output.push_str(&format!("    }},\n"));
            }

            // Remove trailing comma from last feature
            if registry.feature_count() > 0 {
                output.pop(); // Remove \n
                output.pop(); // Remove ,
                output.push('\n');
            }

            output.push_str("  ]\n}\n");
            output
        }
        "yaml" => {
            // Create a YAML representation
            let mut output = String::from("features:\n");

            for feature in registry.feature_list() {
                output.push_str(&format!("  - name: {}\n", feature.name));
                output.push_str(&format!("    description: {}\n", feature.description));
                output.push_str(&format!("    category: {}\n", feature.category));
                output.push_str(&format!("    size_impact: {}\n", feature.size_impact));
                output.push_str(&format!(
                    "    default_enabled: {}\n",
                    feature.default_enabled
                ));

                if !feature.dependencies.is_empty() {
                    output.push_str("    dependencies:\n");
                    for dep in &feature.dependencies {
                        output.push_str(&format!("      - {}\n", dep));
                    }
                }

                if !feature.tags.is_empty() {
                    output.push_str("    tags:\n");
                    for tag in &feature.tags {
                        output.push_str(&format!("      - {}\n", tag));
                    }
                }
            }

            output
        }
        _ => {
            // Default text format
            let mut output = String::from("Available features:\n");
            output.push_str("------------------\n\n");

            let categories = registry.get_categories();

            for category in categories {
                output.push_str(&format!("[{}]\n", category));

                let features = registry.get_features_by_category(&category);
                for feature in features {
                    let status = if registry.is_selected(&feature.name) {
                        "✓"
                    } else {
                        " "
                    };
                    output.push_str(&format!(
                        "[{}] {} - {}\n",
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

                    output.push_str(&format!("    Size: {} KB\n", feature.size_impact));
                }

                output.push_str("\n");
            }

            output
        }
    }
}

// Test feature enabling and dependency resolution
#[test]
fn test_features_cli_enable_with_dependencies() {
    let (_temp_dir, mut registry) = create_test_registry();

    // Initially basic and logging are enabled by default
    assert!(registry.is_selected("basic"));
    assert!(registry.is_selected("logging"));
    assert!(!registry.is_selected("metrics"));

    // Enable metrics (should also ensure logging remains enabled as dependency)
    let result = handle_enable_feature(&mut registry, "metrics");
    assert!(result.is_ok());

    // Verify metrics and its dependency are enabled
    assert!(registry.is_selected("metrics"));
    assert!(registry.is_selected("logging"));

    // Try to disable logging (should fail as metrics depends on it)
    let result = handle_disable_feature(&mut registry, "logging");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("required by"));

    // Disable metrics first
    let result = handle_disable_feature(&mut registry, "metrics");
    assert!(result.is_ok());

    // Now disabling logging should succeed
    let result = handle_disable_feature(&mut registry, "logging");
    assert!(result.is_ok());

    // Verify both are now disabled
    assert!(!registry.is_selected("metrics"));
    assert!(!registry.is_selected("logging"));
}

// Helper function simulating the features_cli enable command
fn handle_enable_feature(registry: &mut FeatureRegistry, feature_name: &str) -> Result<(), String> {
    match registry.select(feature_name) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to enable feature: {}", e)),
    }
}

// Helper function simulating the features_cli disable command
fn handle_disable_feature(
    registry: &mut FeatureRegistry,
    feature_name: &str,
) -> Result<(), String> {
    match registry.deselect(feature_name) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to disable feature: {}", e)),
    }
}

// Test the status display functionality
#[test]
fn test_features_cli_status_display() {
    let (_temp_dir, registry) = create_test_registry();

    // Get formatted status output
    let status_output = get_formatted_status(&registry);

    // Should show basic and logging as enabled by default
    assert!(status_output.contains("Enabled features:"));
    assert!(status_output.contains("basic"));
    assert!(status_output.contains("logging"));
    assert!(!status_output.contains("advanced"));
    assert!(!status_output.contains("metrics"));

    // Should include feature counts
    assert!(status_output.contains("Total features: 4"));
    assert!(status_output.contains("Enabled: 2"));
}

// Helper function simulating the features_cli status command
fn get_formatted_status(registry: &FeatureRegistry) -> String {
    let mut output = String::from("Current Feature Status\n");
    output.push_str("--------------------\n\n");

    // Get enabled and available counts
    let enabled = registry.get_selected();
    let total = registry.feature_count();

    output.push_str("Enabled features:\n");
    for feature in &enabled {
        if let Some(info) = registry.get_feature_info(feature) {
            output.push_str(&format!(
                "- {} ({} KB): {}\n",
                info.name, info.size_impact, info.description
            ));
        }
    }

    // Add summary counts
    output.push_str(&format!("\nTotal features: {}\n", total));
    output.push_str(&format!("Enabled: {}\n", enabled.len()));
    output.push_str(&format!("Disabled: {}\n", total - enabled.len()));

    // Calculate total size impact
    let total_size = enabled
        .iter()
        .filter_map(|f| registry.get_feature_info(f))
        .map(|f| f.size_impact as u64) // Cast to u64 to ensure we can handle any size
        .sum::<u64>();

    output.push_str(&format!("Total size impact: {} KB\n", total_size));

    output
}

// Test the build configuration generation
#[test]
fn test_features_cli_build_configuration() {
    let (_temp_dir, registry) = create_test_registry();

    // Test default build configuration
    let default_build = generate_build_config(&registry, false, false, None);

    // Should contain the basic and logging features
    assert!(default_build.contains("--features"));
    assert!(default_build.contains("basic"));
    assert!(default_build.contains("logging"));
    assert!(!default_build.contains("--release"));

    // Test release mode
    let release_build = generate_build_config(&registry, true, false, None);
    assert!(release_build.contains("--release"));

    // Test with target
    let target_build =
        generate_build_config(&registry, false, false, Some("x86_64-unknown-linux-gnu"));
    assert!(target_build.contains("--target x86_64-unknown-linux-gnu"));

    // Test with dependency optimization
    let optimized_build = generate_build_config(&registry, false, true, None);
    assert!(optimized_build.contains("--features"));
    assert!(optimized_build.contains("basic"));
    assert!(optimized_build.contains("logging"));
}

// Helper function simulating the features_cli build command
fn generate_build_config(
    registry: &FeatureRegistry,
    release: bool,
    optimize_deps: bool,
    target: Option<&str>,
) -> String {
    let config = FeatureConfig::from_registry(registry);
    let features_str = config
        .selected_features
        .iter()
        .cloned()
        .collect::<Vec<_>>()
        .join(",");

    let mut output = String::from("cargo build ");

    // Add features
    if !features_str.is_empty() {
        output.push_str(&format!("--features {} ", features_str));
    }

    // Add release flag if needed
    if release {
        output.push_str("--release ");
    }

    // Add target if specified
    if let Some(target) = target {
        output.push_str(&format!("--target {} ", target));
    }

    // Apply dependency optimization if requested
    if optimize_deps {
        // In a real scenario this would modify Cargo.toml
        // For testing, we just note that optimization was requested
        output.push_str("# with dependency optimization ");
    }

    output.trim().to_string()
}

// Test dependency analysis functionality
#[test]
fn test_features_cli_dependency_analysis() {
    let (_temp_dir, mut registry) = create_test_registry();

    // Enable all features
    registry.select("metrics").unwrap();
    registry.select("advanced").unwrap();

    // Generate dependency analysis
    let analysis = generate_dependency_analysis(&registry);

    // Check that analysis contains expected elements
    assert!(analysis.contains("Feature Dependency Analysis"));
    assert!(analysis.contains("Direct Dependencies"));
    assert!(analysis.contains("metrics depends on: logging"));
    assert!(analysis.contains("advanced depends on: basic"));
}

// Helper function simulating the features_cli analyze-deps command
fn generate_dependency_analysis(registry: &FeatureRegistry) -> String {
    let mut output = String::from("Feature Dependency Analysis\n\n");

    // Direct dependencies
    output.push_str("Direct Dependencies:\n");
    for feature in registry.get_selected() {
        if let Some(info) = registry.get_feature_info(&feature) {
            if !info.dependencies.is_empty() {
                output.push_str(&format!(
                    "{} depends on: {}\n",
                    feature,
                    info.dependencies.join(", ")
                ));
            }
        }
    }

    // Create a simple dependency visualization in text
    output.push_str("\nDependency Tree:\n");

    // Get all enabled features
    let enabled_features = registry.get_selected();
    for feature in &enabled_features {
        if let Some(info) = registry.get_feature_info(feature) {
            // Skip features with no dependencies
            if info.dependencies.is_empty() {
                output.push_str(&format!("• {}\n", feature));
            } else {
                output.push_str(&format!("• {} → ", feature));
                output.push_str(&info.dependencies.join(", "));
                output.push_str("\n");
            }
        }
    }

    output
}

// Test configuration saving and loading
#[test]
fn test_features_cli_config_save_load() {
    let (temp_dir, mut registry) = create_test_registry();

    // Enable the advanced feature
    registry.select("advanced").unwrap();

    // Create a config path
    let config_path = temp_dir.path().join("features.json");
    let config_path_str = config_path.to_string_lossy().to_string();

    // Save the configuration
    let save_result = save_feature_config(&registry, &config_path_str);
    assert!(save_result.is_ok());
    assert!(config_path.exists());

    // Create a new registry and load the config
    let (_, mut new_registry) = create_test_registry();

    // By default, only basic and logging are enabled
    assert!(new_registry.is_selected("basic"));
    assert!(new_registry.is_selected("logging"));
    assert!(!new_registry.is_selected("advanced"));

    // Load the saved config
    let load_result = load_feature_config(&mut new_registry, &config_path_str);
    assert!(load_result.is_ok());

    // Now it should have basic, logging, and advanced enabled
    assert!(new_registry.is_selected("basic"));
    assert!(new_registry.is_selected("logging"));
    assert!(new_registry.is_selected("advanced"));
}

// Helper function simulating config saving
fn save_feature_config(registry: &FeatureRegistry, path: &str) -> Result<(), String> {
    let config = FeatureConfig::from_registry(registry);
    config
        .save(&PathBuf::from(path))
        .map_err(|e| format!("Failed to save config: {}", e))
}

// Helper function simulating config loading
fn load_feature_config(registry: &mut FeatureRegistry, path: &str) -> Result<(), String> {
    let config = FeatureConfig::load(&PathBuf::from(path))
        .map_err(|e| format!("Failed to load config: {}", e))?;

    // Update registry with loaded config
    for feature in &config.selected_features {
        let _ = registry.select(feature);
    }

    Ok(())
}
