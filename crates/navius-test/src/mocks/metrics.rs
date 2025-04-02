use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use thiserror::Error;

use crate::error::TestError;
use crate::error::TestResult;
use crate::mock::MockRegistry;

/// Errors that can occur when working with metrics
#[derive(Debug, Error)]
pub enum MockMetricsError {
    /// Error registering a metric
    #[error("Failed to register metric: {0}")]
    RegisterError(String),

    /// Error recording a metric
    #[error("Failed to record metric: {0}")]
    RecordError(String),

    /// Invalid metric name
    #[error("Invalid metric name: {0}")]
    InvalidName(String),

    /// Invalid metric value
    #[error("Invalid metric value: {0}")]
    InvalidValue(String),

    /// Other error
    #[error("Metrics error: {0}")]
    Other(String),
}

/// Type of metric
#[derive(Debug, Clone, PartialEq)]
pub enum MetricType {
    /// Counter metric
    Counter,
    /// Gauge metric
    Gauge,
    /// Histogram metric
    Histogram,
    /// Summary metric
    Summary,
}

/// Value for a metric
#[derive(Debug, Clone, PartialEq)]
pub enum MetricValue {
    /// Integer value
    Integer(i64),
    /// Float value
    Float(f64),
    /// Boolean value
    Boolean(bool),
}

impl MetricValue {
    /// Get the integer value
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            Self::Integer(v) => Some(*v),
            _ => None,
        }
    }

    /// Get the float value
    pub fn as_float(&self) -> Option<f64> {
        match self {
            Self::Float(v) => Some(*v),
            _ => None,
        }
    }

    /// Get the boolean value
    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            Self::Boolean(v) => Some(*v),
            _ => None,
        }
    }
}

/// Trait for collecting metrics
pub trait MetricsCollector: Send + Sync {
    /// Register a counter metric
    fn register_counter(&self, name: &str, help: &str) -> Result<(), MockMetricsError>;

    /// Register a gauge metric
    fn register_gauge(&self, name: &str, help: &str) -> Result<(), MockMetricsError>;

    /// Register a histogram metric
    fn register_histogram(
        &self,
        name: &str,
        help: &str,
        buckets: &[f64],
    ) -> Result<(), MockMetricsError>;

    /// Register a summary metric
    fn register_summary(&self, name: &str, help: &str) -> Result<(), MockMetricsError>;

    /// Increment a counter
    fn increment_counter(&self, name: &str, value: u64) -> Result<(), MockMetricsError>;

    /// Set a gauge
    fn set_gauge(&self, name: &str, value: f64) -> Result<(), MockMetricsError>;

    /// Record a histogram value
    fn record_histogram(&self, name: &str, value: f64) -> Result<(), MockMetricsError>;

    /// Record a summary value
    fn record_summary(&self, name: &str, value: f64) -> Result<(), MockMetricsError>;

    /// Increment a counter with labels
    fn increment_counter_with_labels(
        &self,
        name: &str,
        value: u64,
        labels: Vec<(&str, String)>,
    ) -> Result<(), MockMetricsError>;

    /// Set a gauge with labels
    fn set_gauge_with_labels(
        &self,
        name: &str,
        value: f64,
        labels: Vec<(&str, String)>,
    ) -> Result<(), MockMetricsError>;

    /// Record a histogram value with labels
    fn record_histogram_with_labels(
        &self,
        name: &str,
        value: f64,
        labels: Vec<(&str, String)>,
    ) -> Result<(), MockMetricsError>;

    /// Record a summary value with labels
    fn record_summary_with_labels(
        &self,
        name: &str,
        value: f64,
        labels: Vec<(&str, String)>,
    ) -> Result<(), MockMetricsError>;
}

/// Trait for exporting metrics
pub trait MetricsExporter: Send + Sync {
    /// Export metrics in Prometheus format
    fn export_prometheus(&self) -> Result<String, MockMetricsError>;
}

/// Mock implementation of the metrics collector and exporter
#[derive(Debug)]
pub struct MockMetrics {
    /// Registered metrics
    registered: Arc<Mutex<HashMap<String, (MetricType, String)>>>,
    /// Recorded metrics
    recorded: Arc<Mutex<HashMap<String, Vec<(MetricValue, Option<HashMap<String, String>>)>>>>,
    /// Should fail to register metrics
    fail_register: Arc<Mutex<bool>>,
    /// Should fail to record metrics
    fail_record: Arc<Mutex<bool>>,
}

