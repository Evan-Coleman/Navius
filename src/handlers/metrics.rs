use axum::extract::State;
use axum::http::header;
use axum::response::{IntoResponse, Response};
use std::sync::Arc;

use crate::app::AppState;
use crate::models::MetricsResponse;

/// Handler for the metrics endpoint
#[utoipa::path(
    get,
    path = "/metrics",
    responses(
        (status = 200, description = "Prometheus metrics in text format", content_type = "text/plain", body = MetricsResponse)
    ),
    tag = "metrics"
)]
pub async fn metrics(State(state): State<Arc<AppState>>) -> Response {
    let metrics_text = crate::metrics::metrics_handler(&state.metrics_handle).await;

    // Return with text/plain content type
    ([(header::CONTENT_TYPE, "text/plain")], metrics_text).into_response()
}
