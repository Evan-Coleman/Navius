//! Packaging system for server customization
//!
//! Provides tools for creating optimized server packages based on selected features.

use crate::core::features::{DependencyAnalyzer, FeatureError, FeatureRegistry};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Build configuration for generating optimized server binaries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
    /// Source code path
    pub source_path: PathBuf,

    /// Output path
    pub output_path: PathBuf,

    /// Selected features
    pub features: HashSet<String>,

    /// Optimization level (debug, release, extreme)
    pub optimization_level: String,

    /// Target platform
    pub target: Option<String>,

    /// Additional build flags
    pub additional_flags: Vec<String>,

    /// Version information
    pub version: VersionInfo,

    /// Container configuration
    pub container: Option<ContainerConfig>,

    /// Whether to optimize dependencies
    pub optimize_dependencies: bool,
}

/// Version information for package tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    /// Major version
    pub major: u32,

    /// Minor version
    pub minor: u32,

    /// Patch version
    pub patch: u32,

    /// Build identifier
    pub build: Option<String>,

    /// Git commit hash
    pub commit: Option<String>,
}

/// Container configuration for Docker/OCI images
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerConfig {
    /// Base image to use
    pub base_image: String,

    /// Container tags
    pub tags: Vec<String>,

    /// Environment variables
    pub env_vars: Vec<EnvVar>,

    /// Exposed ports
    pub ports: Vec<u16>,

    /// Container labels
    pub labels: std::collections::HashMap<String, String>,
}

/// Environment variable for container configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvVar {
    /// Environment variable name
    pub name: String,

    /// Environment variable value
    pub value: String,
}

impl BuildConfig {
    /// Create a new build configuration
    pub fn new(source_path: PathBuf, output_path: PathBuf) -> Self {
        Self {
            source_path,
            output_path,
            features: HashSet::new(),
            optimization_level: "release".to_string(),
            target: None,
            additional_flags: Vec::new(),
            version: VersionInfo::default(),
            container: None,
            optimize_dependencies: false,
        }
    }

    /// Add selected features
    pub fn with_features(mut self, features: HashSet<String>) -> Self {
        self.features = features;
        self
    }

    /// Set optimization level
    pub fn with_optimization(mut self, level: &str) -> Self {
        self.optimization_level = level.to_string();
        self
    }

    /// Set target platform
    pub fn with_target(mut self, target: Option<&str>) -> Self {
        self.target = target.map(|s| s.to_string());
        self
    }

    /// Add additional build flags
    pub fn with_flags(mut self, flags: Vec<String>) -> Self {
        self.additional_flags = flags;
        self
    }

    /// Set version information
    pub fn with_version(mut self, version: VersionInfo) -> Self {
        self.version = version;
        self
    }

    /// Set container configuration
    pub fn with_container(mut self, container: ContainerConfig) -> Self {
        self.container = Some(container);
        self
    }

    /// Enable dependency optimization
    pub fn with_dependency_optimization(mut self, optimize: bool) -> Self {
        self.optimize_dependencies = optimize;
        self
    }

    /// Generate Cargo.toml with selected features
    pub fn generate_cargo_toml(&self) -> Result<(), FeatureError> {
        if self.optimize_dependencies {
            // Create dependency analyzer
            let analyzer = DependencyAnalyzer::new(
                self.source_path.join("Cargo.toml"),
                self.features.clone(),
            )?;

            // Generate optimized Cargo.toml
            let optimized_toml = analyzer.generate_optimized_toml()?;

            // Create optimized build directory
            let build_dir = self.output_path.join("optimized_build");
            std::fs::create_dir_all(&build_dir).map_err(|e| {
                FeatureError::IoError(format!("Failed to create build directory: {}", e))
            })?;

            // Write optimized Cargo.toml
            std::fs::write(build_dir.join("Cargo.toml"), optimized_toml).map_err(|e| {
                FeatureError::IoError(format!("Failed to write optimized Cargo.toml: {}", e))
            })?;

            // Generate dependency tree visualization
            let tree = analyzer.generate_dependency_tree();
            std::fs::write(build_dir.join("dependency_tree.md"), tree).map_err(|e| {
                FeatureError::IoError(format!("Failed to write dependency tree: {}", e))
            })?;
        }

        Ok(())
    }

    /// Generate build command
    pub fn generate_build_command(&self) -> Vec<String> {
        let mut cmd = vec!["cargo".to_string(), "build".to_string()];

        // Add optimization
        if self.optimization_level == "release" {
            cmd.push("--release".to_string());
        }

        // Add target if specified
        if let Some(target) = &self.target {
            cmd.push("--target".to_string());
            cmd.push(target.clone());
        }

        // Add features
        if !self.features.is_empty() {
            let features_str = self.features.iter().cloned().collect::<Vec<_>>().join(",");
            cmd.push("--features".to_string());
            cmd.push(features_str);
        }

        // Add additional flags
        cmd.extend(self.additional_flags.clone());

        cmd
    }

