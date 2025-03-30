//! CORS middleware for the Navius HTTP server.
//!
//! This middleware provides Cross-Origin Resource Sharing (CORS) support,
//! allowing controlled access to resources from different origins.

use axum::http::{HeaderName, HeaderValue, Method};
use std::{collections::HashSet, time::Duration};
use tower_http::cors::{AllowHeaders, AllowOrigin, Any, CorsLayer as TowerCorsLayer};

/// Configuration for CORS middleware.
#[derive(Debug, Clone)]
pub struct CorsConfig {
    /// Allowed origins for CORS requests.
    allowed_origins: HashSet<String>,
    /// Allowed methods for CORS requests.
    allowed_methods: HashSet<Method>,
    /// Allowed headers for CORS requests.
    allowed_headers: HashSet<HeaderName>,
    /// Whether to allow credentials in CORS requests.
    allow_credentials: bool,
    /// Headers exposed to the browser.
    exposed_headers: HashSet<HeaderName>,
    /// Max age for preflight requests in seconds.
    max_age: Option<Duration>,
}

impl Default for CorsConfig {
    fn default() -> Self {
        let allowed_methods = [
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
            Method::HEAD,
        ]
        .iter()
        .cloned()
        .collect();

        // Common allowed headers
        let mut allowed_headers = HashSet::new();
        for header in &[
            "content-type",
            "authorization",
            "accept",
            "x-request-id",
            "x-api-key",
        ] {
            if let Ok(name) = HeaderName::from_bytes(header.as_bytes()) {
                allowed_headers.insert(name);
            }
        }

        // Common exposed headers
        let mut exposed_headers = HashSet::new();
        for header in &["content-type", "content-length", "x-request-id"] {
            if let Ok(name) = HeaderName::from_bytes(header.as_bytes()) {
                exposed_headers.insert(name);
            }
        }

        Self {
            allowed_origins: HashSet::new(), // No default origins, must be specified
            allowed_methods,
            allowed_headers,
            allow_credentials: true,
            exposed_headers,
            max_age: Some(Duration::from_secs(86400)), // 24 hours
        }
    }
}

impl CorsConfig {
    /// Create a new CORS configuration with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the allowed origins.
    pub fn with_allowed_origins(mut self, origins: Vec<String>) -> Self {
        self.allowed_origins = origins.into_iter().collect();
        self
    }

    /// Add an allowed origin.
    pub fn allow_origin(mut self, origin: impl Into<String>) -> Self {
        self.allowed_origins.insert(origin.into());
        self
    }

    /// Set the allowed methods.
    pub fn with_allowed_methods(mut self, methods: Vec<Method>) -> Self {
        self.allowed_methods = methods.into_iter().collect();
        self
    }

    /// Add an allowed method.
    pub fn allow_method(mut self, method: Method) -> Self {
        self.allowed_methods.insert(method);
        self
    }

    /// Set the allowed headers.
    pub fn with_allowed_headers(mut self, headers: Vec<HeaderName>) -> Self {
        self.allowed_headers = headers.into_iter().collect();
        self
    }

    /// Add an allowed header.
    pub fn allow_header(mut self, header: HeaderName) -> Self {
        self.allowed_headers.insert(header);
        self
    }

    /// Set whether to allow credentials.
    pub fn with_credentials(mut self, allow: bool) -> Self {
        self.allow_credentials = allow;
        self
    }

    /// Set the exposed headers.
    pub fn with_exposed_headers(mut self, headers: Vec<HeaderName>) -> Self {
        self.exposed_headers = headers.into_iter().collect();
        self
    }

    /// Add an exposed header.
    pub fn expose_header(mut self, header: HeaderName) -> Self {
        self.exposed_headers.insert(header);
        self
    }

    /// Set the max age for preflight requests.
    pub fn with_max_age(mut self, seconds: u64) -> Self {
        self.max_age = Some(Duration::from_secs(seconds));
        self
    }
}

/// CORS middleware layer.
#[derive(Debug, Clone)]
pub struct CorsLayer {
    inner: TowerCorsLayer,
}

impl CorsLayer {
    /// Create a new CORS layer with default configuration.
    pub fn new() -> Self {
        Self::with_config(CorsConfig::default())
    }

    /// Create a new CORS layer with the provided configuration.
    pub fn with_config(config: CorsConfig) -> Self {
        let origins: AllowOrigin = if config.allowed_origins.is_empty() {
            Any.into()
        } else {
            let origins = config
                .allowed_origins
                .iter()
                .filter_map(|origin| HeaderValue::from_str(origin).ok())
                .collect::<Vec<_>>();
            origins.into()
        };

        let methods = config.allowed_methods.iter().cloned().collect::<Vec<_>>();

        let headers: AllowHeaders = if config.allowed_headers.is_empty() {
            Any.into()
        } else {
            let headers = config.allowed_headers.iter().cloned().collect::<Vec<_>>();
            headers.into()
        };

        let exposed_headers = config.exposed_headers.iter().cloned().collect::<Vec<_>>();

        let mut builder = TowerCorsLayer::new()
            .allow_origin(origins)
            .allow_methods(methods)
            .allow_headers(headers)
            .allow_credentials(config.allow_credentials)
            .expose_headers(exposed_headers);

        if let Some(max_age) = config.max_age {
            builder = builder.max_age(max_age);
        }

        Self { inner: builder }
    }
}

impl<S> tower::Layer<S> for CorsLayer {
    type Service = <TowerCorsLayer as tower::Layer<S>>::Service;

    fn layer(&self, service: S) -> Self::Service {
        self.inner.layer(service)
    }
}

/// Convenience function to create a CORS layer with default configuration.
pub fn cors_layer() -> CorsLayer {
    CorsLayer::new()
}

/// Convenience function to create a permissive CORS layer for development.
pub fn permissive_cors_layer() -> CorsLayer {
    CorsLayer::with_config(
        CorsConfig::default()
            .allow_origin("*".to_string())
            .with_credentials(false), // Wildcard origins cannot have credentials
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = CorsConfig::default();
        assert!(config.allowed_origins.is_empty());
        assert_eq!(config.allowed_methods.len(), 6); // GET, POST, PUT, DELETE, OPTIONS, HEAD
        assert!(config.allow_credentials);
        assert_eq!(config.max_age.unwrap().as_secs(), 86400);
    }

    #[test]
    fn test_custom_config() {
        let config = CorsConfig::default()
            .allow_origin("https://example.com".to_string())
            .allow_method(Method::PATCH)
            .with_credentials(false)
            .with_max_age(3600);

        assert_eq!(config.allowed_origins.len(), 1);
        assert!(config.allowed_origins.contains("https://example.com"));
        assert_eq!(config.allowed_methods.len(), 7); // Added PATCH
        assert!(!config.allow_credentials);
        assert_eq!(config.max_age.unwrap().as_secs(), 3600);
    }
}
