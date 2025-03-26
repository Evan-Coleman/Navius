use crate::core::features::{FeatureError, FeatureInfo, FeatureRegistry};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

/// Cargo dependency analyzer
pub struct DependencyAnalyzer {
    /// Path to Cargo.toml
    cargo_path: PathBuf,

    /// Selected features
    selected_features: HashSet<String>,

    /// Feature-to-dependency mapping
    feature_dependencies: HashMap<String, HashSet<String>>,

    /// Required dependencies based on feature selection
    required_dependencies: HashSet<String>,

    /// Optional dependencies that can be removed
    removable_dependencies: HashSet<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CargoManifest {
    package: Option<CargoPackage>,
    dependencies: Option<HashMap<String, CargoDependency>>,
    features: Option<HashMap<String, Vec<String>>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CargoPackage {
    name: String,
    version: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum CargoDependency {
    Simple(String),
    Detailed(DetailedDependency),
}

#[derive(Debug, Serialize, Deserialize)]
struct DetailedDependency {
    version: Option<String>,
    features: Option<Vec<String>>,
    optional: Option<bool>,
}

impl DependencyAnalyzer {
    /// Create a new dependency analyzer
    pub fn new(
        cargo_path: PathBuf,
        selected_features: HashSet<String>,
    ) -> Result<Self, FeatureError> {
        let mut analyzer = Self {
            cargo_path,
            selected_features,
            feature_dependencies: HashMap::new(),
            required_dependencies: HashSet::new(),
            removable_dependencies: HashSet::new(),
        };

        // Parse Cargo.toml and analyze dependencies
        analyzer.analyze_dependencies()?;

        Ok(analyzer)
    }

    /// Analyze dependencies in Cargo.toml
    fn analyze_dependencies(&mut self) -> Result<(), FeatureError> {
        // Read Cargo.toml
        let cargo_content = std::fs::read_to_string(&self.cargo_path)
            .map_err(|e| FeatureError::IoError(format!("Failed to read Cargo.toml: {}", e)))?;

        // Parse Cargo.toml
        let cargo_toml: CargoManifest = toml::from_str(&cargo_content).map_err(|e| {
            FeatureError::DeserializationError(format!("Failed to parse Cargo.toml: {}", e))
        })?;

        // Extract dependencies
        self.extract_dependencies(&cargo_toml)?;

        // Map dependencies to features
        self.map_dependencies_to_features(&cargo_toml)?;

        // Identify required dependencies based on selected features
        self.identify_required_dependencies()?;

        // Identify dependencies that can be safely removed
        self.identify_removable_dependencies()?;

        Ok(())
    }

    /// Extract all dependencies from Cargo.toml
    fn extract_dependencies(&mut self, cargo_toml: &CargoManifest) -> Result<(), FeatureError> {
        if let Some(deps) = &cargo_toml.dependencies {
            for (name, _) in deps {
                self.required_dependencies.insert(name.clone());
            }
        }

        Ok(())
    }

    /// Map dependencies to features
    fn map_dependencies_to_features(
        &mut self,
        cargo_toml: &CargoManifest,
    ) -> Result<(), FeatureError> {
        if let Some(features) = &cargo_toml.features {
            for (feature, deps) in features {
                let deps_set: HashSet<String> = deps
                    .iter()
                    .filter_map(|s| {
                        // Handle optional dependencies in format 'crate/feature'
                        if s.contains('/') {
                            s.split('/').next().map(|s| s.to_string())
                        } else {
                            Some(s.to_string())
                        }
                    })
                    .collect();

                self.feature_dependencies.insert(feature.clone(), deps_set);
            }
        }

        Ok(())
    }

    /// Identify required dependencies based on selected features
    fn identify_required_dependencies(&mut self) -> Result<(), FeatureError> {
        let mut required = HashSet::new();

        // Add core dependencies that are always required
        required.insert("tokio".to_string());
        required.insert("axum".to_string());
        required.insert("serde".to_string());

        // Add dependencies based on selected features
        for feature in &self.selected_features {
            if let Some(deps) = self.feature_dependencies.get(feature) {
                for dep in deps {
                    required.insert(dep.clone());
                }
            }
        }

        self.required_dependencies = required;

        Ok(())
    }

    /// Identify dependencies that can be safely removed
    fn identify_removable_dependencies(&mut self) -> Result<(), FeatureError> {
        let mut removable = HashSet::new();

        // Start with all dependencies from feature mappings
        for (_, deps) in &self.feature_dependencies {
            for dep in deps {
                removable.insert(dep.clone());
            }
        }

        // Remove required dependencies
        for dep in &self.required_dependencies {
            removable.remove(dep);
        }

        self.removable_dependencies = removable;

        Ok(())
    }

    /// Generate optimized Cargo.toml
    pub fn generate_optimized_toml(&self) -> Result<String, FeatureError> {
        // Read original Cargo.toml
        let cargo_content = std::fs::read_to_string(&self.cargo_path)
            .map_err(|e| FeatureError::IoError(format!("Failed to read Cargo.toml: {}", e)))?;

        // Parse Cargo.toml
        let mut cargo_toml: CargoManifest = toml::from_str(&cargo_content).map_err(|e| {
            FeatureError::DeserializationError(format!("Failed to parse Cargo.toml: {}", e))
        })?;

        // Optimize dependencies section
        if let Some(deps) = cargo_toml.dependencies.as_mut() {
            // Remove unnecessary dependencies
            deps.retain(|name, _| self.required_dependencies.contains(name));
        }

        // Convert back to TOML string
        let optimized_toml = toml::to_string_pretty(&cargo_toml).map_err(|e| {
            FeatureError::SerializationError(format!("Failed to serialize Cargo.toml: {}", e))
        })?;

        Ok(optimized_toml)
    }

    /// Get required dependencies
    pub fn get_required_dependencies(&self) -> &HashSet<String> {
        &self.required_dependencies
    }

    /// Get removable dependencies
    pub fn get_removable_dependencies(&self) -> &HashSet<String> {
        &self.removable_dependencies
    }

    /// Generate dependency tree visualization
    pub fn generate_dependency_tree(&self) -> String {
        let mut tree = String::new();

        tree.push_str("# Dependency Tree\n\n");

        // Add selected features section
        tree.push_str("## Selected Features\n\n");

        for feature in &self.selected_features {
            tree.push_str(&format!("- {}\n", feature));

            // Add dependencies for this feature
            if let Some(deps) = self.feature_dependencies.get(feature) {
                for dep in deps {
                    let status = if self.required_dependencies.contains(dep) {
                        "✅"
                    } else {
                        "❌"
                    };

                    tree.push_str(&format!("  - {} {}\n", status, dep));
                }
            }
        }

        // Add required dependencies section
        tree.push_str("\n## Required Dependencies\n\n");

        for dep in &self.required_dependencies {
            tree.push_str(&format!("- {}\n", dep));
        }

        // Add removable dependencies section
        tree.push_str("\n## Removable Dependencies\n\n");

        for dep in &self.removable_dependencies {
            tree.push_str(&format!("- {}\n", dep));
        }

        tree
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_cargo_toml() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let cargo_path = temp_dir.path().join("Cargo.toml");

        let cargo_content = r#"
[package]
name = "test-package"
version = "0.1.0"

[dependencies]
tokio = "1.0"
axum = "0.5"
serde = "1.0"
metrics = { version = "0.1", optional = true }
tracing = { version = "0.1", optional = true }

[features]
metrics = ["dep:metrics"]
advanced_metrics = ["metrics", "tracing"]
"#;

        std::fs::write(&cargo_path, cargo_content).unwrap();

        (temp_dir, cargo_path)
    }

    #[test]
    fn test_dependency_analysis() {
        let (temp_dir, cargo_path) = create_test_cargo_toml();

        let mut selected_features = HashSet::new();
        selected_features.insert("metrics".to_string());

        let analyzer = DependencyAnalyzer::new(cargo_path, selected_features).unwrap();

        // It could be a feature name that doesn't directly map to a dependency
        let required = analyzer.get_required_dependencies();
        assert!(required.contains("tokio"));
        assert!(required.contains("axum"));
        assert!(required.contains("serde"));

        // Check removable dependencies
        let removable = analyzer.get_removable_dependencies();
        assert!(removable.contains("tracing"));

        // Keep temp_dir in scope
        drop(temp_dir);
    }
}
