use navius_auth::AuthProvider;
use navius_auth_entra::{EntraConfig, EntraConfigBuilder, EntraProvider};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Create a configuration for the Entra provider
    let config = EntraConfigBuilder::new()
        .client_id("your-client-id")
        .tenant_id("your-tenant-id")
        .build()?;

    // Create the provider
    let provider = EntraProvider::new(config)?;
    println!("Created Entra provider: {}", provider.provider_name());

    // In a real application, you would get a token from an HTTP request
    // or another source. For this example, we'll just show the validation process
    // with a placeholder token.
    let token = "your-jwt-token-from-entra";

    // Authenticate the token (this will fail with our placeholder)
    match provider.authenticate(token).await {
        Ok(auth_info) => {
            println!("Authentication successful!");
            println!("User ID: {}", auth_info.user_id);
            println!("User roles: {:?}", auth_info.roles);
        }
        Err(e) => {
            println!("Authentication failed: {}", e);
            // In a real application, you would return an error response
        }
    }

    // Example of authorization check
    let auth_context = navius_auth::AuthorizationContext {
        user_id: "user123".to_string(),
        user_roles: vec!["User".to_string(), "Editor".to_string()],
        required_roles: vec!["Admin".to_string()],
        resource: Some("document/123".to_string()),
        action: Some("delete".to_string()),
    };

    let is_authorized = provider.authorize(&auth_context).await?;
    if is_authorized {
        println!("User is authorized to perform the action");
    } else {
        println!("User is not authorized to perform the action");
    }

    Ok(())
}

// Additional example showing how to create a provider with more advanced configuration
fn _advanced_configuration_example() -> Result<EntraProvider, Box<dyn std::error::Error>> {
    let config = EntraConfigBuilder::new()
        .client_id("your-client-id")
        .tenant_id("your-tenant-id")
        // Customize the issuer URL if needed
        .issuer("https://login.microsoftonline.com/your-tenant-id/v2.0")
        // Specify custom JWKS URI
        .jwks_uri("https://login.microsoftonline.com/your-tenant-id/discovery/v2.0/keys")
        // Add allowed audiences
        .audience(vec!["api://your-app-id".to_string()])
        // Configure JWKS cache settings
        .jwks_cache_duration(std::time::Duration::from_secs(3600))
        .jwks_refresh_ahead_duration(std::time::Duration::from_secs(300))
        // Allow some clock skew for token validation
        .clock_skew(std::time::Duration::from_secs(60))
        .build()?;

    let provider = EntraProvider::new(config)?;
    Ok(provider)
}
