use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use navius_di::{
    AsyncLifecycle, ComponentRef, ComponentRegistry, ComponentScope, Lifecycle, LifecyclePhase,
    Result,
};

#[derive(Debug, Clone)]
struct TestComponent {
    value: String,
}

#[test]
fn test_register_and_get_component() {
    let registry = ComponentRegistry::new();
    let component = TestComponent {
        value: "test".to_string(),
    };

    registry.register(component).unwrap();

    let retrieved = registry.get::<TestComponent>().unwrap();
    assert_eq!(retrieved.value, "test");
}

#[test]
fn test_register_with_qualifier() {
    let registry = ComponentRegistry::new();
    let component = TestComponent {
        value: "qualified".to_string(),
    };

    registry
        .register_with_qualifier(component, "test-qualifier")
        .unwrap();

    let retrieved = registry
        .get_by_qualifier::<TestComponent>("test-qualifier")
        .unwrap();
    assert_eq!(retrieved.value, "qualified");
}

#[test]
fn test_register_factory_singleton() {
    let registry = ComponentRegistry::new();

    registry.register_with_factory(
        || TestComponent {
            value: "factory-created".to_string(),
        },
        ComponentScope::Singleton,
    );

    let first = registry.get::<TestComponent>().unwrap();
    let second = registry.get::<TestComponent>().unwrap();

    assert_eq!(first.value, "factory-created");
    assert_eq!(second.value, "factory-created");

    // For singletons, both references should point to the same instance
    assert_eq!(Arc::as_ptr(&first.0), Arc::as_ptr(&second.0));
}

#[test]
fn test_register_factory_prototype() {
    let registry = ComponentRegistry::new();
    let counter = Arc::new(AtomicUsize::new(0));

    let counter_clone = counter.clone();
    registry.register_with_factory(
        move || {
            let value = counter_clone.fetch_add(1, Ordering::SeqCst);
            TestComponent {
                value: format!("instance-{}", value),
            }
        },
        ComponentScope::Prototype,
    );

    let first = registry.get::<TestComponent>().unwrap();
    let second = registry.get::<TestComponent>().unwrap();

    assert_eq!(first.value, "instance-0");
    assert_eq!(second.value, "instance-1");

    // For prototypes, references should point to different instances
    assert_ne!(Arc::as_ptr(&first.0), Arc::as_ptr(&second.0));
}

#[test]
fn test_component_not_found() {
    let registry = ComponentRegistry::new();
    let result = registry.get::<String>();
    assert!(result.is_err());
}

#[test]
fn test_qualifier_not_found() {
    let registry = ComponentRegistry::new();
    let result = registry.get_by_qualifier::<String>("non-existent");
    assert!(result.is_err());
}

#[test]
fn test_has_component() {
    let registry = ComponentRegistry::new();
    registry
        .register(TestComponent {
            value: "test".to_string(),
        })
        .unwrap();

    assert!(registry.has::<TestComponent>());
    assert!(!registry.has::<String>());
}

#[test]
fn test_has_qualifier() {
    let registry = ComponentRegistry::new();
    registry
        .register_with_qualifier(
            TestComponent {
                value: "qualified".to_string(),
            },
            "test-qualifier",
        )
        .unwrap();

    assert!(registry.has_qualifier("test-qualifier"));
    assert!(!registry.has_qualifier("non-existent"));
}

#[test]
fn test_remove_component() {
    let registry = ComponentRegistry::new();
    registry
        .register(TestComponent {
            value: "test".to_string(),
        })
        .unwrap();

    assert!(registry.has::<TestComponent>());
    registry.remove::<TestComponent>().unwrap();
    assert!(!registry.has::<TestComponent>());
}

#[test]
fn test_remove_by_qualifier() {
    let registry = ComponentRegistry::new();
    registry
        .register_with_qualifier(
            TestComponent {
                value: "qualified".to_string(),
            },
            "test-qualifier",
        )
        .unwrap();

    assert!(registry.has_qualifier("test-qualifier"));
    registry.remove_by_qualifier("test-qualifier").unwrap();
    assert!(!registry.has_qualifier("test-qualifier"));
}

#[test]
fn test_multiple_qualifiers_same_type() {
    let registry = ComponentRegistry::new();

    registry
        .register_with_qualifier(
            TestComponent {
                value: "comp1".to_string(),
            },
            "comp1",
        )
        .unwrap();

    registry
        .register_with_qualifier(
            TestComponent {
                value: "comp2".to_string(),
            },
            "comp2",
        )
        .unwrap();

    let comp1 = registry.get_by_qualifier::<TestComponent>("comp1").unwrap();
    let comp2 = registry.get_by_qualifier::<TestComponent>("comp2").unwrap();

    assert_eq!(comp1.value, "comp1");
    assert_eq!(comp2.value, "comp2");
}

#[test]
fn test_shutdown() {
    let registry = ComponentRegistry::new();
    registry
        .register(TestComponent {
            value: "test".to_string(),
        })
        .unwrap();
    registry
        .register_with_qualifier(
            TestComponent {
                value: "qualified".to_string(),
            },
            "test-qualifier",
        )
        .unwrap();

    assert!(registry.has::<TestComponent>());
    assert!(registry.has_qualifier("test-qualifier"));

    registry.shutdown().unwrap();

    assert!(!registry.has::<TestComponent>());
    assert!(!registry.has_qualifier("test-qualifier"));
}

