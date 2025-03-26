// Core service module exports
pub mod database;
pub mod error;
pub mod health;
pub mod service_traits;

// Re-export key components
pub use database::{DatabaseConfig, DatabaseService, DatabaseServiceProvider, InMemoryDatabase};
pub use health::HealthService;
pub use service_traits::{Lifecycle, Service, ServiceProvider, ServiceRegistry};
