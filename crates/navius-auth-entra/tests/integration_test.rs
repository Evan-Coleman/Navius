use navius_auth::AuthProvider;
use navius_auth_entra::{EntraConfig, EntraConfigBuilder, EntraProvider};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[tokio::test]
async fn test_provider_creation() {
    let config = create_test_config();
    let provider = EntraProvider::new(config);
    assert!(provider.is_ok());

    let provider = provider.unwrap();
    assert_eq!(provider.provider_name(), "entra");
}

#[tokio::test]
async fn test_authentication_with_invalid_token() {
    let config = create_test_config();
    let provider = EntraProvider::new(config).unwrap();

    // Try to authenticate with an invalid token
    let result = provider.authenticate("invalid-token").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_authorization() {
    let config = create_test_config();
    let provider = EntraProvider::new(config).unwrap();

    // Test with missing required roles
    let context = navius_auth::AuthorizationContext {
        user_id: "test-user".to_string(),
        user_roles: vec!["User".to_string()],
        required_roles: vec!["Admin".to_string()],
        resource: None,
        action: None,
    };

    let result = provider.authorize(&context).await;
    assert!(result.is_ok());
    assert!(!result.unwrap());

    // Test with matching roles
    let context = navius_auth::AuthorizationContext {
        user_id: "test-user".to_string(),
        user_roles: vec!["User".to_string(), "Admin".to_string()],
        required_roles: vec!["Admin".to_string()],
        resource: None,
        action: None,
    };

    let result = provider.authorize(&context).await;
    assert!(result.is_ok());
    assert!(result.unwrap());
}

// Helper function to create a test configuration
fn create_test_config() -> EntraConfig {
    EntraConfigBuilder::new()
        .client_id("test-client-id")
        .tenant_id("test-tenant-id")
        .issuer("https://login.microsoftonline.com/test-tenant-id/v2.0")
        .jwks_uri("https://login.microsoftonline.com/test-tenant-id/discovery/v2.0/keys")
        .build()
        .unwrap()
}

#[cfg(feature = "integration_tests")]
mod mock_server_tests {
    use super::*;
    use mockito::Server;
    use serde_json::json;

    // This test requires the "integration_tests" feature to be enabled
    #[tokio::test]
    async fn test_jwks_retrieval() {
        // Start a mock server
        let mut server = Server::new();

        // Mock the JWKS endpoint
        let mock_jwks = json!({
            "keys": [
                {
                    "kty": "RSA",
                    "kid": "test-key-id",
                    "use": "sig",
                    "alg": "RS256",
                    "n": "test-modulus",
                    "e": "AQAB"
                }
            ]
        });

        let mock = server
            .mock("GET", "/keys")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_jwks.to_string())
            .create();

        // Create a provider using the mock server
        let config = EntraConfigBuilder::new()
            .client_id("test-client-id")
            .tenant_id("test-tenant-id")
            .jwks_uri(format!("{}/keys", server.url()))
            .build()
            .unwrap();

        let provider = EntraProvider::new(config).unwrap();

        // The JWKS endpoint should be called when validating a token
        let result = provider
            .authenticate(
                "eyJhbGciOiJSUzI1NiIsImtpZCI6InRlc3Qta2V5LWlkIn0.eyJzdWIiOiJ1c2VyMTIzIn0.signature",
            )
            .await;

        // The authentication should fail because our token is invalid,
        // but the JWKS endpoint should have been called
        assert!(result.is_err());
        mock.assert();
    }
}
