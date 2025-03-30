// Component Registry for Navius Dependency Injection
//
// This module provides a comprehensive component registry for dependency injection
// inspired by spring-rs research. It includes lifecycle management, scoped components,
// and autowiring capabilities.

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt,
    marker::PhantomData,
    ops::Deref,
    sync::{Arc, Mutex, RwLock},
};

use async_trait::async_trait;
use tracing::{debug, info, warn};

use crate::error::{Error, Result};

/// Component lifecycle phase
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecyclePhase {
    /// Component is being created
    Create,
    /// Component is being initialized
    Initialize,
    /// Component is being destroyed
    Destroy,
}

impl fmt::Display for LifecyclePhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LifecyclePhase::Create => write!(f, "Create"),
            LifecyclePhase::Initialize => write!(f, "Initialize"),
            LifecyclePhase::Destroy => write!(f, "Destroy"),
        }
    }
}

/// Trait for components with lifecycle hooks
pub trait Lifecycle: Send + Sync {
    /// Called when the component is created
    fn on_create(&self) -> Result<()> {
        Ok(())
    }

    /// Called when the component is initialized
    fn on_initialize(&self) -> Result<()> {
        Ok(())
    }

    /// Called when the component is destroyed
    fn on_destroy(&self) -> Result<()> {
        Ok(())
    }

    /// Execute a lifecycle phase
    fn execute_phase(&self, phase: LifecyclePhase) -> Result<()> {
        match phase {
            LifecyclePhase::Create => self.on_create(),
            LifecyclePhase::Initialize => self.on_initialize(),
            LifecyclePhase::Destroy => self.on_destroy(),
        }
    }
}

/// Trait for components with async lifecycle hooks
#[async_trait]
pub trait AsyncLifecycle: Send + Sync {
    /// Called when the component is created asynchronously
    async fn on_create_async(&self) -> Result<()> {
        Ok(())
    }

    /// Called when the component is initialized asynchronously
    async fn on_initialize_async(&self) -> Result<()> {
        Ok(())
    }

    /// Called when the component is destroyed asynchronously
    async fn on_destroy_async(&self) -> Result<()> {
        Ok(())
    }

    /// Execute a lifecycle phase asynchronously
    async fn execute_phase_async(&self, phase: LifecyclePhase) -> Result<()> {
        match phase {
            LifecyclePhase::Create => self.on_create_async().await,
            LifecyclePhase::Initialize => self.on_initialize_async().await,
            LifecyclePhase::Destroy => self.on_destroy_async().await,
        }
    }
}

/// Component scope determining how instances are managed
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentScope {
    /// A singleton component is instantiated once and shared
    Singleton,
    /// A prototype component is instantiated each time it's requested
    Prototype,
    /// A request-scoped component is instantiated for each request
    Request,
    /// A session-scoped component is instantiated for each session
    Session,
}

impl Default for ComponentScope {
    fn default() -> Self {
        Self::Singleton
    }
}

impl fmt::Display for ComponentScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ComponentScope::Singleton => write!(f, "Singleton"),
            ComponentScope::Prototype => write!(f, "Prototype"),
            ComponentScope::Request => write!(f, "Request"),
            ComponentScope::Session => write!(f, "Session"),
        }
    }
}

/// A reference to a component in the registry
#[derive(Clone)]
pub struct ComponentRef<T>(Arc<T>);

impl<T> ComponentRef<T> {
    /// Create a new component reference
    pub fn new(component: T) -> Self {
        Self(Arc::new(component))
    }

    /// Get the Arc pointer to the component
    pub fn into_arc(self) -> Arc<T> {
        self.0
    }

    /// Get the raw pointer to the component
    pub fn into_raw(self) -> *const T {
        Arc::into_raw(self.0)
    }
}

impl<T: fmt::Debug> fmt::Debug for ComponentRef<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ComponentRef").field(&self.0).finish()
    }
}

impl<T> Deref for ComponentRef<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// A dynamically typed component reference
#[derive(Clone)]
pub struct DynComponentRef(Arc<dyn Any + Send + Sync>);

impl DynComponentRef {
    /// Create a new dynamic component reference
    pub fn new<T: Any + Send + Sync>(component: T) -> Self {
        Self(Arc::new(component))
    }

    /// Downcast to a specific type
    pub fn downcast<T: Any + Send + Sync>(self) -> Option<ComponentRef<T>> {
        self.0
            .downcast_ref::<T>()
            .map(|_| ComponentRef(self.0.clone().downcast::<T>().unwrap()))
    }

