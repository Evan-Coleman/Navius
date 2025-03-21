//! User authentication extensions and customizations
//! THIS FILE IS NOT USED. ONLY HERE FOR REFERENCE.
//! This module allows developers to customize and extend the core authentication functionality.
//! Add your custom auth logic, role mappings, or domain-specific auth requirements here.

use crate::config::app_config::AppConfig;
use crate::core::auth::{EntraAuthLayer, EntraTokenClient};

/// Example: Create a custom auth layer with specific role requirements
pub fn create_custom_auth_layer(config: &AppConfig) -> EntraAuthLayer {
    // This is an example of how you might create a custom auth layer
    // with specific role requirements for your application

    let required_roles = vec!["CustomRole1".to_string(), "CustomRole2".to_string()];

    // Create auth layer with custom configuration
    EntraAuthLayer::new(
        &config.auth.entra.tenant_id,
        &config.auth.entra.audience,
        &config.auth.entra.issuer,
        required_roles,
    )
}

/// Example: Create a token client for a specific downstream service
pub async fn create_service_client(
    config: &AppConfig,
    service_name: &str,
) -> Result<reqwest::Client, String> {
    // Create a token client
    let token_client = EntraTokenClient::from_config(config);

    // Get appropriate scope for the service from configuration or hardcoded mapping
    let scope = match service_name {
        "service1" => "api://service1/.default",
        "service2" => "api://service2/.default",
        _ => return Err(format!("Unknown service: {}", service_name)),
    };

    // Create authenticated client for the service
    token_client.create_client(scope).await
}
