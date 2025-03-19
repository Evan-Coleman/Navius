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
    // Extract the method
    let method = req.method().to_string();

    // Extract matched route if available (and create an owned copy)
    let matched_path = req
        .extensions()
        .get::<axum::extract::MatchedPath>()
        .map(|mp| mp.as_str().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    debug!("🔍 Request: {} {}", method, matched_path);

    let response = next.run(req).await;

    info!(
        "📋 Response: {} {} - {}",
        method,
        matched_path,
        response.status()
    );

    Ok(response)
}