    /// Execute lifecycle hooks if the component implements Lifecycle
    pub fn execute_lifecycle(&self, phase: LifecyclePhase) -> Result<()> {
        if let Some(lifecycle) = self.0.downcast_ref::<dyn Lifecycle>() {
            return lifecycle.execute_phase(phase);
        }
        Ok(())
    }

    /// Execute async lifecycle hooks if the component implements AsyncLifecycle
    pub async fn execute_lifecycle_async(&self, phase: LifecyclePhase) -> Result<()> {
        if let Some(lifecycle) = self.0.downcast_ref::<dyn AsyncLifecycle>() {
            return lifecycle.execute_phase_async(phase).await;
        }
        Ok(())
    }
}

/// Factory for creating components
pub trait ComponentFactory: Send + Sync {
    /// Create a component instance
    fn create(&self) -> DynComponentRef;

    /// Get the TypeId of the component
    fn type_id(&self) -> TypeId;

    /// Get the type name of the component
    fn type_name(&self) -> &str;

    /// Get the scope of the component
    fn scope(&self) -> ComponentScope;

    /// Initialize a component
    fn initialize(&self, component: &DynComponentRef) -> Result<()> {
        component.execute_lifecycle(LifecyclePhase::Initialize)
    }

    /// Initialize a component asynchronously
    async fn initialize_async(&self, component: &DynComponentRef) -> Result<()> {
        component
            .execute_lifecycle_async(LifecyclePhase::Initialize)
            .await
    }
}

/// Typed component factory
pub struct TypedComponentFactory<T, F> {
    factory: F,
    scope: ComponentScope,
    _marker: PhantomData<T>,
}

impl<T, F> TypedComponentFactory<T, F> {
    /// Create a new typed component factory
    pub fn new(factory: F, scope: ComponentScope) -> Self {
        Self {
            factory,
            scope,
            _marker: PhantomData,
        }
    }
}

impl<T: Any + Send + Sync, F: Fn() -> T + Send + Sync> ComponentFactory
    for TypedComponentFactory<T, F>
{
    fn create(&self) -> DynComponentRef {
        DynComponentRef::new((self.factory)())
    }

    fn type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }

    fn type_name(&self) -> &str {
        std::any::type_name::<T>()
    }

    fn scope(&self) -> ComponentScope {
        self.scope
    }
}

/// Component registry for managing dependencies
#[derive(Default)]
pub struct ComponentRegistry {
    components: RwLock<HashMap<TypeId, DynComponentRef>>,
    factories: RwLock<HashMap<TypeId, Box<dyn ComponentFactory>>>,
    qualifiers: RwLock<HashMap<String, TypeId>>,
}

impl ComponentRegistry {
    /// Create a new empty component registry
    pub fn new() -> Self {
        Self {
            components: RwLock::new(HashMap::new()),
            factories: RwLock::new(HashMap::new()),
            qualifiers: RwLock::new(HashMap::new()),
        }
    }

    /// Register a component instance
    pub fn register<T: Any + Send + Sync>(&self, component: T) -> Result<()> {
        let type_id = TypeId::of::<T>();
        let component_ref = DynComponentRef::new(component);

        // Initialize lifecycle if component supports it
        component_ref.execute_lifecycle(LifecyclePhase::Create)?;
        component_ref.execute_lifecycle(LifecyclePhase::Initialize)?;

        self.components
            .write()
            .unwrap()
            .insert(type_id, component_ref);
        Ok(())
    }

    /// Register a component with a qualifier
    pub fn register_with_qualifier<T: Any + Send + Sync>(
        &self,
        component: T,
        qualifier: &str,
    ) -> Result<()> {
        let type_id = TypeId::of::<T>();
        let component_ref = DynComponentRef::new(component);

        // Initialize lifecycle if component supports it
        component_ref.execute_lifecycle(LifecyclePhase::Create)?;
        component_ref.execute_lifecycle(LifecyclePhase::Initialize)?;

        self.components
            .write()
            .unwrap()
            .insert(type_id, component_ref);
        self.qualifiers
            .write()
            .unwrap()
            .insert(qualifier.to_string(), type_id);
        Ok(())
    }

    /// Register a component factory
    pub fn register_factory<F>(&self, factory: Box<dyn ComponentFactory>) {
        let type_id = factory.type_id();
        self.factories.write().unwrap().insert(type_id, factory);
    }

