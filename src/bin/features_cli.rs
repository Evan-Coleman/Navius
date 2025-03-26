use std::{collections::HashSet, path::PathBuf, time::Duration};

use clap::{Arg, Command};
use colored::Colorize;
use dialoguer::{Confirm, MultiSelect, Select, theme::ColorfulTheme};
use indicatif::{ProgressBar, ProgressStyle};

use navius::core::features::{FeatureConfig, FeatureError, FeatureInfo, FeatureRegistry};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create colored header
    print_header();

    // Parse command line arguments
    let matches = Command::new("Navius Feature CLI")
        .version("1.0")
        .author("Navius Team")
        .about("Feature selection and customization tool")
        .subcommand(
            Command::new("list").about("List available features").arg(
                Arg::new("format")
                    .short('f')
                    .long("format")
                    .help("Output format (text, json, yaml)")
                    .default_value("text")
                    .value_parser(["text", "json", "yaml"]),
            ),
        )
        .subcommand(
            Command::new("enable").about("Enable a feature").arg(
                Arg::new("feature")
                    .required(true)
                    .help("Feature name to enable"),
            ),
        )
        .subcommand(
            Command::new("disable").about("Disable a feature").arg(
                Arg::new("feature")
                    .required(true)
                    .help("Feature name to disable"),
            ),
        )
        .subcommand(Command::new("status").about("Show current feature status"))
        .subcommand(Command::new("interactive").about("Start interactive mode"))
        .subcommand(Command::new("apply").about(
            "Apply feature configuration to the actual project (generates config for cargo build)",
        ))
        .get_matches();

    // Load feature registry from existing config if available, otherwise create new
    let mut registry = load_feature_registry()?;

    // Process commands
    match matches.subcommand() {
        Some(("list", sub_matches)) => list_features(&registry, sub_matches),
        Some(("enable", sub_matches)) => enable_feature(&mut registry, sub_matches)?,
        Some(("disable", sub_matches)) => disable_feature(&mut registry, sub_matches)?,
        Some(("status", _)) => show_feature_status(&registry),
        Some(("interactive", _)) => show_interactive_menu(&mut registry)?,
        Some(("apply", _)) => apply_feature_configuration(&registry)?,
        _ => {
            // No subcommand provided, show interactive menu
            show_interactive_menu(&mut registry)?;
        }
    }

    Ok(())
}

/// Show interactive menu for the CLI
fn show_interactive_menu(registry: &mut FeatureRegistry) -> Result<(), FeatureError> {
    let theme = ColorfulTheme::default();

    loop {
        println!("\n{}\n", "Navius Feature Management".green().bold());

        // Define menu items
        let items = vec![
            "Select Features",
            "Show Feature Status",
            "Enable Feature",
            "Disable Feature",
            "Exit",
        ];

        // Show menu and get selection
        let selection = Select::with_theme(&theme)
            .with_prompt("Choose an action")
            .default(0)
            .items(&items)
            .interact()
            .unwrap_or(4); // Default to Exit on error

        match selection {
            0 => {
                // Interactive feature selection
                interactive_feature_selection(registry)?;
            }
            1 => {
                // Show feature status
                show_feature_status(registry);

                // Wait for user to continue
                pause_for_user();
            }
            2 => {
                // Enable a feature
                let feature = prompt_for_feature(registry, "Select a feature to enable:");
                if let Some(feature_name) = feature {
                    let result = enable_feature_interactive(registry, &feature_name);
                    if let Err(e) = result {
                        println!("\n❌ Error: {}", e);
                    }
                }

                // Wait for user to continue
                pause_for_user();
            }
            3 => {
                // Disable a feature
                let feature = prompt_for_feature(registry, "Select a feature to disable:");
                if let Some(feature_name) = feature {
                    let result = disable_feature_interactive(registry, &feature_name);
                    if let Err(e) = result {
                        println!("\n❌ Error: {}", e);
                    }
                }

                // Wait for user to continue
                pause_for_user();
            }
            4 | _ => {
                // Exit
                println!("\n👋 Goodbye!");
                break;
            }
        }
    }

    Ok(())
}

