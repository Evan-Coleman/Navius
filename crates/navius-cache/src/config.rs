use std::time::Duration;

/// Cache configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Cache URL
    pub url: String,

    /// Cache key prefix for namespacing
    pub key_prefix: Option<String>,

    /// Default TTL (time to live) for cache entries
    pub default_ttl: Option<Duration>,

    /// Whether to enable metrics for cache operations
    pub metrics: bool,

    /// Whether to enable tracing for cache operations
    pub trace: bool,
}

impl CacheConfig {
    /// Create a new cache configuration with custom settings
    pub fn new(url: String, key_prefix: String, default_ttl: Duration) -> Self {
        Self {
            url,
            key_prefix: Some(key_prefix),
            default_ttl: Some(default_ttl),
            metrics: false,
            trace: false,
        }
    }

    /// Validate the cache configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.url.is_empty() {
            return Err("Cache URL cannot be empty".to_string());
        }
        Ok(())
    }

    /// Get the prefixed key
    pub fn prefixed_key(&self, key: &str) -> String {
        if let Some(prefix) = &self.key_prefix {
            if !prefix.is_empty() {
                return format!("{}{}", prefix, key);
            }
        }
        key.to_string()
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

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            url: "memory://".to_string(),
            key_prefix: Some("navius:".to_string()),
            default_ttl: Some(Duration::from_secs(3600)),
            metrics: false,
            trace: false,
        }
    }
}

/// Create a cache configuration for testing with a unique prefix
#[cfg(test)]
pub fn config_for_testing() -> CacheConfig {
    use uuid::Uuid;
    CacheConfig {
        url: "memory://".to_string(),
        key_prefix: Some(format!("navius-test-{}:", Uuid::new_v4())),
        default_ttl: Some(Duration::from_secs(60)),
        metrics: false,
        trace: true,
    }
}
