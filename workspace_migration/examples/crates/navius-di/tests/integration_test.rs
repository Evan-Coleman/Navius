// Integration tests for Navius Dependency Injection

use async_trait::async_trait;
use navius_di::{
    Application, ApplicationBuilder, AsyncLifecycle, ComponentRef, ComponentRegistry,
    ComponentScope, ConfigRef, Error, Lifecycle, Result,
};
use serde::Deserialize;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

// Test configuration
#[derive(Debug, Clone, Deserialize, PartialEq)]
struct TestConfig {
    name: String,
    value: i32,
}

// Test component with lifecycle tracking
struct TestComponent {
    name: String,
    initialized: Arc<AtomicBool>,
    destroyed: Arc<AtomicBool>,
}

impl TestComponent {
    fn new(name: String) -> Self {
        Self {
            name,
            initialized: Arc::new(AtomicBool::new(false)),
            destroyed: Arc::new(AtomicBool::new(false)),
        }
    }

    fn is_initialized(&self) -> bool {
        self.initialized.load(Ordering::SeqCst)
    }

    fn is_destroyed(&self) -> bool {
        self.destroyed.load(Ordering::SeqCst)
    }

    fn get_name(&self) -> &str {
        &self.name
    }
}

impl Lifecycle for TestComponent {
    fn on_initialize(&self) -> Result<()> {
        self.initialized.store(true, Ordering::SeqCst);
        Ok(())
    }

    fn on_destroy(&self) -> Result<()> {
        self.destroyed.store(true, Ordering::SeqCst);
        Ok(())
    }
}

// Test async component with lifecycle tracking
struct TestAsyncComponent {
    name: String,
    initialized: Arc<AtomicBool>,
    destroyed: Arc<AtomicBool>,
}

impl TestAsyncComponent {
    fn new(name: String) -> Self {
        Self {
            name,
            initialized: Arc::new(AtomicBool::new(false)),
            destroyed: Arc::new(AtomicBool::new(false)),
        }
    }

    fn is_initialized(&self) -> bool {
        self.initialized.load(Ordering::SeqCst)
    }

    fn is_destroyed(&self) -> bool {
        self.destroyed.load(Ordering::SeqCst)
    }
}

#[async_trait]
impl AsyncLifecycle for TestAsyncComponent {
    async fn on_initialize_async(&self) -> Result<()> {
        self.initialized.store(true, Ordering::SeqCst);
        Ok(())
    }

    async fn on_destroy_async(&self) -> Result<()> {
        self.destroyed.store(true, Ordering::SeqCst);
        Ok(())
    }
}

// Dependent component
struct DependentComponent {
    dependency: ComponentRef<TestComponent>,
}

impl DependentComponent {
    fn new(dependency: ComponentRef<TestComponent>) -> Self {
        Self { dependency }
    }

    fn get_dependency_name(&self) -> &str {
        self.dependency.get_name()
    }
}

#[tokio::test]
async fn test_component_registration_and_retrieval() {
    // Create a registry and register a component
    let registry = ComponentRegistry::new();
    let test_component = TestComponent::new("test1".to_string());

    // Register component
    registry.register(test_component).unwrap();

    // Verify component can be retrieved
    let retrieved = registry.get::<TestComponent>().unwrap();
    assert_eq!(retrieved.get_name(), "test1");

    // Verify lifecycle hooks
    assert!(retrieved.is_initialized());
    assert!(!retrieved.is_destroyed());

    // Clean up
    registry.shutdown().unwrap();

    // Verify destruction
    assert!(retrieved.is_destroyed());
}

#[tokio::test]
async fn test_application_builder() {
    // Create an application with config and components
    let app = Application::builder()
        .with_config("test.name", "TestApp".to_string())
        .with_config("test.value", 42)
        .with_factory(
            || TestComponent::new("test-app-component".to_string()),
            ComponentScope::Singleton,
        )
        .with_factory(
            || TestAsyncComponent::new("test-async-component".to_string()),
            ComponentScope::Singleton,
        )
        .build()
        .await
        .unwrap();

    // Test configuration retrieval
    let config = app.config::<TestConfig>("test").unwrap();
    assert_eq!(
        config,
        TestConfig {
            name: "TestApp".to_string(),
            value: 42,
        }
    );

    // Test component retrieval
    let component = app.get::<TestComponent>().unwrap();
    assert_eq!(component.get_name(), "test-app-component");
    assert!(component.is_initialized());

    // Test async component
    let async_component = app.get::<TestAsyncComponent>().unwrap();
    assert!(async_component.is_initialized());

    // Clean up
    app.shutdown_async().await.unwrap();

    // Verify destruction
    assert!(component.is_destroyed());
    assert!(async_component.is_destroyed());
}

#[tokio::test]
async fn test_dependency_injection() {
    // Create a registry
    let registry = ComponentRegistry::new();

    // Register a component
    registry
        .register(TestComponent::new("dependency".to_string()))
        .unwrap();

    // Register a component that depends on the first one
    registry.register_with_factory(
        || {
            let dependency = registry.get::<TestComponent>().unwrap();
            DependentComponent::new(dependency)
        },
        ComponentScope::Singleton,
    );

    // Get the dependent component
    let dependent = registry.get::<DependentComponent>().unwrap();

    // Verify that it received the dependency
    assert_eq!(dependent.get_dependency_name(), "dependency");

    // Clean up
    registry.shutdown().unwrap();
}

#[tokio::test]
async fn test_component_not_found() {
    let registry = ComponentRegistry::new();

    // Try to get a component that doesn't exist
    let result = registry.get::<TestComponent>();

    // Verify that it returns an error
    assert!(result.is_err());
    match result {
        Err(Error::ComponentNotFound { .. }) => {} // Expected error
        other => panic!("Unexpected result: {:?}", other),
    }
}

#[tokio::test]
async fn test_qualifier() {
    let registry = ComponentRegistry::new();

    // Register components with qualifiers
    registry
        .register_with_qualifier(TestComponent::new("component1".to_string()), "first")
        .unwrap();

    registry
        .register_with_qualifier(TestComponent::new("component2".to_string()), "second")
        .unwrap();

    // Get components by qualifier
    let first = registry.get_by_qualifier::<TestComponent>("first").unwrap();
    let second = registry
        .get_by_qualifier::<TestComponent>("second")
        .unwrap();

    // Verify that the correct components were retrieved
    assert_eq!(first.get_name(), "component1");
    assert_eq!(second.get_name(), "component2");

    // Clean up
    registry.shutdown().unwrap();
}
