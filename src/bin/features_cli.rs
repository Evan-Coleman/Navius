use std::{collections::HashMap, collections::HashSet, io::Write, path::PathBuf, time::Duration};

use clap::{Arg, Command};
use colored::Colorize;
use dialoguer::{Confirm, MultiSelect, Select, theme::ColorfulTheme};
use indicatif::{ProgressBar, ProgressStyle};

use navius::core::features::{FeatureConfig, FeatureError, FeatureInfo, FeatureRegistry};

fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    // Create colored header
    clear_screen();
    print_header();

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

/// Clear the terminal screen
fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
}

/// Show interactive menu for the CLI
fn show_interactive_menu(registry: &mut FeatureRegistry) -> Result<(), FeatureError> {
    loop {
        clear_screen();
        print_header();
        println!("{}", "Navius Feature Management".green().bold());

        // Menu options
        let items = build_interactive_menu(registry);

        // Use the dialoguer Select component for interactive arrow key selection
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Choose an action")
            .default(0)
            .items(&items)
            .interact()
            .unwrap_or(3); // Default to Exit if interaction fails

        match selection {
            0 => {
                clear_screen();
                print_header();
                let result = manual_feature_selection(registry);
                if let Err(e) = result {
                    eprintln!("Error: {}", e);
                    pause_for_user();
                }
            }
            1 => {
                clear_screen();
                print_header();
                show_feature_status(registry);
                pause_for_user();
            }
            2 => {
                clear_screen();
                print_header();
                if let Err(e) = apply_feature_configuration(registry) {
                    eprintln!("Error: {}", e);
                } else {
                    println!("✅ Configuration applied");
                }
                pause_for_user();
            }
            3 => {
                println!("Exiting...");
                break;
            }
            _ => {
                println!("Invalid choice. Please try again.");
                pause_for_user();
            }
        }
    }

    Ok(())
}

