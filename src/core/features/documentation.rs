//! Documentation generator for the feature system
//!
//! Provides tools for generating documentation based on enabled features.

use crate::core::features::{FeatureError, FeatureInfo, FeatureRegistry, FeatureRegistryExt};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

/// Documentation template format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocTemplate {
    /// Template name
    pub name: String,

    /// Template content
    pub content: String,

    /// Required features for this template
    pub required_features: Vec<String>,

    /// Template variables
    pub variables: HashMap<String, String>,
}

/// Documentation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocConfig {
    /// Output directory for generated documentation
    pub output_dir: PathBuf,

    /// Template directory containing markdown templates
    pub template_dir: PathBuf,

    /// Version information
    pub version: String,

    /// Generate API reference
    pub generate_api_reference: bool,

    /// Generate configuration examples
    pub generate_config_examples: bool,

    /// Generate feature-specific documentation
    pub generate_feature_docs: bool,
}

impl Default for DocConfig {
    fn default() -> Self {
        Self {
            output_dir: PathBuf::from("./docs/generated"),
            template_dir: PathBuf::from("./docs/templates"),
            version: env!("CARGO_PKG_VERSION").to_string(),
            generate_api_reference: true,
            generate_config_examples: true,
            generate_feature_docs: true,
        }
    }
}

/// Documentation generator
pub struct DocGenerator {
    /// Feature registry
    registry: FeatureRegistry,

    /// Documentation configuration
    pub config: DocConfig,

    /// Available templates
    templates: HashMap<String, DocTemplate>,
}

impl DocGenerator {
    /// Create a new documentation generator
    pub fn new(registry: FeatureRegistry, config: DocConfig) -> Result<Self, FeatureError> {
        let mut generator = Self {
            registry,
            config,
            templates: HashMap::new(),
        };

        // Load templates
        generator.load_templates()?;

        Ok(generator)
    }

    /// Load documentation templates from the template directory
    pub fn load_templates(&mut self) -> Result<(), FeatureError> {
        println!("Loading templates from: {:?}", self.config.template_dir);

        if !self.config.template_dir.exists() {
            println!("Template directory not found, creating it...");
            fs::create_dir_all(&self.config.template_dir).map_err(|e| {
                FeatureError::IoError(format!("Failed to create template directory: {}", e))
            })?;

            // Create sample templates
            self.create_sample_templates()?;
        }

        // Read all template files from the template directory
        let entries = fs::read_dir(&self.config.template_dir).map_err(|e| {
            FeatureError::IoError(format!("Failed to read template directory: {}", e))
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| {
                FeatureError::IoError(format!("Failed to read directory entry: {}", e))
            })?;
            let path = entry.path();

            if path.is_file()
                && path
                    .extension()
                    .map_or(false, |ext| ext == "md" || ext == "json")
            {
                self.load_template(&path)?;
            }
        }

        println!("Loaded {} templates", self.templates.len());

        Ok(())
    }

    /// Load a single template from a file
    fn load_template(&mut self, path: &Path) -> Result<(), FeatureError> {
        let file_name = path.file_name().unwrap().to_string_lossy().to_string();
        println!("Loading template: {}", file_name);

        // For JSON templates
        if path.extension().map_or(false, |ext| ext == "json") {
            let content = fs::read_to_string(path).map_err(|e| {
                FeatureError::IoError(format!("Failed to read template file: {}", e))
            })?;

            let template: DocTemplate = serde_json::from_str(&content)
                .map_err(|e| FeatureError::DeserializationError(e.to_string()))?;

            self.templates.insert(template.name.clone(), template);
            return Ok(());
        }

        // For Markdown templates
        let content = fs::read_to_string(path)
            .map_err(|e| FeatureError::IoError(format!("Failed to read template file: {}", e)))?;

        // Extract template metadata from the markdown frontmatter
        let (frontmatter, content) = self.extract_frontmatter(&content)?;

        // Parse frontmatter as YAML
        let frontmatter: serde_yaml::Value = serde_yaml::from_str(&frontmatter)
            .map_err(|e| FeatureError::DeserializationError(e.to_string()))?;

        // Extract required features
        let required_features = if let Some(features) = frontmatter.get("features") {
            let features_vec = features.as_sequence().unwrap_or(&Vec::new()).to_vec();
            features_vec
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        } else {
            Vec::new()
        };

        // Extract variables
        let variables = if let Some(vars) = frontmatter.get("variables") {
            let vars_map = vars
                .as_mapping()
                .unwrap_or(&serde_yaml::Mapping::new())
                .clone();
            vars_map
                .iter()
                .filter_map(|(k, v)| Some((k.as_str()?.to_string(), v.as_str()?.to_string())))
                .collect()
        } else {
            HashMap::new()
        };

        // Create template
        let template = DocTemplate {
            name: path.file_stem().unwrap().to_string_lossy().to_string(),
            content,
            required_features,
            variables,
        };

        self.templates.insert(template.name.clone(), template);

        Ok(())
    }

    /// Extract frontmatter from markdown content
    fn extract_frontmatter(&self, content: &str) -> Result<(String, String), FeatureError> {
        let parts: Vec<&str> = content.splitn(3, "---").collect();

        if parts.len() < 3 {
            // No frontmatter found, return empty frontmatter and original content
            return Ok(("".to_string(), content.to_string()));
        }

        Ok((parts[1].trim().to_string(), parts[2].trim().to_string()))
    }

