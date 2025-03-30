//! Component registry for dependency injection
//!
//! This module provides a component registry for type-safe dependency injection.
//! Components can be registered with the registry and then accessed by their type.

use std::any::{Any, TypeId};
use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::Arc;

use dashmap::DashMap;
use tracing::{instrument, trace, warn};

use crate::error::{PluginError, PluginResult};

/// Defines the scope for component instances.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    /// A single instance is created and shared across all consumers.
    Singleton,
    /// A new instance is created for each consumer.
    Prototype,
}

/// Component key used for identifying components in the registry.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ComponentKey {
    /// Type ID of the component.
    type_id: TypeId,
    /// Optional name for the component.
    name: Option<String>,
}

impl ComponentKey {
    /// Creates a new component key for the given type.
    pub fn new<T: 'static>(name: Option<String>) -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            name,
        }
    }

    /// Creates a new component key from a type ID and optional name.
    pub fn from_type_id(type_id: TypeId, name: Option<String>) -> Self {
        Self { type_id, name }
    }

    /// Returns the type ID of the component.
    pub fn type_id(&self) -> TypeId {
        self.type_id
    }

    /// Returns the name of the component, if any.
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
}

/// Wrapper around a component instance.
#[derive(Debug)]
pub struct Component<T: Send + Sync + 'static> {
    /// The component instance.
    instance: Arc<T>,
    /// Type marker for the component.
    _marker: PhantomData<T>,
}

impl<T: Send + Sync + 'static> Component<T> {
    /// Creates a new component with the given instance.
    pub fn new(instance: T) -> Self {
        Self {
            instance: Arc::new(instance),
            _marker: PhantomData,
        }
    }

    /// Returns a reference to the component instance.
    pub fn instance(&self) -> &T {
        &self.instance
    }

    /// Returns a clone of the Arc containing the instance.
    pub fn clone_instance(&self) -> Arc<T> {
        self.instance.clone()
    }
}

/// Registry for managing components.
///
/// The component registry is responsible for managing component instances
/// and their lifecycles.
#[derive(Default)]
pub struct ComponentRegistry {
    /// Map of component keys to their factory functions.
    factories: DashMap<ComponentKey, Arc<dyn Fn() -> Arc<dyn Any + Send + Sync> + Send + Sync>>,
    /// Map of component keys to their singleton instances.
    singletons: DashMap<ComponentKey, Arc<dyn Any + Send + Sync>>,
}

impl ComponentRegistry {
    /// Creates a new component registry.
    pub fn new() -> Self {
        Self {
            factories: DashMap::new(),
            singletons: DashMap::new(),
        }
    }

