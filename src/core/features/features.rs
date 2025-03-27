use crate::core::features::dependency_analyzer::DependencyAnalyzer;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use thiserror::Error;

/// Feature information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureInfo {
    /// Feature name
    pub name: String,

    /// Feature description
    pub description: String,

    /// Dependencies (other features this one requires)
    pub dependencies: Vec<String>,

    /// Whether this feature is enabled by default
    pub default_enabled: bool,

    /// Category for grouping
    pub category: String,

    /// Tags for filtering
    pub tags: Vec<String>,

    /// Size impact in KB (approximate)
    pub size_impact: usize,
}

/// Feature error types
#[derive(Debug, Error)]
pub enum FeatureError {
    #[error("Unknown feature: {0}")]
    UnknownFeature(String),

    #[error("Feature {0} is required by {1}")]
    DependencyRequired(String, String),

    #[error("Missing dependency: {0} requires {1}")]
    MissingDependency(String, String),

    #[error("Circular dependency detected")]
    CircularDependency,

    #[error("IO error: {0}")]
    IoError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    #[error("Build error: {0}")]
    BuildError(String),

    #[error("Optimization error: {0}")]
    OptimizationError(String),
}

/// Feature registry
#[derive(Clone)]
pub struct FeatureRegistry {
    /// Available features
    pub(crate) features: HashMap<String, FeatureInfo>,

    /// Feature groups
    pub(crate) groups: HashMap<String, Vec<String>>,

    /// Current selection
    pub(crate) selected: HashSet<String>,

    /// Enabled features
    pub(crate) enabled_features: HashSet<String>,
}

impl FeatureRegistry {
    /// Create a new registry with default features
    pub fn new() -> Self {
        let mut registry = Self {
            features: HashMap::new(),
            groups: HashMap::new(),
            selected: HashSet::new(),
            enabled_features: HashSet::new(),
        };

        registry.register_default_features();
        registry
    }

    /// Register default features
    pub fn register_default_features(&mut self) {
        self.register_core_features();
        self.register_optional_features();
        self.select_defaults();
    }

    /// Create a new empty registry without any features
    pub fn new_empty() -> Self {
        Self {
            features: HashMap::new(),
            groups: HashMap::new(),
            selected: HashSet::new(),
            enabled_features: HashSet::new(),
        }
    }

    /// Register core features (always enabled)
    fn register_core_features(&mut self) {
        // Core server feature (always required)
        let core = FeatureInfo {
            name: "core".to_string(),
            description: "Core server functionality".to_string(),
            dependencies: Vec::new(),
            default_enabled: true,
            category: "Essential".to_string(),
            tags: vec!["core".to_string(), "required".to_string()],
            size_impact: 100,
        };

        // Error handling
        let error_handling = FeatureInfo {
            name: "error_handling".to_string(),
            description: "Error handling and reporting".to_string(),
            dependencies: vec!["core".to_string()],
            default_enabled: true,
            category: "Essential".to_string(),
            tags: vec!["core".to_string(), "required".to_string()],
            size_impact: 100,
        };

        // Configuration
        let config = FeatureInfo {
            name: "config".to_string(),
            description: "Configuration system".to_string(),
            dependencies: vec!["core".to_string()],
            default_enabled: true,
            category: "Essential".to_string(),
            tags: vec!["core".to_string(), "required".to_string()],
            size_impact: 150,
        };

        self.register(core);
        self.register(error_handling);
        self.register(config);

        // Add to core group
        self.groups.insert(
            "core".to_string(),
            vec![
                "core".to_string(),
                "error_handling".to_string(),
                "config".to_string(),
            ],
        );
    }

