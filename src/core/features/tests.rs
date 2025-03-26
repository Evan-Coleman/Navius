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

#[cfg(test)]
mod documentation_tests {
    use std::collections::{HashMap, HashSet};
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    use crate::core::features::{
        DocConfig, DocGenerator, DocTemplate, FeatureInfo, FeatureRegistry,
    };

    /// Create a test feature registry with sample features
    fn create_test_registry() -> FeatureRegistry {
        let mut registry = FeatureRegistry::new();

        // Add test features
        registry
            .register(FeatureInfo {
                name: "core".to_string(),
                description: "Core functionality".to_string(),
                dependencies: vec![],
                default_enabled: true,
                category: "core".to_string(),
                tags: vec!["essential".to_string()],
                size_impact: 100,
            })
            .unwrap();

        registry
            .register(FeatureInfo {
                name: "metrics".to_string(),
                description: "Metrics collection and reporting".to_string(),
                dependencies: vec!["core".to_string()],
                default_enabled: true,
                category: "monitoring".to_string(),
                tags: vec!["performance".to_string(), "monitoring".to_string()],
                size_impact: 50,
            })
            .unwrap();

        registry
            .register(FeatureInfo {
                name: "caching".to_string(),
                description: "Data caching capabilities".to_string(),
                dependencies: vec!["core".to_string()],
                default_enabled: false,
                category: "performance".to_string(),
                tags: vec!["performance".to_string(), "optimization".to_string()],
                size_impact: 75,
            })
            .unwrap();

        // Enable features
        registry.enable("core").unwrap();
        registry.enable("metrics").unwrap();

        registry
    }

    /// Setup test environment with temporary directories
    fn setup_test_environment() -> (FeatureRegistry, TempDir, TempDir, DocConfig) {
        let registry = create_test_registry();

        // Create temporary directories for output and templates
        let output_dir = TempDir::new().unwrap();
        let template_dir = TempDir::new().unwrap();

        // Create test config
        let config = DocConfig {
            output_dir: output_dir.path().to_path_buf(),
            template_dir: template_dir.path().to_path_buf(),
            version: "1.0.0".to_string(),
            generate_api_reference: true,
            generate_config_examples: true,
            generate_feature_docs: true,
        };

        (registry, output_dir, template_dir, config)
    }

    /// Create a sample template for testing
    fn create_sample_template(template_dir: &PathBuf, name: &str, content: &str) {
        fs::write(template_dir.join(format!("{}.md", name)), content).unwrap();
    }

    #[test]
    fn test_doc_generator_creation() {
        let (registry, _output_dir, _template_dir, config) = setup_test_environment();

        let generator = DocGenerator::new(registry, config);
        assert!(generator.is_ok());
    }

    #[test]
    fn test_feature_docs_generation() {
        let (registry, output_dir, template_dir, config) = setup_test_environment();

        // Create sample feature template
        create_sample_template(
            &template_dir.path().to_path_buf(),
            "feature-generic",
            "# {{feature.name}} Feature\n\n{{feature.description}}\n\n{{feature.dependencies}}\n",
        );

        // Initialize generator and generate docs
        let generator = DocGenerator::new(registry, config).unwrap();
        let result = generator.generate();

        assert!(result.is_ok());

        // Check that feature docs directory was created
        let feature_docs_dir = output_dir.path().join("features");
        assert!(feature_docs_dir.exists());

        // Check that feature index was created
        let feature_index = feature_docs_dir.join("index.md");
        assert!(feature_index.exists());

        // Check that feature files were created
        let core_doc = feature_docs_dir.join("core.md");
        let metrics_doc = feature_docs_dir.join("metrics.md");

        assert!(core_doc.exists());
        assert!(metrics_doc.exists());

        // Check content of the docs
        let core_content = fs::read_to_string(core_doc).unwrap();
        assert!(core_content.contains("Core Feature"));
        assert!(core_content.contains("Core functionality"));

        let metrics_content = fs::read_to_string(metrics_doc).unwrap();
        assert!(metrics_content.contains("Metrics Feature"));
        assert!(metrics_content.contains("Metrics collection and reporting"));
        assert!(metrics_content.contains("This feature depends on"));
    }

