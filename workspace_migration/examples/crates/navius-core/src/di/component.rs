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

    /// Downcast the dynamic reference to a specific type
    pub fn downcast<T: Any + Send + Sync>(self) -> Option<ComponentRef<T>> {
        self.0.downcast::<T>().ok().map(ComponentRef::new)
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
}

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
    pub fn register<T: Any + Send + Sync>(&mut self, component: T) {
        let type_id = TypeId::of::<T>();
        let component_ref = DynComponentRef::new(component);
        self.components.insert(type_id, component_ref);
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
        self.register_factory(Box::new(typed_factory));
    }

    /// Get a component by type
    pub fn get<T: Any + Send + Sync>(&mut self) -> Result<ComponentRef<T>> {
        let type_id = TypeId::of::<T>();

        // Check if we have a cached instance for singletons
        if let Some(component) = self.components.get(&type_id) {
            if let Some(typed_ref) = component.clone().downcast::<T>() {
                return Ok(typed_ref);
            }
        }

        // Check if we have a factory
        if let Some(factory) = self.factories.get(&type_id) {
            let dyn_ref = factory.create();

            // For singletons, cache the instance
            if factory.scope() == ComponentScope::Singleton {
                self.components.insert(type_id, dyn_ref.clone());
            }

            if let Some(typed_ref) = dyn_ref.downcast::<T>() {
                return Ok(typed_ref);
            }
        }

        Err(Error::new(&format!(
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

        types
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

        registry.register(component);

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
        registry.register(TestComponent {
            value: "test".to_string(),
        });

        assert!(registry.has::<TestComponent>());
        assert!(!registry.has::<String>());
    }
}
