use axum::{
    extract::State,
    routing::{Router, get},
};
use std::sync::Arc;

use crate::{
    core::auth::EntraAuthLayer,
    handlers::{self, actuator, health},
};

use super::AppState;

/// Core router containing essential routes that should not be modified by users
pub struct CoreRouter;

impl CoreRouter {
    /// Create the essential core routes that should not be modified by users
    pub fn create_core_routes(state: Arc<AppState>) -> Router {
        // Define whether auth is enabled
        let auth_enabled = state.config.auth.enabled;

        // Create auth middleware for admin access
        let admin_auth = EntraAuthLayer::from_app_config_require_admin_role(&state.config);

        // Public core routes - accessible without authentication
        let public_routes = Router::new().route("/health", get(health::health_check));

        // Actuator routes - for metrics, health checks, docs, and admin functions
        let actuator_routes = Router::new()
            .route("/health", get(health::detailed_health_check))
            .route("/info", get(actuator::info))
            .route("/docs", get(handlers::docs::swagger_ui_handler))
            .route("/docs/{*file}", get(handlers::docs::openapi_spec_handler));

        // Apply authentication layers if enabled
        let actuator_routes = if auth_enabled {
            actuator_routes.layer(admin_auth)
        } else {
            actuator_routes
        };

        // Return only the core routes
        Router::new()
            .merge(public_routes)
            .nest("/actuator", actuator_routes)
            .with_state(state)
    }
}