    /// Create sample templates to demonstrate usage
    fn create_sample_templates(&self) -> Result<(), FeatureError> {
        let sample_readme = r#"---
title: "README for Generated Documentation"
version: "1.0"
features: []
variables:
  project_name: "Navius Server"
  version: "0.1.0"
---
# {{project_name}} Documentation

This documentation has been automatically generated for version {{version}}.

## Enabled Features

{{feature_list}}

## API Reference

For detailed API documentation, see [API Reference](api/index.md).

## Configuration Examples

For configuration examples, see [Configuration Examples](config/index.md).
"#;

        let metrics_doc = r#"---
title: "Metrics System"
version: "1.0"
features: ["metrics"]
variables:
  port: "8081"
---
# Metrics System

The metrics system provides real-time monitoring of {{project_name}} performance.

## Configuration

Metrics are exposed on port {{port}} by default and can be changed in the configuration:

```yaml
metrics:
  enabled: true
  port: {{port}}
  path: "/metrics"
```

## Available Metrics

{{#if advanced_metrics}}
### Advanced Metrics

With the advanced metrics feature enabled, the following additional metrics are available:

- `request_duration_seconds`: Histogram of request durations
- `request_size_bytes`: Histogram of request sizes
- `response_size_bytes`: Histogram of response sizes
{{/if}}

### Standard Metrics

The following metrics are always available:

- `requests_total`: Counter of total requests
- `errors_total`: Counter of total errors
"#;

        let sample_template_dir = self.config.template_dir.join("basic");
        fs::create_dir_all(&sample_template_dir).map_err(|e| {
            FeatureError::IoError(format!("Failed to create sample template directory: {}", e))
        })?;

        fs::write(sample_template_dir.join("README.md"), sample_readme).map_err(|e| {
            FeatureError::IoError(format!("Failed to write sample README template: {}", e))
        })?;

        let features_dir = self.config.template_dir.join("features");
        fs::create_dir_all(&features_dir).map_err(|e| {
            FeatureError::IoError(format!(
                "Failed to create features template directory: {}",
                e
            ))
        })?;

        fs::write(features_dir.join("metrics.md"), metrics_doc).map_err(|e| {
            FeatureError::IoError(format!("Failed to write metrics template: {}", e))
        })?;

        Ok(())
    }

    /// Generate documentation
    pub fn generate(&self) -> Result<(), FeatureError> {
        println!("Generating documentation for enabled features...");

        // Create output directory if it doesn't exist
        if !self.config.output_dir.exists() {
            fs::create_dir_all(&self.config.output_dir).map_err(|e| {
                FeatureError::IoError(format!("Failed to create output directory: {}", e))
            })?;
        }

        // Generate main documentation index
        self.generate_index()?;

        // Generate feature documentation
        if self.config.generate_feature_docs {
            self.generate_feature_docs()?;
        }

        // Generate API reference
        if self.config.generate_api_reference {
            self.generate_api_reference()?;
        }

        // Generate configuration examples
        if self.config.generate_config_examples {
            self.generate_config_examples()?;
        }

        println!(
            "Documentation generated successfully at: {:?}",
            self.config.output_dir
        );

        Ok(())
    }

    /// Generate the main documentation index
    fn generate_index(&self) -> Result<(), FeatureError> {
        println!("Generating main documentation index...");

        // Look for the README template
        if let Some(template) = self.templates.get("README") {
            // Create feature list for the template
            let feature_list = self
                .registry
                .get_selected()
                .iter()
                .map(|f| {
                    if let Some(info) = self.registry.get_feature_info(f) {
                        format!("- **{}**: {}", info.name, info.description)
                    } else {
                        format!("- {}", f)
                    }
                })
                .collect::<Vec<_>>()
                .join("\n");

            // Create context for the template
            let mut context = self.create_base_context();
            context.insert("feature_list".to_string(), feature_list);

            // Render the template
            let content = self.render_template(&template.content, &context)?;

            // Write the index file
            let output_path = self.config.output_dir.join("index.md");
            fs::write(&output_path, content)
                .map_err(|e| FeatureError::IoError(format!("Failed to write index file: {}", e)))?;

            println!("Generated index at: {:?}", output_path);
        } else {
            println!("README template not found, skipping index generation");
        }

        Ok(())
    }

    /// Generate feature-specific documentation
    fn generate_feature_docs(&self) -> Result<(), FeatureError> {
        println!("Generating feature-specific documentation...");

        // Create output directory for feature docs
        let feature_docs_dir = self.config.output_dir.join("features");
        fs::create_dir_all(&feature_docs_dir).map_err(|e| {
            FeatureError::IoError(format!("Failed to create feature docs directory: {}", e))
        })?;

        // Get enabled features - use get_selected() instead of get_enabled()
        let enabled_features: HashSet<String> = self.registry.get_selected().clone();

        // Track generated docs
        let mut generated_features = HashSet::new();

        // Generate documentation for each enabled feature
        for feature_name in &enabled_features {
            let feature_info = self
                .registry
                .get_feature_info(feature_name)
                .ok_or_else(|| FeatureError::UnknownFeature(feature_name.clone()))?;

            // Skip features we've already generated docs for
            if generated_features.contains(feature_name) {
                continue;
            }

            // Generate documentation for this feature
            self.generate_feature_doc(&feature_info, &feature_docs_dir)?;
            generated_features.insert(feature_name.clone());

            // Generate documentation for related features (dependencies and dependents)
            self.generate_related_features_docs(
                &feature_info,
                &feature_docs_dir,
                &mut generated_features,
            )?;
        }

        // Generate feature index
        self.generate_feature_index(&feature_docs_dir, &enabled_features)?;

        println!("✅ Feature documentation generated successfully.");
        Ok(())
    }

    /// Generate documentation for a specific feature
    fn generate_feature_doc(
        &self,
        feature_info: &FeatureInfo,
        output_dir: &Path,
    ) -> Result<(), FeatureError> {
        println!(
            "Generating documentation for feature: {}",
            feature_info.name
        );

        // Look for a feature-specific template first
        let template_name = format!("feature-{}", feature_info.name);
        let template_content = if let Some(template) = self.templates.get(&template_name) {
            template.content.clone()
        } else if let Some(template) = self.templates.get("feature-generic") {
            template.content.clone()
        } else {
            // Create default template if no template exists
            let default_template = format!(
                "# {} Feature\n\n## Overview\n\n{}\n\n## Configuration\n\n```yaml\n{}: {{}}\n```\n\n## API Reference\n\n## Examples\n\n",
                feature_info.name, feature_info.description, feature_info.name
            );
            default_template
        };

        // Create context for template rendering
        let mut context = self.create_base_context();
        context.insert("feature.name".to_string(), feature_info.name.clone());
        context.insert(
            "feature.description".to_string(),
            feature_info.description.clone(),
        );
        context.insert(
            "feature.category".to_string(),
            feature_info.category.clone(),
        );

        // Add dependencies list
        let dependencies = if feature_info.dependencies.is_empty() {
            "This feature has no dependencies.".to_string()
        } else {
            let mut deps = String::from("This feature depends on:\n\n");
            for dep in &feature_info.dependencies {
                deps.push_str(&format!("- [{}](./{})\n", dep, dep));
            }
            deps
        };
        context.insert("feature.dependencies".to_string(), dependencies);

        // Add tags
        let tags = if feature_info.tags.is_empty() {
            "".to_string()
        } else {
            let mut tag_list = String::from("**Tags**: ");
            tag_list.push_str(&feature_info.tags.join(", "));
            tag_list
        };
        context.insert("feature.tags".to_string(), tags);

        // Render the template
        let rendered = self.render_template(&template_content, &context)?;

        // Write rendered content to file
        let output_file = output_dir.join(format!("{}.md", feature_info.name));
        fs::write(&output_file, rendered).map_err(|e| {
            FeatureError::IoError(format!("Failed to write feature documentation: {}", e))
        })?;

        Ok(())
    }

    /// Generate documentation for related features (dependencies and dependents)
    fn generate_related_features_docs(
        &self,
        feature_info: &FeatureInfo,
        output_dir: &Path,
        generated_features: &mut HashSet<String>,
    ) -> Result<(), FeatureError> {
        // Generate docs for dependencies
        for dep_name in &feature_info.dependencies {
            let dep_name_owned = dep_name.to_string();
            if !generated_features.contains(&dep_name_owned) {
                if let Some(dep_info) = self.registry.get_feature_info(&dep_name_owned) {
                    self.generate_feature_doc(&dep_info, output_dir)
                        .map_err(|e| {
                            FeatureError::IoError(format!(
                                "Failed to generate documentation for dependency {}: {}",
                                dep_name_owned, e
                            ))
                        })?;
                    generated_features.insert(dep_name_owned);
                }
            }
        }

        // Get dependents (features that depend on this one)
        // Since get_dependent_features doesn't exist, find them manually
        let dependent_features = self.find_dependent_features(&feature_info.name);

        for dep_name in &dependent_features {
            let dep_name_owned = dep_name.to_string();
            if !generated_features.contains(&dep_name_owned) {
                if let Some(dep_info) = self.registry.get_feature_info(&dep_name_owned) {
                    self.generate_feature_doc(&dep_info, output_dir)
                        .map_err(|e| {
                            FeatureError::IoError(format!(
                                "Failed to generate documentation for dependent feature {}: {}",
                                dep_name_owned, e
                            ))
                        })?;
                    generated_features.insert(dep_name_owned);
                }
            }
        }

        Ok(())
    }

    /// Find features that depend on a given feature
    fn find_dependent_features(&self, feature_name: &str) -> Vec<String> {
        let mut dependents = Vec::new();

        // Get all features and check if they depend on the given feature
        for feature in self.registry.features() {
            if feature.dependencies.contains(&feature_name.to_string()) {
                dependents.push(feature.name.clone());
            }
        }

        dependents
    }

    /// Generate an index of all features
    fn generate_feature_index(
        &self,
        output_dir: &Path,
        enabled_features: &HashSet<String>,
    ) -> Result<(), FeatureError> {
        println!("Generating feature index...");

        // Create content for feature index
        let mut content = String::from("# Feature Index\n\n");
        content.push_str(
            "This document provides an overview of all available features in the system.\n\n",
        );

        // Group features by category with error handling
        let mut features_by_category: HashMap<String, Vec<&FeatureInfo>> = HashMap::new();

        for feature in self.registry.features() {
            features_by_category
                .entry(feature.category.clone())
                .or_default()
                .push(feature);
        }

        // Add each category and its features
        for (category, features) in features_by_category {
            // Add category header
            content.push_str(&format!("## {}\n\n", category));

            for feature in features {
                // Determine feature status with proper string ownership
                let status = if enabled_features.contains(&feature.name) {
                    "✅ Enabled".to_owned()
                } else {
                    "❌ Disabled".to_owned()
                };

                // Convert tags to a string, handling empty tags gracefully
                let tags_string = if feature.tags.is_empty() {
                    "none".to_owned()
                } else {
                    feature.tags.join(", ")
                };

                // Format feature documentation with error handling for each field
                let feature_doc = format!(
                    "### [{}](./{}.md)\n\n{}\n\n**Status**: {} | **Tags**: {}\n\n",
                    feature.name, feature.name, feature.description, status, tags_string
                );

                content.push_str(&feature_doc);
            }
        }

        // Write to file with detailed error handling
        let output_file = output_dir.join("index.md");

        // Ensure the output directory exists
        if let Some(parent) = output_file.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).map_err(|e| {
                    FeatureError::IoError(format!(
                        "Failed to create directory {:?} for feature index: {}",
                        parent, e
                    ))
                })?;
            }
        }