    #[test]
    fn test_api_reference_generation() {
        let (registry, output_dir, template_dir, config) = setup_test_environment();

        // Create sample API template
        create_sample_template(
            &template_dir.path().to_path_buf(),
            "api-reference",
            "# API Reference\n\n## Features\n\n{{feature_apis}}\n\n## Enabled Features\n\n{{enabled_features}}\n",
        );

        // Initialize generator and generate docs
        let generator = DocGenerator::new(registry, config).unwrap();
        let result = generator.generate_api_reference();

        assert!(result.is_ok());

        // Check that API docs directory was created
        let api_docs_dir = output_dir.path().join("api");
        assert!(api_docs_dir.exists());

        // Check that API index was created
        let api_index = api_docs_dir.join("index.md");
        assert!(api_index.exists());

        // Check content of the API reference
        let api_content = fs::read_to_string(api_index).unwrap();
        assert!(api_content.contains("API Reference"));
        assert!(api_content.contains("core"));
        assert!(api_content.contains("metrics"));

        // Check that feature-specific API files were created
        let core_api = api_docs_dir.join("core.md");
        let metrics_api = api_docs_dir.join("metrics.md");

        assert!(core_api.exists());
        assert!(metrics_api.exists());
    }

    #[test]
    fn test_comprehensive_api_doc_generation() {
        let (registry, _output_dir, _template_dir, _config) = setup_test_environment();

        // Get a feature for testing
        let feature = FeatureInfo {
            name: "test_feature".to_string(),
            description: "Test feature for API docs".to_string(),
            dependencies: vec!["core".to_string()],
            default_enabled: true,
            category: "core".to_string(),
            tags: vec!["test".to_string(), "api".to_string()],
            size_impact: 10,
        };

        // Create generator directly for this test
        let generator = DocGenerator::new(registry, DocConfig::default()).unwrap();

        // Generate API doc for the feature
        let result = generator.generate_comprehensive_api_doc(&feature);

        assert!(result.is_ok());
        let content = result.unwrap();

        // Check content
        assert!(content.contains("# test_feature API Reference"));
        assert!(content.contains("Test feature for API docs"));
        assert!(content.contains("**Category**: core"));
        assert!(content.contains("**Tags**: test, api"));
        assert!(content.contains("This feature depends on:"));
        assert!(content.contains("- [core](core.md)"));
        assert!(content.contains("## API Details"));
        assert!(content.contains("### Module Structure"));
        assert!(content.contains("### Public Functions"));
        assert!(content.contains("## Example Usage"));
    }

    #[test]
    fn test_template_rendering() {
        let (registry, _output_dir, _template_dir, _config) = setup_test_environment();

        // Create generator directly for this test
        let generator = DocGenerator::new(registry, DocConfig::default()).unwrap();

        // Create a test template with variables
        let template =
            "# {{title}}\n\n{{description}}\n\n- Feature: {{feature}}\n- Version: {{version}}";

        // Create context with test values
        let mut context = HashMap::new();
        context.insert("title".to_string(), "Test Document".to_string());
        context.insert("description".to_string(), "This is a test".to_string());
        context.insert("feature".to_string(), "test_feature".to_string());
        context.insert("version".to_string(), "1.0.0".to_string());

        // Render the template
        let result = generator.render_template(template, &context);

        assert!(result.is_ok());
        let rendered = result.unwrap();

        // Check rendered content
        assert_eq!(
            rendered,
            "# Test Document\n\nThis is a test\n\n- Feature: test_feature\n- Version: 1.0.0"
        );
    }
}

// Testing utilities
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
