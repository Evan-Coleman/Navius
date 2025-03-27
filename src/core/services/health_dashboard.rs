use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant, SystemTime};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tracing::{debug, info, warn};

use crate::core::models::DependencyStatus;
use crate::core::router::AppState;
use crate::core::services::error::ServiceError;
use crate::core::services::health_discovery::HealthDiscoveryService;
use crate::core::services::health_provider::{HealthConfig, HealthIndicator};

/// Health status history entry
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HealthStatusHistoryEntry {
    /// Timestamp of the health check
    pub timestamp: SystemTime,

    /// Overall status at this time
    pub status: String,

    /// Component statuses
    pub components: HashMap<String, String>,

    /// Any error message
    pub error: Option<String>,
}

/// Enhanced health service with detailed reporting and dashboard functionality
pub struct HealthDashboardService {
    /// Health discovery service for indicators
    discovery: Arc<RwLock<HealthDiscoveryService>>,

    /// Health status history
    history: Arc<RwLock<Vec<HealthStatusHistoryEntry>>>,

    /// Maximum history entries to keep
    max_history: usize,

    /// Last check time
    last_check: Arc<RwLock<Option<Instant>>>,

    /// Dashboard configuration
    config: HealthDashboardConfig,
}

/// Health dashboard configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HealthDashboardConfig {
    /// Maximum history entries to keep
    pub max_history: usize,

    /// Minimum check interval (to prevent DoS)
    pub min_check_interval: Duration,

    /// Detailed component information
    pub detailed_components: bool,

    /// Include status history in response
    pub include_history: bool,

    /// Include performance metrics
    pub include_performance: bool,
}

impl Default for HealthDashboardConfig {
    fn default() -> Self {
        Self {
            max_history: 100,
            min_check_interval: Duration::from_secs(5),
            detailed_components: true,
            include_history: true,
            include_performance: true,
        }
    }
}

impl HealthDashboardService {
    /// Create a new health dashboard service
    pub fn new(
        discovery: Arc<RwLock<HealthDiscoveryService>>,
        config: HealthDashboardConfig,
    ) -> Self {
        Self {
            discovery,
            history: Arc::new(RwLock::new(Vec::with_capacity(config.max_history))),
            max_history: config.max_history,
            last_check: Arc::new(RwLock::new(None)),
            config,
        }
    }

