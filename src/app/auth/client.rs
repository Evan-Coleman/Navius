// Copyright (c) 2025 Navius Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#[cfg(feature = "auth")]
//! User authentication extensions and customizations
//!
//! This module allows developers to customize and extend the core authentication functionality.
//! Add your custom auth logic, role mappings, or domain-specific auth requirements here.

use crate::core::auth::{EntraAuthLayer, EntraTokenClient};
use crate::core::config::AppConfig;
use crate::core::error::{AppError, ErrorSeverity, Result};

/// Example: Create a custom auth layer with specific role requirements
pub fn create_custom_auth_layer(_config: &AppConfig) -> EntraAuthLayer {
    // This is an example of how you might create a custom auth layer
    // with specific role requirements for your application

    let required_roles = vec!["CustomRole1".to_string(), "CustomRole2".to_string()];

    // Create auth layer with custom configuration - in a real app, you would use EntraAuthConfig
    // from the configuration or create it with proper values
    EntraAuthLayer::require_any_role(required_roles)
}

/// Example: Create a token client for a specific downstream service
pub async fn create_service_client(
    config: &AppConfig,
    service_name: &str,
) -> Result<reqwest::Client> {
    // Create a token client
    let token_client = EntraTokenClient::from_config(config);

    // Get appropriate scope for the service from configuration or hardcoded mapping
    let scope = match service_name {
        "service1" => "api://service1/.default",
        "service2" => "api://service2/.default",
        _ => {
            return Err(AppError::ValidationError(format!(
                "Unknown service: {}",
                service_name
            )));
        }
    };

    // Create authenticated client for the service
    match token_client.create_client(scope).await {
        Ok(client) => Ok(client),
        Err(err) => Err(AppError::Unauthorized(format!(
            "Failed to create token client: {}",
            err
        ))),
    }
}