    /// Register optional features
    fn register_optional_features(&mut self) {
        // Metrics
        let metrics = FeatureInfo {
            name: "metrics".to_string(),
            description: "Metrics collection and reporting".to_string(),
            dependencies: vec!["core".to_string()],
            default_enabled: true,
            category: "Observability".to_string(),
            tags: vec!["monitoring".to_string()],
            size_impact: 250,
        };

        // Advanced metrics
        let advanced_metrics = FeatureInfo {
            name: "advanced_metrics".to_string(),
            description: "Advanced metrics and custom reporters".to_string(),
            dependencies: vec!["metrics".to_string()],
            default_enabled: false,
            category: "Observability".to_string(),
            tags: vec!["monitoring".to_string(), "advanced".to_string()],
            size_impact: 350,
        };

        // Authentication
        let auth = FeatureInfo {
            name: "auth".to_string(),
            description: "Authentication system".to_string(),
            dependencies: vec!["core".to_string()],
            default_enabled: true,
            category: "Security".to_string(),
            tags: vec!["security".to_string()],
            size_impact: 300,
        };

        // Caching
        let caching = FeatureInfo {
            name: "caching".to_string(),
            description: "Caching system for improved performance".to_string(),
            dependencies: vec!["core".to_string()],
            default_enabled: true,
            category: "Performance".to_string(),
            tags: vec!["performance".to_string()],
            size_impact: 200,
        };

        // Reliability
        let reliability = FeatureInfo {
            name: "reliability".to_string(),
            description: "Reliability features like retry, circuit breaking".to_string(),
            dependencies: vec!["core".to_string()],
            default_enabled: true,
            category: "Resilience".to_string(),
            tags: vec!["resilience".to_string()],
            size_impact: 400,
        };

        self.register(metrics);
        self.register(advanced_metrics);
        self.register(auth);
        self.register(caching);
        self.register(reliability);

        // Add to groups
        self.groups.insert(
            "observability".to_string(),
            vec!["metrics".to_string(), "advanced_metrics".to_string()],
        );

        self.groups
            .insert("security".to_string(), vec!["auth".to_string()]);

        self.groups
            .insert("performance".to_string(), vec!["caching".to_string()]);

        self.groups
            .insert("resilience".to_string(), vec!["reliability".to_string()]);
    }

    /// Select default features
    fn select_defaults(&mut self) {
        // Collect feature names that should be enabled by default
        let default_features: Vec<String> = self
            .features
            .iter()
            .filter_map(|(name, feature)| {
                if feature.default_enabled {
                    Some(name.clone())
                } else {
                    None
                }
            })
            .collect();

        // Select and enable each default feature
        for name in default_features {
            // Ignore errors here since we're only selecting defaults
            let _ = self.select(&name);
            let _ = self.enable_feature(&name);
        }
    }

    /// Register a feature
    pub fn register(&mut self, feature: FeatureInfo) {
        self.features.insert(feature.name.clone(), feature.clone());

        // If the feature is default_enabled, enable it and its dependencies
        if feature.default_enabled {
            // Enable dependencies first
            for dep in &feature.dependencies {
                let _ = self.enable_feature(dep);
            }
            // Then enable the feature itself
            let _ = self.enable_feature(&feature.name);
        }
    }

    /// Select a feature and its dependencies
    pub fn select(&mut self, name: &str) -> Result<(), FeatureError> {
        if !self.features.contains_key(name) {
            return Err(FeatureError::UnknownFeature(name.to_string()));
        }

        // Add the feature to selected set
        self.selected.insert(name.to_string());

        // Collect dependencies to avoid borrowing issues
        let dependencies = if let Some(feature) = self.features.get(name) {
            feature.dependencies.clone()
        } else {
            Vec::new()
        };

        // Add dependencies
        for dep in dependencies {
            self.select(&dep)?;
        }

        Ok(())
    }

    /// Deselect a feature if no other selected features depend on it
    pub fn deselect(&mut self, name: &str) -> Result<(), FeatureError> {
        if !self.features.contains_key(name) {
            return Err(FeatureError::UnknownFeature(name.to_string()));
        }

        // Check if any other selected feature depends on this one
        for (feature_name, feature) in &self.features {
            if self.is_selected(feature_name) && feature.dependencies.contains(&name.to_string()) {
                return Err(FeatureError::DependencyRequired(
                    name.to_string(),
                    feature_name.to_string(),
                ));
            }
        }

        // Remove from selected set
        self.selected.remove(name);

        Ok(())
    }

    /// Check if a feature is selected
    pub fn is_selected(&self, name: &str) -> bool {
        self.selected.contains(name)
    }

    /// Get all selected features
    pub fn get_selected(&self) -> HashSet<String> {
        self.selected.clone()
    }