    /// Execute build
    pub fn execute_build(&self) -> Result<PathBuf, FeatureError> {
        // Generate optimized Cargo.toml if requested
        self.generate_cargo_toml()?;

        // Get build command
        let build_cmd = self.generate_build_command();

        // Execute build command
        let status = std::process::Command::new(&build_cmd[0])
            .args(&build_cmd[1..])
            .current_dir(if self.optimize_dependencies {
                self.output_path.join("optimized_build")
            } else {
                self.source_path.clone()
            })
            .status()
            .map_err(|e| {
                FeatureError::BuildError(format!("Failed to execute build command: {}", e))
            })?;

        if !status.success() {
            return Err(FeatureError::BuildError("Build command failed".to_string()));
        }

        // Get the binary path
        self.get_binary_path()
    }

    /// Get the path to the built binary
    fn get_binary_path(&self) -> Result<PathBuf, FeatureError> {
        // Get package name from Cargo.toml or use default
        let binary_name = "navius-server"; // This should be extracted from Cargo.toml

        let binary_path = if self.optimization_level == "release" {
            self.output_path.join("target/release").join(binary_name)
        } else {
            self.output_path.join("target/debug").join(binary_name)
        };

        if !binary_path.exists() {
            return Err(FeatureError::BuildError(format!(
                "Binary not found at expected path: {:?}",
                binary_path
            )));
        }

        Ok(binary_path)
    }

    /// Optimize the binary by stripping debug symbols and applying other optimizations
    fn optimize_binary(&self, binary_path: &Path) -> Result<(), FeatureError> {
        println!("Optimizing binary at: {:?}", binary_path);

        // Strip debug symbols if available
        if cfg!(unix) {
            println!("Stripping debug symbols to reduce binary size...");

            let status = Command::new("strip")
                .arg(binary_path)
                .status()
                .map_err(|e| {
                    FeatureError::OptimizationError(format!("Failed to strip binary: {}", e))
                })?;

            if !status.success() {
                println!(
                    "Warning: Failed to strip debug symbols, continuing with unstripped binary"
                );
            }
        }

        // Apply upx compression if extreme optimization is requested and upx is available
        if self.optimization_level == "extreme" {
            if Command::new("upx").arg("--version").output().is_ok() {
                println!("Applying UPX compression for extreme optimization...");

                let status = Command::new("upx")
                    .arg("--best")
                    .arg(binary_path)
                    .status()
                    .map_err(|e| {
                        FeatureError::OptimizationError(format!("Failed to compress binary: {}", e))
                    })?;

                if !status.success() {
                    println!(
                        "Warning: Failed to apply UPX compression, continuing with uncompressed binary"
                    );
                }
            } else {
                println!("Warning: UPX not found, skipping compression optimization");
            }
        }

        Ok(())
    }

    /// Build container image from the binary
    pub fn build_container(&self, binary_path: &Path) -> Result<String, FeatureError> {
        if let Some(container) = &self.container {
            println!(
                "Building container image with base: {}",
                container.base_image
            );

            // Create a temporary Dockerfile
            let dockerfile_path = self.source_path.join("Dockerfile.tmp");
            self.generate_dockerfile(&dockerfile_path, binary_path, container)?;

            // Build the Docker image
            let tag = if container.tags.is_empty() {
                format!(
                    "navius-server:{}.{}.{}",
                    self.version.major, self.version.minor, self.version.patch
                )
            } else {
                container.tags[0].clone()
            };

            let status = Command::new("docker")
                .arg("build")
                .arg("-t")
                .arg(&tag)
                .arg("-f")
                .arg(&dockerfile_path)
                .arg(self.source_path.to_str().unwrap())
                .status()
                .map_err(|e| {
                    FeatureError::BuildError(format!("Failed to build container: {}", e))
                })?;

            if !status.success() {
                return Err(FeatureError::BuildError(
                    "Container build failed".to_string(),
                ));
            }

            // Clean up temporary Dockerfile
            if dockerfile_path.exists() {
                std::fs::remove_file(dockerfile_path).ok();
            }

            Ok(tag)
        } else {
            Err(FeatureError::BuildError(
                "No container configuration provided".to_string(),
            ))
        }
    }

    /// Generate a Dockerfile for the container
    fn generate_dockerfile(
        &self,
        path: &Path,
        binary_path: &Path,
        config: &ContainerConfig,
    ) -> Result<(), FeatureError> {
        // Basic Dockerfile content
        let mut content = format!(
            "FROM {}\n\nWORKDIR /app\n\nCOPY {} /app/navius-server\n\n",
            config.base_image,
            binary_path.file_name().unwrap().to_str().unwrap()
        );

        // Add environment variables
        for env in &config.env_vars {
            content.push_str(&format!("ENV {}={}\n", env.name, env.value));
        }

        // Add exposed ports
        for port in &config.ports {
            content.push_str(&format!("EXPOSE {}\n", port));
        }

        // Add labels
        for (key, value) in &config.labels {
            content.push_str(&format!("LABEL {}=\"{}\"\n", key, value));
        }

        // Add entrypoint
        content.push_str("ENTRYPOINT [\"/app/navius-server\"]\n");

        // Write the Dockerfile
        std::fs::write(path, content)
            .map_err(|e| FeatureError::IoError(format!("Failed to write Dockerfile: {}", e)))?;

        Ok(())
    }
}

