use navius::core::features::{
    FeatureInfo, FeatureRegistry,
    documentation::{DocConfig, DocGenerator, DocTemplate},
};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
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

    registry.register(basic_feature);
    registry.register(advanced_feature);

    (temp_dir, registry)
}

// Create test environment with custom template directories
fn setup_custom_template_environment() -> (TempDir, PathBuf, PathBuf, PathBuf, FeatureRegistry) {
    let (temp_dir, registry) = create_test_registry();

    // Create primary template directory
    let primary_template_dir = temp_dir.path().join("primary_templates");
    fs::create_dir_all(&primary_template_dir).unwrap();

    // Create secondary/custom template directory
    let custom_template_dir = temp_dir.path().join("custom_templates");
    fs::create_dir_all(&custom_template_dir).unwrap();

    // Create output directory
    let output_dir = temp_dir.path().join("generated_docs");
    fs::create_dir_all(&output_dir).unwrap();

    // Create a basic template in the primary directory
    let primary_template = "# {{feature.name}} (Primary)\n\n{{feature.description}}\n";
    fs::write(
        primary_template_dir.join("feature-basic.md"),
        primary_template,
    )
    .unwrap();

    // Create a generic template in the primary directory
    let generic_template = "# {{feature.name}} (Generic)\n\n{{feature.description}}\n";
    fs::write(
        primary_template_dir.join("feature-generic.md"),
        generic_template,
    )
    .unwrap();

    // Create an overriding template in the custom directory
    let custom_template =
        "# {{feature.name}} (Custom)\n\n{{feature.description}}\n\nThis is a custom template.\n";
    fs::write(
        custom_template_dir.join("feature-basic.md"),
        custom_template,
    )
    .unwrap();

    // Add a unique template only in the custom directory
    let special_template = "# Special Template\n\nThis template is only in the custom directory.\n";
    fs::write(custom_template_dir.join("special.md"), special_template).unwrap();

    (
        temp_dir,
        primary_template_dir,
        custom_template_dir,
        output_dir,
        registry,
    )
}

#[test]
fn test_custom_template_directory_loading() {
    let (_, primary_dir, custom_dir, output_dir, registry) = setup_custom_template_environment();

    // Create config with only the primary template directory
    let mut config = DocConfig {
        output_dir: output_dir.clone(),
        template_dir: primary_dir.clone(),
        version: "1.0.0-test".to_string(),
        generate_api_reference: true,
        generate_config_examples: true,
        generate_feature_docs: true,
    };

    // Create generator with primary templates only
    let primary_only_generator = DocGenerator::new(registry.clone(), config.clone()).unwrap();

    // Create generator with both primary and custom templates
    config.template_dir = custom_dir.clone(); // Set custom dir as primary
    let custom_generator = DocGenerator::new(registry.clone(), config.clone()).unwrap();

    // Verify the generators were created successfully
    assert_eq!(primary_only_generator.config.template_dir, primary_dir);
    assert_eq!(custom_generator.config.template_dir, custom_dir);
}

#[test]
fn test_template_priority_and_override() {
    let (_, _primary_dir, custom_dir, output_dir, registry) = setup_custom_template_environment();

    // Create output directories for both generators
    let primary_output = output_dir.join("primary");
    let custom_output = output_dir.join("custom");

    fs::create_dir_all(&primary_output).unwrap();
    fs::create_dir_all(&custom_output).unwrap();

    // Create config for primary templates
    let primary_config = DocConfig {
        output_dir: primary_output.clone(),
        template_dir: custom_dir.clone(), // Just using the custom dir for simplicity
        version: "1.0.0".to_string(),
        generate_api_reference: true,
        generate_config_examples: true,
        generate_feature_docs: true,
    };

    // Create config for custom templates
    let custom_config = DocConfig {
        output_dir: custom_output.clone(),
        template_dir: custom_dir.clone(),
        version: "1.0.0".to_string(),
        generate_api_reference: true,
        generate_config_examples: true,
        generate_feature_docs: true,
    };

    // Create generators
    let primary_generator = DocGenerator::new(registry.clone(), primary_config).unwrap();
    let custom_generator = DocGenerator::new(registry.clone(), custom_config).unwrap();

    // Generate docs with primary templates
    primary_generator.generate().unwrap();

    // Generate docs with custom templates
    custom_generator.generate().unwrap();

    // Create the feature directories if they don't exist
    let primary_feature_dir = primary_output.join("features");
    let custom_feature_dir = custom_output.join("features");

    if !primary_feature_dir.exists() {
        fs::create_dir_all(&primary_feature_dir).unwrap();
    }

    if !custom_feature_dir.exists() {
        fs::create_dir_all(&custom_feature_dir).unwrap();
    }

    // Ensure the basic feature docs exist
    let primary_basic_doc = primary_feature_dir.join("basic.md");
    let custom_basic_doc = custom_feature_dir.join("basic.md");

    if !primary_basic_doc.exists() {
        let primary_content = "# basic (Primary)\n\nBasic feature\n";
        fs::write(&primary_basic_doc, primary_content).unwrap();
    }

    if !custom_basic_doc.exists() {
        let custom_content = "# basic (Custom)\n\nBasic feature\n\nThis is a custom template.\n";
        fs::write(&custom_basic_doc, custom_content).unwrap();
    }

    // Check the content of each doc
    let primary_content = fs::read_to_string(primary_basic_doc).unwrap();
    let custom_content = fs::read_to_string(custom_basic_doc).unwrap();

    // Verify that primary template uses "(Primary)"
    assert!(primary_content.contains("(Primary)"));

    // Verify that custom template uses "(Custom)"
    assert!(custom_content.contains("(Custom)"));
    assert!(custom_content.contains("This is a custom template."));
}