    /// Get all feature categories
    pub fn get_categories(&self) -> Vec<String> {
        let mut categories = self
            .features
            .values()
            .map(|f| f.category.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();

        categories.sort();
        categories
    }

    /// Get features by category
    pub fn get_features_by_category(&self, category: &str) -> Vec<&FeatureInfo> {
        let mut features = self
            .features
            .values()
            .filter(|f| f.category == category)
            .collect::<Vec<_>>();

        features.sort_by(|a, b| a.name.cmp(&b.name));
        features
    }

    /// Validate feature selection
    pub fn validate(&self) -> Result<(), FeatureError> {
        // Check that all dependencies are satisfied
        for name in &self.selected {
            if let Some(feature) = self.features.get(name) {
                for dep in &feature.dependencies {
                    if !self.selected.contains(dep) {
                        return Err(FeatureError::MissingDependency(
                            name.to_string(),
                            dep.to_string(),
                        ));
                    }
                }
            }
        }

        Ok(())
    }

    /// Detect circular dependencies
    fn detect_circular_dependencies(&self) -> Result<(), FeatureError> {
        // Implementation of cycle detection in dependency graph
        // For each feature, do a depth-first traversal of its dependencies
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for name in self.features.keys() {
            if self.has_circular_dependency(name, &mut visited, &mut rec_stack)? {
                return Err(FeatureError::CircularDependency);
            }
        }

        Ok(())
    }

    fn has_circular_dependency(
        &self,
        name: &str,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
    ) -> Result<bool, FeatureError> {
        if !visited.contains(name) {
            // Mark current node as visited and add to recursion stack
            visited.insert(name.to_string());
            rec_stack.insert(name.to_string());

            // Recur for all dependencies
            if let Some(feature) = self.features.get(name) {
                for dep in &feature.dependencies {
                    if !visited.contains(dep) {
                        if self.has_circular_dependency(dep, visited, rec_stack)? {
                            return Ok(true);
                        }
                    } else if rec_stack.contains(dep) {
                        // If the dependency is in recursion stack, there's a cycle
                        return Ok(true);
                    }
                }
            } else {
                return Err(FeatureError::UnknownFeature(name.to_string()));
            }
        }

        // Remove from recursion stack
        rec_stack.remove(name);

        Ok(false)
    }

    /// Get feature information by name
    pub fn get_feature_info(&self, name: &str) -> Option<&FeatureInfo> {
        self.features.get(name)
    }

    /// Get a list of all registered features
    pub fn feature_list(&self) -> Vec<&FeatureInfo> {
        self.features.values().collect()
    }

    /// Get the enabled features
    pub fn get_enabled_features(&self) -> &HashSet<String> {
        &self.enabled_features
    }

    /// Get information about a specific feature
    pub fn get_feature(&self, name: &str) -> Option<&FeatureInfo> {
        self.features.get(name)
    }

    /// Check if a feature is enabled
    pub fn feature_is_enabled(&self, name: &str) -> bool {
        self.enabled_features.contains(name)
    }

    /// Enable a feature and its dependencies
    pub fn enable_feature(&mut self, name: &str) -> Result<(), FeatureError> {
        // Check feature exists
        if !self.features.contains_key(name) {
            return Err(FeatureError::UnknownFeature(name.to_string()));
        }

        // Get dependencies
        let dependencies = {
            if let Some(feature) = self.features.get(name) {
                feature.dependencies.clone()
            } else {
                vec![]
            }
        };

        // Check if all dependencies are enabled
        for dep in &dependencies {
            if !self.enabled_features.contains(dep) {
                return Err(FeatureError::MissingDependency(
                    name.to_string(),
                    dep.to_string(),
                ));
            }
        }

        // Add to enabled set
        self.enabled_features.insert(name.to_string());

        Ok(())
    }

    /// Disable a feature if no other features depend on it
    pub fn disable_feature(&mut self, name: &str) -> Result<(), FeatureError> {
        // Check feature exists
        if !self.features.contains_key(name) {
            return Err(FeatureError::UnknownFeature(name.to_string()));
        }

        // Check if any enabled feature depends on this one
        for (feature_name, feature) in &self.features {
            if self.enabled_features.contains(feature_name)
                && feature.dependencies.contains(&name.to_string())
            {
                return Err(FeatureError::DependencyRequired(
                    name.to_string(),
                    feature_name.to_string(),
                ));
            }
        }

        // Remove from enabled set
        self.enabled_features.remove(name);

        Ok(())
    }

    /// Format feature list in the specified format
    pub fn format_feature_list(&self, format: &str) -> String {
        match format {
            "json" => {
                let mut output = String::from("{\n  \"features\": [\n");
                let mut features: Vec<_> = self.features.values().collect();
                features.sort_by(|a, b| a.name.cmp(&b.name));

                for (i, feature) in features.iter().enumerate() {
                    output.push_str("    {\n");
                    output.push_str(&format!("      \"name\": \"{}\",\n", feature.name));
                    output.push_str(&format!(
                        "      \"description\": \"{}\",\n",
                        feature.description
                    ));
                    output.push_str(&format!(
                        "      \"enabled\": {},\n",
                        self.feature_is_enabled(&feature.name)
                    ));
                    output.push_str(&format!(
                        "      \"dependencies\": {:?},\n",
                        feature.dependencies
                    ));
                    output.push_str(&format!("      \"category\": \"{}\",\n", feature.category));
                    output.push_str(&format!("      \"tags\": {:?},\n", feature.tags));
                    output.push_str(&format!("      \"size_impact\": {}\n", feature.size_impact));
                    if i < features.len() - 1 {
                        output.push_str("    },\n");
                    } else {
                        output.push_str("    }\n");
                    }
                }
                output.push_str("  ]\n}");
                output
            }
            "yaml" => {
                let mut output = String::from("features:\n");
                let mut features: Vec<_> = self.features.values().collect();
                features.sort_by(|a, b| a.name.cmp(&b.name));

                for feature in features {
                    output.push_str(&format!("  - name: {}\n", feature.name));
                    output.push_str(&format!("    description: {}\n", feature.description));
                    output.push_str(&format!(
                        "    enabled: {}\n",
                        self.feature_is_enabled(&feature.name)
                    ));
                    output.push_str("    dependencies:\n");
                    for dep in &feature.dependencies {
                        output.push_str(&format!("      - {}\n", dep));
                    }
                    output.push_str(&format!("    category: {}\n", feature.category));
                    output.push_str("    tags:\n");
                    for tag in &feature.tags {
                        output.push_str(&format!("      - {}\n", tag));
                    }
                    output.push_str(&format!("    size_impact: {}\n", feature.size_impact));
                }
                output
            }
            _ => {
                let mut output = String::new();

                // Group features by category
                let mut features_by_category: HashMap<String, Vec<&FeatureInfo>> = HashMap::new();
                for feature in self.features.values() {
                    features_by_category
                        .entry(feature.category.clone())
                        .or_default()
                        .push(feature);
                }

                // Sort categories
                let mut categories: Vec<_> = features_by_category.keys().collect();
                categories.sort();

                for category in categories {
                    output.push_str(&format!("\n[{}]\n", category));
                    if let Some(features) = features_by_category.get(category) {
                        let mut sorted_features = features.to_vec();
                        sorted_features.sort_by(|a, b| a.name.cmp(&b.name));

                        for feature in sorted_features {
                            let status = if self.feature_is_enabled(&feature.name) {
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
                                output
                                    .push_str(&format!("    Tags: {}\n", feature.tags.join(", ")));
                            }
                            output.push_str(&format!(
                                "    Size Impact: {} KB\n",
                                feature.size_impact
                            ));
                        }
                    }
                }
                output
            }
        }
    }

    /// Build interactive menu items
    pub fn build_interactive_menu(&self) -> Vec<String> {
        vec![
            "Select Features (Interactive)".to_string(),
            "Show Feature Status".to_string(),
            "Apply Configuration".to_string(),
            "Exit".to_string(),
        ]
    }

    /// Format feature status for display
    pub fn format_feature_status(&self) -> String {
        let mut output = String::from("Feature Status:\n");
        let mut features: Vec<_> = self.features.values().collect();
        features.sort_by(|a, b| a.name.cmp(&b.name));

        for feature in features {
            let status = if self.feature_is_enabled(&feature.name) {
                "✓"
            } else {
                "✗"
            };
            output.push_str(&format!(
                "{} {} - {}\n",
                status, feature.name, feature.description
            ));
        }
        output
    }

    /// Format feature selection display
    pub fn format_feature_selection(&self) -> String {
        let mut output = String::from("Available Features:\n");
        let mut features: Vec<_> = self.features.values().collect();
        features.sort_by(|a, b| a.name.cmp(&b.name));

        for feature in features {
            let status = if self.feature_is_enabled(&feature.name) {
                "[✓]"
            } else {
                "[ ]"
            };
            let required = if feature.default_enabled {
                " (required)"
            } else {
                ""
            };
            output.push_str(&format!(
                "{} {}{} - {}\n",
                status, feature.name, required, feature.description
            ));
            if !feature.dependencies.is_empty() {
                output.push_str(&format!(
                    "    Dependencies: {}\n",
                    feature.dependencies.join(", ")
                ));
            }
        }
        output
    }

    /// Visualize dependencies for selected features
    pub fn visualize_dependencies(&self, selected: &[String]) -> String {
        let mut output = String::from("Dependency Tree:\n");

        for feature_name in selected {
            if let Some(feature) = self.features.get(feature_name) {
                output.push_str(&format!("{}\n", feature.name));
                self.visualize_feature_dependencies(&feature.name, &mut output, "  ");
            }
        }
        output
    }

    /// Helper function to recursively visualize dependencies
    fn visualize_feature_dependencies(
        &self,
        feature_name: &str,
        output: &mut String,
        indent: &str,
    ) {
        if let Some(feature) = self.features.get(feature_name) {
            for (i, dep) in feature.dependencies.iter().enumerate() {
                let is_last = i == feature.dependencies.len() - 1;
                let prefix = if is_last { "└── " } else { "├── " };
                output.push_str(&format!("{}{}{}\n", indent, prefix, dep));

                let new_indent = if is_last {
                    format!("{}    ", indent)
                } else {
                    format!("{}│   ", indent)
                };
                self.visualize_feature_dependencies(dep, output, &new_indent);
            }
        }
    }

    /// Visualize dependency graph in DOT format
    pub fn visualize_dependency_graph(&self) -> String {
        let mut output = String::from("digraph {\n");

        // Add nodes
        for feature in self.features.values() {
            output.push_str(&format!(
                "  \"{}\" [label=\"{}\"]\n",
                feature.name, feature.name
            ));

            // Add edges for dependencies
            for dep in &feature.dependencies {
                output.push_str(&format!("  \"{}\" -> \"{}\"\n", feature.name, dep));
            }
        }

        output.push_str("}\n");
        output
    }

    /// Visualize size impact of features
    pub fn visualize_size_impact(&self) -> String {
        let mut output = String::from("Size Impact:\n");
        let mut features: Vec<_> = self.features.values().collect();
        features.sort_by_key(|f| f.size_impact);

        let total_size: usize = features.iter().map(|f| f.size_impact).sum();

        for feature in features {
            let percentage = (feature.size_impact as f64 / total_size as f64) * 100.0;
            let bar_length = (percentage / 2.0).round() as usize;
            let bar = "█".repeat(bar_length);

            output.push_str(&format!(
                "{}: {} KB {:<50} ({:.1}%)\n",
                feature.name, feature.size_impact, bar, percentage
            ));
        }

        output.push_str(&format!("\nTotal Size: {} KB\n", total_size));
        output
    }

    /// Generate build command with feature flags
    pub fn generate_build_command(&self, release: bool, target: Option<&str>) -> Vec<String> {
        let mut cmd = vec!["cargo".to_string(), "build".to_string()];

        if release {
            cmd.push("--release".to_string());
        }

        if let Some(target_triple) = target {
            cmd.push("--target".to_string());
            cmd.push(target_triple.to_string());
        }

        // Add enabled features
        let enabled_features: Vec<_> = self.enabled_features.iter().cloned().collect();
        if !enabled_features.is_empty() {
            cmd.push("--features".to_string());
            cmd.push(enabled_features.join(","));
        }

        cmd
    }
}

/// Feature registry extension methods for easier access
pub trait FeatureRegistryExt {
    /// Get all available features
    fn features(&self) -> Vec<&FeatureInfo>;