    /// Register a component with a factory function and scope
    pub fn register_with_factory<T, F>(&self, factory: F, scope: ComponentScope)
    where
        T: Any + Send + Sync,
        F: Fn() -> T + Send + Sync + 'static,
    {
        let typed_factory = TypedComponentFactory::new(factory, scope);
        self.register_factory(Box::new(typed_factory));
    }

    /// Register a component with a factory function, scope, and qualifier
    pub fn register_with_factory_and_qualifier<T, F>(
        &self,
        factory: F,
        scope: ComponentScope,
        qualifier: &str,
    ) where
        T: Any + Send + Sync,
        F: Fn() -> T + Send + Sync + 'static,
    {
        let type_id = TypeId::of::<T>();
        let typed_factory = TypedComponentFactory::new(factory, scope);
        self.register_factory(Box::new(typed_factory));
        self.qualifiers
            .write()
            .unwrap()
            .insert(qualifier.to_string(), type_id);
    }

    /// Get a component by type
    pub fn get<T: Any + Send + Sync>(&self) -> Result<ComponentRef<T>> {
        let type_id = TypeId::of::<T>();

        // Check if we have a cached instance for singletons
        if let Some(component) = self.components.read().unwrap().get(&type_id) {
            if let Some(typed_ref) = component.clone().downcast::<T>() {
                return Ok(typed_ref);
            }
        }

        // Check if we have a factory
        if let Some(factory) = self.factories.read().unwrap().get(&type_id).cloned() {
            let dyn_ref = factory.create();

            // Initialize the component
            factory.initialize(&dyn_ref)?;

            // For singletons, cache the instance
            if factory.scope() == ComponentScope::Singleton {
                self.components
                    .write()
                    .unwrap()
                    .insert(type_id, dyn_ref.clone());
            }

            if let Some(typed_ref) = dyn_ref.downcast::<T>() {
                return Ok(typed_ref);
            }
        }

        Err(Error::ComponentNotFound {
            name: std::any::type_name::<T>().to_string(),
        })
    }

    /// Get a component by qualifier
    pub fn get_by_qualifier<T: Any + Send + Sync>(
        &self,
        qualifier: &str,
    ) -> Result<ComponentRef<T>> {
        let qualifiers = self.qualifiers.read().unwrap();
        let type_id = qualifiers
            .get(qualifier)
            .ok_or_else(|| Error::QualifierNotFound {
                qualifier: qualifier.to_string(),
            })?;

        // Continue with the same logic as get<T>() but using the found type_id
        drop(qualifiers); // Release the lock before proceeding

        // Check if we have a cached instance for singletons
        if let Some(component) = self.components.read().unwrap().get(type_id) {
            if let Some(typed_ref) = component.clone().downcast::<T>() {
                return Ok(typed_ref);
            }
        }

        // Check if we have a factory
        if let Some(factory) = self.factories.read().unwrap().get(type_id).cloned() {
            let dyn_ref = factory.create();

            // Initialize the component
            factory.initialize(&dyn_ref)?;

            // For singletons, cache the instance
            if factory.scope() == ComponentScope::Singleton {
                self.components
                    .write()
                    .unwrap()
                    .insert(*type_id, dyn_ref.clone());
            }

            if let Some(typed_ref) = dyn_ref.downcast::<T>() {
                return Ok(typed_ref);
            }
        }

        Err(Error::TypeMismatch {
            expected: std::any::type_name::<T>().to_string(),
            found: format!("Component with qualifier '{}'", qualifier),
        })
    }

    /// Get a component by type asynchronously
    pub async fn get_async<T: Any + Send + Sync>(&self) -> Result<ComponentRef<T>> {
        let type_id = TypeId::of::<T>();

        // Check if we have a cached instance for singletons
        if let Some(component) = self.components.read().unwrap().get(&type_id) {
            if let Some(typed_ref) = component.clone().downcast::<T>() {
                return Ok(typed_ref);
            }
        }

        // Check if we have a factory
        if let Some(factory) = self.factories.read().unwrap().get(&type_id).cloned() {
            let dyn_ref = factory.create();

            // Initialize the component asynchronously
            factory.initialize_async(&dyn_ref).await?;

            // For singletons, cache the instance
            if factory.scope() == ComponentScope::Singleton {
                self.components
                    .write()
                    .unwrap()
                    .insert(type_id, dyn_ref.clone());
            }

            if let Some(typed_ref) = dyn_ref.downcast::<T>() {
                return Ok(typed_ref);
            }
        }

        Err(Error::ComponentNotFound {
            name: std::any::type_name::<T>().to_string(),
        })
    }

