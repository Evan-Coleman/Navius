use axum::{
    Json,
    extract::{Path, State},
    routing::{Router, get, post},
};
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::info;

use crate::{
    core::auth::EntraAuthLayer,
    core::router::{AppState, create_core_app_router, init_app_state},
    handlers::examples::pet,
};

/// Create custom user routes that can be modified by developers
fn create_user_routes(state: Arc<AppState>) -> Router {
    // Define whether auth is enabled
    let auth_enabled = state.config.auth.enabled;

    // Create auth middleware for different access levels
    let readonly_auth = EntraAuthLayer::from_app_config_require_read_only_role(&state.config);
    let fullaccess_auth = EntraAuthLayer::from_app_config_require_full_access_role(&state.config);

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
}

/// Create the application router by combining core routes with user routes
pub fn create_router(state: Arc<AppState>) -> Router {
    // Get user-defined routes
    let user_routes = create_user_routes(state.clone());

    // Create the core app router with user routes
    create_core_app_router(state, user_routes)
}

/// Initialize the application
///
/// Note: The Router returned here has type Router<Arc<AppState>>. When passing to the server
/// in main.rs, it needs to be used with the appropriate serving method.
pub async fn init() -> (Router, SocketAddr) {
    // Initialize app state and get server address
    let (state, addr) = init_app_state().await;

    // Create router with app state
    let app = create_router(state);

    (app, addr)
}
