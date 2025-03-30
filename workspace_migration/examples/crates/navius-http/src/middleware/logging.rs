//! Logging middleware for the Navius HTTP server.
//!
//! This middleware provides structured logging for HTTP requests and responses,
//! using the tracing crate for logging formatted information.

use axum::{
    extract::Request,
    http::{HeaderMap, HeaderName},
    middleware::Next,
    response::Response,
};
use std::time::Instant;
use std::{collections::HashSet, fmt::Display};
use tower::{Layer, Service};
use tracing::{Level, info, warn};

/// Default maximum body size to log (in bytes)
const DEFAULT_MAX_BODY_SIZE: usize = 1024 * 10; // 10KB

/// Configuration for logging middleware.
#[derive(Debug, Clone)]
pub struct LoggingConfig {
    /// The log level to use.
    log_level: Level,
    /// Whether to include request headers in the logs.
    include_headers: bool,
    /// Whether to include response body in the logs (if small enough).
    include_body: bool,
    /// Maximum body size to log (in bytes).
    max_body_length: usize,
    /// Paths that should not be logged.
    excluded_paths: Vec<String>,
    /// Headers that should not be logged (for privacy).
    excluded_headers: HashSet<HeaderName>,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        let mut excluded_headers = HashSet::new();
        // Default sensitive headers to exclude
        for header in &[
            "authorization",
            "cookie",
            "set-cookie",
            "x-api-key",
            "x-auth-token",
        ] {
            if let Ok(header_name) = HeaderName::from_bytes(header.as_bytes()) {
                excluded_headers.insert(header_name);
            }
        }

        Self {
            log_level: Level::INFO,
            include_headers: true,
            include_body: false,
            max_body_length: DEFAULT_MAX_BODY_SIZE,
            excluded_paths: vec!["/health".to_string(), "/metrics".to_string()],
            excluded_headers,
        }
    }
}

impl LoggingConfig {
    /// Create a new logging configuration with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the log level for request logging.
    pub fn with_log_level(mut self, level: Level) -> Self {
        self.log_level = level;
        self
    }

    /// Set whether to include request headers in the logs.
    pub fn with_headers(mut self, include_headers: bool) -> Self {
        self.include_headers = include_headers;
        self
    }

    /// Set whether to include response body in the logs.
    pub fn with_body(mut self, include_body: bool) -> Self {
        self.include_body = include_body;
        self
    }

    /// Set the maximum body size to log.
    pub fn with_max_body_length(mut self, max_length: usize) -> Self {
        self.max_body_length = max_length;
        self
    }

    /// Set the paths that should not be logged.
    pub fn with_excluded_paths(mut self, paths: Vec<String>) -> Self {
        self.excluded_paths = paths;
        self
    }

    /// Add a path that should not be logged.
    pub fn exclude_path(mut self, path: impl Into<String>) -> Self {
        self.excluded_paths.push(path.into());
        self
    }

    /// Set the headers that should not be logged.
    pub fn with_excluded_headers(mut self, headers: Vec<HeaderName>) -> Self {
        self.excluded_headers = headers.into_iter().collect();
        self
    }

    /// Add a header that should not be logged.
    pub fn exclude_header(mut self, header: HeaderName) -> Self {
        self.excluded_headers.insert(header);
        self
    }

    /// Check if a path should be logged.
    fn should_log_path(&self, path: &str) -> bool {
        !self
            .excluded_paths
            .iter()
            .any(|excluded| path.starts_with(excluded))
    }

    /// Filter headers to exclude sensitive information.
    fn filter_headers<'a>(&'a self, headers: &'a HeaderMap) -> impl Display + 'a {
        struct FilteredHeaders<'a> {
            headers: &'a HeaderMap,
            excluded: &'a HashSet<HeaderName>,
        }

        impl<'a> Display for FilteredHeaders<'a> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let mut first = true;
                f.write_str("{")?;
                for (name, value) in self.headers.iter() {
                    if self.excluded.contains(name) {
                        continue;
                    }

                    if !first {
                        f.write_str(", ")?;
                    }
                    first = false;

                    f.write_fmt(format_args!("{}: {:?}", name, value))?;
                }
                f.write_str("}")
            }
        }

        FilteredHeaders {
            headers,
            excluded: &self.excluded_headers,
        }
    }
}

/// Logging middleware layer.
#[derive(Debug, Clone)]
pub struct LoggingLayer {
    config: LoggingConfig,
}

impl LoggingLayer {
    /// Create a new logging layer with default configuration.
    pub fn new() -> Self {
        Self {
            config: LoggingConfig::default(),
        }
    }

    /// Create a new logging layer with the provided configuration.
    pub fn with_config(config: LoggingConfig) -> Self {
        Self { config }
    }
}

impl<S> Layer<S> for LoggingLayer {
    type Service = LoggingService<S>;

    fn layer(&self, service: S) -> Self::Service {
        LoggingService {
            inner: service,
            config: self.config.clone(),
        }
    }
}

