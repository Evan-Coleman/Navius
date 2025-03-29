use axum::{
    extract::{Request, State},
    http::{self, HeaderMap, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use navius_auth::{
    basic::{BasicProvider, BasicProviderConfig, MockUser},
    middleware::{extract_token, AuthChecker, AuthConfig, AuthLayer},
    Credentials, Subject,
};
use serde::{Deserialize, Serialize};
use std::{error::Error, net::SocketAddr, sync::Arc};
use tower_http::cors::{Any, CorsLayer};
use tracing::{info, instrument};

#[derive(Clone)]
struct AppState {
    provider: Arc<BasicProvider>,
}

#[derive(Debug, Serialize)]
struct UserResponse {
    id: String,
    username: String,
    roles: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Debug, Serialize)]
struct LoginResponse {
    token: String,
    user: UserResponse,
}

#[derive(Debug, Serialize)]
struct MessageResponse {
    message: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Set up a basic tracing subscriber for logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Create a Basic authentication provider
    let provider = create_auth_provider();

    // Create app state
    let state = AppState {
        provider: Arc::new(provider),
    };

    // Create the authentication middleware
    let auth_layer = AuthLayer::with_config(
        Arc::clone(&state.provider) as Arc<dyn navius_auth::AuthProvider>,
        AuthConfig {
            required: true,
            header_name: "Authorization".to_string(),
            token_prefix: "Bearer".to_string(),
            include_identity: true,
            exempt_paths: vec!["/api/login".to_string(), "/api/public".to_string()],
        },
    );

    // Create a simple admin middleware
    let admin_middleware = middleware::from_fn(admin_middleware);

    // Create a CORS layer
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([http::Method::GET, http::Method::POST])
        .allow_headers([http::header::AUTHORIZATION, http::header::CONTENT_TYPE]);

    // Build the router
    let app = Router::new()
        .route("/api/login", post(login_handler))
        .route("/api/public", get(public_handler))
        .route("/api/private", get(private_handler))
        .route("/api/admin", get(admin_handler))
        .layer(admin_middleware)
        .with_state(state)
        .layer(cors)
        .layer(auth_layer);

    // Run the server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("Server listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

fn create_auth_provider() -> BasicProvider {
    let config = BasicProviderConfig {
        secret_key: "example-secret-key".to_string(),
        token_expiry: 3600,    // 1 hour
        hash_passwords: false, // No hashing for simplicity in examples
        mock_users: vec![
            MockUser {
                id: "user-1".to_string(),
                username: "user".to_string(),
                password: "password".to_string(),
                display_name: Some("Regular User".to_string()),
                email: Some("user@example.com".to_string()),
                roles: vec!["user".to_string()],
                permissions: vec!["read".to_string()],
            },
            MockUser {
                id: "admin-1".to_string(),
                username: "admin".to_string(),
                password: "admin123".to_string(),
                display_name: Some("Admin User".to_string()),
                email: Some("admin@example.com".to_string()),
                roles: vec!["admin".to_string(), "user".to_string()],
                permissions: vec![
                    "read".to_string(),
                    "write".to_string(),
                    "delete".to_string(),
                ],
            },
        ],
    };

    BasicProvider::new("example-provider".to_string(), config)
}

#[instrument(skip(state))]
async fn login_handler(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Create credentials
    let credentials = Credentials {
        username: request.username,
        password: request.password,
    };

    // Authenticate user
    let identity = state.provider.authenticate(&credentials).await?;

    // Create subject for token generation
    let subject = Subject {
        id: identity.id.clone(),
        subject_type: "user".to_string(),
        name: identity
            .display_name
            .clone()
            .unwrap_or_else(|| identity.username.clone()),
        roles: identity.roles.clone(),
        attributes: None,
    };

    // Generate token
    let token = state.provider.create_token(&subject).await?;

    // Create response
    let user_response = UserResponse {
        id: identity.id,
        username: identity.username,
        roles: identity.roles.into_iter().map(|r| r.name).collect(),
    };

    Ok((
        StatusCode::OK,
        Json(LoginResponse {
            token,
            user: user_response,
        }),
    ))
}

async fn public_handler() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(MessageResponse {
            message: "This is a public endpoint - no authentication required".to_string(),
        }),
    )
}

async fn private_handler(headers: HeaderMap) -> impl IntoResponse {
    // The identity headers will be added by the auth middleware
    let identity = headers
        .get("X-Identity")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");

    (
        StatusCode::OK,
        Json(MessageResponse {
            message: format!(
                "This is a protected endpoint - authenticated as: {}",
                identity
            ),
        }),
    )
}

async fn admin_handler() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(MessageResponse {
            message: "This is an admin endpoint - you have admin privileges".to_string(),
        }),
    )
}

// Custom middleware to check for admin role
async fn admin_middleware(headers: HeaderMap, request: Request, next: Next) -> Response {
    // Check if user has admin role
    let roles = headers
        .get("X-Roles")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();

    if !roles.contains("admin") {
        return AppError::Forbidden(
            "You don't have admin privileges to access this endpoint".to_string(),
        )
        .into_response();
    }

    next.run(request).await
}

// Simple error type for the example
#[derive(Debug)]
enum AppError {
    Authentication(String),
    Forbidden(String),
    Internal(String),
}

impl From<navius_auth::Error> for AppError {
    fn from(err: navius_auth::Error) -> Self {
        if err.is_authentication_error() {
            AppError::Authentication(err.to_string())
        } else if err.is_authorization_error() {
            AppError::Forbidden(err.to_string())
        } else {
            AppError::Internal(err.to_string())
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::Authentication(msg) => (StatusCode::UNAUTHORIZED, msg),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = Json(serde_json::json!({
            "error": {
                "message": message,
            }
        }));

        (status, body).into_response()
    }
}

/*
To test this example:

1. Run the server: `cargo run --example auth_middleware --features http`

2. Public endpoint (no auth required):
   curl http://localhost:3000/api/public

3. Login to get a token:
   curl -X POST http://localhost:3000/api/login \
     -H "Content-Type: application/json" \
     -d '{"username": "user", "password": "password"}'

   This will return a token.

4. Access the protected endpoint with the token:
   curl http://localhost:3000/api/private \
     -H "Authorization: Bearer YOUR_TOKEN_HERE"

5. Try to access the admin endpoint with a regular user token:
   curl http://localhost:3000/api/admin \
     -H "Authorization: Bearer YOUR_TOKEN_HERE"

   This should fail with a 403 Forbidden.

6. Login as admin to get an admin token:
   curl -X POST http://localhost:3000/api/login \
     -H "Content-Type: application/json" \
     -d '{"username": "admin", "password": "admin123"}'

7. Access the admin endpoint with the admin token:
   curl http://localhost:3000/api/admin \
     -H "Authorization: Bearer ADMIN_TOKEN_HERE"

   This should succeed.
*/
