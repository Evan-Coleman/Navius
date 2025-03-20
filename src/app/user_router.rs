use axum::{
    Json,
    extract::{Path, State},
    routing::{Router, get, post},
};
use std::sync::Arc;
use tracing::info;

use crate::{app::AppState, auth::EntraAuthLayer, handlers::examples::pet};

/// Example User Router for demonstrating how to add custom routes
pub struct UserRouter;

impl UserRouter {
    /// Create custom user routes that can be modified by developers
    pub fn create_user_routes(state: Arc<AppState>) -> Router {
        // Define whether auth is enabled
        let auth_enabled = state.config.auth.enabled;

        // Create auth middleware for different access levels
        let readonly_auth = EntraAuthLayer::from_app_config_require_read_only_role(&state.config);
        let fullaccess_auth =
            EntraAuthLayer::from_app_config_require_full_access_role(&state.config);

        // 1. PUBLIC ROUTES - available without authentication
        let public_routes = Router::new()
            .route("/pet/{id}", get(pet::fetch_pet_handler))
            // Add more public routes here
            .route("/hello", get(|| async { "Hello, World!" }));

        // 2. READ-ONLY ROUTES - requires basic authentication
        let readonly_routes = Router::new()
            .route("/pet/{id}", get(pet::fetch_pet_handler))
            // Add more read-only routes here
            ;

        // 3. FULL ACCESS ROUTES - requires full access role
        let fullaccess_routes = Router::new()
            .route("/pet/{id}", get(pet::fetch_pet_handler))
            // Add more full access routes here
            ;

        // Apply authentication layers if enabled
        let (readonly_routes, fullaccess_routes) = if auth_enabled {
            (
                readonly_routes.layer(readonly_auth),
                fullaccess_routes.layer(fullaccess_auth),
            )
        } else {
            // No auth enabled
            (readonly_routes, fullaccess_routes)
        };

        // Combine user-defined routes
        Router::new()
            .merge(public_routes)
            .nest("/read", readonly_routes)
            .nest("/full", fullaccess_routes)
            .with_state(state)
            .with_state(()) // Strip the state to return a Router<()>
    }
}
