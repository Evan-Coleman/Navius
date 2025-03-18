use axum::extract::State;
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use std::sync::Arc;
use tracing::info;

use crate::app::AppState;
use crate::models::MetricsResponse;

/// Get Prometheus metrics
///
/// Returns Prometheus metrics in text format
pub async fn metrics(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    info!("📊 Getting Prometheus metrics");
    let metrics = state.metrics_handle.render();
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/plain")],
        metrics,
    )
}
