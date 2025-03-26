use navius::core::config::app_config::{AppConfig, load_config as app_load_config};
use navius::core::features::{
    BuildConfig, ContainerConfig, DocConfig, DocGenerator, FeatureConfig, FeatureRegistry,
    PackageManager, VersionInfo,
};
use std::collections::{HashMap, HashSet};
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_usage();
        return;
    }

    match args[1].as_str() {
        "list" => list_features(),
        "enable" if args.len() >= 3 => enable_feature(&args[2]),
        "disable" if args.len() >= 3 => disable_feature(&args[2]),
        "status" => show_status(),
        "build" => build_with_features(),
        "package" => package_with_features(),
        "container" => create_container(),
        "update" if args.len() >= 3 => create_update_package(&args[2]),
        "docs" => generate_documentation(),
        "docs:version" if args.len() >= 3 => generate_versioned_docs(&args[2]),
        "save" if args.len() >= 3 => save_config(&args[2]),
        "load" if args.len() >= 3 => load_config(&args[2]),
        "reset" => reset_to_defaults(),
        _ => print_usage(),
    }
}

fn print_usage() {
    println!("Navius Feature Builder");
    println!("Usage:");
    println!("  feature-builder list                - List all available features");
    println!("  feature-builder status              - Show currently selected features");
    println!("  feature-builder enable <feature>    - Enable a feature and its dependencies");
    println!("  feature-builder disable <feature>   - Disable a feature if possible");
    println!("  feature-builder build               - Build with selected features");
    println!(
        "  feature-builder package             - Create optimized package with selected features"
    );
    println!(
        "  feature-builder container           - Create container image with selected features"
    );
    println!(
        "  feature-builder update <output_dir> - Create update package with selected features"
    );
    println!("  feature-builder docs                - Generate documentation for enabled features");
    println!("  feature-builder docs:version <tag>  - Generate versioned documentation");
    println!("  feature-builder save <file>         - Save current feature config");
    println!("  feature-builder load <file>         - Load feature config from file");
    println!("  feature-builder reset               - Reset to default features");
}

fn load_and_create_registry() -> (AppConfig, FeatureRegistry) {
    // Load app config using the standard config system
    let app_config = match app_load_config() {
        Ok(config) => config,
        Err(e) => {
            println!("Error loading configuration: {}", e);
            println!("Using default settings...");
            // Create default app config
            AppConfig::default()
        }
    };

    // Create registry from the standard configuration
    let mut registry = FeatureRegistry::new();

    // Apply config-defined feature selections from both sources
    let mut enabled_features = app_config.enabled_features();

    // Also load features from features.json if it exists
    if let Ok(feature_config) = FeatureConfig::load_default() {
        for feature in &feature_config.selected_features {
            enabled_features.insert(feature.clone());
        }
    }

    // First clear any default selections that aren't in our config
    let default_selected = registry.get_selected();
    for feature in default_selected {
        if !enabled_features.contains(&feature) {
            // Try to deselect, but ignore errors (dependencies, etc.)
            let _ = registry.deselect(&feature);
        }
    }

    // Then enable all configured features
    for feature in &enabled_features {
        let _ = registry.select(feature);
    }

    (app_config, registry)
}

fn list_features() {
    let (_, registry) = load_and_create_registry();

    println!("Available features:");
    println!("------------------");

    let categories = registry.get_categories();

    for category in categories {
        println!("\n[{}]", category);

        let features = registry.get_features_by_category(&category);
        for feature in features {
            let status = if registry.is_selected(&feature.name) {
                "[✓]"
            } else {
                "[ ]"
            };

            println!("{} {} - {}", status, feature.name, feature.description);

            if !feature.dependencies.is_empty() {
                println!("    Dependencies: {}", feature.dependencies.join(", "));
            }

            if !feature.tags.is_empty() {
                println!("    Tags: {}", feature.tags.join(", "));
            }
        }
    }
}

fn enable_feature(name: &str) {
    let (_, mut registry) = load_and_create_registry();

    // Enable the new feature
    match registry.select(name) {
        Ok(_) => {
            println!("Successfully enabled feature: {}", name);

            if let Some(feature) = registry.get_feature_info(name) {
                println!("Dependencies also enabled:");
                for dep in &feature.dependencies {
                    println!("  - {}", dep);
                }
            }

            // Save updated configuration
            save_to_config_file(&registry);
        }
        Err(e) => {
            println!("Error enabling feature: {}", e);
        }
    }
}

