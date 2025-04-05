// Use navius-core for core functionality
use navius_core::{
    // config::Config as NaviusConfig, // Keep commented out until config type fixed
    di::{Application, ApplicationBuilder},
    error::Result,
};

// Use navius-http for server config and middleware stubs
use axum::{
    Router,
    routing::IntoMakeService, // Needed for axum::serve
};
// Middleware imports kept for potential future use, but commented out
// use navius_http::middleware::{
//     logging_layer, permissive_cors_layer, request_id_layer, timeout_layer,
// };
use navius_http::server::{HttpServerConfig, ShutdownReceiver, ShutdownSender};

// Import the necessary macros
use navius_macros::{nest, route};

// Other necessary imports
use axum::extract::State;
use axum::response::IntoResponse;
use std::ops::Deref;
use std::sync::Arc;
use tracing::{error, info};

// Removed DB imports
// use sqlx::postgres::{PgPool, PgPoolOptions};
// use sqlx::ConnectOptions;
// use std::str::FromStr;

// Removed Repository imports
// mod repository;
// use repository::{DbRepository, DynDbRepository, PostgresRepository};

// Config imports
use config::Config as ExternalConfig;
// use config::ConfigError as ExternalConfigError;

// TODO: Re-enable when JWTProvider is added
// use navius_auth::{JWTProvider, TokenProviderConfig};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    info!("Tracing initialized.");

    // --- Configuration ---
    let temp_config: ExternalConfig = ExternalConfig::builder()
        .add_source(config::File::with_name("config/default.toml").required(false))
        .add_source(config::File::with_name("config/local.toml").required(false))
        .add_source(config::Environment::with_prefix("NAVIUS").separator("__"))
        .build()
        .map_err(|e| {
            navius_core::error::Error::configuration(format!("Config build error: {}", e))
        })?;
    info!("Configuration loaded.");

    // --- Application Context & DI ---
    info!("Initializing application builder...");
    // TODO: Use ApplicationBuilder::with_config once config type mismatch is resolved
    let mut app_builder = ApplicationBuilder::new();
    info!("Application builder created.");

    // --- JWT Provider (Placeholder) ---
    // TODO: Fix unresolved type errors (E0412, E0433) and uncomment
    // let jwt_config = temp_config.get::<TokenProviderConfig>("jwt")?;
    // let jwt_provider = Arc::new(JWTProvider::new("jwt_main".to_string(), jwt_config));
    // app_builder.add_component(jwt_provider)?;
    // info!("JWT Provider registered.");

    // --- REMOVED: Database Pool ---

    // --- REMOVED: Repository ---

    // --- Build Application Context & State ---
    info!("Building application...");
    let app: Application = app_builder.build();
    let app_arc = Arc::new(app);
    // AppState now only holds the Application Arc (no db pool/repo needed)
    let app_state = AppState {
        app: app_arc.clone(),
    };
    info!("Application built and state created.");

    // --- Router Setup ---
    info!("Building router manually...");
    let api_router = api::__navius_router_api(); // Get router from #[nest]
    let merged_router = Router::new().nest("/api", api_router); // Nest under /api
    let app_router = merged_router.with_state(app_state); // Apply state
    let app_service = app_router.into_make_service(); // Convert for axum::serve
    info!("Router setup complete.");

    // --- Server Configuration & Shutdown Channel ---
    let host = temp_config
        .get_string("server.host")
        .unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = temp_config.get_int("server.port").unwrap_or(8080) as u16;
    let (shutdown_tx, shutdown_rx) = HttpServerConfig::create_shutdown_channel();
    let server_config = HttpServerConfig::new()
        .with_host_and_port(&host, port)
        .with_shutdown(shutdown_tx.clone());

    // --- Server Start ---
    info!("Binding TCP listener...");
    let listener = server_config.bind_listener().await?;
    let local_addr = listener.local_addr()?;
    info!("Server listening on {}", local_addr);

    info!("Starting Axum server with graceful shutdown...");
    let server_task = tokio::spawn(async move {
        axum::serve(listener, app_service)
            .with_graceful_shutdown(HttpServerConfig::create_shutdown_future(shutdown_rx))
            .await
    });

    // --- Wait for Shutdown Signal (Ctrl+C) ---
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            info!("Ctrl+C received, sending shutdown signal...");
            if let Err(e) = shutdown_tx.send() {
                 error!("Failed to send shutdown signal: {}", e);
            }
        }
        res = server_task => {
            match res {
                Ok(Ok(())) => info!("Server completed normally."),
                Ok(Err(e)) => error!("Axum server error: {}", e),
                Err(e) => error!("Server task join error: {}", e),
            }
        }
    }

    // Wait for the server task to fully finish after signal
    if let Err(e) = server_task.await {
        error!("Final server task join error: {}", e);
    }

    info!("Server shut down gracefully.");
    Ok(())
}

// API Module - Simplified, only hello and context routes
#[nest(prefix = "/v1")]
mod api {
    use super::*;
    use axum::extract::State;
    use navius_macros::route;

    #[route(method = "GET", path = "/hello")]
    pub async fn hello_world() -> &'static str {
        "Hello, world!"
    }

    // Removed db_version and db_time handlers

    // Keep context_info handler as it uses the core DI registry
    #[route(method = "GET", path = "/context")]
    #[axum::debug_handler]
    async fn context_info(State(state): State<AppState>) -> String {
        // Access registry through AppState -> Application
        format!("Components: {:?}", state.app.registry().component_types())
    }
}

// Application state - Simplified, only holds Application Arc
#[derive(Clone)]
struct AppState {
    app: Arc<Application>,
}

// Removed AppState::get_repository impl

// ... Placeholder middleware functions commented out ...