/// Manual feature selection using MultiSelect
fn manual_feature_selection(registry: &mut FeatureRegistry) -> Result<(), FeatureError> {
    clear_screen();
    print_header();
    println!("{}", "Select Features".green().bold());

    // Get all features and categorize them
    let all_features: Vec<FeatureInfo> = registry
        .feature_list()
        .into_iter()
        .map(|f| f.clone())
        .collect();

    // First, we'll separate required features from optional ones
    let mut optional_features: Vec<FeatureInfo> = Vec::new();
    let mut required_features: Vec<FeatureInfo> = Vec::new();

    for feature in &all_features {
        if is_required_feature(feature) {
            required_features.push(feature.clone());
        } else {
            optional_features.push(feature.clone());
        }
    }

    // Sort features by name within each group
    optional_features.sort_by(|a, b| a.name.cmp(&b.name));
    required_features.sort_by(|a, b| a.name.cmp(&b.name));

    // Create display items for MultiSelect with current enabled status
    let mut display_items: Vec<String> = Vec::new();
    let mut selection_indices: Vec<usize> = Vec::new();

    // Track currently enabled features
    let enabled_features: HashSet<String> =
        registry.get_enabled_features().iter().cloned().collect();

    // Add optional features to selection list
    for (i, feature) in optional_features.iter().enumerate() {
        // Format display string with size and dependencies
        let mut display = format!("{} ({} KB)", feature.name, feature.size_impact);

        // Add dependency information if any
        if !feature.dependencies.is_empty() {
            display.push_str(" - Requires ");
            display.push_str(&feature.dependencies.join(", "));
        }

        display_items.push(display);

        // Mark as selected if currently enabled
        if enabled_features.contains(&feature.name) {
            selection_indices.push(i);
        }
    }

    // Show the feature selection interface
    let theme = ColorfulTheme::default();

    // Create a vector of booleans representing selection state
    let mut default_states = vec![false; display_items.len()];
    for &idx in &selection_indices {
        if idx < display_items.len() {
            default_states[idx] = true;
        }
    }

    let selections = MultiSelect::with_theme(&theme)
        .with_prompt("Select features to enable (space to toggle, enter to confirm)")
        .items(&display_items)
        .defaults(&default_states)
        .interact()
        .unwrap_or_else(|_| selection_indices.clone());

    // Convert selections back to feature names and include required features
    let mut selected_features: HashSet<String> = HashSet::new();

    // First add all required features
    for feature in &required_features {
        selected_features.insert(feature.name.clone());
    }

    // Then add selected optional features
    for &index in &selections {
        if index < optional_features.len() {
            let feature = &optional_features[index];
            selected_features.insert(feature.name.clone());

            // Also add all dependencies
            for dep in &feature.dependencies {
                selected_features.insert(dep.clone());
            }
        }
    }

    // Validation: Check for dependency violations before applying changes
    let metrics_deselected =
        registry.feature_is_enabled("metrics") && !selected_features.contains("metrics");
    let advanced_metrics_enabled = registry.feature_is_enabled("advanced_metrics");
    let advanced_metrics_selected = selected_features.contains("advanced_metrics");
    let advanced_metrics_deselected = advanced_metrics_enabled && !advanced_metrics_selected;

    // If both are being deselected, show an informative message but allow it
    if metrics_deselected && advanced_metrics_deselected {
        println!(
            "{}",
            "ℹ️  INFO: Disabling both metrics and advanced_metrics together.".bright_blue()
        );
        println!("This is the correct way to disable features with dependencies.");
    }
    // If metrics is being deselected while advanced_metrics is still enabled (and not being deselected)
    else if metrics_deselected && advanced_metrics_enabled && !advanced_metrics_deselected {
        // Option 1: Block the action
        if !Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Disable advanced_metrics as well? (Required to disable metrics)")
            .default(true)
            .interact()
            .unwrap_or(false)
        {
            println!("❌ Cancelled. No changes were made.");
            pause_for_user();
            return Ok(()); // Exit without making changes
        }

        // User chose to disable both
        println!(
            "{}",
            "ℹ️  Disabling both metrics and advanced_metrics.".yellow()
        );

        // Remove from selected_features
        selected_features.remove("advanced_metrics");
        selected_features.remove("metrics");

        // Directly disable in registry - this fixes a bug where registry changes weren't being applied
        registry.disable_feature("advanced_metrics")?;
        registry.disable_feature("metrics")?;
        println!("✓ Manually disabled: advanced_metrics");
        println!("✓ Manually disabled: metrics");
    }
    // If advanced_metrics is being enabled, ensure metrics is also enabled
    else if advanced_metrics_selected && !selected_features.contains("metrics") {
        println!(
            "{}",
            "ℹ️  Auto-enabling metrics which is required by advanced_metrics.".yellow()
        );
        selected_features.insert("metrics".to_string());
    }

    // Update registry with selections and save
    update_registry_selections(registry, selected_features.clone())?;
    save_feature_configuration(registry)?;

    // Get the updated lists after applying changes
    let enabled_list: HashSet<String> = registry.get_enabled_features().iter().cloned().collect();

    clear_screen();
    print_header();

    // Show which features are now enabled and disabled
    println!("{}", "Feature Configuration Updated".green().bold());
    println!();

    // First display optional enabled features
    println!("{}", "Enabled Optional Features:".green());
    let mut sorted_enabled: Vec<String> = enabled_list
        .iter()
        .filter(|name| {
            if let Some(feature) = registry.get_feature_info(name) {
                !is_required_feature(feature)
            } else {
                true
            }
        })
        .cloned()
        .collect();
    sorted_enabled.sort();

    for feature_name in &sorted_enabled {
        let feature_info = registry.get_feature_info(feature_name).unwrap();
        println!("  [✓] {} ({} KB)", feature_name, feature_info.size_impact);
    }

    // Display the disabled features
    println!();
    println!("{}", "Disabled Features:".red());
    let all_feature_names: HashSet<String> = all_features.iter().map(|f| f.name.clone()).collect();
    let mut disabled_list: Vec<String> = all_feature_names
        .difference(&enabled_list)
        .filter(|name| {
            if let Some(feature) = registry.get_feature_info(name) {
                !is_required_feature(feature)
            } else {
                true
            }
        })
        .cloned()
        .collect();

    disabled_list.sort();

    for feature_name in disabled_list {
        let feature_info = registry.get_feature_info(&feature_name).unwrap();
        println!("  [ ] {} ({} KB)", feature_name, feature_info.size_impact);
    }

    // Always show required features at the bottom
    println!();
    println!("{}", "Required Features (Always Enabled):".yellow().bold());
    let mut required_list: Vec<String> = registry
        .feature_list()
        .iter()
        .filter(|f| is_required_feature(f))
        .map(|f| f.name.clone())
        .collect();
    required_list.sort();

    for feature_name in required_list {
        let feature_info = registry.get_feature_info(&feature_name).unwrap();
        println!(
            "  [✓] {} ({} KB)",
            feature_name.yellow(),
            feature_info.size_impact
        );
    }

    println!();
    println!("✅ Feature configuration saved!");

    pause_for_user();

    Ok(())
}

