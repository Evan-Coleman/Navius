//! # Services module
//!
//! This module provides services that implement business logic.
//! Services use repositories to interact with data and implement business rules.

#[cfg(test)]
use mock_it::Mock;

pub mod error;
pub mod user;
pub use error::ServiceError;
pub use user::{IUserService, UserService};

#[cfg(test)]
mod tests;

#[cfg(test)]
pub type MockUserService = Mock<dyn IUserService, ()>;

/// Type alias for service results
pub type ServiceResult<T> = Result<T, ServiceError>;

use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Trait for service registry with required methods
pub trait ServiceRegistryTrait {
    /// Get the pet service
    fn pet_service(&self) -> &dyn Any;
}

/// Registry for managing application services
pub struct ServiceRegistry {
    services: RwLock<HashMap<String, Arc<dyn Any + Send + Sync>>>,
}

impl ServiceRegistry {
    /// Create a new service registry
    pub fn new() -> Self {
        Self {
            services: RwLock::new(HashMap::new()),
        }
    }

    /// Register a service with the registry
    pub fn register<T: Any + Send + Sync>(&self, name: &str, service: T) {
        let mut services = self.services.write().unwrap();
        services.insert(name.to_string(), Arc::new(service));
    }

    /// Get a service from the registry
    pub fn get<T: Any + Send + Sync>(&self, name: &str) -> Option<Arc<T>> {
        self.services
            .read()
            .unwrap()
            .get(name)
            .cloned()
            .and_then(|service| service.downcast::<T>().ok())
    }

    /// Get a pet service
    pub fn pet_service(
        &self,
    ) -> Option<Arc<dyn crate::app::services::pet_service::IPetService + Send + Sync>> {
        self.get::<crate::app::services::pet_service::PetService>("pet_service")
            .map(|svc| svc as Arc<dyn crate::app::services::pet_service::IPetService + Send + Sync>)
    }
}

impl Default for ServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}
