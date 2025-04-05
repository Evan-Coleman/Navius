// Use navius-core for core functionality
use navius_core::{
    config::Config,
    di::{ApplicationBuilder, ApplicationContext, ComponentRef, ComponentRegistry},
    error::Result,
    logging,
};

// Use navius-auth types and provider
use navius_auth::middleware::AuthLayer;
use navius_auth::token::{JWTProvider, TokenProviderConfig};
use navius_auth::types::Claims; // Import auth middleware

// Use navius-http for server, routing, and middleware
use axum::{
    Router, middleware as axum_middleware,
    routing::{get, post},
};
use navius_http::middleware::{
    cors_layer, default_middleware, logging_layer, permissive_cors_layer, request_id_layer,
    timeout_layer,
}; // Import common middleware
use navius_http::server::HttpServer; // Axum imports for routing

// Other necessary imports
use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::Deserialize;
use std::sync::Arc;
use tower::ServiceBuilder;

// Use navius-db types
use navius_db::config::DatabaseConfig;
use navius_db::pool::Pool;

// Remove mock Claims
// #[derive(Debug, Clone, Deserialize)]
// pub struct Claims {
//     pub uid: u64,
// }

// Remove mock JWT module
// mod jwt_mock {
//     use super::Claims;
//     pub fn encode(claims: Claims) -> Result<String> {
//         Ok(format!("mock_token_for_uid_{}", claims.uid))
//     }
// }

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Initialize Application Context (Config, Logging, DI)
    let app_builder = ApplicationBuilder::new()
        .with_config_path("config/default.toml")
        .with_config_path_optional("config/local.toml")
        .with_env_prefix("APP");

    let temp_config = app_builder.build_config()?;

    // JWT Provider Registration
    let jwt_config = temp_config.get::<TokenProviderConfig>("jwt")?;
    let jwt_provider = Arc::new(JWTProvider::new("jwt_main".to_string(), jwt_config));
    let app_builder = app_builder.register_component(jwt_provider.clone());

    // Database Pool Registration
    let db_config = temp_config.get::<DatabaseConfig>("database")?;
    let db_pool = Pool::new(db_config).await?; // Create the pool
    let app_builder = app_builder.register_component(db_pool); // Register the pool component

    let app_context = Arc::new(app_builder.build()?);
    let config = app_context.config();

    tracing::info!(
        "Navius application starting with config keys: {:?}",
        config.keys()
    );

    // 2. Define Application Routes
    let api_routes = Router::new()
        .route("/", get(hello_world))
        .route("/hello_world", get(hello_world))
        .route("/hello/:name", get(hello).post(hello))
        .route("/login", post(login))
        .route("/user-info", get(protected_user_info))
        .route_layer(axum_middleware::from_fn_with_state(
            jwt_provider.clone(),
            navius_auth::middleware::authenticate,
        ));

    // Define SQL routes
    let sql_router = Router::new()
        .route("/version", get(sql::sqlx_request_handler))
        .route("/now", get(sql::sqlx_time_handler));

    let app_router = Router::new()
        // .nest("/api", api_routes) // Can nest later if needed
        .merge(api_routes)
        .nest("/sql", sql_router) // Nest the SQL routes under /sql
        .layer(
            ServiceBuilder::new()
                // Add layers from navius-http
                .layer(request_id_layer())
                .layer(logging_layer())
                .layer(permissive_cors_layer())
                .layer(timeout_layer()),
        )
        .with_state(app_context.clone());

    // 3. Configure and Start the HTTP Server
    let server_host = config
        .get::<String>("server.host")
        .unwrap_or_else(|_| "127.0.0.1".to_string());
    let server_port = config.get::<u16>("server.port").unwrap_or(3000);
    let addr = format!("{}:{}", server_host, server_port);

    tracing::info!("Starting HTTP server on {}", addr);

    HttpServer::new(addr)
        .serve(app_router)
        .await
        .map_err(|e| navius_core::error::Error::internal(format!("HTTP server failed: {}", e)))?;

    tracing::info!("Server shutdown complete");
    Ok(())
}

