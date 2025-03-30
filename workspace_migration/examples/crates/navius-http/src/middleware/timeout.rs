//! Timeout middleware for the Navius HTTP server.
//!
//! This middleware provides configurable timeout functionality for HTTP requests,
//! automatically terminating requests that take too long to process.

use axum::{
    extract::Request,
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::{future::Future, pin::Pin, time::Duration};
use tower::timeout::TimeoutLayer as TowerTimeoutLayer;
use tower::{Layer, Service};
use tracing::{info, warn};

/// Default timeout duration (30 seconds)
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// Configuration for timeout middleware.
#[derive(Debug, Clone)]
pub struct TimeoutConfig {
    /// The default timeout duration.
    timeout: Duration,
    /// Path-specific timeouts that override the default.
    path_timeouts: Vec<(String, Duration)>,
    /// Paths that are exempt from timeouts.
    excluded_paths: Vec<String>,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            timeout: DEFAULT_TIMEOUT,
            path_timeouts: Vec::new(),
            excluded_paths: Vec::new(),
        }
    }
}

impl TimeoutConfig {
    /// Create a new timeout configuration with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the default timeout duration.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Add a path-specific timeout that overrides the default.
    pub fn with_path_timeout(mut self, path: impl Into<String>, timeout: Duration) -> Self {
        self.path_timeouts.push((path.into(), timeout));
        self
    }

    /// Set the paths that are exempt from timeouts.
    pub fn with_excluded_paths(mut self, paths: Vec<String>) -> Self {
        self.excluded_paths = paths;
        self
    }

    /// Add a path that is exempt from timeouts.
    pub fn exclude_path(mut self, path: impl Into<String>) -> Self {
        self.excluded_paths.push(path.into());
        self
    }

    /// Get the timeout for a specific path, falling back to the default.
    pub fn get_timeout_for_path(&self, path: &str) -> Option<Duration> {
        // Check if the path is excluded
        if self.excluded_paths.iter().any(|p| path.starts_with(p)) {
            return None;
        }

        // Check for path-specific timeout
        for (prefix, timeout) in &self.path_timeouts {
            if path.starts_with(prefix) {
                return Some(*timeout);
            }
        }

        // Fall back to default timeout
        Some(self.timeout)
    }
}

/// Timeout middleware layer.
#[derive(Debug, Clone)]
pub struct TimeoutLayer {
    config: TimeoutConfig,
}

impl TimeoutLayer {
    /// Create a new timeout layer with default configuration.
    pub fn new() -> Self {
        Self {
            config: TimeoutConfig::default(),
        }
    }

    /// Create a new timeout layer with the provided configuration.
    pub fn with_config(config: TimeoutConfig) -> Self {
        Self { config }
    }

    /// Create a new timeout layer with the specified timeout.
    pub fn with_timeout(timeout: Duration) -> Self {
        Self {
            config: TimeoutConfig::default().with_timeout(timeout),
        }
    }
}

impl<S> Layer<S> for TimeoutLayer
where
    S: Service<Request> + Clone + Send + 'static,
    S::Response: Send + 'static,
    S::Error: Into<Box<dyn std::error::Error + Send + Sync>> + Send,
    S::Future: Send + 'static,
{
    type Service = TimeoutService<S>;

    fn layer(&self, service: S) -> Self::Service {
        TimeoutService {
            inner: service,
            config: self.config.clone(),
        }
    }
}

/// Service implementation for the timeout middleware.
#[derive(Debug, Clone)]
pub struct TimeoutService<S> {
    inner: S,
    config: TimeoutConfig,
}

