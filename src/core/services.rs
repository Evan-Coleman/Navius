// Core service module exports
pub mod database_interface;
pub mod database_service;
pub mod error;
pub mod health;
pub mod memory_database;
pub mod service_traits;

// Re-export key components
pub use database_interface::{
    DatabaseConfig, DatabaseOperations, DatabaseProvider, DatabaseProviderRegistry,
};
pub use database_service::{DatabaseService, InMemoryDatabaseServiceProvider};
pub use health::HealthService;
pub use memory_database::{InMemoryDatabase, InMemoryDatabaseProvider};
pub use service_traits::{Lifecycle, Service, ServiceProvider, ServiceRegistry};
