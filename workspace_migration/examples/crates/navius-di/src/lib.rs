//! Navius Dependency Injection
//!
//! This crate provides a lightweight dependency injection system for Navius applications,
//! based on research of the spring-rs framework. It includes a component registry for
//! managing dependencies and lifecycle hooks for components.
//!
//! # Features
//!
//! - Component registry with type-safe dependency resolution
//! - Support for different component scopes (singleton, prototype, request, session)
//! - Component lifecycle hooks (synchronous and asynchronous)
//! - Qualifier support for component disambiguation
//! - Factory-based component creation
//! - Comprehensive error handling
//!
//! # Examples
//!
//! ```rust
//! use navius_di::{ComponentRegistry, ComponentScope, Result};
//!
//! struct DatabaseService {
//!     connection_string: String,
//! }
//!
//! impl DatabaseService {
//!     fn new(connection_string: String) -> Self {
//!         Self { connection_string }
//!     }
//!
//!     fn query(&self, sql: &str) -> String {
//!         format!("Executing '{}' on connection {}", sql, self.connection_string)
//!     }
//! }
//!
//! struct UserService {
//!     db: DatabaseService,
//! }
//!
//! impl UserService {
//!     fn new(db: DatabaseService) -> Self {
//!         Self { db }
//!     }
//!
//!     fn get_user(&self, id: &str) -> String {
//!         self.db.query(&format!("SELECT * FROM users WHERE id = '{}'", id))
//!     }
//! }
//!
//! fn main() -> Result<()> {
//!     // Create a component registry
//!     let registry = ComponentRegistry::new();
//!
//!     // Register a database service as a singleton
//!     registry.register_with_factory(
//!         || DatabaseService::new("jdbc:postgresql://localhost:5432/mydb".to_string()),
//!         ComponentScope::Singleton,
//!     );
//!
//!     // Register a user service that depends on the database service
//!     registry.register_with_factory(
//!         || {
//!             let db = registry.get::<DatabaseService>().unwrap();
//!             UserService::new((*db).clone())
//!         },
//!         ComponentScope::Prototype,
//!     );
//!
//!     // Get the user service and use it
//!     let user_service = registry.get::<UserService>()?;
//!     let user = user_service.get_user("user-1");
//!     println!("User: {}", user);
//!
//!     Ok(())
//! }
//! ```

// Expose the error module
pub mod error;

// Expose the registry module
pub mod registry;

// Expose the application module (will be implemented later)
// pub mod application;

// Re-export common types for convenience
pub use error::{Error, Result};
pub use registry::{
    AsyncLifecycle, ComponentFactory, ComponentRef, ComponentRegistry, ComponentScope,
    DynComponentRef, Lifecycle, LifecyclePhase, TypedComponentFactory,
};
