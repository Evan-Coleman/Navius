use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, RwLock};
use tracing::{debug, info};

use crate::core::router::AppState;
use crate::models::DependencyStatus;
use crate::utils::api_resource::ApiResource;

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
        let resources = match self.resources.read() {
            Ok(resources) => resources,
            Err(_) => {
                debug!("Failed to acquire read lock on resource registry");
                return Vec::new();
            }
        };

        let mut results = Vec::new();

        for (_, registration) in resources.iter() {
            let health_check = &registration.health_check_fn;
            let status = health_check(state).await;
            results.push(status);
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
