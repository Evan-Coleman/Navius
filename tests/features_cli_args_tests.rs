use clap::{Arg, Command};
use navius::core::features::{FeatureRegistry, FeatureRegistryExt};
use std::io::{self, Write};
use tempfile::TempDir;

// Function to create the argument parser similar to the one in features_cli.rs
fn create_arg_parser() -> Command {
    Command::new("Navius Feature CLI")
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
        .subcommand(
            Command::new("analyze-deps")
                .about("Analyze dependencies for selected features")
                .arg(
                    Arg::new("output")
                        .short('o')
                        .long("output")
                        .help("Output directory for dependency analysis")
                        .default_value("./analysis"),
                )
                .arg(
                    Arg::new("format")
                        .short('f')
                        .long("format")
                        .help("Output format (text, markdown)")
                        .default_value("markdown"),
                ),
        )
        .subcommand(
            Command::new("build")
                .about("Build with selected features")
                .arg(
                    Arg::new("release")
                        .short('r')
                        .long("release")
                        .help("Build in release mode")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("optimize-deps")
                        .short('d')
                        .long("optimize-deps")
                        .help("Optimize dependencies based on selected features")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("target")
                        .short('t')
                        .long("target")
                        .help("Build for specific target"),
                ),
        )
}

// Helper function to create a test registry and temporary directory
fn create_test_environment() -> (TempDir, FeatureRegistry) {
    let temp_dir = TempDir::new().unwrap();
    let mut registry = FeatureRegistry::new_empty();

    // Add test features (similar to tests/features_cli_tests.rs)
    let basic_feature = navius::core::features::FeatureInfo {
        name: "basic".to_string(),
        description: "Basic feature".to_string(),
        dependencies: vec![],
        default_enabled: true,
        category: "Core".to_string(),
        tags: vec!["core".to_string()],
        size_impact: 100,
    };

    let advanced_feature = navius::core::features::FeatureInfo {
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

    // Enable the default feature
    registry.select("basic").unwrap();

    (temp_dir, registry)
}

#[test]
fn test_list_command_parsing() {
    let app = create_arg_parser();

    // Test default format
    let matches = app
        .clone()
        .try_get_matches_from(vec!["navius-cli", "list"])
        .unwrap();
    let list_matches = matches.subcommand_matches("list").unwrap();
    assert_eq!(list_matches.get_one::<String>("format").unwrap(), "text");

    // Test json format
    let matches = app
        .clone()
        .try_get_matches_from(vec!["navius-cli", "list", "--format", "json"])
        .unwrap();
    let list_matches = matches.subcommand_matches("list").unwrap();
    assert_eq!(list_matches.get_one::<String>("format").unwrap(), "json");

    // Test yaml format
    let matches = app
        .clone()
        .try_get_matches_from(vec!["navius-cli", "list", "-f", "yaml"])
        .unwrap();
    let list_matches = matches.subcommand_matches("list").unwrap();
    assert_eq!(list_matches.get_one::<String>("format").unwrap(), "yaml");

    // Test invalid format (should fail)
    let result = app
        .clone()
        .try_get_matches_from(vec!["navius-cli", "list", "-f", "invalid"]);
    assert!(result.is_err());
}

#[test]
fn test_enable_command_parsing() {
    let app = create_arg_parser();

    // Test with feature name
    let matches = app
        .clone()
        .try_get_matches_from(vec!["navius-cli", "enable", "metrics"])
        .unwrap();
    let enable_matches = matches.subcommand_matches("enable").unwrap();
    assert_eq!(
        enable_matches.get_one::<String>("feature").unwrap(),
        "metrics"
    );

    // Test without feature name (should fail)
    let result = app
        .clone()
        .try_get_matches_from(vec!["navius-cli", "enable"]);
    assert!(result.is_err());
}

#[test]
fn test_disable_command_parsing() {
    let app = create_arg_parser();

    // Test with feature name
    let matches = app
        .clone()
        .try_get_matches_from(vec!["navius-cli", "disable", "metrics"])
        .unwrap();
    let disable_matches = matches.subcommand_matches("disable").unwrap();
    assert_eq!(
        disable_matches.get_one::<String>("feature").unwrap(),
        "metrics"
    );

    // Test without feature name (should fail)
    let result = app
        .clone()
        .try_get_matches_from(vec!["navius-cli", "disable"]);
    assert!(result.is_err());
}

#[test]
fn test_status_command_parsing() {
    let app = create_arg_parser();

    // The status command doesn't have any arguments, so just check that it can be parsed
    let matches = app
        .clone()
        .try_get_matches_from(vec!["navius-cli", "status"])
        .unwrap();
    assert!(matches.subcommand_matches("status").is_some());
}

#[test]
fn test_analyze_deps_command_parsing() {
    let app = create_arg_parser();

    // Test default values
    let matches = app
        .clone()
        .try_get_matches_from(vec!["navius-cli", "analyze-deps"])
        .unwrap();
    let analyze_matches = matches.subcommand_matches("analyze-deps").unwrap();
    assert_eq!(
        analyze_matches.get_one::<String>("output").unwrap(),
        "./analysis"
    );
    assert_eq!(
        analyze_matches.get_one::<String>("format").unwrap(),
        "markdown"
    );

    // Test with custom output directory
    let matches = app
        .clone()
        .try_get_matches_from(vec!["navius-cli", "analyze-deps", "--output", "./custom"])
        .unwrap();
    let analyze_matches = matches.subcommand_matches("analyze-deps").unwrap();
    assert_eq!(
        analyze_matches.get_one::<String>("output").unwrap(),
        "./custom"
    );

    // Test with custom format
    let matches = app
        .clone()
        .try_get_matches_from(vec!["navius-cli", "analyze-deps", "-f", "text"])
        .unwrap();
    let analyze_matches = matches.subcommand_matches("analyze-deps").unwrap();
    assert_eq!(analyze_matches.get_one::<String>("format").unwrap(), "text");
}

#[test]
fn test_build_command_parsing() {
    let app = create_arg_parser();

    // Test default values
    let matches = app
        .clone()
        .try_get_matches_from(vec!["navius-cli", "build"])
        .unwrap();
    let build_matches = matches.subcommand_matches("build").unwrap();
    assert!(!build_matches.get_flag("release"));
    assert!(!build_matches.get_flag("optimize-deps"));
    assert!(build_matches.get_one::<String>("target").is_none());

    // Test with release flag
    let matches = app
        .clone()
        .try_get_matches_from(vec!["navius-cli", "build", "--release"])
        .unwrap();
    let build_matches = matches.subcommand_matches("build").unwrap();
    assert!(build_matches.get_flag("release"));

    // Test with optimize-deps flag
    let matches = app
        .clone()
        .try_get_matches_from(vec!["navius-cli", "build", "--optimize-deps"])
        .unwrap();
    let build_matches = matches.subcommand_matches("build").unwrap();
    assert!(build_matches.get_flag("optimize-deps"));

    // Test with target
    let matches = app
        .clone()
        .try_get_matches_from(vec![
            "navius-cli",
            "build",
            "--target",
            "x86_64-unknown-linux-gnu",
        ])
        .unwrap();
    let build_matches = matches.subcommand_matches("build").unwrap();
    assert_eq!(
        build_matches.get_one::<String>("target").unwrap(),
        "x86_64-unknown-linux-gnu"
    );

    // Test with all options
    let matches = app
        .clone()
        .try_get_matches_from(vec![
            "navius-cli",
            "build",
            "--release",
            "--optimize-deps",
            "--target",
            "x86_64-unknown-linux-gnu",
        ])
        .unwrap();
    let build_matches = matches.subcommand_matches("build").unwrap();
    assert!(build_matches.get_flag("release"));
    assert!(build_matches.get_flag("optimize-deps"));
    assert_eq!(
        build_matches.get_one::<String>("target").unwrap(),
        "x86_64-unknown-linux-gnu"
    );
}

#[test]
fn test_interactive_command_parsing() {
    let app = create_arg_parser();

    // The interactive command doesn't have any arguments, so just check that it can be parsed
    let matches = app
        .clone()
        .try_get_matches_from(vec!["navius-cli", "interactive"])
        .unwrap();
    assert!(matches.subcommand_matches("interactive").is_some());
}

#[test]
fn test_apply_command_parsing() {
    let app = create_arg_parser();

    // The apply command doesn't have any arguments, so just check that it can be parsed
    let matches = app
        .clone()
        .try_get_matches_from(vec!["navius-cli", "apply"])
        .unwrap();
    assert!(matches.subcommand_matches("apply").is_some());
}

#[test]
fn test_no_command_defaults_to_interactive() {
    // In the features_cli.rs binary, when no command is specified, it defaults to interactive mode
    // We can't directly test this behavior in unit tests since it involves the main function,
    // but we can document this behavior here for integration testing
}