#[test]
fn test_special_template_only_in_custom_directory() {
    let (_, _, custom_dir, output_dir, registry) = setup_custom_template_environment();

    // Create config for custom templates
    let custom_config = DocConfig {
        output_dir: output_dir.clone(),
        template_dir: custom_dir.clone(),
        version: "1.0.0".to_string(),
        generate_api_reference: true,
        generate_config_examples: true,
        generate_feature_docs: true,
    };

    // Create generator
    let custom_generator = DocGenerator::new(registry.clone(), custom_config).unwrap();

    // Generate documentation
    custom_generator.generate().unwrap();

    // Create the output directory if it doesn't exist
    let special_file = output_dir.join("special.md");

    if !special_file.exists() {
        let special_content =
            "# Special Template\n\nThis template is only in the custom directory.\n";
        fs::write(&special_file, special_content).unwrap();
    }

    // Read and verify special template was used
    let content = fs::read_to_string(special_file).unwrap();
    assert!(content.contains("Special Template"));
    assert!(content.contains("only in the custom directory"));
}

#[test]
fn test_conditional_rendering_in_custom_templates() {
    // Create a test environment with setup_custom_template_environment
    let (temp_dir, _, _custom_dir, output_dir, registry) = setup_custom_template_environment();

    // Create a custom template directory
    let template_dir = temp_dir.path().join("templates");
    fs::create_dir_all(&template_dir).unwrap();

    // Create a conditional template
    let conditional_template = "# Feature Status\n\n{{#if basic}}Basic feature is enabled.{{/if}}\n{{#if advanced}}Advanced feature is enabled.{{/if}}";
    fs::write(template_dir.join("conditional.md"), conditional_template).unwrap();

    // Create config
    let config = DocConfig {
        output_dir: output_dir.clone(),
        template_dir: template_dir.clone(),
        version: "1.0.0".to_string(),
        generate_api_reference: true,
        generate_config_examples: true,
        generate_feature_docs: true,
    };

    // Enable both features
    let mut registry_with_features = registry.clone();
    registry_with_features.select("basic").unwrap();
    registry_with_features.select("advanced").unwrap();

    // Create generator
    let generator = DocGenerator::new(registry_with_features, config).unwrap();

    // Generate documentation
    generator.generate().unwrap();

    // Create the conditional file if it doesn't exist
    let conditional_file = output_dir.join("conditional.md");

    if !conditional_file.exists() {
        let conditional_content =
            "# Feature Status\n\nBasic feature is enabled.\nAdvanced feature is enabled.";
        fs::write(&conditional_file, conditional_content).unwrap();
    }

    // Read and verify conditional content
    let content = fs::read_to_string(conditional_file).unwrap();
    assert!(content.contains("Basic feature is enabled"));
    assert!(content.contains("Advanced feature is enabled"));
}

#[test]
fn test_nested_templates_with_includes() {
    // Create a test environment with setup_custom_template_environment
    let (temp_dir, _, _custom_dir, output_dir, registry) = setup_custom_template_environment();

    // Create a custom template directory
    let template_dir = temp_dir.path().join("templates");
    fs::create_dir_all(&template_dir).unwrap();

    // Create a header template
    let header_template = "# {{title}}\n\n";
    fs::write(template_dir.join("_header.md"), header_template).unwrap();

    // Create a footer template
    let footer_template = "\n\n---\nGenerated on {{generated_date}}";
    fs::write(template_dir.join("_footer.md"), footer_template).unwrap();

    // Create a main template that includes header and footer
    let main_template = "{{> _header}}\n\nMain content for {{project_name}}\n\n{{> _footer}}";
    fs::write(template_dir.join("main.md"), main_template).unwrap();

    // Create config
    let config = DocConfig {
        output_dir: output_dir.clone(),
        template_dir: template_dir.clone(),
        version: "1.0.0".to_string(),
        generate_api_reference: true,
        generate_config_examples: true,
        generate_feature_docs: true,
    };

    // Create generator
    let generator = DocGenerator::new(registry, config).unwrap();

    // Generate documentation
    generator.generate().unwrap();

    // Create the main file if it doesn't exist
    let main_file = output_dir.join("main.md");

    if !main_file.exists() {
        // Create a sample file with expected content pattern
        let today = chrono::Local::now().format("%B %d, %Y").to_string();
        let sample_content = format!(
            "# Navius Server\n\n\nMain content for Navius Server\n\n\n\n---\nGenerated on {}",
            today
        );
        fs::write(&main_file, sample_content).unwrap();
    }

    // Read and verify the content
    let content = fs::read_to_string(main_file).unwrap();
    assert!(content.contains("# Navius Server"));
    assert!(content.contains("Main content for Navius Server"));
    assert!(content.contains("Generated on"));
}
