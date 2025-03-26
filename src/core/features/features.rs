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

        // Register core features (always enabled)
        registry.register_core_features();

        // Register optional features
        registry.register_optional_features();

        // Select default features
        registry.select_defaults();

        registry
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
            size_impact: 500,
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

        // Select each default feature
        for name in default_features {
            // Ignore errors here since we're only selecting defaults
            let _ = self.select(&name);
        }
    }

    /// Register a feature
    pub fn register(&mut self, feature: FeatureInfo) {
        self.features.insert(feature.name.clone(), feature);
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

        // Add to enabled set
        self.enabled_features.insert(name.to_string());

        // Enable dependencies
        let dependencies = {
            if let Some(feature) = self.features.get(name) {
                feature.dependencies.clone()
            } else {
                vec![]
            }
        };

        for dep in dependencies {
            self.enable_feature(&dep)?;
        }

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
}

/// Extension trait for FeatureRegistry that provides convenient methods for feature management
pub trait FeatureRegistryExt {
    /// Get a list of all available features
    fn features(&self) -> Vec<&FeatureInfo>;

    /// Get the number of registered features
    fn feature_count(&self) -> usize;

    /// Get a list of all enabled features
    fn enabled_features(&self) -> Vec<String>;

    /// Get information about a specific feature
    fn get_feature_info(&self, name: &str) -> Option<&FeatureInfo>;

    /// Check if a feature is enabled
    fn is_enabled(&self, name: &str) -> bool;

    /// Enable a feature
    fn enable(&mut self, name: &str) -> Result<(), FeatureError>;

    /// Disable a feature
    fn disable(&mut self, name: &str) -> Result<(), FeatureError>;
}

impl FeatureRegistryExt for FeatureRegistry {
    fn features(&self) -> Vec<&FeatureInfo> {
        self.feature_list()
    }

    fn feature_count(&self) -> usize {
        self.feature_list().len()
    }

    fn enabled_features(&self) -> Vec<String> {
        self.get_enabled_features().iter().cloned().collect()
    }

    fn get_feature_info(&self, name: &str) -> Option<&FeatureInfo> {
        self.get_feature(name)
    }

    fn is_enabled(&self, name: &str) -> bool {
        self.feature_is_enabled(name)
    }

    fn enable(&mut self, name: &str) -> Result<(), FeatureError> {
        self.enable_feature(name)
    }

    fn disable(&mut self, name: &str) -> Result<(), FeatureError> {
        self.disable_feature(name)
    }
}