impl Default for MockMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl MockMetrics {
    /// Create a new mock metrics collector and exporter
    pub fn new() -> Self {
        Self {
            registered: Arc::new(Mutex::new(HashMap::new())),
            recorded: Arc::new(Mutex::new(HashMap::new())),
            fail_register: Arc::new(Mutex::new(false)),
            fail_record: Arc::new(Mutex::new(false)),
        }
    }

    /// Register the mock with a registry
    pub fn register(self, _registry: &MockRegistry) -> TestResult<Arc<Self>> {
        Ok(Arc::new(self))
    }

    /// Set whether registering metrics should fail
    pub fn set_register_failure(&self, should_fail: bool) {
        let mut fail_register = self.fail_register.lock().unwrap();
        *fail_register = should_fail;
    }

    /// Set whether recording metrics should fail
    pub fn set_record_failure(&self, should_fail: bool) {
        let mut fail_record = self.fail_record.lock().unwrap();
        *fail_record = should_fail;
    }

    /// Get all registered metrics
    pub fn get_registered_metrics(&self) -> HashMap<String, (MetricType, String)> {
        let registered = self.registered.lock().unwrap();
        registered.clone()
    }

    /// Get recorded values for a metric
    pub fn get_recorded_values(
        &self,
        name: &str,
    ) -> Vec<(MetricValue, Option<HashMap<String, String>>)> {
        let recorded = self.recorded.lock().unwrap();
        recorded.get(name).cloned().unwrap_or_default()
    }

    /// Clear all recorded metrics
    pub fn clear_recorded(&self) {
        let mut recorded = self.recorded.lock().unwrap();
        recorded.clear();
    }

    /// Assert that a metric was registered
    pub fn assert_registered(&self, name: &str, metric_type: MetricType) -> TestResult<()> {
        let registered = self.registered.lock().unwrap();
        if let Some((t, _)) = registered.get(name) {
            if t == &metric_type {
                return Ok(());
            }
            return Err(TestError::AssertionFailed(format!(
                "Metric {} was registered as {:?}, expected {:?}",
                name, t, metric_type
            )));
        }
        Err(TestError::AssertionFailed(format!(
            "Metric {} was not registered",
            name
        )))
    }

    /// Assert that a metric was recorded with a value
    pub fn assert_recorded(
        &self,
        name: &str,
        value: MetricValue,
        labels: Option<HashMap<String, String>>,
    ) -> TestResult<()> {
        let recorded = self.recorded.lock().unwrap();
        if let Some(values) = recorded.get(name) {
            for (v, l) in values {
                if v == &value && labels_match(l, &labels) {
                    return Ok(());
                }
            }
            return Err(TestError::AssertionFailed(format!(
                "Metric {} with value {:?} and labels {:?} was not recorded",
                name, value, labels
            )));
        }
        Err(TestError::AssertionFailed(format!(
            "Metric {} was not recorded",
            name
        )))
    }
}

/// Check if labels match
fn labels_match(
    actual: &Option<HashMap<String, String>>,
    expected: &Option<HashMap<String, String>>,
) -> bool {
    match (actual, expected) {
        (None, None) => true,
        (Some(_), None) => true,
        (None, Some(_)) => false,
        (Some(a), Some(b)) => {
            for (k, v) in b {
                if let Some(av) = a.get(k) {
                    if av != v {
                        return false;
                    }
                } else {
                    return false;
                }
            }
            true
        }
    }
}

impl MetricsCollector for MockMetrics {
    fn register_counter(&self, name: &str, help: &str) -> Result<(), MockMetricsError> {
        if *self.fail_register.lock().unwrap() {
            return Err(MockMetricsError::RegisterError(format!(
                "Failed to register counter {}",
                name
            )));
        }
        self.registered
            .lock()
            .unwrap()
            .insert(name.to_string(), (MetricType::Counter, help.to_string()));
        Ok(())
    }

    fn register_gauge(&self, name: &str, help: &str) -> Result<(), MockMetricsError> {
        if *self.fail_register.lock().unwrap() {
            return Err(MockMetricsError::RegisterError(format!(
                "Failed to register gauge {}",
                name
            )));
        }
        self.registered
            .lock()
            .unwrap()
            .insert(name.to_string(), (MetricType::Gauge, help.to_string()));
        Ok(())
    }

    fn register_histogram(
        &self,
        name: &str,
        help: &str,
        _buckets: &[f64],
    ) -> Result<(), MockMetricsError> {
        if *self.fail_register.lock().unwrap() {
            return Err(MockMetricsError::RegisterError(format!(
                "Failed to register histogram {}",
                name
            )));
        }
        self.registered
            .lock()
            .unwrap()
            .insert(name.to_string(), (MetricType::Histogram, help.to_string()));
        Ok(())
    }

