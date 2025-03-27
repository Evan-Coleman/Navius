use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;

use crate::core::models::DependencyStatus;
use crate::core::router::AppState;
use crate::core::services::error::ServiceError;

/// Enhanced HealthIndicator trait for health checks
pub trait HealthIndicator: Send + Sync + 'static {
    /// Get the name of this health indicator
    fn name(&self) -> String;

    /// Check the health of this component
    fn check_health(&self, state: &Arc<AppState>) -> DependencyStatus;

    /// Get metadata about this indicator
    fn metadata(&self) -> HashMap<String, String> {
        HashMap::new()
    }

    /// Get the order in which this indicator should be checked
    fn order(&self) -> i32 {
        0
    }

    /// Whether this indicator is critical (failure means system is down)
    fn is_critical(&self) -> bool;

    /// Clone this indicator
    fn clone_box(&self) -> Box<dyn HealthIndicator>;
}

/// Configuration for health service
#[derive(Debug, Clone)]
pub struct HealthConfig {
    /// Show details in health response
    pub show_details: bool,

    /// Show components in health response
    pub show_components: bool,

    /// Show disk space
    pub show_disk_space: bool,

    /// Show cache status
    pub show_cache: bool,

    /// Show environment information
    pub show_environment: bool,

    /// Show service registry status
    pub show_service_registry: bool,

    /// Provider-specific configuration
    pub provider_config: HashMap<String, String>,
}

impl Default for HealthConfig {
    fn default() -> Self {
        Self {
            show_details: true,
            show_components: true,
            show_disk_space: true,
            show_cache: true,
            show_environment: true,
            show_service_registry: true,
            provider_config: HashMap::new(),
        }
    }
}

/// Health indicator provider trait
pub trait HealthIndicatorProvider: Send + Sync + 'static {
    /// Create health indicators for the application
    fn create_indicators(&self, config: &HealthConfig) -> Vec<Box<dyn HealthIndicator>>;

    /// Whether this provider is enabled
    fn is_enabled(&self, config: &HealthConfig) -> bool;

    /// Get the name of this provider
    fn name(&self) -> String;
}

/// Registry for health indicator providers
pub struct HealthIndicatorProviderRegistry {
    providers: Vec<Box<dyn HealthIndicatorProvider>>,
}

impl HealthIndicatorProviderRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
        }
    }

    /// Register a provider
    pub fn register(&mut self, provider: Box<dyn HealthIndicatorProvider>) {
        self.providers.push(provider);
    }

    /// Get all indicators from enabled providers
    pub fn get_indicators(&self, config: &HealthConfig) -> Vec<Box<dyn HealthIndicator>> {
        let mut indicators = Vec::new();

        for provider in &self.providers {
            if provider.is_enabled(config) {
                let provider_indicators = provider.create_indicators(config);
                indicators.extend(provider_indicators);
            }
        }

        // Sort by order
        indicators.sort_by_key(|i| i.order());

        indicators
    }

    /// Get all provider names
    pub fn get_provider_names(&self) -> Vec<String> {
        self.providers.iter().map(|p| p.name()).collect()
    }
}

/// Enhanced health service that uses the provider system
pub struct HealthServiceV2 {
    registry: Arc<HealthIndicatorProviderRegistry>,
    config: HealthConfig,
}

impl HealthServiceV2 {
    /// Create a new health service with the given registry and config
    pub fn new(registry: Arc<HealthIndicatorProviderRegistry>, config: HealthConfig) -> Self {
        Self { registry, config }
    }

