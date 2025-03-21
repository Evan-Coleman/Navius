use super::*;

// Test functionality for the core config module
#[test]
fn test_constants() {
    // Test that constants are properly defined
    assert!(!constants::auth::env_vars::TENANT_ID.is_empty());
    assert!(!constants::auth::env_vars::CLIENT_ID.is_empty());
    assert!(!constants::auth::env_vars::CLIENT_SECRET.is_empty());
}
