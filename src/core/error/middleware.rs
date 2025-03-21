use axum::{
    body::Body,
    extract::MatchedPath,
    http::{
        Request, Response, StatusCode,
        header::{self, HeaderValue},
    },
    response::IntoResponse,
};
use metrics::counter;
use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    time::Instant,
};
use tower::{Layer, Service};
use tracing::{Span, error, info};
use tracing_futures::Instrument;
use uuid::Uuid;

use crate::error::{AppError, ErrorResponse, log_error};

/// Generate a unique request ID
pub fn generate_request_id() -> String {
    Uuid::new_v4().to_string()
}

/// Request tracking middleware
#[derive(Clone)]
pub struct RequestTrackingLayer {
    pub service_name: String,
}

impl RequestTrackingLayer {
    pub fn new(service_name: &str) -> Self {
        Self {
            service_name: service_name.to_string(),
        }
    }
}

impl<S> Layer<S> for RequestTrackingLayer {
    type Service = RequestTrackingMiddleware<S>;

    fn layer(&self, service: S) -> Self::Service {
        RequestTrackingMiddleware {
            inner: service,
            service_name: self.service_name.clone(),
        }
    }
}

#[derive(Clone)]
pub struct RequestTrackingMiddleware<S> {
    inner: S,
    service_name: String,
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for RequestTrackingMiddleware<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>> + Send + 'static,
    S::Future: Send + 'static,
    ReqBody: Send + 'static,
    ResBody: Send + 'static,
    S::Error: Into<Box<dyn std::error::Error + Send + Sync>> + Send,
{
    type Response = Response<ResBody>;
    type Error = Box<dyn std::error::Error + Send + Sync>;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, mut req: Request<ReqBody>) -> Self::Future {
        // Generate unique request ID
        let request_id = generate_request_id();
        req.extensions_mut().insert(RequestId(request_id.clone()));

        // Get path for metrics
        let path = req
            .extensions()
            .get::<MatchedPath>()
            .map(|matched_path| matched_path.as_str())
            .unwrap_or_else(|| req.uri().path())
            .to_string();

        // Record request metrics
        let _ = counter!("http.requests.total", "path" => path.clone(), "method" => req.method().to_string());

        // Create a span for tracing
        let span = tracing::info_span!(
            "request",
            service = %self.service_name,
            request_id = %request_id,
            method = %req.method(),
            path = %path,
            remote_addr = %req.extensions().get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
                .map(|connect_info| connect_info.0.to_string())
                .unwrap_or_else(|| "unknown".to_string()),
        );

        info!(parent: &span, "Request started");

        let start = Instant::now();
        let future = self.inner.call(req);

        let service_name = self.service_name.clone();
        let path_string = path;

        // Process the response
        let future = async move {
            // Use instrumentation to attach the span to the future
            match future.instrument(span.clone()).await {
                Ok(mut response) => {
                    let status = response.status();
                    let duration = start.elapsed();

                    // Add request id header to response
                    let header_value = HeaderValue::from_str(&request_id)
                        .unwrap_or_else(|_| HeaderValue::from_static("invalid-id"));
                    response.headers_mut().insert(
                        header::HeaderName::from_static("x-request-id"),
                        header_value,
                    );

                    // Record response metrics
                    let _ = counter!(
                        "http.responses.total",
                        "path" => path_string.clone(),
                        "status" => status.as_u16().to_string()
                    );

                    if status.is_server_error() {
                        error!(
                            parent: &span,
                            status = %status.as_u16(),
                            duration_ms = %duration.as_millis(),
                            service = %service_name,
                            request_id = %request_id,
                            "Request failed with server error"
                        );
                    } else {
                        info!(
                            parent: &span,
                            status = %status.as_u16(),
                            duration_ms = %duration.as_millis(),
                            service = %service_name,
                            request_id = %request_id,
                            "Request completed successfully"
                        );
                    }

                    Ok(response)
                }
                Err(err) => {
                    let err = err.into();
                    let app_err = AppError::InternalError(format!("{}", err));
                    let status = app_err.status_code();
                    let duration = start.elapsed();

                    // Log the error
                    log_error(
                        &app_err,
                        Some(format!("Path: {}", path_string)),
                        Some(request_id.clone()),
                    );

                    // Record error metrics
                    let _ = counter!(
                        "http.errors.total",
                        "path" => path_string.clone(),
                        "status" => status.as_u16().to_string(),
                        "error_type" => app_err.error_type()
                    );

                    error!(
                        parent: &span,
                        status = %status.as_u16(),
                        duration_ms = %duration.as_millis(),
                        service = %service_name,
                        request_id = %request_id,
                        error_type = %app_err.error_type(),
                        "Request failed with error"
                    );

                    // We need to convert from Response<Body> to Response<ResBody>
                    Err(Box::new(app_err) as Box<dyn std::error::Error + Send + Sync>)
                }
            }
        };

        Box::pin(future)
    }
}

/// Request ID for tracking
#[derive(Debug, Clone)]
pub struct RequestId(pub String);

/// Extension trait to get request ID from request extensions
pub trait RequestIdExt {
    fn request_id(&self) -> Option<&str>;
}

impl<B> RequestIdExt for Request<B> {
    fn request_id(&self) -> Option<&str> {
        self.extensions().get::<RequestId>().map(|id| id.0.as_str())
    }
}