        // Write the content with detailed error handling
        fs::write(&output_file, &content).map_err(|e| {
            FeatureError::IoError(format!(
                "Failed to write feature index at {:?}: {}. Error details: {}",
                output_file,
                e,
                e.to_string()
            ))
        })?;

        println!(
            "✅ Feature index generated successfully at: {:?}",
            output_file
        );
        Ok(())
    }

    /// Generate API reference documentation for enabled features
    fn generate_api_reference(&self) -> Result<(), FeatureError> {
        println!("Generating API reference documentation...");

        // Create output directory for API docs
        let api_docs_dir = self.config.output_dir.join("api");
        fs::create_dir_all(&api_docs_dir).map_err(|e| {
            FeatureError::IoError(format!("Failed to create API docs directory: {}", e))
        })?;

        // Get enabled features - use get_selected() instead of get_enabled()
        let enabled_features = self.registry.get_selected();

        // Load API reference templates
        let api_template_name = "api-reference";
        let api_template = if let Some(template) = self.templates.get(api_template_name) {
            template.content.clone()
        } else {
            // Create default API reference template if not found
            String::from(
                "# API Reference\n\n## Core API\n\n## Feature-Specific APIs\n\n{{feature_apis}}\n",
            )
        };

        // Generate API sections for each feature
        let mut feature_apis = String::new();

        // Group features by category
        let mut features_by_category: HashMap<String, Vec<String>> = HashMap::new();

        for feature_name in &enabled_features {
            if let Some(feature) = self.registry.get_feature_info(feature_name) {
                features_by_category
                    .entry(feature.category.clone())
                    .or_default()
                    .push(feature_name.to_string()); // Convert &str to String explicitly
            }
        }

        // Add API sections for each category
        for (category, feature_names) in &features_by_category {
            feature_apis.push_str(&format!("### {}\n\n", category));

            for feature_name in feature_names {
                // Try to find feature-specific API template
                let api_key = format!("api-{}", feature_name);
                let api_content = if let Some(template) = self.templates.get(&api_key) {
                    // Use the specific template
                    template.content.clone()
                } else {
                    // Generate a standard API section
                    self.generate_standard_api_section(feature_name)?
                };

                feature_apis.push_str(&api_content);
                feature_apis.push_str("\n\n");
            }
        }

        // Create context for rendering
        let mut context = self.create_base_context();
        context.insert("feature_apis".to_string(), feature_apis);

        // Insert list of enabled features
        let enabled_list = enabled_features
            .iter()
            .map(|f| format!("- {}", f))
            .collect::<Vec<String>>()
            .join("\n");
        context.insert("enabled_features".to_string(), enabled_list);

        // Render the main API reference template
        let rendered = self.render_template(&api_template, &context)?;

        // Write to file
        let output_file = api_docs_dir.join("index.md");
        fs::write(&output_file, rendered)
            .map_err(|e| FeatureError::IoError(format!("Failed to write API reference: {}", e)))?;

        // Generate feature-specific API reference files
        self.generate_feature_api_files(&api_docs_dir, &enabled_features)?;

        println!("✅ API reference documentation generated successfully.");
        Ok(())
    }

    /// Generate standard API section for a feature
    fn generate_standard_api_section(&self, feature_name: &str) -> Result<String, FeatureError> {
        let feature = self
            .registry
            .get_feature_info(feature_name)
            .ok_or_else(|| FeatureError::UnknownFeature(feature_name.to_string()))?;

        let mut content = format!("### {} API\n\n", feature_name);
        content.push_str(&format!("{}\n\n", feature.description));

        // Add example usage
        content.push_str("#### Example Usage\n\n");
        content.push_str("```rust\n");

        // Generate different example code based on feature category
        match feature.category.as_str() {
            "core" => {
                content.push_str(&format!("// Core feature: {}\n", feature_name));
                content.push_str(&format!("use navius::core::{}::*;\n\n", feature_name));
                content.push_str(&format!("let result = {}::initialize();\n", feature_name));
            }
            "api" => {
                content.push_str(&format!("// API feature: {}\n", feature_name));
                content.push_str("use navius::core::api::*;\n\n");
                content.push_str(&format!("let router = build_router();\n"));
                content.push_str(&format!("router.route({}_routes());\n", feature_name));
            }
            "security" => {
                content.push_str(&format!("// Security feature: {}\n", feature_name));
                content.push_str("use navius::core::security::*;\n\n");
                content.push_str(&format!("let guard = {}::create_guard();\n", feature_name));
                content.push_str(&format!("router.with(guard);\n"));
            }
            _ => {
                content.push_str(&format!("// Feature: {}\n", feature_name));
                content.push_str(&format!(
                    "use navius::core::features::{}::*;\n\n",
                    feature_name
                ));
                content.push_str(&format!("let feature = {}::new();\n", feature_name));
                content.push_str(&format!("feature.initialize();\n"));
            }
        }

        content.push_str("```\n\n");

        // Add common operations
        content.push_str("#### Common Operations\n\n");
        content.push_str("| Operation | Description |\n");
        content.push_str("|-----------|-------------|\n");
        content.push_str(&format!(
            "| `{}::initialize()` | Initialize the feature |\n",
            feature_name
        ));
        content.push_str(&format!(
            "| `{}::configure(config)` | Configure the feature |\n",
            feature_name
        ));

        // Add feature-specific operations based on category
        match feature.category.as_str() {
            "core" => {
                content.push_str(&format!(
                    "| `{}::start()` | Start the core service |\n",
                    feature_name
                ));
                content.push_str(&format!(
                    "| `{}::stop()` | Stop the core service |\n",
                    feature_name
                ));
            }
            "api" => {
                content.push_str(&format!(
                    "| `{}::routes()` | Get API routes |\n",
                    feature_name
                ));
                content.push_str(&format!(
                    "| `{}::handlers()` | Get API handlers |\n",
                    feature_name
                ));
            }
            "security" => {
                content.push_str(&format!(
                    "| `{}::verify(token)` | Verify security token |\n",
                    feature_name
                ));
                content.push_str(&format!(
                    "| `{}::authorize(user, resource)` | Check authorization |\n",
                    feature_name
                ));
            }
            _ => {
                content.push_str(&format!(
                    "| `{}::enable()` | Enable the feature |\n",
                    feature_name
                ));
                content.push_str(&format!(
                    "| `{}::disable()` | Disable the feature |\n",
                    feature_name
                ));
            }
        }

        Ok(content)
    }

    /// Generate detailed API files for each feature
    fn generate_feature_api_files(
        &self,
        output_dir: &Path,
        enabled_features: &HashSet<String>,
    ) -> Result<(), FeatureError> {
        for feature_name in enabled_features {
            let feature = self
                .registry
                .get_feature_info(feature_name)
                .ok_or_else(|| FeatureError::UnknownFeature(feature_name.to_string()))?;

            // Template selection
            let template_name = format!("api-{}-detailed", feature_name);
            let api_content = if let Some(template) = self.templates.get(&template_name) {
                // Use the specific template
                template.content.clone()
            } else {
                // Generate a comprehensive API doc
                self.generate_comprehensive_api_doc(&feature)?
            };

            // Create context for rendering
            let mut context = self.create_base_context();
            context.insert("feature.name".to_string(), feature.name.clone());
            context.insert(
                "feature.description".to_string(),
                feature.description.clone(),
            );
            context.insert("feature.category".to_string(), feature.category.clone());

            // Render the template
            let rendered = self.render_template(&api_content, &context)?;

            // Write to file
            let output_file = output_dir.join(format!("{}.md", feature_name));
            fs::write(&output_file, rendered).map_err(|e| {
                FeatureError::IoError(format!("Failed to write feature API doc: {}", e))
            })?;
        }

        Ok(())
    }

    /// Generate comprehensive API documentation for a feature
    fn generate_comprehensive_api_doc(
        &self,
        feature: &FeatureInfo,
    ) -> Result<String, FeatureError> {
        let mut content = format!("# {} API Reference\n\n", feature.name);
        content.push_str(&format!("## Overview\n\n{}\n\n", feature.description));

        // Category and tags
        content.push_str(&format!("**Category**: {}\n\n", feature.category));

        if !feature.tags.is_empty() {
            content.push_str(&format!("**Tags**: {}\n\n", feature.tags.join(", ")));
        }

        // Dependencies
        if !feature.dependencies.is_empty() {
            content.push_str("## Dependencies\n\n");
            content.push_str("This feature depends on:\n\n");

            for dep in &feature.dependencies {
                content.push_str(&format!("- [{0}]({0}.md)\n", dep));
            }
            content.push_str("\n");
        }

        // API details
        content.push_str("## API Details\n\n");

        // Module structure
        content.push_str("### Module Structure\n\n");
        content.push_str("```rust\n");
        content.push_str(&format!("navius::core::{}::\n", feature.name));
        content.push_str("├── types       // Type definitions\n");
        content.push_str("├── errors      // Error types\n");
        content.push_str("├── config      // Configuration\n");
        content.push_str("└── api         // Public API\n");
        content.push_str("```\n\n");

        // Public functions
        content.push_str("### Public Functions\n\n");
        content.push_str("```rust\n");

        // Add standard functions based on feature category
        match feature.category.as_str() {
            "core" => {
                content.push_str(&format!("/// Initialize the {} feature\n", feature.name));
                content.push_str(&format!(
                    "pub fn initialize() -> Result<(), Error> {{}}\n\n"
                ));

                content.push_str(&format!("/// Configure the {} feature\n", feature.name));
                content.push_str(&format!(
                    "pub fn configure(config: {}Config) -> Result<(), Error> {{}}\n\n",
                    feature.name
                ));

                content.push_str(&format!("/// Start the {} service\n", feature.name));
                content.push_str(&format!("pub fn start() -> Result<(), Error> {{}}\n\n"));

                content.push_str(&format!("/// Stop the {} service\n", feature.name));
                content.push_str(&format!("pub fn stop() -> Result<(), Error> {{}}\n"));
            }
            "api" => {
                content.push_str(&format!("/// Get routes for the {} API\n", feature.name));
                content.push_str(&format!("pub fn routes() -> Router {{}}\n\n"));

                content.push_str(&format!("/// Get handlers for the {} API\n", feature.name));
                content.push_str(&format!("pub fn handlers() -> Handlers {{}}\n\n"));

                content.push_str(&format!("/// Handle {} requests\n", feature.name));
                content.push_str(&format!(
                    "pub async fn handle(req: Request) -> Response {{}}\n"
                ));
            }
            "security" => {
                content.push_str(&format!(
                    "/// Create a security guard for the {} feature\n",
                    feature.name
                ));
                content.push_str(&format!("pub fn create_guard() -> Guard {{}}\n\n"));

                content.push_str("/// Verify a security token\n");
                content.push_str(&format!(
                    "pub fn verify(token: &str) -> Result<Claims, Error> {{}}\n\n"
                ));

                content.push_str("/// Check authorization\n");
                content.push_str(&format!(
                    "pub fn authorize(user: &User, resource: &Resource) -> Result<(), Error> {{}}\n"
                ));
            }
            _ => {
                content.push_str(&format!(
                    "/// Create a new instance of the {} feature\n",
                    feature.name
                ));
                content.push_str(&format!("pub fn new() -> {} {{}}\n\n", feature.name));

                content.push_str(&format!("/// Initialize the {} feature\n", feature.name));
                content.push_str(&format!(
                    "pub fn initialize(&self) -> Result<(), Error> {{}}\n\n"
                ));

                content.push_str(&format!("/// Enable the {} feature\n", feature.name));
                content.push_str(&format!(
                    "pub fn enable(&mut self) -> Result<(), Error> {{}}\n\n"
                ));

                content.push_str(&format!("/// Disable the {} feature\n", feature.name));
                content.push_str(&format!(
                    "pub fn disable(&mut self) -> Result<(), Error> {{}}\n"
                ));
            }
        }
        content.push_str("```\n\n");

        // Example usage
        content.push_str("## Example Usage\n\n");
        content.push_str("```rust\n");

        // Generate different example code based on feature category
        match feature.category.as_str() {
            "core" => {
                content.push_str(&format!("use navius::core::{}::*;\n\n", feature.name));
                content.push_str("fn main() -> Result<(), Error> {\n");
                content.push_str(&format!("    // Initialize the {} feature\n", feature.name));
                content.push_str(&format!("    {}::initialize()?;\n\n", feature.name));
                content.push_str(&format!("    // Configure the feature\n"));
                content.push_str(&format!("    let config = {}Config {{\n", feature.name));
                content.push_str(&format!("        // Configure options\n"));
                content.push_str(&format!("        enabled: true,\n"));
                content.push_str(&format!("        // ... other options\n"));
                content.push_str(&format!("    }};\n"));
                content.push_str(&format!("    {}::configure(config)?;\n\n", feature.name));
                content.push_str(&format!("    // Start the service\n"));
                content.push_str(&format!("    {}::start()?;\n\n", feature.name));
                content.push_str(&format!("    // ... application code ...\n\n"));
                content.push_str(&format!("    // Stop the service when done\n"));
                content.push_str(&format!("    {}::stop()?;\n\n", feature.name));
                content.push_str(&format!("    Ok(())\n"));
                content.push_str(&format!("}}"));
            }
            "api" => {
                content.push_str(&format!("use navius::core::api::*;\n"));
                content.push_str(&format!("use navius::core::{}::*;\n\n", feature.name));
                content.push_str("async fn configure_api() {\n");
                content.push_str(&format!("    // Get a router instance\n"));
                content.push_str(&format!("    let mut app = Router::new();\n\n"));
                content.push_str(&format!("    // Add routes for the {} API\n", feature.name));
                content.push_str(&format!(
                    "    let {}_routes = {}::routes();\n",
                    feature.name, feature.name
                ));
                content.push_str(&format!(
                    "    app = app.nest(\"/{}\", {}_routes);\n\n",
                    feature.name, feature.name
                ));
                content.push_str(&format!("    // ... configure other API routes ...\n\n"));
                content.push_str(&format!("    // Build the application\n"));
                content.push_str(&format!("    let app = app.build();\n"));
                content.push_str(&format!("}}"));
            }
            "security" => {
                content.push_str(&format!("use navius::core::security::*;\n"));
                content.push_str(&format!("use navius::core::{}::*;\n\n", feature.name));
                content.push_str("async fn configure_security(router: &mut Router) {\n");
                content.push_str(&format!("    // Create a security guard\n"));
                content.push_str(&format!(
                    "    let guard = {}::create_guard();\n\n",
                    feature.name
                ));
                content.push_str(&format!("    // Apply the guard to protected routes\n"));
                content.push_str(&format!(
                    "    router.route(\"/protected\", get(protected_handler))\n"
                ));
                content.push_str(&format!("        .with(guard);\n\n"));
                content.push_str(&format!("    // Verify a token\n"));
                content.push_str(&format!("    let token = \"user_token_here\";\n"));
                content.push_str(&format!("    match {}::verify(token) {{\n", feature.name));
                content.push_str(&format!(
                    "        Ok(claims) => println!(\"Valid token: {{:?}}\", claims),\n"
                ));
                content.push_str(&format!(
                    "        Err(e) => println!(\"Invalid token: {{}}\", e),\n"
                ));
                content.push_str(&format!("    }}\n"));
                content.push_str(&format!("}}"));
            }
            _ => {
                content.push_str(&format!(
                    "use navius::core::features::{}::*;\n\n",
                    feature.name
                ));
                content.push_str("fn main() -> Result<(), Error> {\n");
                content.push_str(&format!("    // Create a new instance of the feature\n"));
                content.push_str(&format!(
                    "    let mut {} = {}::new();\n\n",
                    feature.name.to_lowercase(),
                    feature.name
                ));
                content.push_str(&format!("    // Initialize the feature\n"));
                content.push_str(&format!(
                    "    {}.initialize()?;\n\n",
                    feature.name.to_lowercase()
                ));
                content.push_str(&format!("    // Enable the feature\n"));
                content.push_str(&format!(
                    "    {}.enable()?;\n\n",
                    feature.name.to_lowercase()
                ));
                content.push_str(&format!("    // ... use the feature ...\n\n"));
                content.push_str(&format!("    // Disable when done\n"));
                content.push_str(&format!(
                    "    {}.disable()?;\n\n",
                    feature.name.to_lowercase()
                ));
                content.push_str(&format!("    Ok(())\n"));
                content.push_str(&format!("}}"));
            }
        }
        content.push_str("\n```\n\n");

        Ok(content)
    }

    /// Create base context for template rendering
    fn create_base_context(&self) -> HashMap<String, String> {
        let mut context = HashMap::new();

        // Add basic variables
        context.insert("version".to_string(), self.config.version.clone());
        context.insert(
            "generated_date".to_string(),
            chrono::Local::now().format("%B %d, %Y").to_string(),
        );
        context.insert("project_name".to_string(), "Navius Server".to_string());

        // Include variables from the template
        context
    }

    /// Create feature-specific context for template rendering
    fn create_feature_context(
        &self,
        selected_features: &HashSet<String>,
    ) -> HashMap<String, String> {
        let mut context = self.create_base_context();

        // Add feature flags to the context
        for feature in selected_features {
            context.insert(feature.clone(), "true".to_string());
        }

        // Add feature descriptions
        for feature in selected_features {
            if let Some(info) = self.registry.get_feature_info(feature) {
                context.insert(format!("{}_description", feature), info.description.clone());
            }
        }

        context
    }

    /// Render a template with the given context
    fn render_template(
        &self,
        template: &str,
        context: &HashMap<String, String>,
    ) -> Result<String, FeatureError> {
        let mut result = template.to_string();

        // Simple template rendering - replace {{variable}} with value
        for (key, value) in context {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, value);
        }

        // Handle conditionals - {{#if feature}}...{{/if}}
        let regex = regex::Regex::new(r"\{\{#if\s+([^\}]+)\}\}(.*?)\{\{/if\}\}").unwrap();

        while regex.is_match(&result) {
            result = regex
                .replace_all(&result, |caps: &regex::Captures| {
                    let condition = &caps[1];
                    let content = &caps[2];

                    if context.get(condition).map_or(false, |v| v == "true") {
                        content.to_string()
                    } else {
                        "".to_string()
                    }
                })
                .to_string();
        }

        Ok(result)
    }

    /// Generate a standalone version of the documentation
    pub fn generate_versioned(&self, version_tag: &str) -> Result<PathBuf, FeatureError> {
        // Create version-specific directory
        let versioned_dir = self.config.output_dir.join("versions").join(version_tag);

        if !versioned_dir.exists() {
            fs::create_dir_all(&versioned_dir).map_err(|e| {
                FeatureError::IoError(format!("Failed to create versioned directory: {}", e))
            })?;
        }

        // Create a version-specific config
        let versioned_config = DocConfig {
            output_dir: versioned_dir.clone(),
            version: version_tag.to_string(),
            ..self.config.clone()
        };

        // We need to use a new generator with the same registry
        // Use the registry settings from this instance
        let registry = FeatureRegistry::new();

        // Create a new generator with the version-specific config
        let mut versioned_generator = DocGenerator::new(registry, versioned_config)?;

        // Copy the selected features from the original registry
        for feature in self.registry.get_selected() {
            let _ = versioned_generator.registry.select(&feature);
        }

        // Generate the documentation
        versioned_generator.generate()?;

        println!(
            "Generated versioned documentation for {} at: {:?}",
            version_tag, versioned_dir
        );

        Ok(versioned_dir)
    }

    /// Generate configuration examples
    fn generate_config_examples(&self) -> Result<(), FeatureError> {
        println!("Generating configuration examples...");

        // Create configuration directory
        let config_dir = self.config.output_dir.join("config");
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir).map_err(|e| {
                FeatureError::IoError(format!("Failed to create config directory: {}", e))
            })?;
        }

        // Generate configuration examples for each environment
        self.generate_config_example("development", &config_dir)?;
        self.generate_config_example("production", &config_dir)?;
        self.generate_config_example("testing", &config_dir)?;

        // Generate configuration index
        let config_index = r#"# Configuration Examples