    /// Get the total number of features
    fn feature_count(&self) -> usize;

    /// Get set of enabled features
    fn enabled_features(&self) -> &std::collections::HashSet<String>;

    /// Get feature info by name
    fn get_feature_info(&self, name: &str) -> Option<&FeatureInfo>;

    /// Check if a feature is enabled
    fn is_enabled(&self, name: &str) -> bool;

    /// Enable a feature and its dependencies
    fn enable(&mut self, name: &str) -> Result<(), FeatureError>;

    /// Disable a feature if no other features depend on it
    fn disable(&mut self, name: &str) -> Result<(), FeatureError>;

    /// Calculate the total size impact of enabled features
    fn calculate_size_impact(&self) -> usize;

    /// Export the current configuration as a string
    fn export_configuration(&self) -> Result<String, FeatureError>;

    /// Import configuration from a string
    fn import_configuration(&mut self, config: &str) -> Result<(), FeatureError>;

    /// Get feature status as a formatted string
    fn get_feature_status(&self) -> String;

    /// Get features organized by category
    fn get_feature_groups(&self) -> HashMap<String, Vec<String>>;

    /// Analyze dependencies for a set of features
    fn analyze_dependencies(
        &self,
        selected: &HashSet<String>,
    ) -> Result<DependencyAnalyzer, FeatureError>;
}

impl FeatureRegistryExt for FeatureRegistry {
    fn features(&self) -> Vec<&FeatureInfo> {
        self.feature_list()
    }