/// Helper function to determine if a feature is required
fn is_required_feature(feature: &FeatureInfo) -> bool {
    feature.tags.contains(&"required".to_string())
}

/// Update registry with new selections
fn update_registry_selections(
    registry: &mut FeatureRegistry,
    selections: HashSet<String>,
) -> Result<(), FeatureError> {
    // Track which features we need to operate on
    let feature_names: Vec<String> = registry
        .feature_list()
        .iter()
        .map(|f| f.name.clone())
        .collect();

    // First collect the changes we need to make
    let mut to_disable = Vec::new();
    let mut to_enable = Vec::new();

    // Build dependency graph
    let mut dependents: HashMap<String, Vec<String>> = HashMap::new();
    for feature_name in &feature_names {
        if let Some(feature) = registry.get_feature_info(feature_name) {
            for dep in &feature.dependencies {
                dependents
                    .entry(dep.clone())
                    .or_default()
                    .push(feature_name.clone());
            }
        }
    }

    // Identify features to disable - any currently enabled feature not in selections
    let currently_enabled: HashSet<String> =
        registry.get_enabled_features().iter().cloned().collect();
    for feature_name in currently_enabled.iter() {
        let feature_info = match registry.get_feature_info(feature_name) {
            Some(info) => info,
            None => continue,
        };

        let is_required = is_required_feature(feature_info);

        // Skip required features
        if is_required {
            continue;
        }

        // If the feature is enabled but not in selections, disable it
        if !selections.contains(feature_name) {
            to_disable.push(feature_name.clone());
        }
    }

    // Identify features to enable - any feature in selections not currently enabled
    for feature_name in selections.iter() {
        if !currently_enabled.contains(feature_name) {
            to_enable.push(feature_name.clone());
        }
    }

    // Make sure all required features are enabled
    for feature_name in &feature_names {
        let feature_info = match registry.get_feature_info(feature_name) {
            Some(info) => info,
            None => continue,
        };

        if is_required_feature(feature_info)
            && !to_enable.contains(feature_name)
            && !currently_enabled.contains(feature_name)
        {
            to_enable.push(feature_name.clone());
        }
    }

    // Now perform the operations, tracking any errors
    let mut operation_errors = Vec::new();

    // First disable features (do this first to avoid dependency conflicts)
    // We need to sort to_disable so that dependent features are disabled before their dependencies
    // For example, advanced_metrics should be disabled before metrics

    // Sort the to_disable list so dependents are disabled before dependencies
    let mut sorted_to_disable = to_disable.clone();
    sorted_to_disable.sort_by(|a, b| {
        // Try to get the dependency relationship between a and b
        let a_depends_on_b = dependents.get(b).map_or(false, |deps| deps.contains(a));
        let b_depends_on_a = dependents.get(a).map_or(false, |deps| deps.contains(b));

        if a_depends_on_b {
            // a depends on b, so a should be disabled first
            std::cmp::Ordering::Less
        } else if b_depends_on_a {
            // b depends on a, so b should be disabled first
            std::cmp::Ordering::Greater
        } else {
            // No direct dependency, just use alphabetical order
            a.cmp(b)
        }
    });

    for feature_name in &sorted_to_disable {
        match registry.disable_feature(feature_name) {
            Ok(_) => println!("✓ Disabled: {}", feature_name),
            Err(e) => {
                println!("✗ Failed to disable {}: {}", feature_name, e);
                operation_errors.push(format!("Failed to disable {}: {}", feature_name, e));
            }
        }
    }

    // Then enable new features
    for feature_name in &to_enable {
        match registry.enable_feature(feature_name) {
            Ok(_) => println!("✓ Enabled: {}", feature_name),
            Err(e) => {
                println!("✗ Failed to enable {}: {}", feature_name, e);
                operation_errors.push(format!("Failed to enable {}: {}", feature_name, e));
            }
        }
    }

    // Save configuration regardless of any individual feature errors
    save_feature_configuration(registry)?;

    // Report any feature operation errors
    if !operation_errors.is_empty() {
        let error_message = operation_errors.join("\n");
        println!("\n⚠️ Some operations failed:");
        println!("{}", error_message);
    } else {
        println!("✅ Features updated successfully!");
    }

    Ok(())
}

