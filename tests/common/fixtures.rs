use navius::core::features::{
    FeatureConfig, FeatureInfo, FeatureRegistry,
    documentation::{DocConfig, DocGenerator},
};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Create a test feature registry with sample features
pub fn create_test_registry() -> (TempDir, FeatureRegistry) {
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

/// Test environment for interactive mode testing
pub struct FeatureTestEnvironment {
    pub registry: FeatureRegistry,
    pub temp_dir: TempDir,
    pub config_path: PathBuf,
}

impl FeatureTestEnvironment {
    /// Create a new test environment with sample features
    pub fn new() -> Self {
        let (temp_dir, registry) = create_test_registry();

        // Set up a config path
        let config_path = temp_dir.path().join("features.json");

        Self {
            registry,
            temp_dir,
            config_path,
        }
    }

    /// Save current configuration to file
    pub fn save_config(&self) -> Result<(), String> {
        let config = FeatureConfig::from_registry(&self.registry);
        config
            .save(&self.config_path)
            .map_err(|e| format!("Failed to save config: {}", e))
    }

    /// Load configuration from file
    pub fn load_config(&mut self) -> Result<(), String> {
        let config = FeatureConfig::load(&self.config_path)
            .map_err(|e| format!("Failed to load config: {}", e))?;

        // Update registry with loaded config
        for feature in &config.selected_features {
            let _ = self.registry.select(feature);
        }

        Ok(())
    }
}

/// Create a test documentation generator
pub fn create_test_doc_generator(temp_dir: &TempDir) -> (DocGenerator, PathBuf, PathBuf) {
    let (_, registry) = create_test_registry();

    // Create template directory
    let template_dir = temp_dir.path().join("templates");
    fs::create_dir_all(&template_dir).unwrap();

    // Create output directory
    let output_dir = temp_dir.path().join("generated_docs");
    fs::create_dir_all(&output_dir).unwrap();

    // Create a test config
    let config = DocConfig {
        output_dir: output_dir.clone(),
        template_dir: template_dir.clone(),
        version: "1.0.0-test".to_string(),
        generate_api_reference: true,
        generate_config_examples: true,
        generate_feature_docs: true,
    };

    // Create sample templates
    create_sample_templates(&template_dir);

    // Create a DocGenerator instance
    let doc_gen = DocGenerator::new(registry, config).unwrap();

    (doc_gen, template_dir, output_dir)
}

/// Create sample templates for testing
pub fn create_sample_templates(template_dir: &PathBuf) {
    // Create simple test templates
    let readme_template = "# Documentation\n\nEnabled features:\n{{feature_list}}\n";
    fs::write(template_dir.join("README.md"), readme_template).unwrap();

    let feature_template = "# {{feature.name}}\n\n{{feature.description}}\n\n## Dependencies\n\n{{feature.dependencies}}\n";
    fs::write(template_dir.join("feature-generic.md"), feature_template).unwrap();

    // Feature-specific template
    let metrics_template = "# Metrics Feature\n\n{{feature.description}}\n\n## Usage\n\nEnabling this feature provides metrics collection capabilities.\n";
    fs::write(template_dir.join("feature-metrics.md"), metrics_template).unwrap();
}