    /// Check the health of all components and generate a detailed report
    pub async fn check_health(&self, state: &Arc<AppState>) -> Result<Value, ServiceError> {
        // Respect rate limiting
        {
            let mut last_check = self
                .last_check
                .write()
                .map_err(|e| ServiceError::other(format!("Failed to acquire lock: {}", e)))?;

            if let Some(check_time) = *last_check {
                let elapsed = check_time.elapsed();
                if elapsed < self.config.min_check_interval {
                    debug!(
                        "Rate limiting health check, last check was {} ms ago",
                        elapsed.as_millis()
                    );
                    // Return cached result from history
                    if let Ok(history) = self.history.read() {
                        if let Some(last) = history.last() {
                            return Ok(self.format_cached_result(last, elapsed));
                        }
                    }
                }
            }

            // Update last check time
            *last_check = Some(Instant::now());
        }

        // Get all health indicators
        let discovery_read = self.discovery.read().map_err(|e| {
            ServiceError::other(format!("Failed to acquire discovery read lock: {}", e))
        })?;

        let indicators = discovery_read.get_all_indicators();

        // Track overall status and performance
        let mut aggregated_status = "UP".to_string();
        let mut components = serde_json::Map::new();
        let mut component_statuses = HashMap::new();
        let mut check_start = Instant::now();

        // Check each health indicator and measure performance
        for indicator in indicators {
            let indicator_name = indicator.name();
            let indicator_start = Instant::now();

            // Check health
            let result = indicator.check_health(state);
            let indicator_duration = indicator_start.elapsed();

            // Store component status for history
            component_statuses.insert(indicator_name.clone(), result.status.clone());

            // If any critical component is down, the overall status is down
            if result.status != "UP" && indicator.is_critical() {
                aggregated_status = "DOWN".to_string();
            }

            // Add component details to response
            if self.config.detailed_components {
                let mut component_details = serde_json::Map::new();
                component_details.insert(
                    "status".to_string(),
                    serde_json::Value::String(result.status),
                );

                if let Some(details) = result.details {
                    component_details
                        .insert("details".to_string(), serde_json::Value::String(details));
                }

                // Add metadata
                let metadata = indicator.metadata();
                for (k, v) in metadata {
                    component_details.insert(k, serde_json::Value::String(v));
                }

                // Add performance info
                if self.config.include_performance {
                    component_details.insert(
                        "responseTime".to_string(),
                        serde_json::Value::Number(serde_json::Number::from(
                            indicator_duration.as_millis() as u64,
                        )),
                    );
                }

                components.insert(indicator_name, serde_json::Value::Object(component_details));
            }
        }

        // Calculate total health check duration
        let check_duration = check_start.elapsed();

        // Update history
        {
            let mut history = self.history.write().map_err(|e| {
                ServiceError::other(format!("Failed to acquire history write lock: {}", e))
            })?;

            // Add new entry
            history.push(HealthStatusHistoryEntry {
                timestamp: SystemTime::now(),
                status: aggregated_status.clone(),
                components: component_statuses,
                error: None,
            });

            // Trim if necessary
            if history.len() > self.max_history {
                history.remove(0);
            }
        }

        // Build the response
        let mut response = serde_json::Map::new();
        response.insert(
            "status".to_string(),
            serde_json::Value::String(aggregated_status),
        );

        if self.config.detailed_components {
            response.insert(
                "components".to_string(),
                serde_json::Value::Object(components),
            );
        }

        // Add performance metrics
        if self.config.include_performance {
            response.insert(
                "responseTime".to_string(),
                serde_json::Value::Number(serde_json::Number::from(
                    check_duration.as_millis() as u64
                )),
            );
        }

        // Add history if requested
        if self.config.include_history {
            if let Ok(history) = self.history.read() {
                let history_json = json!(*history);
                response.insert("history".to_string(), history_json);
            }
        }

        Ok(serde_json::Value::Object(response))
    }

    /// Format a cached result with information about the cache
    fn format_cached_result(&self, entry: &HealthStatusHistoryEntry, age: Duration) -> Value {
        let mut response = serde_json::Map::new();

        // Add status
        response.insert(
            "status".to_string(),
            serde_json::Value::String(entry.status.clone()),
        );

        // Add cache information
        response.insert("cached".to_string(), serde_json::Value::Bool(true));

        response.insert(
            "cacheAge".to_string(),
            serde_json::Value::Number(serde_json::Number::from(age.as_millis() as u64)),
        );

        // Add timestamp
        response.insert("timestamp".to_string(), json!(entry.timestamp));

        // Add components if detailed
        if self.config.detailed_components {
            let mut components = serde_json::Map::new();

            for (name, status) in &entry.components {
                let mut component_details = serde_json::Map::new();
                component_details.insert(
                    "status".to_string(),
                    serde_json::Value::String(status.clone()),
                );

                components.insert(name.clone(), serde_json::Value::Object(component_details));
            }

            response.insert(
                "components".to_string(),
                serde_json::Value::Object(components),
            );
        }

        serde_json::Value::Object(response)
    }

    /// Get health status history
    pub fn get_history(&self) -> Result<Vec<HealthStatusHistoryEntry>, ServiceError> {
        let history = self.history.read().map_err(|e| {
            ServiceError::other(format!("Failed to acquire history read lock: {}", e))
        })?;

        Ok(history.clone())
    }

    /// Clear history
    pub fn clear_history(&self) -> Result<(), ServiceError> {
        let mut history = self.history.write().map_err(|e| {
            ServiceError::other(format!("Failed to acquire history write lock: {}", e))
        })?;

        history.clear();
        Ok(())
    }

    /// Register a dynamic health indicator
    pub fn register_indicator(
        &self,
        indicator: Box<dyn HealthIndicator>,
    ) -> Result<(), ServiceError> {
        let mut discovery = self.discovery.write().map_err(|e| {
            ServiceError::other(format!("Failed to acquire discovery write lock: {}", e))
        })?;

        discovery.register_dynamic_indicator(indicator);
        Ok(())
    }