/// Enable a feature interactively
fn enable_feature_interactive(
    registry: &mut FeatureRegistry,
    feature: &str,
) -> Result<(), FeatureError> {
    println!("Enabling feature: {}", feature.green());

    // Enable the feature
    let result = match registry.enable_feature(feature) {
        Ok(_) => {
            println!("✅ Feature enabled successfully!");

            // Save the configuration to disk
            save_feature_configuration(registry)?;
            Ok(())
        }
        Err(e) => {
            println!("❌ Error: {}", e);

            // Show more info for dependency errors
            if let FeatureError::MissingDependency(_, missing) = &e {
                println!("⚠️  Missing dependency: {}", missing.yellow());
                println!("Enable that feature first.");
            }

            Err(e)
        }
    };

    result
}

/// Disable feature (interactive mode)
fn disable_feature_interactive(
    registry: &mut FeatureRegistry,
    feature: &str,
) -> Result<(), FeatureError> {
    println!("Disabling feature: {}", feature.yellow());

    // Check if feature is required
    let feature_info = registry.get_feature_info(feature).unwrap();
    if is_required_feature(feature_info) {
        println!("❌ Cannot disable required feature: {}", feature);
        return Err(FeatureError::DependencyRequired(
            feature.to_string(),
            "system core".to_string(),
        ));
    }

    // Check if any other enabled feature depends on this one
    let mut dependent_features = Vec::new();
    for (feature_name, other_feature) in registry.feature_list().iter().map(|f| (f.name.clone(), f))
    {
        if registry.feature_is_enabled(&feature_name)
            && other_feature.dependencies.contains(&feature.to_string())
        {
            dependent_features.push(feature_name);
        }
    }

    // If there are dependent features, ask the user if they want to disable them too
    if !dependent_features.is_empty() {
        // Special case warning for metrics when advanced_metrics depends on it
        if feature == "metrics" && dependent_features.contains(&"advanced_metrics".to_string()) {
            println!(
                "{}",
                "⚠️  WARNING: Advanced metrics depends on basic metrics!"
                    .bright_red()
                    .bold()
            );
            println!("Disabling metrics will also disable advanced metrics functionality.");
        }

        println!("⚠️  The following enabled features depend on {}:", feature);
        for dep in &dependent_features {
            println!("   - {}", dep);
        }

        let confirm = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(&format!("Disable {} and all dependent features?", feature))
            .interact()
            .unwrap_or(false);

        if !confirm {
            println!("❌ Aborted. No features were disabled.");
            return Ok(());
        }

        // User confirmed, so disable all dependent features first
        for dep_feature in &dependent_features {
            match registry.disable_feature(dep_feature) {
                Ok(_) => println!("✓ Disabled dependent feature: {}", dep_feature),
                Err(e) => {
                    println!("❌ Error disabling {}: {}", dep_feature, e);
                    return Err(e);
                }
            }
        }
    }

    // Now disable the requested feature
    match registry.disable_feature(feature) {
        Ok(_) => {
            println!("✅ Feature {} disabled successfully!", feature);

            // Save the configuration to disk
            save_feature_configuration(registry)?;
            Ok(())
        }
        Err(e) => {
            println!("❌ Error: {}", e);
            Err(e)
        }
    }
}

