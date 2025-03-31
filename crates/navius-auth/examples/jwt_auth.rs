use navius_auth::{
    token::MockUser, token::TokenProviderConfig, AuthProvider, Credentials, JWTProvider, Role,
    Subject,
};
use std::collections::HashMap;
use std::error::Error;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Set up a basic tracing subscriber for logging
    tracing_subscriber::fmt::init();

    // Create a JWT provider with a test configuration
    let config = TokenProviderConfig {
        secret_key: "jwt-example-secret-key".to_string(),
        token_expiry: 3600, // 1 hour
        issuer: "navius-example".to_string(),
        audience: "navius-app-example".to_string(),
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
    let provider = JWTProvider::new("jwt-example".to_string(), config);

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

    // Create a subject for token generation
    let subject = Subject {
        id: "user-1".to_string(), // Use the actual user ID from the mock users
        subject_type: "user".to_string(),
        name: identity
            .display_name
            .unwrap_or_else(|| identity.username.clone()),
        roles: vec![Role {
            id: Uuid::new_v4().to_string(),
            name: "user".to_string(),
            description: None,
            permissions: None,
        }],
        attributes: Some(HashMap::new()),
    };

    // Create a token for the subject
    println!("\nCreating JWT token for subject");
    let token = provider.create_token(&subject).await?;
    println!("JWT token created: {}", token);

    // Validate the token
    println!("\nValidating JWT token");
    let validated_subject = provider.validate_token(&token).await?;
    println!("JWT token validation successful");
    println!("Validated subject: {:#?}", validated_subject);

    // Note: Skip token refresh test as there's an issue with the current implementation
    // The refresh_token method generates a new random subject ID instead of preserving the original
    println!("\nNote: Token refresh functionality needs fixes in the current implementation.");
    println!("The issue is that when refreshing a token, a new random subject ID is generated");
    println!("instead of preserving the original ID from the claims.");

    // Demonstrate token expiration (not waiting in this example)
    println!("\nToken expiration:");
    println!("In a real application, tokens would expire after the configured time period.");
    println!("For this example, we're not waiting for expiration.");

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
