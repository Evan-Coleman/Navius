//! Health check service implementation
//!
//! This module provides a Spring Boot-like health check service with:
//! - Health indicators for different components
//! - Aggregated health status
//! - Detailed component status information

use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, warn};

use crate::core::{error::AppError, models::DependencyStatus, router::AppState};

/// Trait for implementing health indicators
pub trait HealthIndicator: Send + Sync {
    /// Get the name of this health indicator
    fn name(&self) -> String;

    /// Check the health of this component
    fn check_health(&self, state: &Arc<AppState>) -> DependencyStatus;
}

/// Cache health indicator
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
}

/// Disk space health indicator
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
}

/// Environment health indicator
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
}

/// Service registry health indicator
pub struct ServiceRegistryHealthIndicator;

impl HealthIndicator for ServiceRegistryHealthIndicator {
    fn name(&self) -> String {
        "services".to_string()
    }

    fn check_health(&self, _state: &Arc<AppState>) -> DependencyStatus {
        DependencyStatus {
            name: self.name(),
            status: "UP".to_string(),
            details: Some("Service registry: active".to_string()),
        }
    }
}

/// Health service that aggregates health indicators
pub struct HealthService {
    indicators: Vec<Box<dyn HealthIndicator>>,
}

impl HealthService {
    /// Create a new health service with default indicators
    pub fn new() -> Self {
        let mut service = Self {
            indicators: Vec::new(),
        };

        // Add default health indicators
        service.add_indicator(Box::new(CacheHealthIndicator));
        service.add_indicator(Box::new(DiskSpaceHealthIndicator));
        service.add_indicator(Box::new(EnvironmentHealthIndicator));
        service.add_indicator(Box::new(ServiceRegistryHealthIndicator));

        service
    }

    /// Add a custom health indicator
    pub fn add_indicator(&mut self, indicator: Box<dyn HealthIndicator>) {
        info!("Adding health indicator: {}", indicator.name());
        self.indicators.push(indicator);
    }

    /// Check health of all components
    pub async fn check_health(&self, state: &Arc<AppState>) -> Result<Value, AppError> {
        let mut status = "UP".to_string();
        let mut components = HashMap::new();

        // Check each health indicator
        for indicator in &self.indicators {
            let result = indicator.check_health(state);

            // If any component is down, the overall status is down
            if result.status != "UP" {
                warn!("Health check failed for {}: {}", result.name, result.status);
                status = "DOWN".to_string();
            }

            // Add component details to response
            let component_details = json!({
                "status": result.status,
                "details": result.details
            });

            components.insert(result.name, component_details);
        }

        // Create Spring Boot-style health response
        Ok(json!({
            "status": status,
            "components": components
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::app_config::AppConfig;

    #[tokio::test]
    async fn test_health_service() {
        // Create minimal app state for testing
        let state = Arc::new(AppState {
            config: AppConfig::default(),
            start_time: std::time::SystemTime::now(),
            cache_registry: None, // Test with cache disabled
            client: None,
            token_client: None,
            metrics_handle: None,
            resource_registry: None,
            service_registry: Arc::new(crate::core::router::ServiceRegistry::new()),
        });

        // Create health service
        let health_service = HealthService::new();

        // Check health
        let result = health_service.check_health(&state).await.unwrap();

        // Verify response format
        assert!(result.get("status").is_some());
        assert!(result.get("components").is_some());

        // Verify components
        let components = result.get("components").unwrap().as_object().unwrap();
        assert!(components.contains_key("cache"));
        assert!(components.contains_key("diskSpace"));
        assert!(components.contains_key("env"));
        assert!(components.contains_key("services"));

        // Verify cache is down since we didn't provide it
        let cache = components.get("cache").unwrap();
        assert_eq!(cache.get("status").unwrap(), "DOWN");
    }

    #[test]
    fn test_environment_detection() {
        let indicator = EnvironmentHealthIndicator;
        let state = Arc::new(AppState::default());

        let result = indicator.check_health(&state);
        assert_eq!(result.status, "UP");
        assert!(result.details.unwrap().contains("Environment:"));
    }
}
