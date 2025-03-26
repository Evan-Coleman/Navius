//! Documentation generator for the feature system
//!
//! Provides tools for generating documentation based on enabled features.

use crate::core::features::{FeatureError, FeatureInfo, FeatureRegistry};
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

        // Create features directory
        let features_dir = self.config.output_dir.join("features");
        if !features_dir.exists() {
            fs::create_dir_all(&features_dir).map_err(|e| {
                FeatureError::IoError(format!("Failed to create features directory: {}", e))
            })?;
        }

        // Generate a list of all features with documentation
        let mut feature_list = Vec::new();

        // Get selected features
        let selected_features = self.registry.get_selected();

        // Process each template
        for template in self.templates.values() {
            // Check if all required features are enabled
            if !template.required_features.is_empty() {
                let all_required = template
                    .required_features
                    .iter()
                    .all(|f| selected_features.contains(f));

                if !all_required {
                    // Skip this template if not all required features are enabled
                    continue;
                }
            }

            // If the template is for a feature, add it to the feature docs
            if template.name.starts_with("features/") || template.name.contains("feature") {
                // Create context for template rendering
                let context = self.create_feature_context(&selected_features);

                // Render the template
                let content = self.render_template(&template.content, &context)?;

                // Get just the feature name from the template name
                let feature_name = template.name.split('/').last().unwrap_or(&template.name);

                // Write the feature documentation
                let output_path = features_dir.join(format!("{}.md", feature_name));
                fs::write(&output_path, content).map_err(|e| {
                    FeatureError::IoError(format!("Failed to write feature documentation: {}", e))
                })?;

                println!("Generated feature documentation: {}", feature_name);

                // Add to feature list
                feature_list.push(format!(
                    "- [{}](features/{}.md)",
                    feature_name, feature_name
                ));
            }
        }

        // Generate features index
        let features_index = format!(
            "# Feature Documentation\n\nThis documentation covers the following enabled features:\n\n{}\n",
            feature_list.join("\n")
        );

        fs::write(features_dir.join("index.md"), features_index)
            .map_err(|e| FeatureError::IoError(format!("Failed to write features index: {}", e)))?;

        println!("Generated features index");

        Ok(())
    }

    /// Generate API reference documentation
    fn generate_api_reference(&self) -> Result<(), FeatureError> {
        println!("Generating API reference documentation...");

        // Create API reference directory
        let api_dir = self.config.output_dir.join("api");
        if !api_dir.exists() {
            fs::create_dir_all(&api_dir).map_err(|e| {
                FeatureError::IoError(format!("Failed to create API directory: {}", e))
            })?;
        }

        // Generate API index with enabled features
        let selected_features = self.registry.get_selected();
        let mut api_endpoints = Vec::new();

        // Define API endpoints for each feature
        // In a real implementation, this would read from API documentation
        // or generate it from code analysis
        if selected_features.contains("metrics") {
            api_endpoints
                .push("## Metrics API\n\n- `GET /metrics` - Retrieve metrics in Prometheus format");
        }

        if selected_features.contains("health") {
            api_endpoints.push("## Health API\n\n- `GET /health` - Retrieve health status");
        }

        if selected_features.contains("auth") {
            api_endpoints.push("## Authentication API\n\n- `POST /auth/login` - Authenticate user\n- `POST /auth/refresh` - Refresh authentication token");
        }

        // Generate the API index
        let api_index = format!(
            "# API Reference\n\nThis API reference includes endpoints for enabled features only.\n\n{}\n",
            api_endpoints.join("\n\n")
        );

        fs::write(api_dir.join("index.md"), api_index)
            .map_err(|e| FeatureError::IoError(format!("Failed to write API index: {}", e)))?;

        println!("Generated API reference");

        Ok(())
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
