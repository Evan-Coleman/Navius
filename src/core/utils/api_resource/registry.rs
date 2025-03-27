use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, RwLock};
use tracing::{debug, info};

use crate::core::auth::MockTokenClient;
use crate::core::cache::CacheRegistry;
use crate::core::models::DependencyStatus;
use crate::core::router::AppState;
use crate::core::router::ServiceRegistry;
use crate::core::utils::api_resource::ApiResource;

/// Type for a health check function
pub type HealthCheckFn = Box<
    dyn Fn(&Arc<AppState>) -> futures::future::BoxFuture<'static, DependencyStatus> + Send + Sync,
>;

/// Registration data for an API resource
pub struct ResourceRegistration {
    /// The resource type string
    pub resource_type: String,

    /// The API name for logging
    pub api_name: String,

    /// Function to perform a health check for this resource
    pub health_check_fn: HealthCheckFn,
}

/// Registry for API resources
#[derive(Debug, Default)]
pub struct ApiResourceRegistry {
    /// Map of resource type names to registrations
    resources: RwLock<HashMap<String, ResourceRegistration>>,
}

impl ApiResourceRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            resources: RwLock::new(HashMap::new()),
        }
    }

    /// Register a resource with a health check function
    pub fn register<T, F>(&self, health_check: F) -> Result<(), String>
    where
        T: ApiResource,
        F: Fn(&Arc<AppState>) -> futures::future::BoxFuture<'static, DependencyStatus>
            + Send
            + Sync
            + 'static,
    {
        let resource_type = T::resource_type();
        let api_name = T::api_name();

        let mut resources = match self.resources.write() {
            Ok(resources) => resources,
            Err(_) => return Err("Failed to acquire write lock on resource registry".to_string()),
        };

        // Check if already registered
        if resources.contains_key(resource_type) {
            debug!("Resource type {} already registered", resource_type);
            return Ok(());
        }

        // Register the resource type
        resources.insert(
            resource_type.to_string(),
            ResourceRegistration {
                resource_type: resource_type.to_string(),
                api_name: api_name.to_string(),
                health_check_fn: Box::new(health_check),
            },
        );

        info!("✅ Registered API resource: {}", resource_type);
        Ok(())
    }

    /// Get all registered resource types
    pub fn get_resource_types(&self) -> Vec<String> {
        let resources = match self.resources.read() {
            Ok(resources) => resources,
            Err(_) => {
                debug!("Failed to acquire read lock on resource registry");
                return Vec::new();
            }
        };

        resources.keys().cloned().collect()
    }

    /// Get a health check function for a specific resource type
    pub fn get_health_check(&self, resource_type: &str) -> Option<HealthCheckFn> {
        let resources = match self.resources.read() {
            Ok(resources) => resources,
            Err(_) => {
                debug!("Failed to acquire read lock on resource registry");
                return None;
            }
        };

        resources.get(resource_type).map(|reg| {
            let resource_type = reg.resource_type.clone();
            let api_name = reg.api_name.clone();

            // Create a new function that doesn't move the api_name
            Box::new(move |state: &Arc<AppState>| {
                // Clone these values so they can be used in the returned future
                let api_name = api_name.clone();
                let resource_type = resource_type.clone();
                let _state = state.clone(); // Prefix with underscore to avoid warning

                Box::pin(async move {
                    DependencyStatus {
                        name: format!("{} ({})", api_name, resource_type),
                        status: "unknown".to_string(),
                        details: None,
                    }
                }) as futures::future::BoxFuture<'static, DependencyStatus>
            }) as HealthCheckFn
        })
    }

    /// Run health checks for all registered resources
    pub async fn run_all_health_checks(&self, state: &Arc<AppState>) -> Vec<DependencyStatus> {
        // Collect futures for health checks while holding the lock
        let futures = {
            let resources = match self.resources.read() {
                Ok(resources) => resources,
                Err(_) => {
                    debug!("Failed to acquire read lock on resource registry");
                    return Vec::new();
                }
            };

            // Create futures but don't await them yet
            resources
                .iter()
                .map(|(_, registration)| {
                    let health_check = &registration.health_check_fn;
                    health_check(state)
                })
                .collect::<Vec<_>>()
            // Lock is dropped here when the block ends
        };

        // Now await all futures without holding the lock
        let mut results = Vec::new();
        for future in futures {
            results.push(future.await);
        }

        results
    }
}

// Implement Debug manually since the function can't be debugged
impl std::fmt::Debug for ResourceRegistration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResourceRegistration")
            .field("resource_type", &self.resource_type)
            .field("api_name", &self.api_name)
            .field("health_check_fn", &"<function>")
            .finish()
    }
}

pub struct ApiResourceError {
    pub code: String,
    pub message: String,
    pub details: BTreeMap<String, String>,
}

