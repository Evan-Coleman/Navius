use navius_auth::{
    basic::BasicProviderConfig, basic::MockUser, AuthProvider, BasicProvider, Credentials, Subject,
};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Set up a basic tracing subscriber for logging
    tracing_subscriber::fmt::init();

    // Create a Basic provider with a test configuration
    let config = BasicProviderConfig {
        secret_key: "example-secret-key".to_string(),
        token_expiry: 3600,    // 1 hour
        hash_passwords: false, // No hashing for simplicity in examples
        mock_users: vec![
            MockUser {
                id: "user-1".to_string(),
                username: "alice".to_string(),
                password: "password123".to_string(),
                display_name: Some("Alice Smith".to_string()),
                email: Some("alice@example.com".to_string()),
                roles: vec!["user".to_string()],
                permissions: vec!["read".to_string()],
            },
            MockUser {
                id: "user-2".to_string(),
                username: "bob".to_string(),
                password: "letmein".to_string(),
                display_name: Some("Bob Johnson".to_string()),
                email: Some("bob@example.com".to_string()),
                roles: vec!["admin".to_string(), "user".to_string()],
                permissions: vec![
                    "read".to_string(),
                    "write".to_string(),
                    "delete".to_string(),
                ],
            },
        ],
    };

    // Create the authentication provider
    let provider = BasicProvider::new("example-provider".to_string(), config);

    // Authenticate a user
    let credentials = Credentials {
        username: "alice".to_string(),
        password: "password123".to_string(),
    };

    // Authenticate and get user identity
    println!("Authenticating user: {}", credentials.username);
    let identity = provider.authenticate(&credentials).await?;
    println!("Authentication successful for {}", identity.username);
    println!("User identity: {:#?}", identity);

    // Create a subject from the identity for token generation
    let subject = Subject {
        id: "user-1".to_string(), // Use the actual ID from the mock user
        subject_type: "user".to_string(),
        name: identity
            .display_name
            .unwrap_or_else(|| identity.username.clone()),
        roles: identity.roles.clone(),
        attributes: None,
    };

    // Create a token for the subject
    println!("\nCreating token for subject");
    let token = provider.create_token(&subject).await?;
    println!("Token created: {}", token);

    // Validate the token
    println!("\nValidating token");
    let validated_subject = provider.validate_token(&token).await?;
    println!("Token validation successful");
    println!("Validated subject: {:#?}", validated_subject);

    // Revoke the token
    println!("\nRevoking token");
    provider.revoke_token(&token).await?;
    println!("Token revoked");

    // Try to validate the revoked token (should fail)
    println!("\nTrying to validate revoked token");
    match provider.validate_token(&token).await {
        Ok(_) => println!("Token validation succeeded (unexpected)"),
        Err(e) => println!("Token validation failed as expected: {}", e),
    }

    // Test with incorrect credentials
    let invalid_credentials = Credentials {
        username: "alice".to_string(),
        password: "wrongpassword".to_string(),
    };

    println!("\nTrying to authenticate with incorrect credentials");
    match provider.authenticate(&invalid_credentials).await {
        Ok(_) => println!("Authentication succeeded (unexpected)"),
        Err(e) => println!("Authentication failed as expected: {}", e),
    }

    Ok(())
}
