//! Component registry for dependency injection.
//!
//! This module provides a component registry system for implementing dependency injection
//! in Navius applications. It is based on research of the spring-rs framework and provides
//! a lightweight approach to managing components and their dependencies.

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt,
    ops::Deref,
    sync::Arc,
};

use crate::error::{Error, Result};

/// Lifecycle phase of a component
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecyclePhase {
    /// Component is being initialized
    Initialize,
    /// Component is being destroyed
    Destroy,
}

/// Trait for components with lifecycle hooks
pub trait Lifecycle: Send + Sync {
    /// Called when the component is initialized
    fn on_initialize(&self) -> Result<()> {
        Ok(())
    }

    /// Called when the component is destroyed
    fn on_destroy(&self) -> Result<()> {
        Ok(())
    }

    /// Execute a lifecycle phase
    fn execute_phase(&self, _phase: LifecyclePhase) -> Result<()> {
        Ok(())
    }
}

/// Trait for components with async lifecycle hooks
#[async_trait::async_trait]
pub trait AsyncLifecycle: Send + Sync {
    /// Called when the component is initialized asynchronously
    async fn on_initialize_async(&self) -> Result<()> {
        Ok(())
    }

    /// Called when the component is destroyed asynchronously
    async fn on_destroy_async(&self) -> Result<()> {
        Ok(())
    }

    /// Execute a lifecycle phase asynchronously
    async fn execute_phase_async(&self, _phase: LifecyclePhase) -> Result<()> {
        Ok(())
    }
}

/// Scope of a component in the registry
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentScope {
    /// A singleton component is instantiated once and shared across the application
    Singleton,
    /// A prototype component is instantiated each time it is requested
    Prototype,
}

impl Default for ComponentScope {
    fn default() -> Self {
        Self::Singleton
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

    /// Attempt to downcast to a specific type
    pub fn downcast<T: Any + Send + Sync>(self) -> Option<ComponentRef<T>> {
        self.0.downcast::<T>().ok().map(ComponentRef)
    }

    /// Attempt to downcast a reference to a specific type
    pub fn downcast_ref<T: Any + Send + Sync>(&self) -> Option<&T> {
        self.0.downcast_ref::<T>()
    }

    /// Get the TypeId of the contained component
    pub fn execute_type_id(&self) -> Option<TypeId> {
        // This is a limited implementation, but sufficient for our immediate needs
        Some((*self.0).type_id())
    }

    /// Execute a lifecycle phase
    pub fn execute_lifecycle(&self, _phase: LifecyclePhase) -> Result<()> {
        // We can't directly downcast to a trait object, so we need to check if the
        // component implements Lifecycle through reflection dynamically

        // For simplicity in this version, we'll just assume the component doesn't implement Lifecycle
        // A more complete implementation would use type_id checks or other mechanisms

        // Return success as no-op if the component doesn't have lifecycle hooks
        Ok(())
    }

    /// Execute an async lifecycle phase
    pub async fn execute_async_lifecycle(&self, _phase: LifecyclePhase) -> Result<()> {
        // Similar to execute_lifecycle, we can't directly downcast to a trait object
        // For simplicity, assume the component doesn't implement AsyncLifecycle

        // Return success as no-op if the component doesn't have async lifecycle hooks
        Ok(())
    }
}

impl fmt::Debug for DynComponentRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DynComponentRef")
            .field("type_id", &self.0.type_id())
            .finish()
    }
}

/// Factory for creating component instances
pub trait ComponentFactory: Send + Sync {
    /// Create a new instance of the component
    fn create(&self) -> DynComponentRef;
    /// Get the type ID of the component
    fn type_id(&self) -> TypeId;
    /// Get the human-readable name of the component
    fn type_name(&self) -> &str;
    /// Get the scope of the component
    fn scope(&self) -> ComponentScope;
    /// Execute initialization lifecycle hook
    fn initialize(&self, component: &DynComponentRef) -> Result<()> {
        component.execute_lifecycle(LifecyclePhase::Initialize)
    }
    /// Execute destruction lifecycle hook
    fn destroy(&self, component: &DynComponentRef) -> Result<()> {
        component.execute_lifecycle(LifecyclePhase::Destroy)
    }
}

/// Async factory extension for executing async lifecycle methods
pub trait AsyncComponentFactory: ComponentFactory {
    /// Execute async initialization lifecycle hook
    async fn initialize_async(&self, component: &DynComponentRef) -> Result<()> {
        component
            .execute_async_lifecycle(LifecyclePhase::Initialize)
            .await
    }
    /// Execute async destruction lifecycle hook
    async fn destroy_async(&self, component: &DynComponentRef) -> Result<()> {
        component
            .execute_async_lifecycle(LifecyclePhase::Destroy)
            .await
    }
}

// Implement AsyncComponentFactory for all ComponentFactory implementors
impl<T: ComponentFactory> AsyncComponentFactory for T {}

/// Factory for creating instances of a specific component type
pub struct TypedComponentFactory<T: Any + Send + Sync, F: Fn() -> T + Send + Sync> {
    factory: F,
    scope: ComponentScope,
}

