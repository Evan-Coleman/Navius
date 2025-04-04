use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{delete, get, post},
};
use navius_cache::{Cache, CacheKey, CacheOperations, CacheOptions};
use navius_cache_redis_plugin::{RedisCache, RedisCacheConfig};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

/// This example demonstrates how to integrate the Redis cache with Axum web framework
///
/// To run this example:
/// ```bash
/// # Start a Redis server if not already running
/// docker run --name redis-test -p 6379:6379 -d redis
///
/// # Run the example
/// cargo run --example axum_integration
///
/// # Then use curl or a browser to interact with the API:
/// # Get the user (will be empty initially)
/// curl http://localhost:3000/users/123
///
/// # Create a user
/// curl -X POST -H "Content-Type: application/json" \
///      -d '{"name":"John Doe","email":"john@example.com"}' \
///      http://localhost:3000/users/123
///
/// # Get the user again
/// curl http://localhost:3000/users/123
///
/// # Delete the user
/// curl -X DELETE http://localhost:3000/users/123
/// ```

// Define a User model for our API
#[derive(Debug, Serialize, Deserialize, Clone)]
struct User {
    id: String,
    name: String,
    email: String,
}

// Create an AppState that holds the Redis cache
struct AppState {
    cache: Arc<RedisCache>,
}

// API handlers
async fn get_user(
    Path(user_id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let cache_key = format!("user:{}", user_id);

    match state.cache.get::<_, User>(&cache_key).await {
        Ok(Some(user)) => {
            // User found in cache
            (StatusCode::OK, Json(user)).into_response()
        }
        Ok(None) => {
            // User not found in cache
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "error": "User not found"
                })),
            )
                .into_response()
        }
        Err(err) => {
            // Cache error occurred
            eprintln!("Cache error: {}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Internal server error"
                })),
            )
                .into_response()
        }
    }
}

#[derive(Debug, Deserialize)]
struct CreateUserRequest {
    name: String,
    email: String,
}

async fn create_user(
    Path(user_id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateUserRequest>,
) -> impl IntoResponse {
    let user = User {
        id: user_id.clone(),
        name: payload.name,
        email: payload.email,
    };

    let cache_key = format!("user:{}", user_id);

    // Store user in cache with a TTL of 1 hour
    let options = CacheOptions {
        ttl: Some(Duration::from_secs(3600)), // 1 hour
    };

    match state.cache.set(&cache_key, &user, Some(options)).await {
        Ok(_) => (StatusCode::CREATED, Json(user)).into_response(),
        Err(err) => {
            eprintln!("Cache error: {}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Internal server error"
                })),
            )
                .into_response()
        }
    }
}

async fn delete_user(
    Path(user_id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let cache_key = format!("user:{}", user_id);

    match state.cache.delete(&cache_key).await {
        Ok(true) => {
            // User was successfully deleted
            StatusCode::NO_CONTENT.into_response()
        }
        Ok(false) => {
            // User not found
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "error": "User not found"
                })),
            )
                .into_response()
        }
        Err(err) => {
            // Cache error occurred
            eprintln!("Cache error: {}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Internal server error"
                })),
            )
                .into_response()
        }
    }
}

async fn health_check(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match state.cache.health_check().await {
        Ok(_) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "status": "UP",
                "cache": "Redis connection healthy"
            })),
        )
            .into_response(),
        Err(err) => {
            eprintln!("Health check error: {}", err);
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({
                    "status": "DOWN",
                    "cache": format!("Redis connection error: {}", err)
                })),
            )
                .into_response()
        }
    }
}

// App router configuration
fn app_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route(
            "/users/:id",
            get(get_user).post(create_user).delete(delete_user),
        )
        .route("/health", get(health_check))
        .with_state(app_state)
}

#[tokio::main]
async fn main() {
    // Initialize tracing for better logging
    tracing_subscriber::fmt::init();

    println!("Redis Cache with Axum Example");
    println!("----------------------------");

    // Create Redis cache configuration
    let config = RedisCacheConfig::new(
        "redis://localhost:6379".to_string(),
        "axum-example:".to_string(),
        Duration::from_secs(3600), // 1 hour
    );

    // Create Redis cache instance
    println!("Connecting to Redis...");
    let cache = match RedisCache::new(config).await {
        Ok(cache) => {
            println!("Successfully connected to Redis");
            cache
        }
        Err(err) => {
            eprintln!("Failed to connect to Redis: {}", err);
            eprintln!("Please make sure Redis is running at localhost:6379");
            std::process::exit(1);
        }
    };

    // Create the application state
    let app_state = Arc::new(AppState {
        cache: Arc::new(cache),
    });

    // Configure the API router
    let app = app_router(app_state);

    // Define the address to listen on
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    println!("Starting server at http://{}", addr);
    println!("Available routes:");
    println!("  GET    /users/:id      - Get a user by ID");
    println!("  POST   /users/:id      - Create a user with ID");
    println!("  DELETE /users/:id      - Delete a user by ID");
    println!("  GET    /health         - Health check");

    // Start the HTTP server
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