/// Interactive feature selection
fn interactive_feature_selection(registry: &mut FeatureRegistry) -> Result<(), FeatureError> {
    let theme = ColorfulTheme::default();

    // Get all features
    let features = registry.feature_list();

    // Prepare for selection
    let mut items: Vec<String> = Vec::new();
    for f in &features {
        let status = if registry.feature_is_enabled(&f.name) {
            "[✓]"
        } else {
            "[ ]"
        };
        items.push(format!("{} {} - {}", status, f.name, f.description));
    }

    // Mark currently enabled features
    let defaults: Vec<bool> = features
        .iter()
        .map(|f| registry.feature_is_enabled(&f.name))
        .collect();

    // Show selection menu
    println!("\n{}\n", "Select features to enable:".green().bold());

    let selections = MultiSelect::with_theme(&theme)
        .with_prompt("Use space to toggle, enter to confirm")
        .items(&items)
        .defaults(&defaults)
        .interact()
        .unwrap_or_default();

    if selections.is_empty() {
        println!("No features selected or operation cancelled");
        return Ok(());
    }

    // Convert selections to feature set
    let mut selected_features: HashSet<String> = HashSet::new();
    for i in selections {
        selected_features.insert(features[i].name.clone());
    }

    // Update registry with new selections
    update_registry_selections(registry, selected_features)?;

    // Ask to save
    if Confirm::with_theme(&theme)
        .with_prompt("Save these feature selections?")
        .default(true)
        .interact()
        .unwrap_or(false)
    {
        println!("✅ Feature configuration saved!");
    }

    Ok(())
}

/// Update registry with new selections
fn update_registry_selections(
    registry: &mut FeatureRegistry,
    selections: HashSet<String>,
) -> Result<(), FeatureError> {
    // Reset current selections first by copying feature names to a separate vector
    let feature_names: Vec<String> = registry
        .feature_list()
        .iter()
        .map(|f| f.name.clone())
        .collect();

    for feature_name in feature_names {
        let _ = registry.disable_feature(&feature_name);
    }

    // Enable selected features
    let progress = ProgressBar::new(selections.len() as u64);
    progress.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} features",
            )
            .unwrap()
            .progress_chars("#>-"),
    );

    for feature in selections {
        progress.inc(1);
        // Use a scoped message instead of a temporary format string
        if let Err(e) = registry.enable_feature(&feature) {
            let message = format!("⚠️  Error enabling feature {}: {}", feature, e);
            progress.finish_with_message(message);
        }
    }

    progress.finish_with_message("✅ Features updated successfully!".to_string());

    // Save the configuration to disk
    save_feature_configuration(registry)?;

    Ok(())
}

/// Prompt for a feature name
fn prompt_for_feature(registry: &FeatureRegistry, prompt: &str) -> Option<String> {
    let theme = ColorfulTheme::default();

    // Get feature names
    let feature_names: Vec<String> = registry
        .feature_list()
        .iter()
        .map(|f| f.name.clone())
        .collect();

    // Check if we have any features to select
    if feature_names.is_empty() {
        println!("No features available to select.");
        return None;
    }

    // Show selection
    let selection = Select::with_theme(&theme)
        .with_prompt(prompt)
        .default(0)
        .items(&feature_names)
        .interact();

    match selection {
        Ok(index) => Some(feature_names[index].clone()),
        Err(_) => None,
    }
}