impl<S> Service<Request> for TimeoutService<S>
where
    S: Service<Request> + Clone + Send + 'static,
    S::Response: Send + 'static,
    S::Error: Into<Box<dyn std::error::Error + Send + Sync>> + Send,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = Box<dyn std::error::Error + Send + Sync>;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let path = req.uri().path().to_string();
        let inner = self.inner.clone();

        // Determine timeout for this path
        match self.config.get_timeout_for_path(&path) {
            Some(timeout) => {
                info!(
                    target: "navius::http",
                    path = %path,
                    timeout_ms = %timeout.as_millis(),
                    "Request timeout configured"
                );

                // Apply timeout using tower's TimeoutLayer
                let timeout_layer = TowerTimeoutLayer::new(timeout);
                let mut timeout_svc = timeout_layer.layer(TimeoutInnerService(inner));

                Box::pin(async move {
                    timeout_svc.call(req).await.map_err(|e| {
                        // Check if this is a timeout error
                        if e.to_string().contains("operation timed out") {
                            warn!(
                                target: "navius::http",
                                path = %path,
                                timeout_ms = %timeout.as_millis(),
                                "Request timed out"
                            );
                        }
                        e
                    })
                })
            }
            None => {
                // No timeout for this path
                info!(
                    target: "navius::http",
                    path = %path,
                    "Request exempt from timeout"
                );

                // Pass through without timeout
                let mut cloned_inner = inner;
                Box::pin(async move { cloned_inner.call(req).await.map_err(Into::into) })
            }
        }
    }
}

// Helper service to convert inner service error type
#[derive(Clone)]
struct TimeoutInnerService<S>(S);

impl<S> Service<Request> for TimeoutInnerService<S>
where
    S: Service<Request> + Clone + Send + 'static,
    S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = Box<dyn std::error::Error + Send + Sync>;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.0.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let mut inner = self.0.clone();
        Box::pin(async move { inner.call(req).await.map_err(Into::into) })
    }
}

/// Axum middleware function for request timeouts.
pub async fn timeout_middleware(request: Request, next: Next, timeout: Duration) -> Response {
    let path = request.uri().path().to_string();

    // Create a tokio timeout future
    let timeout_fut = tokio::time::timeout(timeout, next.run(request));

    match timeout_fut.await {
        Ok(response) => {
            // Request completed within timeout
            response
        }
        Err(_) => {
            // Request timed out
            warn!(
                target: "navius::http",
                path = %path,
                timeout_ms = %timeout.as_millis(),
                "Request timed out"
            );

            // Return timeout response
            (
                axum::http::StatusCode::REQUEST_TIMEOUT,
                format!("Request timed out after {}ms", timeout.as_millis()),
            )
                .into_response()
        }
    }
}

/// Convenience function to create a timeout layer with default configuration.
pub fn timeout_layer() -> TimeoutLayer {
    TimeoutLayer::new()
}

/// Convenience function to create a timeout layer with a specific timeout.
pub fn timeout_layer_with_duration(timeout: Duration) -> TimeoutLayer {
    TimeoutLayer::with_timeout(timeout)
}

/// Function to create a timeout middleware with a specific timeout.
pub fn with_timeout(
    timeout: Duration,
) -> impl FnOnce(Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>>
+ Clone {
    move |req, next| Box::pin(timeout_middleware(req, next, timeout))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = TimeoutConfig::default();
        assert_eq!(config.timeout, DEFAULT_TIMEOUT);
        assert!(config.path_timeouts.is_empty());
        assert!(config.excluded_paths.is_empty());
    }

    #[test]
    fn test_custom_config() {
        let short_timeout = Duration::from_secs(5);
        let long_timeout = Duration::from_secs(60);

        let config = TimeoutConfig::default()
            .with_timeout(short_timeout)
            .with_path_timeout("/api/slow", long_timeout)
            .exclude_path("/webhooks");

        assert_eq!(config.timeout, short_timeout);
        assert_eq!(config.path_timeouts.len(), 1);
        assert_eq!(config.excluded_paths.len(), 1);

        // Test path-specific timeouts
        assert_eq!(
            config.get_timeout_for_path("/api/users"),
            Some(short_timeout)
        );
        assert_eq!(
            config.get_timeout_for_path("/api/slow/operation"),
            Some(long_timeout)
        );
        assert_eq!(config.get_timeout_for_path("/webhooks/github"), None);
    }
}
