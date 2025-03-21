use super::*;

// Test functionality for the config module
#[test]
fn test_default_config() {
    let config = AppConfig::default();
    assert!(!config.server.host.is_empty());
}
