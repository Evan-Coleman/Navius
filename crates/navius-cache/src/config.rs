use std::time::Duration;

/// Cache configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Cache URL
    pub url: String,

    /// Cache prefix for key namespacing
    pub prefix: String,

    /// Default TTL (time to live) for cache entries
    pub default_ttl: Duration,

    /// Connection timeout
    pub connect_timeout: Duration,

    /// Maximum number of connections in the pool
    pub max_connections: usize,

    /// Whether to enable tracing for cache operations
    pub trace: bool,

    /// Whether to enable cache metrics
    pub metrics: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            url: "redis://127.0.0.1:6379".to_string(),
            prefix: "navius:".to_string(),
            default_ttl: Duration::from_secs(3600),
            connect_timeout: Duration::from_secs(5),
            max_connections: 10,
            trace: false,
            metrics: false,
        }
    }
}

impl CacheConfig {
    /// Create a new cache configuration with custom settings
    pub fn new(url: String, prefix: String, default_ttl: Duration) -> Self {
        Self {
            url,
            prefix,
            default_ttl,
            ..Default::default()
        }
    }

    /// Create a cache configuration for testing with a unique prefix
    #[cfg(test)]
    pub fn for_testing() -> Self {
        use uuid::Uuid;
        Self {
            url: "redis://127.0.0.1:6379".to_string(),
            prefix: format!("navius-test-{}:", Uuid::new_v4()),
            default_ttl: Duration::from_secs(60),
            connect_timeout: Duration::from_secs(1),
            max_connections: 5,
            trace: true,
            metrics: false,
        }
    }

    /// Get the prefixed key
    pub fn prefixed_key(&self, key: &str) -> String {
        format!("{}{}", self.prefix, key)
    }

    /// Set the connection timeout
    pub fn with_connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = timeout;
        self
    }

    /// Set the maximum number of connections
    pub fn with_max_connections(mut self, max_connections: usize) -> Self {
        self.max_connections = max_connections;
        self
    }

    /// Enable or disable tracing
    pub fn with_trace(mut self, trace: bool) -> Self {
        self.trace = trace;
        self
    }

    /// Enable or disable metrics
    pub fn with_metrics(mut self, metrics: bool) -> Self {
        self.metrics = metrics;
        self
    }
}
