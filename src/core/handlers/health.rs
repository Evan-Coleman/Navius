use axum::{Json, extract::State};
use chrono;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use crate::core::router::AppState;
use crate::models::{DependencyStatus, DetailedHealthResponse, HealthCheckResponse};

/// Handler for the simple health check endpoint
///
/// This endpoint is designed for load balancers and monitoring systems.
/// It returns minimal information, usually just whether the service is running.
/// It also includes a basic check of the Petstore API's availability.
pub async fn health_check(State(state): State<Arc<AppState>>) -> Json<HealthCheckResponse> {
    // Calculate uptime
    let uptime = SystemTime::now()
        .duration_since(state.start_time)
        .unwrap_or_default();

    // Check Petstore API connectivity
    let mut deps = Vec::new();

    // Add a quick check of the Petstore API
    if !state.config.api.petstore_url.is_empty() {
        let petstore_status = check_petstore_connectivity(&state).await;
        deps.push(petstore_status);
    }

    // Simple health check with dependency info
    Json(HealthCheckResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: uptime.as_secs(),
        dependencies: Some(deps),
    })
}

/// Check if the Petstore API is available
///
/// Performs a lightweight health check to the Petstore API
async fn check_petstore_connectivity(state: &Arc<AppState>) -> DependencyStatus {
    let url = format!("{}/store/inventory", state.config.api.petstore_url);

    // Set a short timeout for health checks (2 seconds)
    let timeout = std::time::Duration::from_secs(2);

    let client = &state.client;
    let mut builder = client.get(&url).timeout(timeout);

    // Add API key if configured
    if let Some(api_key) = &state.config.api.api_key {
        builder = builder.header("api_key", api_key);
    }

    // Perform the request
    let status = match builder.send().await {
        Ok(response) => {
            if response.status().is_success() {
                "up".to_string()
            } else {
                format!("degraded (status {})", response.status())
            }
        }
        Err(e) => {
            let error_message = e.to_string();
            let truncated_message = if error_message.len() > 100 {
                format!("{}...", &error_message[..100])
            } else {
                error_message
            };
            format!("down ({})", truncated_message)
        }
    };

    // Add details with timestamp of the health check
    let mut details = BTreeMap::new();
    details.insert("checked_at".to_string(), chrono::Utc::now().to_rfc3339());

    details.insert("endpoint".to_string(), "/store/inventory".to_string());

    DependencyStatus {
        name: "petstore_api".to_string(),
        status,
        details: Some(details),
    }
}