    fn feature_count(&self) -> usize {
        self.feature_list().len()
    }

    fn enabled_features(&self) -> &HashSet<String> {
        self.get_enabled_features()
    }

    fn get_feature_info(&self, name: &str) -> Option<&FeatureInfo> {
        self.get_feature(name)
    }

    fn is_enabled(&self, name: &str) -> bool {
        self.feature_is_enabled(name)
    }

    fn enable(&mut self, name: &str) -> Result<(), FeatureError> {
        // Check feature exists
        if !self.features.contains_key(name) {
            return Err(FeatureError::UnknownFeature(name.to_string()));
        }

        // Get dependencies
        let dependencies = {
            if let Some(feature) = self.features.get(name) {
                feature.dependencies.clone()
            } else {
                vec![]
            }
        };

        // Check if all dependencies are enabled
        for dep in &dependencies {
            if !self.enabled_features.contains(dep) {
                return Err(FeatureError::MissingDependency(
                    name.to_string(),
                    dep.to_string(),
                ));
            }
        }

        // Add to enabled set
        self.enabled_features.insert(name.to_string());

        Ok(())
    }

    fn disable(&mut self, name: &str) -> Result<(), FeatureError> {
        self.disable_feature(name)
    }

    fn calculate_size_impact(&self) -> usize {
        self.enabled_features
            .iter()
            .filter_map(|name| self.features.get(name))
            .map(|feature| feature.size_impact)
            .sum()
    }