impl<T: Any + Send + Sync, F: Fn() -> T + Send + Sync> TypedComponentFactory<T, F> {
    /// Create a new typed component factory
    pub fn new(factory: F, scope: ComponentScope) -> Self {
        Self { factory, scope }
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
    components: HashMap<TypeId, DynComponentRef>,
    factories: HashMap<TypeId, Box<dyn ComponentFactory>>,
}

impl ComponentRegistry {
    /// Create a new empty component registry
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
            factories: HashMap::new(),
        }
    }

    /// Register a component instance
    pub fn register<T: Any + Send + Sync>(&mut self, component: T) -> Result<()> {
        let type_id = TypeId::of::<T>();
        let component_ref = DynComponentRef::new(component);

        // Initialize lifecycle if component supports it
        component_ref.execute_lifecycle(LifecyclePhase::Initialize)?;

        self.components.insert(type_id, component_ref);
        Ok(())
    }

    /// Register a component factory
    pub fn register_factory<F>(&mut self, factory: Box<dyn ComponentFactory>) {
        let type_id = factory.type_id();
        self.factories.insert(type_id, factory);
    }

    /// Register a component with a factory function and scope
    pub fn register_with_factory<T, F>(&mut self, factory: F, scope: ComponentScope)
    where
        T: Any + Send + Sync,
        F: Fn() -> T + Send + Sync + 'static,
    {
        let typed_factory = TypedComponentFactory::new(factory, scope);
        self.register_factory::<TypedComponentFactory<T, F>>(Box::new(typed_factory));
    }

    /// Get a component by type. Returns an error if the component is not registered.
    pub fn get<T: Send + Sync + Clone + 'static>(&self) -> Result<ComponentRef<T>> {
        let type_id = TypeId::of::<T>();

        // First check if we have it as a direct component
        if let Some(component) = self.components.get(&type_id) {
            if let Some(typed_component) = component.downcast_ref::<T>() {
                return Ok(ComponentRef::new(typed_component.clone()));
            }
        }

        // If not found, create a meaningful error message
        Err(Error::component(&format!(
            "Component not found: {}",
            std::any::type_name::<T>()
        )))
    }

    /// Try to get a component by type. Returns None if the component is not registered.
    pub fn try_get<T: Send + Sync + Clone + 'static>(&self) -> Result<Option<ComponentRef<T>>> {
        let type_id = TypeId::of::<T>();

        // Check if we have it as a direct component
        if let Some(component) = self.components.get(&type_id) {
            if let Some(typed_component) = component.downcast_ref::<T>() {
                return Ok(Some(ComponentRef::new(typed_component.clone())));
            }
        }

        // If not found, return None
        Ok(None)
    }

    /// Get a component by type with async initialization
    pub async fn get_async<T: 'static + Send + Sync + Clone>(&self) -> Result<ComponentRef<T>> {
        let type_id = TypeId::of::<T>();

        // Try to get the component similar to the synchronous version
        if let Some(component) = self.components.get(&type_id) {
            if let Some(typed_component) = component.downcast_ref::<T>() {
                return Ok(ComponentRef::new(typed_component.clone()));
            }
        }

        // If not found, return an error
        Err(Error::component(&format!(
            "Component not found: {}",
            std::any::type_name::<T>()
        )))
    }

    /// Check if a component is registered
    pub fn has<T: Any + Send + Sync>(&self) -> bool {
        let type_id = TypeId::of::<T>();
        self.components.contains_key(&type_id) || self.factories.contains_key(&type_id)
    }

    /// Get all registered component types
    pub fn component_types(&self) -> Vec<&str> {
        let mut types = Vec::new();

        for factory in self.factories.values() {
            types.push(factory.type_name());
        }

        for component in self.components.keys() {
            if !self.factories.contains_key(component) {
                types.push(std::any::type_name_of_val(&component));
            }
        }

        types
    }

    /// Shutdown the registry and destroy all components
    pub fn shutdown(&mut self) -> Result<()> {
        // Execute destroy lifecycle for all components
        for component in self.components.values() {
            if let Err(e) = component.execute_lifecycle(LifecyclePhase::Destroy) {
                eprintln!("Error destroying component: {}", e);
                // Continue with other components even if one fails
            }
        }

        // Clear the registry
        self.components.clear();
        self.factories.clear();

        Ok(())
    }

    /// Shutdown the registry asynchronously and destroy all components
    pub async fn shutdown_async(&mut self) -> Result<()> {
        // Get a copy of all component refs to avoid borrowing issues
        let components: Vec<_> = self.components.values().cloned().collect();

        // Destroy all components asynchronously
        for component in components {
            if let Err(e) = component
                .execute_async_lifecycle(LifecyclePhase::Destroy)
                .await
            {
                eprintln!("Error destroying component: {}", e);
            }
        }

        // Clear all registrations
        self.components.clear();
        self.factories.clear();
        Ok(())
    }

    /// Get all component references
    pub fn values(&self) -> impl Iterator<Item = &DynComponentRef> {
        self.components.values()
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
        let mut registry = ComponentRegistry::new();
        let component = TestComponent {
            value: "test".to_string(),
        };

        registry.register(component).unwrap();

        let retrieved = registry.get::<TestComponent>().unwrap();
        assert_eq!(retrieved.value, "test");
    }

    #[test]
    fn register_factory_singleton() {
        let mut registry = ComponentRegistry::new();

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
        let mut registry = ComponentRegistry::new();
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
        let mut registry = ComponentRegistry::new();
        let result = registry.get::<String>();
        assert!(result.is_err());
    }

    #[test]
    fn has_component() {
        let mut registry = ComponentRegistry::new();
        registry
            .register(TestComponent {
                value: "test".to_string(),
            })
            .unwrap();

        assert!(registry.has::<TestComponent>());
        assert!(!registry.has::<String>());
    }
}