/// Enable a feature interactively
fn enable_feature_interactive(
    registry: &mut FeatureRegistry,
    feature: &str,
) -> Result<(), FeatureError> {
    println!("\n{}", format!("Enabling feature: {}", feature).green());

    // Enable the feature
    let result = registry.enable_feature(feature);

    match &result {
        Ok(_) => {
            println!("✅ Feature enabled successfully!");

            let enabled_count = registry.get_enabled_features().len();
            let total_count = registry.feature_list().len();

            println!(
                "Current configuration has {} of {} features enabled",
                enabled_count, total_count
            );

            // Save the configuration to disk
            save_feature_configuration(registry)?;
        }
        Err(e) => {
            println!("❌ Error: {}", e);

            // Show more info for dependency errors
            if let FeatureError::MissingDependency(_, missing) = e {
                println!("\n⚠️  Missing required dependency: {}", missing.yellow());
                println!("   Enable that feature first.");
            }
        }
    }

    result
}

/// Disable feature (interactive mode)
fn disable_feature_interactive(
    registry: &mut FeatureRegistry,
    feature: &str,
) -> Result<(), FeatureError> {
    println!("\n{}", format!("Disabling feature: {}", feature).yellow());

    // Disable the feature
    let result = registry.disable_feature(feature);

    match &result {
        Ok(_) => {
            println!("✅ Feature disabled successfully!");

            let enabled_count = registry.get_enabled_features().len();
            let total_count = registry.feature_list().len();

            println!(
                "Current configuration has {} of {} features enabled",
                enabled_count, total_count
            );

            // Save the configuration to disk
            save_feature_configuration(registry)?;
        }
        Err(e) => {
            println!("❌ Error: {}", e);

            // Show more info for dependency errors
            if let FeatureError::DependencyRequired(_, dependent) = &e {
                println!("\n⚠️  The feature is required by: {}", dependent.yellow());
                println!("   You must disable that feature first.");
            }
        }
    }

    result
}

/// Pause and wait for user to continue
fn pause_for_user() {
    let theme = ColorfulTheme::default();
    dialoguer::Input::<String>::with_theme(&theme)
        .with_prompt("Press Enter to continue")
        .allow_empty(true)
        .interact()
        .ok();
}

/// Print colorful header
fn print_header() {
    println!("{}", "=".repeat(60).bright_blue());
    println!(
        "{}",
        "     NAVIUS FEATURE CUSTOMIZATION SYSTEM     "
            .bright_white()
            .on_bright_blue()
    );
    println!("{}", "=".repeat(60).bright_blue());
    println!();
}

/// List all available features
fn list_features(registry: &FeatureRegistry, matches: &clap::ArgMatches) {
    let format = matches.get_one::<String>("format").unwrap();

    match format.as_str() {
        "json" => {
            // Simple JSON output
            println!("{{");
            println!("  \"features\": [");

            let features = registry.feature_list();
            for (i, feature) in features.iter().enumerate() {
                println!("    {{");
                println!("      \"name\": \"{}\",", feature.name);
                println!("      \"description\": \"{}\",", feature.description);
                println!(
                    "      \"enabled\": {},",
                    registry.feature_is_enabled(&feature.name)
                );
                println!("      \"dependencies\": {:?}", feature.dependencies);
                if i < features.len() - 1 {
                    println!("    }},");
                } else {
                    println!("    }}");
                }
            }

            println!("  ]");
            println!("}}");
        }
        "yaml" => {
            // Simple YAML output
            println!("features:");

            for feature in registry.feature_list() {
                println!("  - name: {}", feature.name);
                println!("    description: {}", feature.description);
                println!(
                    "    enabled: {}",
                    registry.feature_is_enabled(&feature.name)
                );
                println!("    dependencies:");
                for dep in &feature.dependencies {
                    println!("      - {}", dep);
                }
            }
        }
        _ => {
            // Default text output with colors
            println!("{}", "Available Features:".green().bold());
            println!("{}", "=".repeat(60));

            for feature in registry.feature_list() {
                let status = if registry.feature_is_enabled(&feature.name) {
                    "✅ ENABLED ".green()
                } else {
                    "❌ DISABLED".red()
                };

                println!(
                    "{} - {} ({})",
                    status,
                    feature.name.yellow(),
                    feature.description
                );

                if !feature.dependencies.is_empty() {
                    println!(
                        "   {}",
                        format!("Dependencies: {}", feature.dependencies.join(", ")).dimmed()
                    );
                }

                println!();
            }
        }
    }
}

