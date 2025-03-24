//! Middleware module for Navius application

// Re-export middleware components from their respective modules
pub use crate::core::auth::middleware::*;

/// Auth middleware
pub mod auth {
    use axum::http::StatusCode;
    use axum::response::IntoResponse;

    #[derive(Debug, Clone)]
    pub enum AuthError {
        InvalidToken,
        MissingToken,
        TokenExpired,
        UnauthorizedRole,
        Other(String),
    }

    #[derive(Debug, Clone)]
    pub enum TokenError {
        RequestError(String),
        ResponseError(String),
        UnexpectedResponse(String),
    }

    /// Auth check handler
    pub async fn auth_check() -> impl IntoResponse {
        StatusCode::OK
    }
}

/// Metrics middleware
pub mod metrics {
    use axum::extract::Request;
    use axum::http::StatusCode;
    use axum::middleware::Next;
    use axum::response::IntoResponse;

    pub async fn metrics_middleware(req: Request, next: Next) -> impl IntoResponse {
        // Record metrics about the request
        let response = next.run(req).await;
        response
    }
}

/// Router middleware
pub mod router {
    use axum::extract::Request;
    use axum::http::StatusCode;
    use axum::middleware::Next;
    use axum::response::IntoResponse;

    pub async fn router_middleware(req: Request, next: Next) -> impl IntoResponse {
        // Process the request through the router
        let response = next.run(req).await;
        response
    }
}

/// Health check middleware
pub mod health {
    use axum::http::StatusCode;
    use axum::response::IntoResponse;

    /// Health check handler
    pub async fn health_check() -> impl IntoResponse {
        StatusCode::OK
    }
}