fn disable_feature(name: &str) {
    let (_, mut registry) = load_and_create_registry();

    // Disable the feature
    match registry.deselect(name) {
        Ok(_) => {
            println!("Successfully disabled feature: {}", name);

            // Save updated configuration
            save_to_config_file(&registry);
        }
        Err(e) => {
            println!("Error disabling feature: {}", e);

            if e.to_string().contains("required by") {
                // Parse the error to extract which features depend on this
                if let Some(other_feature) = e.to_string().split("required by").nth(1) {
                    println!("The feature is required by: {}", other_feature.trim());
                    println!("You must disable those features first.");
                }
            }
        }
    }
}

fn show_status() {
    let (_, registry) = load_and_create_registry();
    let selected = registry.get_selected();

    println!("Currently selected features:");
    println!("--------------------------");

    for feature in &selected {
        println!("- {}", feature);
    }

    println!("\nTotal features enabled: {}", selected.len());

    if let Ok(path) = FeatureConfig::default_config_path() {
        println!("\nConfiguration file: {}", path.display());
    }
}

fn build_with_features() {
    let (_, registry) = load_and_create_registry();
    let config = FeatureConfig::from_registry(&registry);

    println!("Building with selected features...");

    let flags = config.generate_build_flags();

    let mut args = vec!["build"];
    for flag in &flags {
        args.push(flag.as_str());
    }

    println!("Running: cargo {}", args.join(" "));

    let status = Command::new("cargo")
        .args(args)
        .status()
        .expect("Failed to execute cargo build");

    if status.success() {
        println!("Build completed successfully!");
    } else {
        println!("Build failed with exit code: {:?}", status.code());
    }
}

fn save_to_config_file(registry: &FeatureRegistry) {
    // Create and save a FeatureConfig for backward compatibility
    let feature_config = FeatureConfig::from_registry(registry);
    if let Err(e) = feature_config.save_default() {
        println!("Error saving feature configuration: {}", e);
    } else {
        println!("Feature configuration saved.");
    }

    // Note: In a real system, we would update the app_config.features section
    // and save it using the standard config system, but that requires more code
    // to read, modify, and write YAML files correctly.
}

fn save_config(path: &str) {
    let (_, registry) = load_and_create_registry();
    let config = FeatureConfig::from_registry(&registry);

    match config.save(Path::new(path)) {
        Ok(_) => {
            println!("Feature configuration saved to: {}", path);
        }
        Err(e) => {
            println!("Error saving configuration: {}", e);
        }
    }
}

fn load_config(path: &str) {
    match FeatureConfig::load(Path::new(path)) {
        Ok(config) => {
            println!("Feature configuration loaded from: {}", path);
            println!("Selected features:");

            for feature in &config.selected_features {
                println!("- {}", feature);
            }

            // Create and update registry
            let mut registry = FeatureRegistry::new();

            // Apply feature selections from the loaded config
            for feature in &config.selected_features {
                if let Err(e) = registry.select(feature) {
                    println!("Warning: Couldn't select feature '{}': {}", feature, e);
                }
            }

            // Save to config file
            save_to_config_file(&registry);

            println!("\nConfiguration applied and saved.");
        }
        Err(e) => {
            println!("Error loading configuration: {}", e);
        }
    }
}

fn reset_to_defaults() {
    let registry = FeatureRegistry::new();

    println!("Configuration reset to defaults.");
    println!("Default features enabled:");

    for feature in registry.get_selected() {
        println!("- {}", feature);
    }

    // Save to config file
    save_to_config_file(&registry);
}

fn package_with_features() {
    let (_, registry) = load_and_create_registry();

    println!("Creating optimized package with selected features...");

    // Get current directory for source path
    let source_path = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    // Configure build options
    let build_config = BuildConfig::new(source_path.clone(), source_path)
        .with_optimization("release")
        .with_features(registry.get_selected())
        .with_version(get_version_info());

    // Create package manager
    let package_manager = PackageManager::new(registry, build_config);

    // Build the package
    match package_manager.build_package() {
        Ok(binary_path) => {
            println!("Package built successfully!");
            println!("Binary path: {:?}", binary_path);
        }
        Err(e) => {
            println!("Failed to build package: {}", e);
        }
    }
}

