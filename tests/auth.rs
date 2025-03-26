use navius::auth::AuthError;
use navius::auth::providers::ProviderRegistry;
use navius::config::AppConfig;
use navius::config::app_config::ProviderConfig;
use serde_json;
use std::collections::HashMap;

// Helper function to load test configuration
fn load_test_config() -> AppConfig {
    let mut config = AppConfig::default();
    config.auth.enabled = true;
    config.auth.default_provider = "entra".to_string();

    // Create a minimal providers config
    let provider_config = serde_json::from_str::<ProviderConfig>(
        r#"{
            "enabled": true,
            "client_id": "test-client",
            "jwks_uri": "https://example.com/.well-known/jwks.json", 
            "issuer_url": "https://example.com/issuer",
            "audience": "api://test",
            "role_mappings": {
                "admin": ["admin_role"],
                "read_only": ["reader_role"],
                "full_access": ["full_role"]
            },
            "provider_specific": {
                "tenant_id": "test-tenant"
            }
        }"#,
    )
    .expect("Failed to parse provider config");

    let mut providers = HashMap::new();
    providers.insert("entra".to_string(), provider_config);
    config.auth.providers = providers;

    config
}

// Helper function to generate a valid test token
fn generate_valid_token() -> String {
    // For tests, we return a static token that the mock provider will accept
    "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJ0ZXN0LXVzZXIiLCJuYW1lIjoiVGVzdCBVc2VyIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c".to_string()
}

#[tokio::test]
async fn test_provider_registry_creation() {
    let config = load_test_config();
    let registry_result = ProviderRegistry::from_app_config(&config);

    // Just verify that we can create the registry without errors
    assert!(
        registry_result.is_ok(),
        "Should be able to create ProviderRegistry"
    );

    let registry = registry_result.unwrap();
    assert_eq!(registry.default_provider, "entra");
}