    /// Get a component by qualifier asynchronously
    pub async fn get_by_qualifier_async<T: Any + Send + Sync>(
        &self,
        qualifier: &str,
    ) -> Result<ComponentRef<T>> {
        let qualifiers = self.qualifiers.read().unwrap();
        let type_id = qualifiers
            .get(qualifier)
            .ok_or_else(|| Error::QualifierNotFound {
                qualifier: qualifier.to_string(),
            })?;

        let type_id = *type_id;
        drop(qualifiers); // Release the lock before proceeding

        // Continue with the same logic as get_async<T>() but using the found type_id
        // Check if we have a cached instance for singletons
        if let Some(component) = self.components.read().unwrap().get(&type_id) {
            if let Some(typed_ref) = component.clone().downcast::<T>() {
                return Ok(typed_ref);
            }
        }

        // Check if we have a factory
        if let Some(factory) = self.factories.read().unwrap().get(&type_id).cloned() {
            let dyn_ref = factory.create();

            // Initialize the component asynchronously
            factory.initialize_async(&dyn_ref).await?;

            // For singletons, cache the instance
            if factory.scope() == ComponentScope::Singleton {
                self.components
                    .write()
                    .unwrap()
                    .insert(type_id, dyn_ref.clone());
            }

            if let Some(typed_ref) = dyn_ref.downcast::<T>() {
                return Ok(typed_ref);
            }
        }

        Err(Error::TypeMismatch {
            expected: std::any::type_name::<T>().to_string(),
            found: format!("Component with qualifier '{}'", qualifier),
        })
    }

    /// Check if a component is registered
    pub fn has<T: Any + Send + Sync>(&self) -> bool {
        let type_id = TypeId::of::<T>();
        self.components.read().unwrap().contains_key(&type_id)
            || self.factories.read().unwrap().contains_key(&type_id)
    }

    /// Check if a qualifier is registered
    pub fn has_qualifier(&self, qualifier: &str) -> bool {
        self.qualifiers.read().unwrap().contains_key(qualifier)
    }

    /// Get all registered component types
    pub fn component_types(&self) -> Vec<&str> {
        let mut types = Vec::new();

        for factory in self.factories.read().unwrap().values() {
            types.push(factory.type_name());
        }

        // Add components that don't have factories
        for component in self.components.read().unwrap().keys() {
            if !self.factories.read().unwrap().contains_key(component) {
                types.push(std::any::type_name_of_val(component));
            }
        }

        types
    }

    /// Get all registered qualifiers
    pub fn qualifiers(&self) -> Vec<String> {
        self.qualifiers.read().unwrap().keys().cloned().collect()
    }

    /// Remove a component by type
    pub fn remove<T: Any + Send + Sync>(&self) -> Result<()> {
        let type_id = TypeId::of::<T>();

        // Remove from components if it exists
        if let Some(component) = self.components.write().unwrap().remove(&type_id) {
            // Call lifecycle hook
            component.execute_lifecycle(LifecyclePhase::Destroy)?;
        }

        // Remove from factories
        self.factories.write().unwrap().remove(&type_id);

        // Remove any qualifiers pointing to this type
        let mut qualifiers_to_remove = Vec::new();
        for (qualifier, id) in self.qualifiers.read().unwrap().iter() {
            if *id == type_id {
                qualifiers_to_remove.push(qualifier.clone());
            }
        }

        for qualifier in qualifiers_to_remove {
            self.qualifiers.write().unwrap().remove(&qualifier);
        }

        Ok(())
    }

    /// Remove a component by qualifier
    pub fn remove_by_qualifier(&self, qualifier: &str) -> Result<()> {
        let type_id = {
            let qualifiers = self.qualifiers.read().unwrap();
            match qualifiers.get(qualifier) {
                Some(id) => *id,
                None => {
                    return Err(Error::QualifierNotFound {
                        qualifier: qualifier.to_string(),
                    });
                }
            }
        };

        // Remove from components if it exists
        if let Some(component) = self.components.write().unwrap().remove(&type_id) {
            // Call lifecycle hook
            component.execute_lifecycle(LifecyclePhase::Destroy)?;
        }

        // Remove from factories
        self.factories.write().unwrap().remove(&type_id);

        // Remove the qualifier
        self.qualifiers.write().unwrap().remove(qualifier);

        Ok(())
    }