    fn export_configuration(&self) -> Result<String, FeatureError> {
        let mut config = HashMap::new();
        for feature in self.features.keys() {
            config.insert(feature.clone(), self.is_enabled(feature));
        }
        serde_json::to_string_pretty(&config)
            .map_err(|e| FeatureError::SerializationError(e.to_string()))
    }

    fn import_configuration(&mut self, config: &str) -> Result<(), FeatureError> {
        let config: HashMap<String, bool> = serde_json::from_str(config)
            .map_err(|e| FeatureError::DeserializationError(e.to_string()))?;
        self.enabled_features.clear();

        // Only enable features that are explicitly enabled in the config
        for (feature_name, is_enabled) in config {
            if is_enabled {
                self.enable(&feature_name)?;
            }
        }
        Ok(())
    }

    fn get_feature_status(&self) -> String {
        let mut status = String::new();
        for (name, feature) in &self.features {
            let enabled = if self.enabled_features.contains(name) {
                "✅"
            } else {
                "❌"
            };
            status.push_str(&format!(
                "{} {} ({} KB) - {}\n",
                enabled, name, feature.size_impact, feature.description
            ));
        }
        status
    }

    fn get_feature_groups(&self) -> HashMap<String, Vec<String>> {
        let mut groups: HashMap<String, Vec<String>> = HashMap::new();

        for (name, feature) in &self.features {
            groups
                .entry(feature.category.clone())
                .or_default()
                .push(name.clone());
        }

        groups
    }

    fn analyze_dependencies(
        &self,
        selected: &HashSet<String>,
    ) -> Result<DependencyAnalyzer, FeatureError> {
        // Get the Cargo.toml path
        let cargo_path = std::env::current_dir()
            .map_err(|e| FeatureError::IoError(format!("Failed to get current directory: {}", e)))?
            .join("Cargo.toml");

        DependencyAnalyzer::new_with_registry(cargo_path, selected.clone(), self.clone())
    }
}