    /// Remove a dynamic health indicator
    pub fn remove_indicator(&self, name: &str) -> Result<bool, ServiceError> {
        let mut discovery = self.discovery.write().map_err(|e| {
            ServiceError::other(format!("Failed to acquire discovery write lock: {}", e))
        })?;

        Ok(discovery.remove_dynamic_indicator(name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::app_config::AppConfig;
    use crate::core::services::health_provider::HealthIndicatorProviderRegistry;

    #[derive(Clone)]
    struct TestHealthIndicator {
        name: String,
        status: String,
    }

    impl HealthIndicator for TestHealthIndicator {
        fn name(&self) -> String {
            self.name.clone()
        }

        fn check_health(&self, _state: &Arc<AppState>) -> DependencyStatus {
            DependencyStatus {
                name: self.name(),
                status: self.status.clone(),
                details: Some("Test indicator".to_string()),
            }
        }

        fn is_critical(&self) -> bool {
            true
        }

        fn clone_box(&self) -> Box<dyn HealthIndicator> {
            Box::new(self.clone())
        }
    }

    #[tokio::test]
    async fn test_health_dashboard() {
        // Create app state
        let state = Arc::new(AppState {
            config: AppConfig::default(),
            start_time: SystemTime::now(),
            cache_registry: None,
            client: None,
            token_client: None,
            metrics_handle: None,
            resource_registry: None,
            service_registry: Arc::new(crate::core::router::ServiceRegistry::new()),
        });

        // Create registry
        let registry = Arc::new(HealthIndicatorProviderRegistry::new());

        // Create discovery service with test config
        let discovery = Arc::new(RwLock::new(HealthDiscoveryService::new(
            registry,
            HealthConfig::default(),
        )));

        // Register a test indicator
        {
            let mut discovery = discovery.write().unwrap();
            discovery.register_dynamic_indicator(Box::new(TestHealthIndicator {
                name: "test1".to_string(),
                status: "UP".to_string(),
            }));
        }

        // Create dashboard service with fast check interval for testing
        let dashboard_config = HealthDashboardConfig {
            min_check_interval: Duration::from_millis(50),
            ..HealthDashboardConfig::default()
        };

        let dashboard = HealthDashboardService::new(discovery, dashboard_config);

        // Check health
        let result = dashboard.check_health(&state).await.unwrap();

        // Verify result
        assert_eq!(result["status"], "UP");
        assert!(result["components"].is_object());
        assert!(result["components"]["test1"].is_object());
        assert_eq!(result["components"]["test1"]["status"], "UP");

        // History should have one entry
        let history = dashboard.get_history().unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].status, "UP");

        // Add a second indicator that's DOWN
        dashboard
            .register_indicator(Box::new(TestHealthIndicator {
                name: "test2".to_string(),
                status: "DOWN".to_string(),
            }))
            .unwrap();

        // Wait for rate limit to expire
        tokio::time::sleep(Duration::from_millis(60)).await;

        // Check health again
        let result = dashboard.check_health(&state).await.unwrap();

        // Status should be DOWN because of test2
        assert_eq!(result["status"], "DOWN");
        assert!(result["components"]["test2"].is_object());
        assert_eq!(result["components"]["test2"]["status"], "DOWN");

        // History should have two entries
        let history = dashboard.get_history().unwrap();
        assert_eq!(history.len(), 2);
        assert_eq!(history[1].status, "DOWN");

        // Test getting cached result
        let cached = dashboard.check_health(&state).await.unwrap();
        assert_eq!(cached["cached"], true);

        // Clear history
        dashboard.clear_history().unwrap();

        // History should be empty
        let history = dashboard.get_history().unwrap();
        assert_eq!(history.len(), 0);

        // Remove test2
        let removed = dashboard.remove_indicator("test2").unwrap();
        assert!(removed);

        // Wait for rate limit
        tokio::time::sleep(Duration::from_millis(60)).await;

        // Check health again
        let result = dashboard.check_health(&state).await.unwrap();

        // Status should be UP again
        assert_eq!(result["status"], "UP");
        assert!(result["components"]["test1"].is_object());
        assert!(!result.get("components").unwrap().get("test2").is_some());
    }
}