    /// Registers a component factory with the registry.
    ///
    /// - `scope`: The scope of the component.
    /// - `name`: Optional name for the component.
    /// - `factory`: A function that creates a new instance of the component.
    ///
    /// Returns a result indicating success or failure.
    #[instrument(skip(self, factory), fields(component_type = std::any::type_name::<T>(), name = ?name))]
    pub fn register<T: Send + Sync + 'static + Clone>(
        &self,
        scope: Scope,
        name: Option<String>,
        factory: impl Fn() -> T + Send + Sync + 'static,
    ) -> PluginResult<()> {
        let key = ComponentKey::new::<T>(name);

        // Create a factory that produces Arc<dyn Any>
        let factory = Arc::new(move || -> Arc<dyn Any + Send + Sync> {
            let component = factory();
            Arc::new(component) as Arc<dyn Any + Send + Sync>
        });

        // For singletons, create the instance immediately
        if scope == Scope::Singleton {
            let instance = factory();
            self.singletons.insert(key.clone(), instance);
        }

        // Store the factory
        self.factories.insert(key, factory);

        trace!("Component registered successfully");
        Ok(())
    }

    /// Registers a singleton component instance with the registry.
    ///
    /// - `name`: Optional name for the component.
    /// - `instance`: The component instance to register.
    ///
    /// Returns a result indicating success or failure.
    #[instrument(skip(self, instance), fields(component_type = std::any::type_name::<T>(), name = ?name))]
    pub fn register_instance<T: Send + Sync + 'static>(
        &self,
        name: Option<String>,
        instance: T,
    ) -> PluginResult<()> {
        let key = ComponentKey::new::<T>(name);

        // Create an Arc around the instance
        let instance_arc = Arc::new(instance) as Arc<dyn Any + Send + Sync>;

        // Store as a singleton
        self.singletons.insert(key.clone(), instance_arc.clone());

        // Also create a factory that returns the same instance
        let instance_clone = instance_arc.clone();
        let factory = Arc::new(move || -> Arc<dyn Any + Send + Sync> { instance_clone.clone() });

        self.factories.insert(key, factory);

        trace!("Component instance registered successfully");
        Ok(())
    }

    /// Retrieves a component from the registry.
    ///
    /// - `key`: The key of the component to retrieve.
    ///
    /// Returns the component if found, otherwise an error.
    #[instrument(skip(self), fields(component_type = std::any::type_name::<T>(), key = ?key))]
    pub fn get<T: Send + Sync + 'static + Clone>(
        &self,
        key: &ComponentKey,
    ) -> PluginResult<Component<T>> {
        // For singletons, return the existing instance if available
        if let Some(singleton) = self.singletons.get(key) {
            trace!("Retrieved singleton component");

            // Clone the Arc to avoid borrowing issues
            let singleton_clone = singleton.clone();

            // Attempt to downcast to the requested type
            match (*singleton_clone).downcast_ref::<T>() {
                Some(component) => {
                    return Ok(Component {
                        instance: Arc::new(component.clone()),
                        _marker: PhantomData,
                    });
                }
                None => {
                    return Err(PluginError::ComponentTypeMismatch {
                        requested: std::any::type_name::<T>().to_string(),
                        actual: format!("{:?}", singleton_clone.type_id()),
                    });
                }
            }
        }

        // For prototypes or singletons not yet created, use the factory
        if let Some(factory) = self.factories.get(key) {
            let instance = factory();

            // Attempt to downcast to the requested type
            match (*instance).downcast_ref::<T>() {
                Some(component) => {
                    trace!("Created new component instance from factory");
                    return Ok(Component {
                        instance: Arc::new(component.clone()),
                        _marker: PhantomData,
                    });
                }
                None => {
                    return Err(PluginError::ComponentTypeMismatch {
                        requested: std::any::type_name::<T>().to_string(),
                        actual: format!("{:?}", instance.type_id()),
                    });
                }
            }
        }

        // Component not found
        warn!("Component not found");
        Err(PluginError::ComponentNotFound {
            component_type: std::any::type_name::<T>().to_string(),
            name: key.name().map(String::from),
        })
    }

    /// Retrieves a component by type and optional name.
    ///
    /// - `name`: Optional name for the component.
    ///
    /// Returns the component if found, otherwise an error.
    #[instrument(skip(self), fields(component_type = std::any::type_name::<T>(), name = ?name))]
    pub fn get_by_type<T: Send + Sync + 'static + Clone>(
        &self,
        name: Option<&str>,
    ) -> PluginResult<Component<T>> {
        let key = ComponentKey::new::<T>(name.map(String::from));
        self.get::<T>(&key)
    }

    /// Checks if a component exists in the registry.
    ///
    /// - `key`: The key of the component to check.
    ///
    /// Returns true if the component exists, otherwise false.
    pub fn contains(&self, key: &ComponentKey) -> bool {
        self.factories.contains_key(key) || self.singletons.contains_key(key)
    }

    /// Removes a component from the registry.
    ///
    /// - `key`: The key of the component to remove.
    ///
    /// Returns true if the component was removed, otherwise false.
    pub fn remove(&self, key: &ComponentKey) -> bool {
        let factory_removed = self.factories.remove(key).is_some();
        let singleton_removed = self.singletons.remove(key).is_some();
        factory_removed || singleton_removed
    }

    /// Clears all components from the registry.
    pub fn clear(&self) {
        self.factories.clear();
        self.singletons.clear();
    }

    /// Returns all component keys currently registered.
    pub fn get_all_keys(&self) -> Vec<ComponentKey> {
        self.factories
            .iter()
            .map(|entry| entry.key().clone())
            .collect()
    }
}

impl Debug for ComponentRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ComponentRegistry")
            .field("factories_count", &self.factories.len())
            .field("singletons_count", &self.singletons.len())
            .finish()
    }
}
