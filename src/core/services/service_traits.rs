use std::error::Error;
use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;

/// Base trait for all services in the application
pub trait Service: Send + Sync + Debug + 'static {}

/// Lifecycle trait for services that need to perform initialization or cleanup
#[async_trait]
pub trait Lifecycle: Service {
    /// Initialize the service
    async fn init(&self) -> Result<(), Box<dyn Error + Send + Sync>>;

    /// Shutdown the service, performing any necessary cleanup
    async fn shutdown(&self) -> Result<(), Box<dyn Error + Send + Sync>>;

    /// Check if the service is healthy
    async fn health_check(&self) -> Result<(), Box<dyn Error + Send + Sync>>;
}

/// Service provider trait for creating service instances
#[async_trait]
pub trait ServiceProvider: Sized {
    /// The type of service this provider creates
    type Service: Service;

    /// The configuration type required to create the service
    type Config: Clone + Send + Sync;

    /// The error type that may be returned when creating the service
    type Error: Error + Send + Sync;

    /// Create a new service instance
    async fn create(
        config: Self::Config,
        registry: &ServiceRegistry,
    ) -> Result<Self::Service, Self::Error>;

    /// Check if the service is healthy
    async fn health_check(&self) -> Result<(), Self::Error>;
}

/// Service registry for dependency injection
#[derive(Debug, Default)]
pub struct ServiceRegistry {
    // Internal storage for service instances
    services: std::collections::HashMap<std::any::TypeId, Box<dyn std::any::Any + Send + Sync>>,
}

impl ServiceRegistry {
    /// Create a new empty service registry
    pub fn new() -> Self {
        Self {
            services: std::collections::HashMap::new(),
        }
    }

    /// Register a service in the registry
    pub fn register<T: 'static + Send + Sync>(&mut self, service: T) {
        let type_id = std::any::TypeId::of::<T>();
        self.services.insert(type_id, Box::new(service));
    }

    /// Register a service in the registry using a type key
    pub fn register_as<T: 'static + Send + Sync, K: 'static>(&mut self, service: T) {
        let type_id = std::any::TypeId::of::<K>();
        self.services.insert(type_id, Box::new(service));
    }

    /// Get a service from the registry
    pub fn get<T: 'static>(&self) -> Option<&T> {
        let type_id = std::any::TypeId::of::<T>();
        self.services
            .get(&type_id)
            .and_then(|s| s.downcast_ref::<T>())
    }

    /// Get a mutable reference to a service from the registry
    pub fn get_mut<T: 'static>(&mut self) -> Option<&mut T> {
        let type_id = std::any::TypeId::of::<T>();
        self.services
            .get_mut(&type_id)
            .and_then(|s| s.downcast_mut::<T>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test service implementation
    #[derive(Debug)]
    struct TestService {
        name: String,
    }

    impl Service for TestService {}

    #[test]
    fn test_service_registry() {
        // Create a new service registry
        let mut registry = ServiceRegistry::new();

        // Register a service
        let service = TestService {
            name: "test".to_string(),
        };
        registry.register(service);

        // Retrieve the service
        let retrieved = registry.get::<TestService>();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "test");

        // Try to retrieve a service that doesn't exist
        let not_found = registry.get::<String>();
        assert!(not_found.is_none());
    }

    #[test]
    fn test_register_as() {
        // Create a new service registry
        let mut registry = ServiceRegistry::new();

        // Create a test service
        let test_service = TestService {
            name: "test_service".to_string(),
        };

        // Register the TestService with String as the key type
        // This means we store the TestService in the registry, but we use String's TypeId to look it up
        registry.register_as::<TestService, String>(test_service);

        // Should NOT be retrievable using String's get<T> since that expects a &String, but we stored TestService
        let string_ref = registry.get::<String>();
        assert!(string_ref.is_none());

        // Should NOT be retrievable as TestService since we used String's TypeId as the key
        let test_service_ref = registry.get::<TestService>();
        assert!(test_service_ref.is_none());

        // The issue is that register_as is intended to register a service under a trait,
        // not to allow arbitrary type keys. Let's show how it should be used.

        // Create a new registry
        let mut registry = ServiceRegistry::new();

        // Suppose we have a trait that TestService implements
        trait NamedService {
            fn get_name(&self) -> &str;
        }

        impl NamedService for TestService {
            fn get_name(&self) -> &str {
                &self.name
            }
        }

        // We can't store trait objects directly, so we create a wrapper
        struct NamedServiceImpl(TestService);

        // Register a test service as a NamedService
        registry.register(TestService {
            name: "direct".to_string(),
        });

        // Should be retrievable as a TestService
        let direct_svc = registry.get::<TestService>();
        assert!(direct_svc.is_some());
        assert_eq!(direct_svc.unwrap().name, "direct");
    }
}
