use navius::auth::AuthError;
use navius::auth::providers::ProviderConfig;
use navius::auth::providers::ProviderRegistry;
use navius::config::AppConfig;
use serde_json;

// Helper function to load test configuration
fn load_test_config() -> AppConfig {
    let mut config = AppConfig::default();
    config.auth.enabled = true;
    config.auth.default_provider = "test".to_string();

    // Create a minimal providers config - structure may vary based on your actual implementation
    // This is just a placeholder that will need to be updated based on your actual ProviderConfig structure
    config.auth.providers = Some(
        serde_json::from_str(
            r#"[
        {
            "name": "test",
            "jwks_url": "https://example.com/.well-known/jwks.json", 
            "tenant_id": "test-tenant",
            "audience": "api://test",
            "issuer": "https://example.com/issuer",
            "jwks_refresh_minutes": 60,
            "debug": true
        }
    ]"#,
        )
        .expect("Failed to parse provider config"),
    );

    config
}

// Helper function to generate a valid test token
fn generate_valid_token() -> String {
    // For tests, we return a static token that the mock provider will accept
    "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJ0ZXN0LXVzZXIiLCJuYW1lIjoiVGVzdCBVc2VyIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c".to_string()
}

#[tokio::test]
async fn full_auth_workflow() {
    let config = load_test_config();
    let registry = ProviderRegistry::from_app_config(&config).unwrap();

    // Test valid token flow
    let valid_token = generate_valid_token();
    let claims = registry
        .default_provider()
        .validate_token(&valid_token)
        .await
        .unwrap();
    assert_eq!(claims.sub, "test-user");

    // Test invalid token
    assert!(matches!(
        registry.default_provider().validate_token("invalid").await,
        Err(AuthError::ValidationFailed(_))
    ));

    // Test rate limiting
    for _ in 0..5 {
        assert!(registry.default_provider().refresh_jwks().await.is_ok());
    }
    assert!(matches!(
        registry.default_provider().refresh_jwks().await,
        Err(AuthError::RateLimited(_))
    ));

    // Test circuit breaker
    for _ in 0..5 {
        let _ = registry.default_provider().validate_token("invalid").await;
    }
    assert!(matches!(
        registry.default_provider().validate_token("valid").await,
        Err(AuthError::CircuitOpen)
    ));
}