    /// Shutdown the registry and destroy all components
    pub fn shutdown(&self) -> Result<()> {
        let components = {
            // Take ownership of all components
            let mut components_write = self.components.write().unwrap();
            std::mem::take(&mut *components_write)
        };

        // Clear factories and qualifiers as well
        self.factories.write().unwrap().clear();
        self.qualifiers.write().unwrap().clear();

        // Call destroy on all components
        for (_, component) in components {
            component.execute_lifecycle(LifecyclePhase::Destroy)?;
        }

        Ok(())
    }

    /// Shutdown the registry and destroy all components asynchronously
    pub async fn shutdown_async(&self) -> Result<()> {
        let components = {
            // Take ownership of all components
            let mut components_write = self.components.write().unwrap();
            std::mem::take(&mut *components_write)
        };

        // Clear factories and qualifiers as well
        self.factories.write().unwrap().clear();
        self.qualifiers.write().unwrap().clear();

        // Call destroy on all components asynchronously
        for (_, component) in components {
            component
                .execute_lifecycle_async(LifecyclePhase::Destroy)
                .await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    struct TestComponent {
        value: String,
    }

    #[test]
    fn register_and_get_component() {
        let registry = ComponentRegistry::new();
        let component = TestComponent {
            value: "test".to_string(),
        };

        registry.register(component).unwrap();

        let retrieved = registry.get::<TestComponent>().unwrap();
        assert_eq!(retrieved.value, "test");
    }

    #[test]
    fn register_with_qualifier() {
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
    fn register_factory_singleton() {
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
    fn register_factory_prototype() {
        let registry = ComponentRegistry::new();
        let counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));

        let counter_clone = counter.clone();
        registry.register_with_factory(
            move || {
                let value = counter_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
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
    fn component_not_found() {
        let registry = ComponentRegistry::new();
        let result = registry.get::<String>();
        assert!(result.is_err());
    }

    #[test]
    fn qualifier_not_found() {
        let registry = ComponentRegistry::new();
        let result = registry.get_by_qualifier::<String>("non-existent");
        assert!(result.is_err());
    }

    #[test]
    fn has_component() {
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
    fn has_qualifier() {
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
    fn remove_component() {
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
    fn remove_by_qualifier() {
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
    fn shutdown() {
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

    // Example of a component with lifecycle hooks
    struct LifecycleComponent {
        initialized: std::sync::atomic::AtomicBool,
        destroyed: std::sync::atomic::AtomicBool,
    }

    impl LifecycleComponent {
        fn new() -> Self {
            Self {
                initialized: std::sync::atomic::AtomicBool::new(false),
                destroyed: std::sync::atomic::AtomicBool::new(false),
            }
        }

        fn is_initialized(&self) -> bool {
            self.initialized.load(std::sync::atomic::Ordering::SeqCst)
        }

        fn is_destroyed(&self) -> bool {
            self.destroyed.load(std::sync::atomic::Ordering::SeqCst)
        }
    }

    impl Lifecycle for LifecycleComponent {
        fn on_initialize(&self) -> Result<()> {
            self.initialized
                .store(true, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        }

        fn on_destroy(&self) -> Result<()> {
            self.destroyed
                .store(true, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        }
    }

    #[test]
    fn lifecycle_hooks() {
        let registry = ComponentRegistry::new();
        let component = LifecycleComponent::new();

        registry.register(component).unwrap();

        let retrieved = registry.get::<LifecycleComponent>().unwrap();
        assert!(retrieved.is_initialized());
        assert!(!retrieved.is_destroyed());

        registry.remove::<LifecycleComponent>().unwrap();
        // Can't check destroyed state here as the component is dropped

        // Using a factory to test lifecycle
        let registry = ComponentRegistry::new();
        let component = Arc::new(LifecycleComponent::new());
        let component_clone = component.clone();

        registry.register_with_factory(move || component_clone.clone(), ComponentScope::Singleton);

        let retrieved = registry.get::<Arc<LifecycleComponent>>().unwrap();
        assert!(retrieved.is_initialized());
        assert!(!retrieved.is_destroyed());

        registry.shutdown().unwrap();
        assert!(component.is_destroyed());
    }

    #[tokio::test]
    async fn async_lifecycle() {
        // Implementing a struct with AsyncLifecycle would follow the same pattern
        // This test just verifies the async methods exist and can be called
        let registry = ComponentRegistry::new();
        let component = TestComponent {
            value: "async".to_string(),
        };

        registry.register(component).unwrap();

        let retrieved = registry.get_async::<TestComponent>().await.unwrap();
        assert_eq!(retrieved.value, "async");

        registry.shutdown_async().await.unwrap();
    }
}