    fn register_summary(&self, name: &str, help: &str) -> Result<(), MockMetricsError> {
        if *self.fail_register.lock().unwrap() {
            return Err(MockMetricsError::RegisterError(format!(
                "Failed to register summary {}",
                name
            )));
        }
        self.registered
            .lock()
            .unwrap()
            .insert(name.to_string(), (MetricType::Summary, help.to_string()));
        Ok(())
    }

    fn increment_counter(&self, name: &str, value: u64) -> Result<(), MockMetricsError> {
        if *self.fail_record.lock().unwrap() {
            return Err(MockMetricsError::RecordError(format!(
                "Failed to increment counter {}",
                name
            )));
        }
        let mut recorded = self.recorded.lock().unwrap();
        let values = recorded.entry(name.to_string()).or_insert_with(Vec::new);
        values.push((MetricValue::Integer(value as i64), None));
        Ok(())
    }

    fn set_gauge(&self, name: &str, value: f64) -> Result<(), MockMetricsError> {
        if *self.fail_record.lock().unwrap() {
            return Err(MockMetricsError::RecordError(format!(
                "Failed to set gauge {}",
                name
            )));
        }
        let mut recorded = self.recorded.lock().unwrap();
        let values = recorded.entry(name.to_string()).or_insert_with(Vec::new);
        values.push((MetricValue::Float(value), None));
        Ok(())
    }

    fn record_histogram(&self, name: &str, value: f64) -> Result<(), MockMetricsError> {
        if *self.fail_record.lock().unwrap() {
            return Err(MockMetricsError::RecordError(format!(
                "Failed to record histogram {}",
                name
            )));
        }
        let mut recorded = self.recorded.lock().unwrap();
        let values = recorded.entry(name.to_string()).or_insert_with(Vec::new);
        values.push((MetricValue::Float(value), None));
        Ok(())
    }

    fn record_summary(&self, name: &str, value: f64) -> Result<(), MockMetricsError> {
        if *self.fail_record.lock().unwrap() {
            return Err(MockMetricsError::RecordError(format!(
                "Failed to record summary {}",
                name
            )));
        }
        let mut recorded = self.recorded.lock().unwrap();
        let values = recorded.entry(name.to_string()).or_insert_with(Vec::new);
        values.push((MetricValue::Float(value), None));
        Ok(())
    }

    fn increment_counter_with_labels(
        &self,
        name: &str,
        value: u64,
        labels: Vec<(&str, String)>,
    ) -> Result<(), MockMetricsError> {
        if *self.fail_record.lock().unwrap() {
            return Err(MockMetricsError::RecordError(format!(
                "Failed to increment counter {} with labels",
                name
            )));
        }
        let mut recorded = self.recorded.lock().unwrap();
        let values = recorded.entry(name.to_string()).or_insert_with(Vec::new);
        let labels_map = labels
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect();
        values.push((MetricValue::Integer(value as i64), Some(labels_map)));
        Ok(())
    }

    fn set_gauge_with_labels(
        &self,
        name: &str,
        value: f64,
        labels: Vec<(&str, String)>,
    ) -> Result<(), MockMetricsError> {
        if *self.fail_record.lock().unwrap() {
            return Err(MockMetricsError::RecordError(format!(
                "Failed to set gauge {} with labels",
                name
            )));
        }
        let mut recorded = self.recorded.lock().unwrap();
        let values = recorded.entry(name.to_string()).or_insert_with(Vec::new);
        let labels_map = labels
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect();
        values.push((MetricValue::Float(value), Some(labels_map)));
        Ok(())
    }

    fn record_histogram_with_labels(
        &self,
        name: &str,
        value: f64,
        labels: Vec<(&str, String)>,
    ) -> Result<(), MockMetricsError> {
        if *self.fail_record.lock().unwrap() {
            return Err(MockMetricsError::RecordError(format!(
                "Failed to record histogram {} with labels",
                name
            )));
        }
        let mut recorded = self.recorded.lock().unwrap();
        let values = recorded.entry(name.to_string()).or_insert_with(Vec::new);
        let labels_map = labels
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect();
        values.push((MetricValue::Float(value), Some(labels_map)));
        Ok(())
    }