    /// Check health of all components
    pub async fn check_health(&self, state: &Arc<AppState>) -> Result<Value, ServiceError> {
        let indicators = self.registry.get_indicators(&self.config);
        let mut aggregated_status = "UP".to_string();
        let mut components = serde_json::Map::new();

        // Check each health indicator
        for indicator in indicators {
            let result = indicator.check_health(state);

            // If any critical component is down, the overall status is down
            if result.status != "UP" && indicator.is_critical() {
                aggregated_status = "DOWN".to_string();
            }

            // If we're showing components, add to response
            if self.config.show_components {
                let mut component_details = serde_json::Map::new();
                component_details.insert(
                    "status".to_string(),
                    serde_json::Value::String(result.status),
                );

                if self.config.show_details && result.details.is_some() {
                    component_details.insert(
                        "details".to_string(),
                        serde_json::Value::String(result.details.unwrap()),
                    );
                }

                // Add metadata if present
                let metadata = indicator.metadata();
                if !metadata.is_empty() {
                    for (k, v) in metadata {
                        component_details.insert(k, serde_json::Value::String(v));
                    }
                }

                components.insert(
                    indicator.name(),
                    serde_json::Value::Object(component_details),
                );
            }
        }

        // Build the response
        let mut response = serde_json::Map::new();
        response.insert(
            "status".to_string(),
            serde_json::Value::String(aggregated_status),
        );

        if self.config.show_components {
            response.insert(
                "components".to_string(),
                serde_json::Value::Object(components),
            );
        }

        Ok(serde_json::Value::Object(response))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::app_config::AppConfig;
    use crate::core::router::ServiceRegistry;
    use std::time::SystemTime;

    // Test health indicator
    #[derive(Clone)]
    struct TestHealthIndicator {
        name: String,
        status: String,
        is_crit: bool,
    }

    impl TestHealthIndicator {
        fn new(name: &str, status: &str, is_critical: bool) -> Self {
            Self {
                name: name.to_string(),
                status: status.to_string(),
                is_crit: is_critical,
            }
        }
    }

    impl HealthIndicator for TestHealthIndicator {
        fn name(&self) -> String {
            self.name.clone()
        }

        fn check_health(&self, _state: &Arc<AppState>) -> DependencyStatus {
            DependencyStatus {
                name: self.name(),
                status: self.status.clone(),
                details: Some("Test details".to_string()),
            }
        }

        fn order(&self) -> i32 {
            0
        }

        fn is_critical(&self) -> bool {
            self.is_crit
        }

        fn clone_box(&self) -> Box<dyn HealthIndicator> {
            Box::new(self.clone())
        }
    }

    // Test provider that always returns UP
    struct TestUpProvider;

    impl HealthIndicatorProvider for TestUpProvider {
        fn create_indicators(&self, _config: &HealthConfig) -> Vec<Box<dyn HealthIndicator>> {
            vec![Box::new(TestHealthIndicator::new("test", "UP", true))]
        }

        fn is_enabled(&self, _config: &HealthConfig) -> bool {
            true
        }

        fn name(&self) -> String {
            "test-up".to_string()
        }
    }

    // Test provider that always returns DOWN
    struct TestDownProvider;

    impl HealthIndicatorProvider for TestDownProvider {
        fn create_indicators(&self, _config: &HealthConfig) -> Vec<Box<dyn HealthIndicator>> {
            vec![Box::new(TestHealthIndicator::new("test", "DOWN", true))]
        }

        fn is_enabled(&self, _config: &HealthConfig) -> bool {
            true
        }

        fn name(&self) -> String {
            "test-down".to_string()
        }
    }

    #[tokio::test]
    async fn test_health_service_v2() {
        // Create app state
        let state = Arc::new(AppState {
            config: AppConfig::default(),
            start_time: std::time::SystemTime::now(),
            cache_registry: None,
            client: None,
            token_client: None,
            metrics_handle: None,
            resource_registry: None,
            service_registry: Arc::new(crate::core::router::ServiceRegistry::new()),
        });

        // Create registry with test provider
        let mut registry = HealthIndicatorProviderRegistry::new();
        registry.register(Box::new(TestUpProvider));
        let registry = Arc::new(registry);

        // Create health service
        let health_service = HealthServiceV2::new(registry, HealthConfig::default());

        // Check health
        let result = health_service.check_health(&state).await.unwrap();

        // Check response
        assert_eq!(result["status"], "UP");
        assert!(result["components"].is_object());
        assert!(result["components"]["test"].is_object());
        assert_eq!(result["components"]["test"]["status"], "UP");
        assert_eq!(result["components"]["test"]["details"], "Test details");
    }

    #[tokio::test]
    async fn test_health_service_with_down_component() {
        // Create app state
        let state = Arc::new(AppState {
            config: AppConfig::default(),
            start_time: std::time::SystemTime::now(),
            cache_registry: None,
            client: None,
            token_client: None,
            metrics_handle: None,
            resource_registry: None,
            service_registry: Arc::new(crate::core::router::ServiceRegistry::new()),
        });

        // Create registry with down provider
        let mut registry = HealthIndicatorProviderRegistry::new();
        registry.register(Box::new(TestDownProvider));
        let registry = Arc::new(registry);

        // Create health service
        let health_service = HealthServiceV2::new(registry, HealthConfig::default());

        // Check health
        let result = health_service.check_health(&state).await.unwrap();

        // Check response - should be DOWN because the indicator is critical
        assert_eq!(result["status"], "DOWN");
        assert!(result["components"].is_object());
        assert!(result["components"]["test"].is_object());
        assert_eq!(result["components"]["test"]["status"], "DOWN");
    }

    #[tokio::test]
    async fn test_health_service_with_hidden_components() {
        // Create app state
        let state = Arc::new(AppState {
            config: AppConfig::default(),
            start_time: std::time::SystemTime::now(),
            cache_registry: None,
            client: None,
            token_client: None,
            metrics_handle: None,
            resource_registry: None,
            service_registry: Arc::new(crate::core::router::ServiceRegistry::new()),
        });

        // Create registry with test provider
        let mut registry = HealthIndicatorProviderRegistry::new();
        registry.register(Box::new(TestUpProvider));
        let registry = Arc::new(registry);

        // Create health service with components hidden
        let mut config = HealthConfig::default();
        config.show_components = false;
        let health_service = HealthServiceV2::new(registry, config);

        // Check health
        let result = health_service.check_health(&state).await.unwrap();

        // Check response - should not have components
        assert_eq!(result["status"], "UP");
        assert!(result.get("components").is_none());
    }
}
