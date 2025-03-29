use std::collections::HashMap;
use std::time::Duration;

/// Configuration for the observability system
#[derive(Debug, Clone)]
pub struct ObservabilityConfig {
    /// The provider to use (e.g., "prometheus", "dynatrace")
    pub provider: String,

    /// Whether metrics export is enabled
    pub export_enabled: bool,

    /// Export interval in seconds
    pub export_interval: Duration,

    /// Service name for attribution
    pub service_name: String,

    /// Environment (e.g., "production", "staging")
    pub environment: String,

    /// Whether tracing is enabled
    pub tracing_enabled: bool,

    /// Whether profiling is enabled
    pub profiling_enabled: bool,

    /// Sample rate for tracing (0.0 - 1.0)
    pub trace_sample_rate: f64,

    /// Whether correlation between metrics and traces is enabled
    pub correlation_enabled: bool,

    /// Distributed tracing endpoint (for providers that support it)
    pub tracing_endpoint: Option<String>,

    /// Propagation headers for distributed tracing (trace context or similar)
    pub propagation_headers: Vec<String>,

    /// Provider-specific configuration
    pub provider_config: HashMap<String, String>,
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            provider: "prometheus".to_string(),
            export_enabled: true,
            export_interval: Duration::from_secs(60),
            service_name: "navius".to_string(),
            environment: "development".to_string(),
            tracing_enabled: true,
            profiling_enabled: false,
            trace_sample_rate: 0.1,
            correlation_enabled: true,
            tracing_endpoint: None,
            propagation_headers: vec!["traceparent".to_string(), "tracestate".to_string()],
            provider_config: HashMap::new(),
        }
    }
}

impl ObservabilityConfig {
    /// Create a new configuration
    pub fn new(provider: &str, service_name: &str) -> Self {
        Self {
            provider: provider.to_string(),
            service_name: service_name.to_string(),
            ..Default::default()
        }
    }

    /// Set export enabled
    pub fn with_export_enabled(mut self, enabled: bool) -> Self {
        self.export_enabled = enabled;
        self
    }

    /// Set export interval
    pub fn with_export_interval(mut self, interval: Duration) -> Self {
        self.export_interval = interval;
        self
    }

    /// Set environment
    pub fn with_environment(mut self, environment: &str) -> Self {
        self.environment = environment.to_string();
        self
    }

    /// Set tracing enabled
    pub fn with_tracing_enabled(mut self, enabled: bool) -> Self {
        self.tracing_enabled = enabled;
        self
    }

    /// Set profiling enabled
    pub fn with_profiling_enabled(mut self, enabled: bool) -> Self {
        self.profiling_enabled = enabled;
        self
    }

    /// Set trace sample rate
    pub fn with_trace_sample_rate(mut self, rate: f64) -> Self {
        self.trace_sample_rate = rate.max(0.0).min(1.0);
        self
    }

    /// Set correlation enabled
    pub fn with_correlation_enabled(mut self, enabled: bool) -> Self {
        self.correlation_enabled = enabled;
        self
    }

    /// Set tracing endpoint
    pub fn with_tracing_endpoint(mut self, endpoint: Option<&str>) -> Self {
        self.tracing_endpoint = endpoint.map(|s| s.to_string());
        self
    }

    /// Set propagation headers
    pub fn with_propagation_headers(mut self, headers: Vec<String>) -> Self {
        self.propagation_headers = headers;
        self
    }

    /// Add provider-specific configuration
    pub fn with_provider_config(mut self, key: &str, value: &str) -> Self {
        self.provider_config
            .insert(key.to_string(), value.to_string());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ObservabilityConfig::default();

        assert_eq!(config.provider, "prometheus");
        assert_eq!(config.service_name, "navius");
        assert_eq!(config.environment, "development");
        assert!(config.export_enabled);
        assert_eq!(config.export_interval, Duration::from_secs(60));
        assert!(config.tracing_enabled);
        assert!(!config.profiling_enabled);
        assert!((config.trace_sample_rate - 0.1).abs() < f64::EPSILON);
        assert!(config.correlation_enabled);
        assert_eq!(config.tracing_endpoint, None);
        assert_eq!(config.propagation_headers.len(), 2);
        assert!(config.provider_config.is_empty());
    }

    #[test]
    fn test_config_builder() {
        let config = ObservabilityConfig::new("dynatrace", "test-service")
            .with_export_enabled(false)
            .with_export_interval(Duration::from_secs(30))
            .with_environment("staging")
            .with_tracing_enabled(false)
            .with_profiling_enabled(true)
            .with_trace_sample_rate(0.5)
            .with_correlation_enabled(false)
            .with_tracing_endpoint(Some("http://jaeger:14268/api/traces"))
            .with_propagation_headers(vec!["x-trace-id".to_string()])
            .with_provider_config("api-key", "test-key");

        assert_eq!(config.provider, "dynatrace");
        assert_eq!(config.service_name, "test-service");
        assert_eq!(config.environment, "staging");
        assert!(!config.export_enabled);
        assert_eq!(config.export_interval, Duration::from_secs(30));
        assert!(!config.tracing_enabled);
        assert!(config.profiling_enabled);
        assert!((config.trace_sample_rate - 0.5).abs() < f64::EPSILON);
        assert!(!config.correlation_enabled);
        assert_eq!(
            config.tracing_endpoint,
            Some("http://jaeger:14268/api/traces".to_string())
        );
        assert_eq!(config.propagation_headers, vec!["x-trace-id".to_string()]);
        assert_eq!(
            config.provider_config.get("api-key"),
            Some(&"test-key".to_string())
        );
    }

    #[test]
    fn test_trace_sample_rate_clamping() {
        let config1 = ObservabilityConfig::default().with_trace_sample_rate(-0.5);
        let config2 = ObservabilityConfig::default().with_trace_sample_rate(1.5);

        assert!((config1.trace_sample_rate - 0.0).abs() < f64::EPSILON);
        assert!((config2.trace_sample_rate - 1.0).abs() < f64::EPSILON);
    }
}
