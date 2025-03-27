use std::collections::HashMap;
use std::sync::Arc;

use crate::core::models::DependencyStatus;
use crate::core::router::AppState;
use crate::core::services::health_provider::{
    HealthConfig, HealthIndicator, HealthIndicatorProvider,
};

//------------------------------------------------------------------------------
// Core Health Indicators
//------------------------------------------------------------------------------

/// Cache health indicator
#[derive(Clone)]
pub struct CacheHealthIndicator;

impl HealthIndicator for CacheHealthIndicator {
    fn name(&self) -> String {
        "cache".to_string()
    }

    fn check_health(&self, state: &Arc<AppState>) -> DependencyStatus {
        DependencyStatus {
            name: self.name(),
            status: match &state.cache_registry {
                Some(_) => "UP".to_string(),
                None => "DOWN".to_string(),
            },
            details: Some(format!(
                "Cache {}",
                match &state.cache_registry {
                    Some(_) => "enabled",
                    None => "disabled",
                }
            )),
        }
    }

    fn order(&self) -> i32 {
        10
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut metadata = HashMap::new();
        metadata.insert("type".to_string(), "cache".to_string());
        metadata
    }

    fn clone_box(&self) -> Box<dyn HealthIndicator> {
        Box::new(self.clone())
    }

    fn is_critical(&self) -> bool {
        false
    }
}

/// Disk space health indicator
#[derive(Clone)]
pub struct DiskSpaceHealthIndicator;

impl HealthIndicator for DiskSpaceHealthIndicator {
    fn name(&self) -> String {
        "diskSpace".to_string()
    }

    fn check_health(&self, _state: &Arc<AppState>) -> DependencyStatus {
        // In a real implementation, this would use sys-info or similar
        // to get actual disk information
        DependencyStatus {
            name: self.name(),
            status: "UP".to_string(),
            details: Some("total: 10GB, free: 5GB, threshold: 100MB".to_string()),
        }
    }

    fn order(&self) -> i32 {
        20
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut metadata = HashMap::new();
        metadata.insert("type".to_string(), "disk".to_string());
        metadata
    }

    fn clone_box(&self) -> Box<dyn HealthIndicator> {
        Box::new(self.clone())
    }

    fn is_critical(&self) -> bool {
        true // Disk space issues are critical
    }
}

/// Environment health indicator
#[derive(Clone)]
pub struct EnvironmentHealthIndicator;

impl EnvironmentHealthIndicator {
    fn detect_environment() -> String {
        if std::env::var("PRODUCTION").is_ok() {
            "production".to_string()
        } else if std::env::var("STAGING").is_ok() {
            "staging".to_string()
        } else if std::env::var("CI").is_ok() {
            "ci".to_string()
        } else {
            "development".to_string()
        }
    }
}

impl HealthIndicator for EnvironmentHealthIndicator {
    fn name(&self) -> String {
        "env".to_string()
    }

    fn check_health(&self, _state: &Arc<AppState>) -> DependencyStatus {
        let env = Self::detect_environment();
        DependencyStatus {
            name: self.name(),
            status: "UP".to_string(),
            details: Some(format!("Environment: {}", env)),
        }
    }

    fn order(&self) -> i32 {
        5
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut metadata = HashMap::new();
        metadata.insert("type".to_string(), "environment".to_string());
        metadata.insert("environment".to_string(), Self::detect_environment());
        metadata
    }

    fn clone_box(&self) -> Box<dyn HealthIndicator> {
        Box::new(self.clone())
    }

    fn is_critical(&self) -> bool {
        false
    }
}

/// Service registry health indicator
#[derive(Clone)]
pub struct ServiceRegistryHealthIndicator;

impl HealthIndicator for ServiceRegistryHealthIndicator {
    fn name(&self) -> String {
        "services".to_string()
    }

    fn check_health(&self, state: &Arc<AppState>) -> DependencyStatus {
        let service_count = state.service_registry.service_count();
        DependencyStatus {
            name: self.name(),
            status: "UP".to_string(),
            details: Some(format!(
                "Service registry: {} services active",
                service_count
            )),
        }
    }

    fn order(&self) -> i32 {
        30
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut metadata = HashMap::new();
        metadata.insert("type".to_string(), "services".to_string());
        metadata
    }

    fn clone_box(&self) -> Box<dyn HealthIndicator> {
        Box::new(self.clone())
    }

    fn is_critical(&self) -> bool {
        true // Service registry is critical
    }
}

