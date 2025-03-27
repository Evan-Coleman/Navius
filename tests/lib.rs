// Main test module for integration tests
// This re-exports all test modules for proper test discovery

// Authentication tests
pub mod auth;

// CLI tests
pub mod cli_command_tests;
pub mod cli_tests;
pub mod feature_builder_tests;
pub mod features_cli_args_tests;
pub mod features_cli_interactive_tests;
pub mod features_cli_main_tests;
pub mod features_cli_tests;

// Documentation tests
pub mod documentation_custom_templates_tests;
pub mod documentation_edge_cases_tests;
pub mod documentation_tests;

// Feature system tests
pub mod feature_system_tests;
