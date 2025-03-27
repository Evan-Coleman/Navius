use std::collections::HashMap;
use std::sync::Arc;

use tracing::{info, warn};

use crate::core::services::health_provider::{
    HealthConfig, HealthIndicator, HealthIndicatorProvider, HealthIndicatorProviderRegistry,
};

/// Health indicator discovery service
///
/// This service enables dynamic discovery and registration of health indicators
pub struct HealthDiscoveryService {
    registry: Arc<HealthIndicatorProviderRegistry>,
    config: HealthConfig,
    dynamic_indicators: HashMap<String, Box<dyn HealthIndicator>>,
}

impl HealthDiscoveryService {
    /// Create a new health discovery service
    pub fn new(registry: Arc<HealthIndicatorProviderRegistry>, config: HealthConfig) -> Self {
        Self {
            registry,
            config,
            dynamic_indicators: HashMap::new(),
        }
    }

    /// Register a dynamic health indicator at runtime
    pub fn register_dynamic_indicator(&mut self, indicator: Box<dyn HealthIndicator>) {
        let name = indicator.name();
        info!("Registering dynamic health indicator: {}", name);
        self.dynamic_indicators.insert(name, indicator);
    }

    /// Remove a dynamic health indicator
    pub fn remove_dynamic_indicator(&mut self, name: &str) -> bool {
        if self.dynamic_indicators.remove(name).is_some() {
            info!("Removed dynamic health indicator: {}", name);
            true
        } else {
            warn!(
                "Attempted to remove non-existent dynamic health indicator: {}",
                name
            );
            false
        }
    }

    /// Get all health indicators including dynamic ones
    pub fn get_all_indicators(&self) -> Vec<Box<dyn HealthIndicator>> {
        // Get indicators from providers
        let mut indicators = self.registry.get_indicators(&self.config);

        // Add dynamic indicators
        for (_, indicator) in &self.dynamic_indicators {
            indicators.push(indicator.clone_box());
        }

        // Sort by order
        indicators.sort_by_key(|i| i.order());

        indicators
    }

    /// Get a specific dynamic indicator by name
    pub fn get_dynamic_indicator(&self, name: &str) -> Option<&Box<dyn HealthIndicator>> {
        self.dynamic_indicators.get(name)
    }

    /// Get count of dynamic indicators
    pub fn dynamic_indicator_count(&self) -> usize {
        self.dynamic_indicators.len()
    }
}

/// Auto-discovery trait for health indicators
pub trait HealthIndicatorDiscovery {
    /// Discover health indicators in the application
    fn discover(&self) -> Vec<Box<dyn HealthIndicator>>;
}

/// Dynamic health indicator provider
pub struct DynamicHealthIndicatorProvider {
    indicators: Vec<Box<dyn HealthIndicator>>,
}

impl DynamicHealthIndicatorProvider {
    /// Create a new dynamic health indicator provider
    pub fn new() -> Self {
        Self {
            indicators: Vec::new(),
        }
    }

    /// Add an indicator
    pub fn add_indicator(&mut self, indicator: Box<dyn HealthIndicator>) {
        self.indicators.push(indicator);
    }
}

impl Clone for DynamicHealthIndicatorProvider {
    fn clone(&self) -> Self {
        let cloned_indicators = self.indicators.iter().map(|i| i.clone_box()).collect();

        Self {
            indicators: cloned_indicators,
        }
    }
}

impl HealthIndicatorProvider for DynamicHealthIndicatorProvider {
    fn create_indicators(&self, _config: &HealthConfig) -> Vec<Box<dyn HealthIndicator>> {
        // Return clones of our indicators
        self.indicators.iter().map(|i| i.clone_box()).collect()
    }

    fn is_enabled(&self, _config: &HealthConfig) -> bool {
        true
    }

    fn name(&self) -> String {
        "dynamic".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::DependencyStatus;
    use crate::core::router::AppState;

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

        fn clone_box(&self) -> Box<dyn HealthIndicator> {
            Box::new(self.clone())
        }

        fn is_critical(&self) -> bool {
            false
        }
    }

    #[test]
    fn test_dynamic_indicator_registration() {
        // Create registry
        let registry = Arc::new(HealthIndicatorProviderRegistry::new());

        // Create discovery service
        let mut discovery = HealthDiscoveryService::new(registry, HealthConfig::default());

        // Create test indicator
        let indicator = Box::new(TestHealthIndicator {
            name: "test1".to_string(),
            status: "UP".to_string(),
        });

        // Register it
        discovery.register_dynamic_indicator(indicator);

        // Should have one dynamic indicator
        assert_eq!(discovery.dynamic_indicator_count(), 1);

        // Get it
        let retrieved = discovery.get_dynamic_indicator("test1");
        assert!(retrieved.is_some());

        // Remove it
        let removed = discovery.remove_dynamic_indicator("test1");
        assert!(removed);

        // Should have no dynamic indicators
        assert_eq!(discovery.dynamic_indicator_count(), 0);
    }

    #[test]
    fn test_dynamic_provider() {
        // Create provider
        let mut provider = DynamicHealthIndicatorProvider::new();

        // Add indicators
        provider.add_indicator(Box::new(TestHealthIndicator {
            name: "test1".to_string(),
            status: "UP".to_string(),
        }));

        provider.add_indicator(Box::new(TestHealthIndicator {
            name: "test2".to_string(),
            status: "DOWN".to_string(),
        }));

        // Get indicators
        let indicators = provider.create_indicators(&HealthConfig::default());

        // Should have two
        assert_eq!(indicators.len(), 2);
    }
}