/// Handler for the detailed health check endpoint
///
/// This endpoint provides comprehensive information about the service's health,
/// including component statuses, configs, and internal metrics.
/// This is typically secured in production environments.
pub async fn detailed_health_check(
    State(state): State<Arc<AppState>>,
) -> Json<DetailedHealthResponse> {
    // Calculate uptime
    let uptime = SystemTime::now()
        .duration_since(state.start_time)
        .unwrap_or_default();

    // Gather status of dependencies
    let mut dependencies = Vec::new();

    // Database status
    if state.config.database.enabled {
        if let Some(db_pool) = &state.db_pool {
            // Check database connectivity
            let db_status = match db_pool.ping().await {
                Ok(_) => {
                    let mut details = BTreeMap::new();

                    // Add database connection info
                    details.insert(
                        "connection_url".to_string(),
                        if state.config.endpoint_security.expose_sensitive_info {
                            state.config.database.url.clone()
                        } else {
                            // Hide sensitive connection details
                            let parts: Vec<&str> = state.config.database.url.split('@').collect();
                            if parts.len() > 1 {
                                format!("postgres://*****@{}", parts[1])
                            } else {
                                "postgres://*****".to_string()
                            }
                        },
                    );

                    // Add connection stats if available
                    // The stats method is only available in the old DatabaseConnection trait
                    // We'll need to add this info in a different way for the new PgPool trait
                    // For now, just add a placeholder
                    details.insert(
                        "max_connections".to_string(),
                        state.config.database.max_connections.to_string(),
                    );

                    // Add connection timeout info
                    details.insert(
                        "connect_timeout_seconds".to_string(),
                        state.config.database.connect_timeout_seconds.to_string(),
                    );

                    if let Some(idle_timeout) = state.config.database.idle_timeout_seconds {
                        details
                            .insert("idle_timeout_seconds".to_string(), idle_timeout.to_string());
                    }

                    DependencyStatus {
                        name: "database".to_string(),
                        status: "up".to_string(),
                        details: Some(details),
                    }
                }
                Err(e) => {
                    let mut details = BTreeMap::new();
                    details.insert("error".to_string(), e.to_string());
                    details.insert("checked_at".to_string(), chrono::Utc::now().to_rfc3339());

                    DependencyStatus {
                        name: "database".to_string(),
                        status: "down".to_string(),
                        details: Some(details),
                    }
                }
            };

            dependencies.push(db_status);
        } else {
            // Database is enabled but not initialized
            let mut details = BTreeMap::new();
            details.insert(
                "error".to_string(),
                "Database enabled but not initialized".to_string(),
            );

            dependencies.push(DependencyStatus {
                name: "database".to_string(),
                status: "down".to_string(),
                details: Some(details),
            });
        }
    } else {
        // Database not enabled
        let mut details = BTreeMap::new();
        details.insert("enabled".to_string(), "false".to_string());

        dependencies.push(DependencyStatus {
            name: "database".to_string(),
            status: "disabled".to_string(),
            details: Some(details),
        });
    }

    // Cache status with only enabled flag
    let mut cache_details = BTreeMap::new();
    cache_details.insert(
        "enabled".to_string(),
        state.config.cache.enabled.to_string(),
    );

    dependencies.push(DependencyStatus {
        name: "cache".to_string(),
        status: if state.config.cache.enabled {
            "up"
        } else {
            "disabled"
        }
        .to_string(),
        details: Some(cache_details),
    });

    // Auth status with enhanced details
    let auth_details = if state.config.auth.enabled {
        let mut details = BTreeMap::new();
        details.insert("provider".to_string(), "Entra ID".to_string());

        // Include client ID status (not the actual ID)
        details.insert(
            "client_id_status".to_string(),
            if state.config.auth.entra.client_id.is_empty() {
                "not set".to_string()
            } else {
                "configured".to_string()
            },
        );

        // Include tenant ID status
        details.insert(
            "tenant_id_status".to_string(),
            if state.config.auth.entra.tenant_id.is_empty() {
                "not set".to_string()
            } else {
                "configured".to_string()
            },
        );

        // Include audience status
        details.insert(
            "audience_status".to_string(),
            if state.config.auth.entra.audience.is_empty() {
                "not set".to_string()
            } else {
                "configured".to_string()
            },
        );

        // Include role configurations
        details.insert(
            "admin_roles_configured".to_string(),
            (!state.config.auth.entra.admin_roles.is_empty()).to_string(),
        );

        details.insert(
            "read_only_roles_configured".to_string(),
            (!state.config.auth.entra.read_only_roles.is_empty()).to_string(),
        );

        details.insert(
            "full_access_roles_configured".to_string(),
            (!state.config.auth.entra.full_access_roles.is_empty()).to_string(),
        );

        // Include auth debug mode
        details.insert(
            "debug_mode".to_string(),
            state.config.auth.debug.to_string(),
        );

        Some(details)
    } else {
        None
    };

    dependencies.push(DependencyStatus {
        name: "authentication".to_string(),
        status: if state.config.auth.enabled {
            "enabled"
        } else {
            "disabled"
        }
        .to_string(),
        details: auth_details,
    });

    // Build full response
    Json(DetailedHealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: uptime.as_secs(),
        environment: state.config.environment.to_string(),
        dependencies,
    })
}