    fn record_summary_with_labels(
        &self,
        name: &str,
        value: f64,
        labels: Vec<(&str, String)>,
    ) -> Result<(), MockMetricsError> {
        if *self.fail_record.lock().unwrap() {
            return Err(MockMetricsError::RecordError(format!(
                "Failed to record summary {} with labels",
                name
            )));
        }
        let mut recorded = self.recorded.lock().unwrap();
        let values = recorded.entry(name.to_string()).or_insert_with(Vec::new);
        let labels_map = labels
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect();
        values.push((MetricValue::Float(value), Some(labels_map)));
        Ok(())
    }
}

impl MetricsExporter for MockMetrics {
    fn export_prometheus(&self) -> Result<String, MockMetricsError> {
        let registered = self.registered.lock().unwrap();
        let recorded = self.recorded.lock().unwrap();

        let mut output = String::new();
        for (name, (typ, help)) in registered.iter() {
            // Add help comment
            output.push_str(&format!("# HELP {} {}\n", name, help));

            // Add type comment
            let type_str = match typ {
                MetricType::Counter => "counter",
                MetricType::Gauge => "gauge",
                MetricType::Histogram => "histogram",
                MetricType::Summary => "summary",
            };
            output.push_str(&format!("# TYPE {} {}\n", name, type_str));

            // Add metrics
            if let Some(values) = recorded.get(name) {
                for (value, labels) in values {
                    let value_str = match value {
                        MetricValue::Integer(v) => v.to_string(),
                        MetricValue::Float(v) => v.to_string(),
                        MetricValue::Boolean(v) => (if *v { 1 } else { 0 }).to_string(),
                    };

                    if let Some(labels) = labels {
                        let labels_str = labels
                            .iter()
                            .map(|(k, v)| format!("{}=\"{}\"", k, v))
                            .collect::<Vec<_>>()
                            .join(",");
                        output.push_str(&format!("{}{{{}}} {}\n", name, labels_str, value_str));
                    } else {
                        output.push_str(&format!("{} {}\n", name, value_str));
                    }
                }
            }
        }

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_metrics() -> TestResult<()> {
        let metrics = MockMetrics::new();

        // Register metrics
        metrics.register_counter("test_counter", "Test counter")?;
        metrics.register_gauge("test_gauge", "Test gauge")?;
        metrics.register_histogram("test_histogram", "Test histogram", &[0.1, 0.5, 1.0])?;
        metrics.register_summary("test_summary", "Test summary")?;

        // Record metrics
        metrics.increment_counter("test_counter", 10)?;
        metrics.set_gauge("test_gauge", 20.5)?;
        metrics.record_histogram("test_histogram", 0.75)?;
        metrics.record_summary("test_summary", 100.0)?;

        // Record metrics with labels
        metrics.increment_counter_with_labels(
            "test_counter",
            5,
            vec![("service", "api".to_string()), ("env", "test".to_string())],
        )?;

        // Assert registrations
        metrics.assert_registered("test_counter", MetricType::Counter)?;
        metrics.assert_registered("test_gauge", MetricType::Gauge)?;
        metrics.assert_registered("test_histogram", MetricType::Histogram)?;
        metrics.assert_registered("test_summary", MetricType::Summary)?;

        // Assert recorded values
        metrics.assert_recorded("test_counter", MetricValue::Integer(10), None)?;
        metrics.assert_recorded("test_gauge", MetricValue::Float(20.5), None)?;
        metrics.assert_recorded("test_histogram", MetricValue::Float(0.75), None)?;
        metrics.assert_recorded("test_summary", MetricValue::Float(100.0), None)?;

        // Assert recorded values with labels
        let mut labels = HashMap::new();
        labels.insert("service".to_string(), "api".to_string());
        labels.insert("env".to_string(), "test".to_string());
        metrics.assert_recorded("test_counter", MetricValue::Integer(5), Some(labels))?;

        // Get all recorded values
        let counter_values = metrics.get_recorded_values("test_counter");
        assert_eq!(counter_values.len(), 2);

        // Export to Prometheus format
        let prometheus = metrics.export_prometheus()?;
        assert!(prometheus.contains("# HELP test_counter Test counter"));
        assert!(prometheus.contains("# TYPE test_counter counter"));
        assert!(prometheus.contains("test_counter 10"));
        assert!(prometheus.contains("test_counter{service=\"api\",env=\"test\"} 5"));

        // Test failure modes
        metrics.set_register_failure(true);
        assert!(
            metrics
                .register_counter("should_fail", "Should fail")
                .is_err()
        );

        metrics.set_record_failure(true);
        assert!(metrics.increment_counter("test_counter", 1).is_err());

        // Clear recorded metrics
        metrics.clear_recorded();
        assert_eq!(metrics.get_recorded_values("test_counter").len(), 0);

        Ok(())
    }
}