/// Enable a specific feature
fn enable_feature(
    registry: &mut FeatureRegistry,
    matches: &clap::ArgMatches,
) -> Result<(), FeatureError> {
    let feature = matches
        .get_one::<String>("feature")
        .expect("Required argument");

    enable_feature_interactive(registry, feature)
}

/// Disable a specific feature
fn disable_feature(
    registry: &mut FeatureRegistry,
    matches: &clap::ArgMatches,
) -> Result<(), FeatureError> {
    let feature = matches
        .get_one::<String>("feature")
        .expect("Required argument");

    disable_feature_interactive(registry, feature)
}

/// Show current feature status
fn show_feature_status(registry: &FeatureRegistry) {
    println!("\n{}", "Feature Status:".green().bold());

    let enabled_count = registry.get_enabled_features().len();
    let total_count = registry.feature_list().len();
    let percentage = (enabled_count as f32 / total_count as f32) * 100.0;

    println!(
        "\nEnabled: {} of {} features ({:.0}%)\n",
        enabled_count.to_string().green().bold(),
        total_count,
        percentage
    );

    // Group features by category
    let mut categories: std::collections::HashMap<String, Vec<FeatureInfo>> =
        std::collections::HashMap::new();

    for feature in registry.feature_list() {
        categories
            .entry(feature.category.clone())
            .or_default()
            .push(feature.clone());
    }

    // Display features by category
    for (category, features) in categories {
        println!("{}", category.cyan().bold());

        for feature in features {
            let status = if registry.feature_is_enabled(&feature.name) {
                "✅ ENABLED ".green()
            } else {
                "❌ DISABLED".red()
            };

            println!("  {} - {} ({})", status, feature.name, feature.description);
        }

        println!();
    }
}

/// Load a feature registry from config file or create new one if not found
fn load_feature_registry() -> Result<FeatureRegistry, FeatureError> {
    // Try to load from config file first
    let config_result = FeatureConfig::load_default();

    match config_result {
        Ok(config) => {
            // Create an empty feature registry
            let mut registry = FeatureRegistry::new();

            // Add sample features
            add_sample_features(&mut registry)?;

            // Update registry with saved selections
            for feature in &config.selected_features {
                // Ignore errors in case feature no longer exists
                let _ = registry.enable_feature(feature);
            }

            println!("✅ Loaded feature configuration from file");
            Ok(registry)
        }
        Err(_) => {
            // Create a new registry with default features
            let mut registry = FeatureRegistry::new();
            add_sample_features(&mut registry)?;

            // Save the initial configuration
            save_feature_configuration(&registry)?;

            println!("✅ Created new feature configuration");
            Ok(registry)
        }
    }
}

/// Save feature configuration to disk
fn save_feature_configuration(registry: &FeatureRegistry) -> Result<(), FeatureError> {
    // Create configuration from registry
    let config = FeatureConfig::from_registry(registry);

    // Save to default location
    config.save_default()?;

    Ok(())
}

/// Apply feature configuration to the project
fn apply_feature_configuration(registry: &FeatureRegistry) -> Result<(), FeatureError> {
    // Create configuration from registry
    let config = FeatureConfig::from_registry(registry);

    // Save the configuration
    config.save_default()?;

    // Generate and display Cargo build command
    let flags = config.generate_build_flags();
    let cmd = format!("cargo build {}", flags.join(" "));

    println!("\n{}", "Feature configuration applied!".green().bold());
    println!("\nTo build with these features, run:");
    println!("{}", cmd.cyan());

    // Ask if user wants to build now
    let theme = ColorfulTheme::default();
    if Confirm::with_theme(&theme)
        .with_prompt("Do you want to build the project now with these features?")
        .default(false)
        .interact()
        .unwrap_or(false)
    {
        // Run the build command
        println!("\n{}", "Building project...".green());

        let progress = ProgressBar::new_spinner();
        progress.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap(),
        );
        progress.set_message("Building with custom features...");

        // Execute the build command
        let output = std::process::Command::new("cargo")
            .arg("build")
            .args(flags)
            .output();

        match output {
            Ok(output) => {
                if output.status.success() {
                    progress.finish_with_message("✅ Build successful!".to_string());
                    println!("{}", String::from_utf8_lossy(&output.stdout));
                } else {
                    progress.finish_with_message("❌ Build failed!".to_string());
                    println!("{}", String::from_utf8_lossy(&output.stderr));
                }
            }
            Err(e) => {
                progress.finish_with_message("❌ Failed to execute build command!".to_string());
                println!("Error: {}", e);
            }
        }
    }

    Ok(())
}