// Components with lifecycle hooks
struct LifecycleComponent {
    created: AtomicBool,
    initialized: AtomicBool,
    destroyed: AtomicBool,
}

impl LifecycleComponent {
    fn new() -> Self {
        Self {
            created: AtomicBool::new(false),
            initialized: AtomicBool::new(false),
            destroyed: AtomicBool::new(false),
        }
    }

    fn is_created(&self) -> bool {
        self.created.load(Ordering::SeqCst)
    }

    fn is_initialized(&self) -> bool {
        self.initialized.load(Ordering::SeqCst)
    }

    fn is_destroyed(&self) -> bool {
        self.destroyed.load(Ordering::SeqCst)
    }
}

impl Lifecycle for LifecycleComponent {
    fn on_create(&self) -> Result<()> {
        self.created.store(true, Ordering::SeqCst);
        Ok(())
    }

    fn on_initialize(&self) -> Result<()> {
        self.initialized.store(true, Ordering::SeqCst);
        Ok(())
    }

    fn on_destroy(&self) -> Result<()> {
        self.destroyed.store(true, Ordering::SeqCst);
        Ok(())
    }
}

#[test]
fn test_lifecycle_hooks() {
    let registry = ComponentRegistry::new();
    let component = Arc::new(LifecycleComponent::new());
    let component_clone = component.clone();

    registry.register(component_clone).unwrap();

    assert!(component.is_created());
    assert!(component.is_initialized());
    assert!(!component.is_destroyed());

    registry.shutdown().unwrap();
    assert!(component.is_destroyed());
}

#[test]
fn test_lifecycle_with_factory() {
    let registry = ComponentRegistry::new();
    let component = Arc::new(LifecycleComponent::new());
    let component_clone = component.clone();

    registry.register_with_factory(move || component_clone.clone(), ComponentScope::Singleton);

    let retrieved = registry.get::<Arc<LifecycleComponent>>().unwrap();
    assert!(retrieved.is_created());
    assert!(retrieved.is_initialized());
    assert!(!retrieved.is_destroyed());

    registry.shutdown().unwrap();
    assert!(component.is_destroyed());
}

// Async lifecycle tests
struct AsyncLifecycleComponent {
    created: AtomicBool,
    initialized: AtomicBool,
    destroyed: AtomicBool,
}

impl AsyncLifecycleComponent {
    fn new() -> Self {
        Self {
            created: AtomicBool::new(false),
            initialized: AtomicBool::new(false),
            destroyed: AtomicBool::new(false),
        }
    }

    fn is_created(&self) -> bool {
        self.created.load(Ordering::SeqCst)
    }

    fn is_initialized(&self) -> bool {
        self.initialized.load(Ordering::SeqCst)
    }

    fn is_destroyed(&self) -> bool {
        self.destroyed.load(Ordering::SeqCst)
    }
}

#[async_trait::async_trait]
impl AsyncLifecycle for AsyncLifecycleComponent {
    async fn on_create_async(&self) -> Result<()> {
        self.created.store(true, Ordering::SeqCst);
        Ok(())
    }

    async fn on_initialize_async(&self) -> Result<()> {
        self.initialized.store(true, Ordering::SeqCst);
        Ok(())
    }

    async fn on_destroy_async(&self) -> Result<()> {
        self.destroyed.store(true, Ordering::SeqCst);
        Ok(())
    }
}

#[tokio::test]
async fn test_async_lifecycle() {
    let registry = ComponentRegistry::new();
    let component = Arc::new(AsyncLifecycleComponent::new());

    registry.register(component.clone()).unwrap();

    // Async initialization is called in get_async
    let retrieved = registry
        .get_async::<Arc<AsyncLifecycleComponent>>()
        .await
        .unwrap();

    // For now, just verify we can retrieve the component
    assert!(Arc::ptr_eq(&component, &retrieved));

    registry.shutdown_async().await.unwrap();
}

// Test complex dependency chains
struct ServiceA {
    name: String,
}

impl ServiceA {
    fn new(name: String) -> Self {
        Self { name }
    }
}

struct ServiceB {
    service_a: ComponentRef<ServiceA>,
    name: String,
}

impl ServiceB {
    fn new(service_a: ComponentRef<ServiceA>, name: String) -> Self {
        Self { service_a, name }
    }
}

struct ServiceC {
    service_b: ComponentRef<ServiceB>,
    name: String,
}

impl ServiceC {
    fn new(service_b: ComponentRef<ServiceB>, name: String) -> Self {
        Self { service_b, name }
    }
}

#[test]
fn test_dependency_chain() {
    let registry = ComponentRegistry::new();

    // Register ServiceA
    registry.register(ServiceA::new("A".to_string())).unwrap();

    // Register ServiceB, depends on ServiceA
    registry.register_with_factory(
        || {
            let service_a = registry.get::<ServiceA>().unwrap();
            ServiceB::new(service_a, "B".to_string())
        },
        ComponentScope::Singleton,
    );

    // Register ServiceC, depends on ServiceB
    registry.register_with_factory(
        || {
            let service_b = registry.get::<ServiceB>().unwrap();
            ServiceC::new(service_b, "C".to_string())
        },
        ComponentScope::Singleton,
    );

    // Get ServiceC, which should pull in the entire chain
    let service_c = registry.get::<ServiceC>().unwrap();

    assert_eq!(service_c.name, "C");
    assert_eq!(service_c.service_b.name, "B");
    assert_eq!(service_c.service_b.service_a.name, "A");
}
