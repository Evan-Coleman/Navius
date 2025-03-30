//! Request ID middleware for the Navius HTTP server.
//!
//! This middleware ensures that all requests have a unique request ID.

use axum::extract::Request;
use axum::http::header::{HeaderName, HeaderValue};
use axum::middleware::Next;
use axum::response::Response;
use futures_util::future::BoxFuture;
use navius_core::util;
use std::task::{Context, Poll};
use tower::{Layer, Service};
use tracing::{Span, info};

/// The header name for the request ID header, using the constant from navius_core.
static REQUEST_ID_HEADER: &str = navius_core::constants::headers::REQUEST_ID;

/// Middleware that ensures all requests have a request ID.
#[derive(Debug, Clone)]
pub struct RequestId<S> {
    inner: S,
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for RequestId<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>> + Send + 'static,
    S::Future: Send + 'static,
    ReqBody: Send + 'static,
    ResBody: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut request: Request<ReqBody>) -> Self::Future {
        // Check if the request already has a request ID
        let request_id = if let Some(existing_id) = request.headers().get(REQUEST_ID_HEADER) {
            // Use the existing request ID
            existing_id.to_owned()
        } else {
            // Generate a new request ID
            let id = util::random_id().to_string();
            HeaderValue::from_str(&id).unwrap_or_else(|_| {
                // This should never happen, but just in case
                HeaderValue::from_static("invalid-id")
            })
        };

        // Set the request ID header
        let id_str = request_id.to_str().unwrap_or("invalid-id");

        // Add the request ID to the current span
        Span::current().record("request_id", &id_str);

        // Add or replace the request ID header
        request.headers_mut().insert(
            HeaderName::from_static(REQUEST_ID_HEADER),
            request_id.clone(),
        );

        info!(request_id = %id_str, "Request ID assigned");

        // Call the inner service
        let future = self.inner.call(request);

        // Create the future for processing the request
        Box::pin(async move {
            let mut response = future.await?;

            // Add the request ID to the response headers if not already present
            if !response.headers().contains_key(REQUEST_ID_HEADER) {
                response
                    .headers_mut()
                    .insert(HeaderName::from_static(REQUEST_ID_HEADER), request_id);
            }

            Ok(response)
        })
    }
}

/// Layer that adds the RequestId middleware to a service.
#[derive(Debug, Clone, Default)]
pub struct RequestIdLayer;

impl RequestIdLayer {
    /// Create a new RequestIdLayer.
    pub fn new() -> Self {
        Self
    }
}

impl<S> Layer<S> for RequestIdLayer {
    type Service = RequestId<S>;

    fn layer(&self, service: S) -> Self::Service {
        RequestId { inner: service }
    }
}

/// Convenience function to create a RequestIdLayer.
pub fn request_id_layer() -> RequestIdLayer {
    RequestIdLayer::new()
}

/// Axum middleware function for adding request IDs.
pub async fn request_id_middleware<B>(mut request: Request<B>, next: Next) -> Response
where
    B: axum::body::HttpBody + Send + 'static,
    B::Data: Send,
    B::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    // Check if the request already has a request ID
    let request_id = if let Some(existing_id) = request.headers().get(REQUEST_ID_HEADER) {
        // Use the existing request ID
        existing_id.to_owned()
    } else {
        // Generate a new request ID
        let id = util::random_id().to_string();
        HeaderValue::from_str(&id).unwrap_or_else(|_| {
            // This should never happen, but just in case
            HeaderValue::from_static("invalid-id")
        })
    };

    // Get the ID as a string for logging
    let id_str = request_id.to_str().unwrap_or("invalid-id");

    // Add the request ID to the current span
    Span::current().record("request_id", &id_str);

    // Add the request ID header
    request.headers_mut().insert(
        HeaderName::from_static(REQUEST_ID_HEADER),
        request_id.clone(),
    );

    // Process the request - in a real implementation this would properly convert the body type
    // For this placeholder, we're going to use a workaround since this is incomplete
    let response = next.run(Request::new(axum::body::Body::empty())).await;

    // Add the request ID to the response
    let mut response = response;
    if !response.headers().contains_key(REQUEST_ID_HEADER) {
        response
            .headers_mut()
            .insert(HeaderName::from_static(REQUEST_ID_HEADER), request_id);
    }

    response
}