This directory contains configuration examples for different environments:

- [Development Configuration](development.md)
- [Production Configuration](production.md)
- [Testing Configuration](testing.md)

These examples include only configuration for enabled features.
"#;

        fs::write(config_dir.join("index.md"), config_index)
            .map_err(|e| FeatureError::IoError(format!("Failed to write config index: {}", e)))?;

        println!("Generated configuration examples");
        Ok(())
    }

    /// Generate configuration example for a specific environment
    fn generate_config_example(
        &self,
        environment: &str,
        config_dir: &Path,
    ) -> Result<(), FeatureError> {
        let selected_features = self.registry.get_selected();
        let mut config_sections = Vec::new();

        // Add standard configuration
        config_sections.push(format!("# {} Configuration\n", environment.to_uppercase()));
        config_sections.push("```yaml\n# Server configuration\nserver:\n  host: \"0.0.0.0\"\n  port: 8080\n  \n# Logging configuration\nlogging:\n  level: \"info\"\n  format: \"json\"\n```\n".to_string());

        // Add feature-specific configuration
        if selected_features.contains("metrics") {
            config_sections.push("## Metrics Configuration\n\n```yaml\nmetrics:\n  enabled: true\n  endpoint: \"/metrics\"\n  port: 8081\n```\n".to_string());
        }

        if selected_features.contains("auth") {
            config_sections.push("## Authentication Configuration\n\n```yaml\nauth:\n  enabled: true\n  jwt_secret: \"<your-secret-here>\"\n  token_expiry: 3600\n```\n".to_string());
        }

        if selected_features.contains("caching") {
            config_sections.push("## Caching Configuration\n\n```yaml\ncaching:\n  enabled: true\n  redis_url: \"redis://localhost:6379\"\n  default_ttl: 300\n```\n".to_string());
        }

        // Write the configuration example
        let content = config_sections.join("\n");
        fs::write(config_dir.join(format!("{}.md", environment)), content).map_err(|e| {
            FeatureError::IoError(format!("Failed to write {} config: {}", environment, e))
        })?;

        println!("Generated {} configuration example", environment);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// Setup function to create test resources
    fn setup() -> (TempDir, TempDir, FeatureRegistry) {
        let output_dir = TempDir::new().unwrap();
        let template_dir = TempDir::new().unwrap();

        // Create feature registry
        let mut registry = FeatureRegistry::new();
        registry.select("core").unwrap();
        registry.select("metrics").unwrap();

        (output_dir, template_dir, registry)
    }

    #[test]
    fn test_doc_generator_creation() {
        let (output_dir, template_dir, registry) = setup();

        let config = DocConfig {
            output_dir: output_dir.path().to_path_buf(),
            template_dir: template_dir.path().to_path_buf(),
            version: "0.1.0".to_string(),
            generate_api_reference: true,
            generate_config_examples: true,
            generate_feature_docs: true,
        };

        let result = DocGenerator::new(registry, config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_extract_frontmatter() {
        let (output_dir, template_dir, registry) = setup();

        let config = DocConfig {
            output_dir: output_dir.path().to_path_buf(),
            template_dir: template_dir.path().to_path_buf(),
            version: "0.1.0".to_string(),
            generate_api_reference: true,
            generate_config_examples: true,
            generate_feature_docs: true,
        };

        let generator = DocGenerator::new(registry, config).unwrap();

        let content = r#"---
title: Test
version: 1.0
---
# Content
This is the content.
"#;

        let result = generator.extract_frontmatter(content).unwrap();
        assert_eq!(result.0, "title: Test\nversion: 1.0");
        assert_eq!(result.1, "# Content\nThis is the content.");
    }

    #[test]
    fn test_template_rendering() {
        let (output_dir, template_dir, registry) = setup();

        let config = DocConfig {
            output_dir: output_dir.path().to_path_buf(),
            template_dir: template_dir.path().to_path_buf(),
            version: "0.1.0".to_string(),
            generate_api_reference: true,
            generate_config_examples: true,
            generate_feature_docs: true,
        };

        let generator = DocGenerator::new(registry, config).unwrap();

        let template = "Hello {{name}}! {{#if condition}}Condition is true.{{/if}}";
        let mut context = HashMap::new();
        context.insert("name".to_string(), "World".to_string());
        context.insert("condition".to_string(), "true".to_string());

        let result = generator.render_template(template, &context).unwrap();
        assert_eq!(result, "Hello World! Condition is true.");
    }
}
