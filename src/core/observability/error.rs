use std::error::Error;
use std::fmt;

/// Error type for observability operations
#[derive(Debug)]
pub enum ObservabilityError {
    /// Provider not found
    ProviderNotFound(String),
    /// Provider not supported (feature flag not enabled)
    ProviderNotSupported(String),
    /// Configuration is not supported
    UnsupportedConfiguration(String),
    /// No default provider set
    NoDefaultProvider(String),
    /// Failed to initialize
    InitializationError(String),
    /// Failed to record metric
    MetricRecordingError(String),
    /// Failed to query metric
    MetricQueryError(String),
    /// Failed to export metrics
    MetricExportError(String),
    /// Metric type mismatch
    MetricError(String),
    /// Failed to create span
    SpanCreationError(String),
    /// Failed to start profiling
    ProfilingError(String),
    /// Internal error
    InternalError(String),
}

impl fmt::Display for ObservabilityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            ObservabilityError::ProviderNotFound(msg) => format!("Provider not found: {}", msg),
            ObservabilityError::ProviderNotSupported(msg) => {
                format!("Provider not supported: {}", msg)
            }
            ObservabilityError::UnsupportedConfiguration(msg) => {
                format!("Unsupported configuration: {}", msg)
            }
            ObservabilityError::NoDefaultProvider(msg) => format!("No default provider: {}", msg),
            ObservabilityError::InitializationError(msg) => {
                format!("Initialization error: {}", msg)
            }
            ObservabilityError::MetricRecordingError(msg) => {
                format!("Metric recording error: {}", msg)
            }
            ObservabilityError::MetricQueryError(msg) => format!("Metric query error: {}", msg),
            ObservabilityError::MetricExportError(msg) => format!("Metric export error: {}", msg),
            ObservabilityError::MetricError(msg) => format!("Metric error: {}", msg),
            ObservabilityError::SpanCreationError(msg) => format!("Span creation error: {}", msg),
            ObservabilityError::ProfilingError(msg) => format!("Profiling error: {}", msg),
            ObservabilityError::InternalError(msg) => format!("Internal error: {}", msg),
        };
        write!(f, "{}", message)
    }
}

impl Error for ObservabilityError {}

impl From<&str> for ObservabilityError {
    fn from(msg: &str) -> Self {
        ObservabilityError::InternalError(msg.to_string())
    }
}

impl From<String> for ObservabilityError {
    fn from(msg: String) -> Self {
        ObservabilityError::InternalError(msg)
    }
}

impl From<std::io::Error> for ObservabilityError {
    fn from(err: std::io::Error) -> Self {
        ObservabilityError::InternalError(format!("IO error: {}", err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let error = ObservabilityError::ProviderNotFound("test-provider".to_string());
        assert_eq!(error.to_string(), "Provider not found: test-provider");

        let error =
            ObservabilityError::ProviderNotSupported("jaeger feature not enabled".to_string());
        assert_eq!(
            error.to_string(),
            "Provider not supported: jaeger feature not enabled"
        );

        let error = ObservabilityError::MetricError("wrong metric type".to_string());
        assert_eq!(error.to_string(), "Metric error: wrong metric type");
    }

    #[test]
    fn test_from_str() {
        let error: ObservabilityError = "test error".into();
        assert!(matches!(error, ObservabilityError::InternalError(_)));
    }

    #[test]
    fn test_from_string() {
        let error: ObservabilityError = "test error".to_string().into();
        assert!(matches!(error, ObservabilityError::InternalError(_)));
    }
}