/// Pause and wait for user to continue
fn pause_for_user() {
    println!("\nPress Enter to continue...");
    let mut input = String::new();
    let _ = std::io::stdin().read_line(&mut input);
}

/// Print colorful header (more compact version)
fn print_header() {
    println!("{}", "=== NAVIUS FEATURE CUSTOMIZATION ===".bright_blue());
}

/// List all features function - customized for interactive mode
fn list_features_interactive(registry: &FeatureRegistry) {
    println!("{}", "Available Features".green().bold());
    println!("{}", "=================".green());

    // Get and sort features
    let mut features = registry.feature_list();
    features.sort_by(|a, b| {
        let a_required = is_required_feature(a);
        let b_required = is_required_feature(b);

        // Use a safe comparison approach
        match (a_required, b_required) {
            (true, false) => std::cmp::Ordering::Greater,
            (false, true) => std::cmp::Ordering::Less,
            _ => a.name.cmp(&b.name), // If both have same required status, sort by name
        }
    });

    // Display features with colors and status indicators
    for feature in features {
        let is_enabled = registry.feature_is_enabled(&feature.name);
        let is_required = is_required_feature(&feature);

        let status = if is_enabled {
            "[✓]".green()
        } else {
            "[ ]".red()
        };

        let name = if is_required {
            format!("{} (required)", feature.name).yellow()
        } else {
            feature.name.normal()
        };

        println!("{} {} - {}", status, name, feature.description);

        // Show dependencies
        if !feature.dependencies.is_empty() {
            println!(
                "   Dependencies: {}",
                feature.dependencies.join(", ").blue()
            );
        }

        // Show size impact (it's a usize, not an Option)
        println!(
            "   Size Impact: {}",
            format!("{} KB", feature.size_impact).cyan()
        );

        // Empty line between features for readability
        println!();
    }
}