impl Default for VersionInfo {
    fn default() -> Self {
        Self {
            major: 0,
            minor: 1,
            patch: 0,
            build: None,
            commit: None,
        }
    }
}

/// Package manager for creating and distributing server packages
pub struct PackageManager {
    /// Feature registry
    registry: FeatureRegistry,

    /// Build configuration
    build_config: BuildConfig,
}

impl PackageManager {
    /// Create a new package manager
    pub fn new(registry: FeatureRegistry, build_config: BuildConfig) -> Self {
        Self {
            registry,
            build_config,
        }
    }

    /// Build a server package with selected features
    pub fn build_package(&self) -> Result<PathBuf, FeatureError> {
        // Validate feature selection
        self.registry.validate()?;

        // Execute the build
        let binary_path = self.build_config.execute_build()?;

        println!("Package built successfully at: {:?}", binary_path);

        Ok(binary_path)
    }

    /// Create a containerized deployment
    pub fn create_container(&self) -> Result<String, FeatureError> {
        // Build the binary first
        let binary_path = self.build_package()?;

        // Build the container
        let tag = self.build_config.build_container(&binary_path)?;

        println!("Container built successfully with tag: {}", tag);

        Ok(tag)
    }

    /// Create update package for existing deployments
    pub fn create_update_package(&self, output_path: &Path) -> Result<PathBuf, FeatureError> {
        // Build the binary first
        let binary_path = self.build_package()?;

        // Create a version manifest
        let manifest = self.create_version_manifest()?;

        // Create package directory
        let package_dir = output_path.join(format!(
            "navius-update-{}.{}.{}",
            self.build_config.version.major,
            self.build_config.version.minor,
            self.build_config.version.patch
        ));

        std::fs::create_dir_all(&package_dir).map_err(|e| {
            FeatureError::IoError(format!("Failed to create package directory: {}", e))
        })?;

        // Copy binary to package directory
        let target_binary = package_dir.join("navius-server");
        std::fs::copy(&binary_path, &target_binary)
            .map_err(|e| FeatureError::IoError(format!("Failed to copy binary: {}", e)))?;

        // Write manifest to package directory
        let manifest_path = package_dir.join("manifest.json");
        std::fs::write(
            &manifest_path,
            serde_json::to_string_pretty(&manifest)
                .map_err(|e| FeatureError::SerializationError(e.to_string()))?,
        )
        .map_err(|e| FeatureError::IoError(format!("Failed to write manifest: {}", e)))?;

        println!("Update package created at: {:?}", package_dir);

        Ok(package_dir)
    }

    /// Create version manifest for the package
    fn create_version_manifest(&self) -> Result<serde_json::Value, FeatureError> {
        let manifest = serde_json::json!({
            "version": {
                "major": self.build_config.version.major,
                "minor": self.build_config.version.minor,
                "patch": self.build_config.version.patch,
                "build": self.build_config.version.build,
                "commit": self.build_config.version.commit,
            },
            "features": self.registry.get_selected(),
            "created_at": chrono::Utc::now().to_rfc3339(),
        });

        Ok(manifest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// Create a test build configuration
    fn create_test_build_config() -> (TempDir, BuildConfig) {
        let temp_dir = TempDir::new().unwrap();
        let source_path = std::env::current_dir().unwrap();
        let output_path = temp_dir.path().to_path_buf();

        let build_config = BuildConfig::new(source_path, output_path)
            .with_optimization("debug")
            .with_features(
                vec!["core".to_string(), "metrics".to_string()]
                    .into_iter()
                    .collect(),
            )
            .with_version(VersionInfo {
                major: 0,
                minor: 1,
                patch: 0,
                build: Some("test".to_string()),
                commit: Some("abcdef".to_string()),
            });

        (temp_dir, build_config)
    }

    #[test]
    fn test_build_command_generation() {
        let (_, config) = create_test_build_config();

        let cmd = config.generate_build_command();

        assert_eq!(cmd[0], "cargo");
        assert_eq!(cmd[1], "build");

        // Check that features are included
        let features_index = cmd.iter().position(|arg| arg == "--features").unwrap();
        let features = &cmd[features_index + 1];
        assert!(features.contains("core"));
        assert!(features.contains("metrics"));
    }

    #[test]
    fn test_version_info_default() {
        let version = VersionInfo::default();

        assert_eq!(version.major, 0);
        assert_eq!(version.minor, 1);
        assert_eq!(version.patch, 0);
        assert!(version.build.is_none());
        assert!(version.commit.is_none());
    }
}
