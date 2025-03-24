//! # Services module
//!
//! This module provides services that implement business logic.
//! Services use repositories to interact with data and implement business rules.

#[cfg(test)]
use mock_it::Mock;

pub mod error;
pub mod health;
pub mod metrics;
pub mod pet;
pub use error::ServiceError;
pub use pet::PetService as CorePetService;

#[cfg(test)]
mod tests;

#[cfg(test)]
pub type MockUserService = Mock<dyn IUserService, ()>;

/// Type alias for service results
pub type ServiceResult<T> = Result<T, ServiceError>;

use crate::app::database::repositories::pet_repository::PetRepository;
use crate::app::database::repositories::pet_repository::PgPetRepository;
use crate::app::services::PetService as AppPetService;
use crate::app::services::pet_service::PetService;
use crate::core::database::connection::DatabaseConnection;
use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Trait for service registry with required methods
pub trait ServiceRegistryTrait {
    /// Get the pet service
    fn pet_service(&self) -> &dyn Any;
}

/// Registry for managing application services
#[derive(Clone)]
pub struct ServiceRegistry {
    pub pet_service: Arc<dyn Any + Send + Sync>,
    services: Arc<RwLock<HashMap<String, Arc<dyn Any + Send + Sync>>>>,
}

impl ServiceRegistry {
    /// Create a new service registry
    pub fn new<R: PetRepository + 'static>(pet_service: Arc<PetService<R>>) -> Self {
        let mut services = HashMap::new();
        services.insert(
            "pet_service".to_string(),
            pet_service.clone() as Arc<dyn Any + Send + Sync>,
        );

        Self {
            pet_service: pet_service as Arc<dyn Any + Send + Sync>,
            services: Arc::new(RwLock::new(services)),
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
    pub fn pet_service(&self) -> Option<Arc<dyn crate::app::services::IPetService + Send + Sync>> {
        self.get::<dyn crate::app::services::IPetService + Send + Sync>("pet_service")
    }
}

impl Default for ServiceRegistry {
    fn default() -> Self {
        panic!("ServiceRegistry must be initialized with required services")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::services::pet_service::MockPetRepository;

    #[test]
    fn test_service_registry_new() {
        let mock_repository = Arc::new(MockPetRepository::default());
        let pet_service = Arc::new(PetService::new(mock_repository));
        let registry = ServiceRegistry::new(pet_service);
        assert!(Arc::strong_count(&registry.pet_service) > 0);
    }

    #[test]
    #[should_panic(expected = "ServiceRegistry must be initialized with required services")]
    fn test_service_registry_default_panics() {
        let _registry = ServiceRegistry::default();
    }
}

#[cfg(test)]
pub mod mock;

pub struct Services {
    db_connection: Option<Arc<dyn DatabaseConnection>>,
    pet_service: Option<Arc<dyn crate::app::services::IPetService + Send + Sync>>,
}

impl Services {
    pub fn new(db_connection: Option<Arc<dyn DatabaseConnection>>) -> Self {
        let pet_service = db_connection.as_ref().map(|conn| {
            Arc::new(AppPetService::new(conn.clone()))
                as Arc<dyn crate::app::services::IPetService + Send + Sync>
        });

        Self {
            db_connection,
            pet_service,
        }
    }

    pub fn get_pet_service(
        &self,
    ) -> Option<Arc<dyn crate::app::services::IPetService + Send + Sync>> {
        self.pet_service.clone()
    }
}