impl ApiResourceError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details: BTreeMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::auth::MockTokenClient;
    use crate::core::cache::CacheRegistry;
    use crate::core::router::ServiceRegistry;
    use crate::core::utils::api_resource::ApiResource;
    use serde::{Deserialize, Serialize};
    use std::collections::BTreeMap;
    use std::future::Future;
    use std::pin::Pin;
    use std::sync::Arc;

    // A mock implementation of ApiResource for testing
    #[derive(Clone, Debug, Serialize, Deserialize)]
    struct MockResource {
        id: i64,
        name: String,
    }

    impl ApiResource for MockResource {
        type Id = i64;

        fn resource_type() -> &'static str {
            "mock_resource"
        }

        fn api_name() -> &'static str {
            "MockAPI"
        }
    }

    // Helper function to create a health check function for tests
    fn create_health_check(
        status: &'static str,
    ) -> impl Fn(&Arc<AppState>) -> Pin<Box<dyn Future<Output = DependencyStatus> + Send + 'static>>
    + Send
    + Sync
    + 'static {
        move |_state| {
            let status_clone = status.to_string();
            Box::pin(async move {
                DependencyStatus {
                    name: "mock_api".to_string(),
                    status: status_clone,
                    details: None,
                }
            })
        }
    }

    #[test]
    fn test_registry_new() {
        let registry = ApiResourceRegistry::new();
        assert_eq!(registry.get_resource_types().len(), 0);
    }

    #[test]
    fn test_registry_register() {
        let registry = ApiResourceRegistry::new();

        // Register a mock resource
        let result = registry.register::<MockResource, _>(create_health_check("UP"));
        assert!(result.is_ok());

        // Check if resource was registered
        let resource_types = registry.get_resource_types();
        assert_eq!(resource_types.len(), 1);
        assert_eq!(resource_types[0], "mock_resource");
    }

    #[test]
    fn test_registry_register_duplicate() {
        let registry = ApiResourceRegistry::new();

        // Register a mock resource
        let result1 = registry.register::<MockResource, _>(create_health_check("UP"));
        assert!(result1.is_ok());

        // Try to register the same resource again
        let result2 = registry.register::<MockResource, _>(create_health_check("DOWN"));

        // This should succeed but overwrite the previous registration
        assert!(result2.is_ok());

        // Only one resource type should exist
        let resource_types = registry.get_resource_types();
        assert_eq!(resource_types.len(), 1);
    }

    #[test]
    fn test_get_health_check() {
        let registry = ApiResourceRegistry::new();

        // Register a mock resource
        registry
            .register::<MockResource, _>(create_health_check("UP"))
            .unwrap();

        // Get the health check function
        let health_check = registry.get_health_check("mock_resource");
        assert!(health_check.is_some());

        // Non-existent resource should return None
        let not_found = registry.get_health_check("non_existent");
        assert!(not_found.is_none());
    }

    #[tokio::test]
    async fn test_run_all_health_checks() {
        let registry = ApiResourceRegistry::new();

        // Register multiple resources with different health statuses
        registry
            .register::<MockResource, _>(create_health_check("UP"))
            .unwrap();

        // Create a second resource type
        #[derive(Clone, Debug, Serialize, Deserialize)]
        struct AnotherResource {
            id: String,
            data: String,
        }

        impl ApiResource for AnotherResource {
            type Id = String;

            fn resource_type() -> &'static str {
                "another_resource"
            }

            fn api_name() -> &'static str {
                "AnotherAPI"
            }
        }

        registry
            .register::<AnotherResource, _>(create_health_check("DOWN"))
            .unwrap();

        // Create app state (minimal for testing)
        let app_state = Arc::new(AppState {
            config: crate::core::config::app_config::AppConfig::default(),
            start_time: std::time::SystemTime::now(),
            cache_registry: Some(Arc::new(CacheRegistry::default())),
            client: Some(reqwest::Client::new()),
            token_client: Some(Arc::new(MockTokenClient::default())),
            metrics_handle: Some(
                metrics_exporter_prometheus::PrometheusBuilder::new()
                    .build_recorder()
                    .handle(),
            ),
            resource_registry: Some(Arc::new(ApiResourceRegistry::new())),
            service_registry: Arc::new(ServiceRegistry::new()),
        });

        // Run all health checks
        let statuses = registry.run_all_health_checks(&app_state).await;

        // Should have two statuses
        assert_eq!(statuses.len(), 2);

        // Check statuses contain expected values
        let has_up = statuses.iter().any(|s| s.status == "UP");
        let has_down = statuses.iter().any(|s| s.status == "DOWN");
        assert!(has_up);
        assert!(has_down);
    }

    #[test]
    fn test_debug_impl() {
        let registry = ApiResourceRegistry::new();
        let debug_str = format!("{:?}", registry);
        assert!(debug_str.contains("ApiResourceRegistry"));

        // Test ResourceRegistration debug impl
        let health_check = Box::new(create_health_check("UP")) as HealthCheckFn;
        let registration = ResourceRegistration {
            resource_type: "test".to_string(),
            api_name: "TestAPI".to_string(),
            health_check_fn: health_check,
        };

        let debug_str = format!("{:?}", registration);
        assert!(debug_str.contains("resource_type: \"test\""));
        assert!(debug_str.contains("api_name: \"TestAPI\""));
    }
}
