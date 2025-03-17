use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use tracing::{debug, info};

use crate::app::AppState;

/// Middleware for logging requests
pub async fn log_request(
    State(_state): State<Arc<AppState>>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let path = req.uri().path().to_string();
    let method = req.method().to_string();

    debug!("Request: {} {}", method, path);

    let response = next.run(req).await;

    info!("Response: {} {} - {}", method, path, response.status());

    Ok(response)
}