fn create_container() {
    let (_, registry) = load_and_create_registry();

    println!("Creating container image with selected features...");

    // Get current directory for source path
    let source_path = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    // Configure container options
    let container_config = ContainerConfig {
        base_image: "debian:bullseye-slim".to_string(),
        tags: vec![format!("navius-server:{}", env!("CARGO_PKG_VERSION"))],
        env_vars: vec![],
        ports: vec![8080, 8081],
        labels: HashMap::from([
            (
                "org.opencontainers.image.title".to_string(),
                "Navius Server".to_string(),
            ),
            (
                "org.opencontainers.image.version".to_string(),
                env!("CARGO_PKG_VERSION").to_string(),
            ),
            (
                "org.opencontainers.image.vendor".to_string(),
                "Navius".to_string(),
            ),
        ]),
    };

    // Configure build options
    let build_config = BuildConfig::new(source_path.clone(), source_path)
        .with_optimization("release")
        .with_features(registry.get_selected())
        .with_version(get_version_info())
        .with_container(container_config);

    // Create package manager
    let package_manager = PackageManager::new(registry, build_config);

    // Build the container
    match package_manager.create_container() {
        Ok(tag) => {
            println!("Container built successfully!");
            println!("Image tag: {}", tag);
            println!("\nYou can run it with:");
            println!("docker run -p 8080:8080 -p 8081:8081 {}", tag);
        }
        Err(e) => {
            println!("Failed to build container: {}", e);
        }
    }
}

fn create_update_package(output_dir: &str) {
    let (_, registry) = load_and_create_registry();

    println!("Creating update package with selected features...");

    // Get current directory for source path
    let source_path = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let output_path = PathBuf::from(output_dir);

    // Configure build options
    let build_config = BuildConfig::new(source_path.clone(), source_path)
        .with_optimization("release")
        .with_features(registry.get_selected())
        .with_version(get_version_info());

    // Create package manager
    let package_manager = PackageManager::new(registry, build_config);

    // Create update package
    match package_manager.create_update_package(&output_path) {
        Ok(package_path) => {
            println!("Update package created successfully!");
            println!("Package path: {:?}", package_path);
        }
        Err(e) => {
            println!("Failed to create update package: {}", e);
        }
    }
}

fn get_version_info() -> VersionInfo {
    // Get version from environment
    let version_str = env!("CARGO_PKG_VERSION");
    let parts: Vec<&str> = version_str.split('.').collect();

    let major = parts.get(0).and_then(|s| s.parse().ok()).unwrap_or(0);
    let minor = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(1);
    let patch = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);

    // Try to get git commit hash
    let commit = get_git_commit();

    VersionInfo {
        major,
        minor,
        patch,
        build: Some(format!("build-{}", chrono::Utc::now().format("%Y%m%d%H%M"))),
        commit,
    }
}

fn get_git_commit() -> Option<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output();

    match output {
        Ok(output) if output.status.success() => {
            let hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !hash.is_empty() { Some(hash) } else { None }
        }
        _ => None,
    }
}

fn generate_documentation() {
    let (_, registry) = load_and_create_registry();

    println!("Generating documentation for selected features...");

    // Use default doc configuration
    let config = DocConfig::default();

    // Create documentation generator
    match DocGenerator::new(registry, config) {
        Ok(generator) => {
            // Generate documentation
            match generator.generate() {
                Ok(()) => {
                    println!("Documentation generated successfully!");
                    println!("Output directory: {:?}", generator.config.output_dir);
                }
                Err(e) => {
                    println!("Failed to generate documentation: {}", e);
                }
            }
        }
        Err(e) => {
            println!("Failed to create documentation generator: {}", e);
        }
    }
}

fn generate_versioned_docs(version_tag: &str) {
    let (_, registry) = load_and_create_registry();

    println!("Generating versioned documentation for selected features...");
    println!("Version tag: {}", version_tag);

    // Use default doc configuration
    let config = DocConfig::default();

    // Create documentation generator
    match DocGenerator::new(registry, config) {
        Ok(generator) => {
            // Generate versioned documentation
            match generator.generate_versioned(version_tag) {
                Ok(output_dir) => {
                    println!("Versioned documentation generated successfully!");
                    println!("Output directory: {:?}", output_dir);
                }
                Err(e) => {
                    println!("Failed to generate versioned documentation: {}", e);
                }
            }
        }
        Err(e) => {
            println!("Failed to create documentation generator: {}", e);
        }
    }
}