//------------------------------------------------------------------------------
// Health Indicator Providers
//------------------------------------------------------------------------------

/// Core health indicator provider for basic system indicators
pub struct CoreHealthIndicatorProvider;

impl HealthIndicatorProvider for CoreHealthIndicatorProvider {
    fn create_indicators(&self, config: &HealthConfig) -> Vec<Box<dyn HealthIndicator>> {
        let mut indicators: Vec<Box<dyn HealthIndicator>> = Vec::new();

        // Add environment indicator
        if config.show_environment {
            indicators.push(Box::new(EnvironmentHealthIndicator));
        }

        // Add cache indicator
        if config.show_cache {
            indicators.push(Box::new(CacheHealthIndicator));
        }

        // Add disk space indicator
        if config.show_disk_space {
            indicators.push(Box::new(DiskSpaceHealthIndicator));
        }

        // Add service registry indicator
        if config.show_service_registry {
            indicators.push(Box::new(ServiceRegistryHealthIndicator));
        }

        indicators
    }

    fn is_enabled(&self, _config: &HealthConfig) -> bool {
        true // Core provider is always enabled
    }

    fn name(&self) -> String {
        "core".to_string()
    }
}

/// Database health indicator provider
pub struct DatabaseHealthIndicatorProvider;

impl DatabaseHealthIndicatorProvider {
    /// Create a new database health indicator provider
    pub fn new() -> Self {
        Self
    }
}

impl HealthIndicatorProvider for DatabaseHealthIndicatorProvider {
    fn create_indicators(&self, _config: &HealthConfig) -> Vec<Box<dyn HealthIndicator>> {
        vec![Box::new(DatabaseHealthIndicator)]
    }

    fn is_enabled(&self, _config: &HealthConfig) -> bool {
        true // Always enabled for now
    }

    fn name(&self) -> String {
        "database".to_string()
    }
}

/// Database health indicator
#[derive(Clone)]
pub struct DatabaseHealthIndicator;

impl HealthIndicator for DatabaseHealthIndicator {
    fn name(&self) -> String {
        "db".to_string()
    }

    fn check_health(&self, _state: &Arc<AppState>) -> DependencyStatus {
        // In a real implementation, this would check the database connection
        // For now, we just return DISABLED since we've removed the database
        DependencyStatus {
            name: self.name(),
            status: "UNKNOWN".to_string(),
            details: Some("Database connection is DISABLED".to_string()),
        }
    }

    fn order(&self) -> i32 {
        15
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut metadata = HashMap::new();
        metadata.insert("type".to_string(), "database".to_string());
        metadata.insert("status".to_string(), "disabled".to_string());
        metadata
    }

    fn clone_box(&self) -> Box<dyn HealthIndicator> {
        Box::new(self.clone())
    }

    fn is_critical(&self) -> bool {
        true // Database connectivity is critical
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::app_config::AppConfig;

    #[test]
    fn test_cache_health_indicator() {
        // Create app state with no cache registry
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

        let indicator = CacheHealthIndicator;
        let result = indicator.check_health(&state);

        assert_eq!(result.name, "cache");
        assert_eq!(result.status, "DOWN");
        assert_eq!(result.details, Some("Cache disabled".to_string()));
    }

    #[test]
    fn test_disk_space_health_indicator() {
        let state = Arc::new(AppState::default());
        let indicator = DiskSpaceHealthIndicator;
        let result = indicator.check_health(&state);

        assert_eq!(result.name, "diskSpace");
        assert_eq!(result.status, "UP");
        assert!(result.details.is_some());
    }

    #[test]
    fn test_environment_health_indicator() {
        let state = Arc::new(AppState::default());
        let indicator = EnvironmentHealthIndicator;
        let result = indicator.check_health(&state);

        assert_eq!(result.name, "env");
        assert_eq!(result.status, "UP");
        assert!(result.details.unwrap().contains("Environment:"));
    }

    #[test]
    fn test_core_provider() {
        let provider = CoreHealthIndicatorProvider;
        let config = HealthConfig::default();

        // Provider should be enabled
        assert!(provider.is_enabled(&config));

        // Should create indicators based on config
        let indicators = provider.create_indicators(&config);
        assert!(!indicators.is_empty());

        // With everything disabled, should return empty list
        let empty_config = HealthConfig {
            show_cache: false,
            show_disk_space: false,
            show_environment: false,
            show_service_registry: false,
            ..HealthConfig::default()
        };
        let empty_indicators = provider.create_indicators(&empty_config);
        assert!(empty_indicators.is_empty());
    }
}