/// Add sample features to the registry
fn add_sample_features(registry: &mut FeatureRegistry) -> Result<(), FeatureError> {
    // Core functionality (always required)
    let core_info = FeatureInfo {
        name: "core".to_string(),
        description: "Core functionality".to_string(),
        category: "Core".to_string(),
        dependencies: vec![],
        default_enabled: true,
        tags: vec!["essential".to_string()],
        size_impact: 100,
    };
    registry.register(core_info);

    // Metrics
    let metrics_info = FeatureInfo {
        name: "metrics".to_string(),
        description: "Basic metrics collection".to_string(),
        category: "Observability".to_string(),
        dependencies: vec![],
        default_enabled: true,
        tags: vec!["monitoring".to_string()],
        size_impact: 250,
    };
    registry.register(metrics_info);

    // Advanced metrics (depends on metrics)
    let advanced_metrics_info = FeatureInfo {
        name: "advanced_metrics".to_string(),
        description: "Advanced metrics with histograms and gauges".to_string(),
        category: "Observability".to_string(),
        dependencies: vec!["metrics".to_string()],
        default_enabled: false,
        tags: vec!["monitoring".to_string(), "advanced".to_string()],
        size_impact: 400,
    };
    registry.register(advanced_metrics_info);

    // Caching
    let caching_info = FeatureInfo {
        name: "caching".to_string(),
        description: "Data caching functionality".to_string(),
        category: "Performance".to_string(),
        dependencies: vec![],
        default_enabled: true,
        tags: vec!["performance".to_string()],
        size_impact: 200,
    };
    registry.register(caching_info);

    // Auth (security)
    let auth_info = FeatureInfo {
        name: "auth".to_string(),
        description: "Authentication and authorization".to_string(),
        category: "Security".to_string(),
        dependencies: vec![],
        default_enabled: true,
        tags: vec!["security".to_string()],
        size_impact: 350,
    };
    registry.register(auth_info);

    // Error handling
    let error_handling_info = FeatureInfo {
        name: "error_handling".to_string(),
        description: "Error handling and reporting".to_string(),
        category: "Essential".to_string(),
        dependencies: vec![],
        default_enabled: false,
        tags: vec!["essential".to_string()],
        size_impact: 150,
    };
    registry.register(error_handling_info);

    // Config system
    let config_info = FeatureInfo {
        name: "config".to_string(),
        description: "Configuration system".to_string(),
        category: "Essential".to_string(),
        dependencies: vec![],
        default_enabled: false,
        tags: vec!["essential".to_string()],
        size_impact: 180,
    };
    registry.register(config_info);

    // Reliability
    let reliability_info = FeatureInfo {
        name: "reliability".to_string(),
        description: "Reliability features like retry, circuit breaking".to_string(),
        category: "Resilience".to_string(),
        dependencies: vec![],
        default_enabled: false,
        tags: vec!["resilience".to_string()],
        size_impact: 300,
    };
    registry.register(reliability_info);

    // Enable some features by default
    registry.enable_feature("core")?;
    registry.enable_feature("metrics")?;
    registry.enable_feature("auth")?;
    registry.enable_feature("caching")?;

    Ok(())
}