// --- Route Handlers ---

// Handlers now receive AppContext via State extractor
async fn hello_world(State(_app_context): State<Arc<ApplicationContext>>) -> impl IntoResponse {
    "hello world"
}

async fn hello(
    State(_app_context): State<Arc<ApplicationContext>>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    format!("hello {name}")
}

#[derive(Deserialize)]
struct LoginCredentials {
    username: String,
    password: String,
}

// Login handler uses ComponentRef from AppContext implicitly now
async fn login(
    State(app_context): State<Arc<ApplicationContext>>,
    Json(credentials): Json<LoginCredentials>,
) -> Result<impl IntoResponse> {
    let jwt_provider = app_context.get_component::<JWTProvider>()?;
    let LoginCredentials { username, password } = credentials;

    if username == "root" && password == "correct_password" {
        let user_id = "user-1000";
        match jwt_provider
            .generate_token(user_id, Some(vec!["admin".to_string()]))
            .await
        {
            Ok(token) => Ok((StatusCode::OK, token)),
            Err(e) => {
                tracing::error!("Failed to generate token: {}", e);
                Err(navius_core::error::Error::internal(
                    "Failed to generate token",
                ))
            }
        }
    } else {
        Err(navius_core::error::Error::authentication(
            "Username or password incorrect",
        ))
    }
}

// Placeholder for custom config structure (if needed beyond core config)
#[derive(Deserialize, Debug)] // Added Debug
struct CustomConfig {
    user_info_detail: String,
}

// Protected route handler - Claims should be injected by middleware
// The AuthLayer applied earlier should handle this.
async fn protected_user_info(
    State(app_context): State<Arc<ApplicationContext>>,
    claims: Claims, // Extracted by navius_auth::middleware::authenticate
) -> Result<impl IntoResponse> {
    // Example: Get custom config if registered as a component
    // let custom_config = app_context.get_component::<CustomConfig>()?;
    // let details = &custom_config.user_info_detail;
    let details = "Mock details from handler"; // Placeholder

    let user_id = claims.sub;
    Ok(format!("get user info of id#{}: {}", user_id, details))
}

// --- Route Handlers (SQL) ---
mod sql {
    use axum::extract::State;
    use axum::response::IntoResponse;
    use navius_core::di::ApplicationContext;
    use navius_core::error::Result;
    use navius_db::pool::Pool; // Import Pool
    use sqlx::postgres::PgPool;
    use std::sync::Arc; // Need specific pool type if using raw sqlx

    // #[get("/version")] // Target style
    // Handler now receives the Pool component via AppContext
    pub async fn sqlx_request_handler(
        State(app_context): State<Arc<ApplicationContext>>,
    ) -> Result<impl IntoResponse> {
        let pool = app_context.get_component::<Pool>()?;
        // Use the pool (from navius-db) to execute a query
        // Example: Fetch PostgreSQL version using raw sqlx query
        // Note: Requires sqlx dependency directly if using raw queries
        let version: (String,) = sqlx::query_as("SELECT version()")
            .fetch_one(pool.inner::<PgPool>()?) // Get underlying PgPool
            .await
            .map_err(|e| navius_core::error::Error::database(format!("DB query failed: {}", e)))?;
        Ok(version.0)
    }

    // #[get("/now")] // Target style
    pub async fn sqlx_time_handler(
        State(app_context): State<Arc<ApplicationContext>>,
    ) -> Result<impl IntoResponse> {
        let pool = app_context.get_component::<Pool>()?;
        // Example: Fetch current timestamp
        let now: (chrono::DateTime<chrono::Utc>,) = sqlx::query_as("SELECT NOW()")
            .fetch_one(pool.inner::<PgPool>()?)
            .await
            .map_err(|e| navius_core::error::Error::database(format!("DB query failed: {}", e)))?;
        Ok(now.0.to_rfc3339())
    }
}

// Remove the old App struct and AppBuilder as they are replaced by navius-core and navius-http
// pub mod app { ... }
// pub mod plugins { ... }
