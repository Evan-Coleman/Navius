use navius_auth::{
    basic::{BasicProvider, BasicProviderConfig, MockUser},
    AuthProvider, Credentials, Error, Role, Subject,
};
use std::sync::Arc;
use tracing::Level;

async fn main_func() -> Result<(), Error> {
    // Set up basic tracing
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    // Create a mock provider for testing
    let provider = create_auth_provider();
    let provider_arc = Arc::new(provider) as Arc<dyn AuthProvider>;

    // Authenticate users with different credentials
    println!("\nTesting authentication:");

    // Authenticate with valid credentials
    match authenticate(Arc::clone(&provider_arc), "user", "password").await {
        Ok(_) => println!("✅ Regular user authenticated successfully"),
        Err(e) => println!("❌ Regular user authentication failed: {}", e),
    }

    // Authenticate with valid admin credentials
    match authenticate(Arc::clone(&provider_arc), "admin", "admin123").await {
        Ok(_) => println!("✅ Admin user authenticated successfully"),
        Err(e) => println!("❌ Admin user authentication failed: {}", e),
    }

    // Authenticate with invalid credentials
    match authenticate(Arc::clone(&provider_arc), "user", "wrong-password").await {
        Ok(_) => println!("❓ Authentication succeeded with wrong password (unexpected)"),
        Err(e) => println!("✅ Authentication failed as expected: {}", e),
    }

    // Check roles and permissions
    println!("\nTesting authorization:");

    // Get user identity
    let user_identity = match provider_arc
        .authenticate(&Credentials {
            username: "user".to_string(),
            password: "password".to_string(),
        })
        .await
    {
        Ok(identity) => {
            println!("✅ Got user identity");
            identity
        }
        Err(e) => {
            println!("❌ Failed to get user identity: {}", e);
            return Err(e);
        }
    };

    // Get admin identity
    let admin_identity = match provider_arc
        .authenticate(&Credentials {
            username: "admin".to_string(),
            password: "admin123".to_string(),
        })
        .await
    {
        Ok(identity) => {
            println!("✅ Got admin identity");
            identity
        }
        Err(e) => {
            println!("❌ Failed to get admin identity: {}", e);
            return Err(e);
        }
    };

    // Display roles
    println!("\nUser roles:");
    for role in &user_identity.roles {
        println!("- {}", role.name);
    }

    println!("\nAdmin roles:");
    for role in &admin_identity.roles {
        println!("- {}", role.name);
    }

    // Simple role check
    let has_admin = admin_identity.roles.iter().any(|r| r.name == "admin");
    let has_user = user_identity.roles.iter().any(|r| r.name == "admin");

    println!("\nRole checks:");
    println!("Regular user has admin role: {}", has_user);
    println!("Admin user has admin role: {}", has_admin);

    Ok(())
}

fn create_auth_provider() -> BasicProvider {
    let config = BasicProviderConfig {
        secret_key: "example-secret-key".to_string(),
        token_expiry: 3600,    // 1 hour
        hash_passwords: false, // No hashing for simplicity in examples
        mock_users: vec![
            MockUser {
                id: "user-1".to_string(),
                username: "user".to_string(),
                password: "password".to_string(),
                display_name: Some("Regular User".to_string()),
                email: Some("user@example.com".to_string()),
                roles: vec!["user".to_string()],
                permissions: vec!["read".to_string()],
            },
            MockUser {
                id: "admin-1".to_string(),
                username: "admin".to_string(),
                password: "admin123".to_string(),
                display_name: Some("Admin User".to_string()),
                email: Some("admin@example.com".to_string()),
                roles: vec!["admin".to_string(), "user".to_string()],
                permissions: vec![
                    "read".to_string(),
                    "write".to_string(),
                    "delete".to_string(),
                ],
            },
        ],
    };

    BasicProvider::new("example-provider".to_string(), config)
}

async fn authenticate(
    provider: Arc<dyn AuthProvider>,
    username: &str,
    password: &str,
) -> Result<(), Error> {
    let credentials = Credentials {
        username: username.to_string(),
        password: password.to_string(),
    };

    // Just try authentication and return success/failure
    provider.authenticate(&credentials).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if let Err(err) = main_func().await {
        eprintln!("Error: {}", err);
    }

    Ok(())
}

/*
 * To run this example:
 *
 * ```
 * cargo run --example auth_checker --features basic
 * ```
 */