/// List all available features
fn list_features(registry: &FeatureRegistry, matches: &clap::ArgMatches) {
    let format = matches.get_one::<String>("format").unwrap();

    match format.as_str() {
        "json" => {
            // Simple JSON output
            println!("{{");
            println!("  \"features\": [");

            // Get sorted features
            let mut features = registry.feature_list();
            features.sort_by(|a, b| {
                let a_required = is_required_feature(a);
                let b_required = is_required_feature(b);

                // Use a safe comparison approach
                match (a_required, b_required) {
                    (true, false) => std::cmp::Ordering::Greater,
                    (false, true) => std::cmp::Ordering::Less,
                    _ => a.name.cmp(&b.name), // If both have same required status, sort by name
                }
            });

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

            // Get sorted features
            let mut features = registry.feature_list();
            features.sort_by(|a, b| {
                let a_required = is_required_feature(a);
                let b_required = is_required_feature(b);

                // Use a safe comparison approach
                match (a_required, b_required) {
                    (true, false) => std::cmp::Ordering::Greater,
                    (false, true) => std::cmp::Ordering::Less,
                    _ => a.name.cmp(&b.name), // If both have same required status, sort by name
                }
            });

            for feature in features {
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

            // Group features by category
            let mut categories: std::collections::HashMap<String, Vec<FeatureInfo>> =
                std::collections::HashMap::new();

            for feature in registry.feature_list() {
                categories
                    .entry(feature.category.clone())
                    .or_default()
                    .push(feature.clone());
            }

            // Display features by category, with required features at bottom
            for (category, mut features) in categories {
                println!("{}", category.cyan().bold());

                // Sort features by required status using a safe comparison
                features.sort_by(|a, b| {
                    let a_required = is_required_feature(a);
                    let b_required = is_required_feature(b);

                    // Use a safe comparison approach
                    match (a_required, b_required) {
                        (true, false) => std::cmp::Ordering::Greater,
                        (false, true) => std::cmp::Ordering::Less,
                        _ => a.name.cmp(&b.name), // If both have same required status, sort by name
                    }
                });

                for feature in features {
                    let status = if registry.feature_is_enabled(&feature.name) {
                        "✅".green()
                    } else {
                        "❌".red()
                    };

                    let required_label = if is_required_feature(&feature) {
                        " (required)"
                    } else {
                        ""
                    };

                    println!(
                        "{} {}{} - {}",
                        status,
                        feature.name,
                        required_label.yellow(),
                        feature.description
                    );
                }
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

    // Check if the feature is already enabled
    if registry.feature_is_enabled(feature) {
        println!("Feature '{}' is already enabled.", feature);
        return Ok(());
    }

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

    // Check if the feature is already disabled
    if !registry.feature_is_enabled(feature) {
        println!("Feature '{}' is already disabled.", feature);
        return Ok(());
    }

    disable_feature_interactive(registry, feature)
}

/// Show current feature status
fn show_feature_status(registry: &FeatureRegistry) {
    println!("{}", "Feature Status".green().bold());

    let enabled_count = registry.get_enabled_features().len();
    let total_count = registry.feature_list().len();
    let percentage = (enabled_count as f32 / total_count as f32) * 100.0;

    println!(
        "Enabled: {} of {} features ({:.0}%)",
        enabled_count.to_string().green(),
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

    // Display features by category, with required features at bottom
    for (category, mut features) in categories {
        println!("{}", category.cyan().bold());

        // Sort features by required status using a safe comparison
        features.sort_by(|a, b| {
            let a_required = is_required_feature(a);
            let b_required = is_required_feature(b);

            // Use a safe comparison approach
            match (a_required, b_required) {
                (true, false) => std::cmp::Ordering::Greater,
                (false, true) => std::cmp::Ordering::Less,
                _ => a.name.cmp(&b.name), // If both have same required status, sort by name
            }
        });

        for feature in features {
            let status = if registry.feature_is_enabled(&feature.name) {
                "✅".green()
            } else {
                "❌".red()
            };

            let required_label = if is_required_feature(&feature) {
                " (required)"
            } else {
                ""
            };

            println!(
                "{} {}{} - {}",
                status,
                feature.name,
                required_label.yellow(),
                feature.description
            );
        }
    }
}

/// Load a feature registry from config file or create new one if not found
fn load_feature_registry() -> Result<FeatureRegistry, FeatureError> {
    // Create a fresh registry first with default feature definitions
    let mut registry = FeatureRegistry::new();

    // Add sample features - this just defines them but doesn't enable them yet
    add_sample_features(&mut registry)?;

    // Get list of required features
    let required_features: HashSet<String> = registry
        .feature_list()
        .iter()
        .filter(|f| is_required_feature(f))
        .map(|f| f.name.clone())
        .collect();

    // First disable all features to start from a clean slate
    let feature_names: Vec<String> = registry
        .feature_list()
        .iter()
        .map(|f| f.name.clone())
        .collect();

    for feature_name in &feature_names {
        let _ = registry.disable_feature(feature_name);
    }

    // Try to load enabled features from config file
    let config_result = FeatureConfig::load_default();

    match config_result {
        Ok(config) => {
            // If we have saved selections in the config file, use those
            if !config.selected_features.is_empty() {
                // Create a combined set of features to enable - from config plus required
                let mut features_to_enable = config.selected_features.clone();
                for req_feature in &required_features {
                    features_to_enable.insert(req_feature.clone());
                }

                // Enable features in dependency order
                let mut enabled = HashSet::new();
                let mut retry_count = 0;
                let max_retries = features_to_enable.len() * 2; // Avoid infinite loops

                while enabled.len() < features_to_enable.len() && retry_count < max_retries {
                    for feature in &features_to_enable {
                        if !enabled.contains(feature) {
                            if let Ok(_) = registry.enable_feature(feature) {
                                enabled.insert(feature.clone());
                            }
                        }
                    }
                    retry_count += 1;
                }

                // Save the configuration to ensure it's in sync
                save_feature_configuration(&registry)?;
                println!("✅ Loaded feature configuration from file");
            } else {
                // If config file exists but is empty, enable required features
                ensure_required_features_enabled(&mut registry, &required_features)?;
                save_feature_configuration(&registry)?;
                println!("✅ Updated empty configuration with defaults");
            }
        }
        Err(_) => {
            // No existing config found - enable required features
            ensure_required_features_enabled(&mut registry, &required_features)?;
            save_feature_configuration(&registry)?;
            println!("✅ Created new feature configuration");
        }
    }

    Ok(registry)
}

/// Ensure required features are enabled
fn ensure_required_features_enabled(
    registry: &mut FeatureRegistry,
    required_features: &HashSet<String>,
) -> Result<(), FeatureError> {
    for feature in required_features {
        if !registry.feature_is_enabled(feature) {
            match registry.enable_feature(feature) {
                Ok(_) => println!("✓ Enabled required feature: {}", feature),
                Err(e) => return Err(e),
            }
        }
    }
    Ok(())
}

/// Ensure default features are enabled
fn ensure_default_features_enabled(registry: &mut FeatureRegistry) -> Result<(), FeatureError> {
    registry.enable_feature("core")?;
    registry.enable_feature("metrics")?;
    registry.enable_feature("auth")?;
    registry.enable_feature("caching")?;
    Ok(())
}

/// Save feature configuration to disk
fn save_feature_configuration(registry: &FeatureRegistry) -> Result<(), FeatureError> {
    // Get list of currently enabled features
    let selected_features = registry
        .get_enabled_features()
        .iter()
        .cloned()
        .collect::<HashSet<_>>();

    // Create a new config with these selected features
    let config = FeatureConfig {
        selected_features,
        build_config: std::collections::HashMap::new(),
    };

    // Save the configuration
    config.save_default()
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

    println!("{}", "Feature configuration applied!".green());
    println!("To build with these features, run:");
    println!("{}", cmd.cyan());

    // Ask using Select instead of Confirm
    let build_options = vec!["Yes, build now", "No, I'll build later"];
    let build_choice = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Build the project now with these features?")
        .default(1)
        .items(&build_options)
        .interact()
        .unwrap_or(1);

    if build_choice == 0 {
        // Run the build command
        println!("{}", "Building project...".green());

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
        tags: vec!["essential".to_string(), "required".to_string()], // Add required tag
        size_impact: 100,
    };
    registry.register(core_info);

    // Error handling (also required)
    let error_handling_info = FeatureInfo {
        name: "error_handling".to_string(),
        description: "Error handling and reporting".to_string(),
        category: "Essential".to_string(),
        dependencies: vec![],
        default_enabled: true, // Changed to true since it's required
        tags: vec!["essential".to_string(), "required".to_string()], // Add required tag
        size_impact: 150,
    };
    registry.register(error_handling_info);

    // Config system (also required)
    let config_info = FeatureInfo {
        name: "config".to_string(),
        description: "Configuration system".to_string(),
        category: "Essential".to_string(),
        dependencies: vec![],
        default_enabled: true, // Changed to true since it's required
        tags: vec!["essential".to_string(), "required".to_string()], // Add required tag
        size_impact: 180,
    };
    registry.register(config_info);

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

    // We'll let load_feature_registry handle enabling default features
    Ok(())
}

/// Build the main interactive menu
fn build_interactive_menu(_registry: &FeatureRegistry) -> Vec<String> {
    // Create menu items
    let menu_items = vec![
        "Select Features (Interactive)".to_string(),
        "Show Feature Status".to_string(),
        "Apply Configuration".to_string(),
        "Exit".to_string(),
    ];

    menu_items
}