/// Service implementation for the logging middleware.
#[derive(Debug, Clone)]
pub struct LoggingService<S> {
    inner: S,
    config: LoggingConfig,
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for LoggingService<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    ReqBody: Send + 'static,
    ResBody: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        // Clone things needed for the async block
        let inner = self.inner.clone();
        let config = self.config.clone();
        let path = req.uri().path().to_string();
        let method = req.method().clone();
        let version = format!("{:?}", req.version());
        let headers = req.headers().clone();

        // Skip logging for excluded paths
        if !config.should_log_path(&path) {
            // Just pass through the request
            let mut inner = self.inner.clone();
            return Box::pin(async move { inner.call(req).await });
        }

        // Create a span for request tracing
        let request_id = match headers.get("x-request-id") {
            Some(value) => value.to_str().unwrap_or("unknown").to_string(),
            None => "unknown".to_string(),
        };

        // Log the request
        info!(
            target: "navius::http",
            request_id = %request_id,
            method = %method,
            path = %path,
            version = %version,
            headers = %config.filter_headers(&headers),
            "Request received"
        );

        let start_time = Instant::now();

        // Execute the inner service
        let mut inner = inner;

        Box::pin(async move {
            let result = inner.call(req).await;
            let elapsed = start_time.elapsed();

            match result {
                Ok(response) => {
                    // Log the response
                    let status = response.status();

                    let log_level = if status.is_success() || status.is_redirection() {
                        config.log_level
                    } else if status.is_client_error() {
                        Level::WARN
                    } else {
                        Level::ERROR
                    };

                    match log_level {
                        Level::INFO => {
                            info!(
                                target: "navius::http",
                                request_id = %request_id,
                                status = %status.as_u16(),
                                elapsed_ms = %elapsed.as_millis(),
                                path = %path,
                                "Response sent"
                            );
                        }
                        Level::WARN => {
                            warn!(
                                target: "navius::http",
                                request_id = %request_id,
                                status = %status.as_u16(),
                                elapsed_ms = %elapsed.as_millis(),
                                path = %path,
                                "Client error response"
                            );
                        }
                        _ => {
                            tracing::error!(
                                target: "navius::http",
                                request_id = %request_id,
                                status = %status.as_u16(),
                                elapsed_ms = %elapsed.as_millis(),
                                path = %path,
                                "Server error response"
                            );
                        }
                    }

                    Ok(response)
                }
                Err(err) => Err(err),
            }
        })
    }
}

use std::future::Future;
use std::pin::Pin;

/// Axum middleware function for request logging.
pub async fn logging_middleware(request: Request, next: Next) -> Response {
    let start_time = Instant::now();
    let path = request.uri().path().to_string();
    let method = request.method().clone();

    // Log the request
    let request_id = match request.headers().get("x-request-id") {
        Some(value) => value.to_str().unwrap_or("unknown").to_string(),
        None => "unknown".to_string(),
    };

    info!(
        target: "navius::http",
        request_id = %request_id,
        method = %method,
        path = %path,
        "Request received"
    );

    // Process the request
    let response = next.run(request).await;
    let status = response.status();
    let elapsed = start_time.elapsed();

    // Log the response
    let log_level = if status.is_success() || status.is_redirection() {
        Level::INFO
    } else if status.is_client_error() {
        Level::WARN
    } else {
        Level::ERROR
    };

    match log_level {
        Level::INFO => {
            info!(
                target: "navius::http",
                request_id = %request_id,
                status = %status.as_u16(),
                elapsed_ms = %elapsed.as_millis(),
                path = %path,
                "Response sent"
            );
        }
        Level::WARN => {
            warn!(
                target: "navius::http",
                request_id = %request_id,
                status = %status.as_u16(),
                elapsed_ms = %elapsed.as_millis(),
                path = %path,
                "Client error response"
            );
        }
        _ => {
            tracing::error!(
                target: "navius::http",
                request_id = %request_id,
                status = %status.as_u16(),
                elapsed_ms = %elapsed.as_millis(),
                path = %path,
                "Server error response"
            );
        }
    }

    response
}

/// Convenience function to create a logging layer with default configuration.
pub fn logging_layer() -> LoggingLayer {
    LoggingLayer::new()
}

/// Convenience function to create a logging layer with detailed configuration.
pub fn detailed_logging_layer() -> LoggingLayer {
    LoggingLayer::with_config(LoggingConfig::default().with_headers(true).with_body(true))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = LoggingConfig::default();
        assert_eq!(config.log_level, Level::INFO);
        assert_eq!(config.include_headers, true);
        assert_eq!(config.include_body, false);
        assert_eq!(config.max_body_length, DEFAULT_MAX_BODY_SIZE);
        assert_eq!(config.excluded_paths.len(), 2);
    }

    #[test]
    fn test_custom_config() {
        let config = LoggingConfig::default()
            .with_log_level(Level::DEBUG)
            .with_headers(false)
            .with_body(true)
            .with_max_body_length(100)
            .exclude_path("/ping");

        assert_eq!(config.log_level, Level::DEBUG);
        assert_eq!(config.include_headers, false);
        assert_eq!(config.include_body, true);
        assert_eq!(config.max_body_length, 100);
        assert!(config.excluded_paths.contains(&"/ping".to_string()));
    }
}
