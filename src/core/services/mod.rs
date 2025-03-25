//! # Services module
//!
//! This module provides services that implement business logic.
//! Services use repositories to interact with data and implement business rules.

use crate::app::database::repositories::pet_repository::PgPetRepository;
use crate::app::services::pet_service::{IPetService, PetService};
use crate::core::config::app_config::DatabaseConfig;
use crate::core::database::PgPool;
use crate::core::error::AppError;
use reqwest::Client;
use sqlx::{Pool, Postgres};
use std::any::Any;
use std::sync::Arc;

pub mod error;
pub mod health;
pub mod metrics;
pub mod pet;

pub use error::ServiceError;
pub use pet::PetService as CorePetService;

/// Type alias for service results
pub type ServiceResult<T> = Result<T, ServiceError>;

/// Trait for service registry with required methods
pub trait ServiceRegistryTrait {
    /// Get the pet service
    fn pet_service(&self) -> &dyn Any;
}

/// Registry for managing application services
pub struct ServiceRegistry {
    pet_service: Arc<dyn IPetService + Send + Sync>,
}

impl ServiceRegistry {
    /// Create a new service registry
    pub fn new(db_pool: Arc<Pool<Postgres>>) -> Self {
        let pet_repository = Arc::new(PgPetRepository::new(db_pool));
        let pet_service = Arc::new(PetService::new(pet_repository));

        Self { pet_service }
    }

    #[cfg(test)]
    pub fn new_with_services(pet_service: Arc<dyn IPetService + Send + Sync>) -> Self {
        Self { pet_service }
    }

    /// Get a pet service
    pub fn get_pet_service(&self) -> Result<Arc<dyn IPetService + Send + Sync>, AppError> {
        Ok(self.pet_service.clone())
    }
}

impl Default for ServiceRegistry {
    fn default() -> Self {
        panic!("ServiceRegistry must be initialized with required services")
    }
}

// Backward compatibility for Services type
pub struct Services {
    db_connection: Option<Arc<dyn PgPool>>,
    pet_service: Option<Arc<dyn IPetService + Send + Sync>>,
}

impl Services {
    pub fn new(db_connection: Option<Arc<dyn PgPool>>) -> Self {
        // Create a default Postgres pool
        let pool = Arc::new(Pool::<Postgres>::connect_lazy_with(
            sqlx::postgres::PgConnectOptions::new()
                .host("localhost")
                .port(5432)
                .database("postgres")
                .username("postgres")
                .password("postgres"),
        ));

        let pet_repository = Arc::new(PgPetRepository::new(pool));
        let pet_service =
            Some(Arc::new(PetService::new(pet_repository)) as Arc<dyn IPetService + Send + Sync>);

        Self {
            db_connection,
            pet_service,
        }
    }

    pub fn get_pet_service(&self) -> Option<Arc<dyn IPetService + Send + Sync>> {
        self.pet_service.clone()
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::app::database::repositories::pet_repository::tests::MockPetRepository;

    pub fn create_test_registry() -> Arc<ServiceRegistry> {
        let mock_repository = Arc::new(MockPetRepository::new(vec![]));
        let pet_service = Arc::new(PetService::new(mock_repository));

        Arc::new(ServiceRegistry { pet_service })
    }

    #[test]
    #[ignore] // Ignore this test as it requires database connection
    fn test_service_registry_new() {
        // Create a mock db_pool - in a real test we would use a more sophisticated mock
        let db_pool = Arc::new(Pool::<Postgres>::connect_lazy_with(
            sqlx::postgres::PgConnectOptions::new()
                .host("localhost")
                .port(5432)
                .database("test_db")
                .username("postgres")
                .password("postgres"),
        ));
        let registry = ServiceRegistry::new(db_pool);
        assert!(Arc::strong_count(&registry.pet_service) > 0);
    }

    #[test]
    #[should_panic(expected = "ServiceRegistry must be initialized with required services")]
    fn test_service_registry_default_panics() {
        let _registry = ServiceRegistry::default();
    }
}
