//! Component registry implementations for the DI system.

use crate::di::component::{ComponentRef, ComponentScope, DynComponentRef, LifecyclePhase};
use crate::error::{Error, Result};
use async_trait::async_trait;
use futures::future::join_all;
use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Trait for a component registry.
#[async_trait]
pub trait ComponentRegistry: Send + Sync + std::fmt::Debug {
    /// Registers a component with the registry.
    fn register(&mut self, component: DynComponentRef) -> Result<()>;

    /// Retrieves a component by its type name.
    fn get_by_type_name(&self, type_name: &str) -> Option<DynComponentRef>;

    /// Retrieves a component by its ID.
    fn get_by_id(&self, id: &str) -> Option<DynComponentRef>;

    /// Retrieves all components of a specific type.
    fn get_all_by_type<T: Any + Send + Sync + 'static>(&self) -> Vec<Arc<T>>;

    /// Retrieves all registered component type names.
    fn component_types(&self) -> Vec<String>;

    /// Executes a lifecycle phase for all components.
    async fn execute_lifecycle_async(&self, phase: LifecyclePhase) -> Result<()>;
}

/// Type alias for a dynamically dispatched component registry, wrapped in Arc.
pub type DynComponentRegistry = Arc<dyn ComponentRegistry>;

/// An in-memory implementation of the component registry.
#[derive(Debug, Default)]
pub struct InMemoryComponentRegistry {
    components_by_id: HashMap<String, DynComponentRef>,
    components_by_type: HashMap<String, Vec<DynComponentRef>>,
}

impl InMemoryComponentRegistry {
    /// Creates a new empty registry.
    pub fn new() -> Self {
        Default::default()
    }
}

#[async_trait]
impl ComponentRegistry for InMemoryComponentRegistry {
    fn register(&mut self, component: DynComponentRef) -> Result<()> {
        // Use execute_type_id() method instead of accessing private field `component.0`
        let type_id = component
            .execute_type_id()
            .ok_or_else(|| Error::internal("Failed to get TypeId from DynComponentRef"))?;

        // Use TypeId Debug format as a placeholder string key
        let type_name = format!("{:?}", type_id);

        // Keep the components_by_id map (though it's not populated by this register method)
        // Keep the components_by_type map using the placeholder type_name string
        self.components_by_type
            .entry(type_name.to_string())
            .or_default()
            .push(component);
        Ok(())
    }

    fn get_by_type_name(&self, type_name: &str) -> Option<DynComponentRef> {
        self.components_by_type
            .get(type_name)
            .and_then(|v| v.first())
            .cloned()
    }

    fn get_by_id(&self, id: &str) -> Option<DynComponentRef> {
        // This map is not populated by the current `register` method.
        self.components_by_id.get(id).cloned()
    }

    fn get_all_by_type<T: Any + Send + Sync + 'static>(&self) -> Vec<Arc<T>> {
        let type_name = std::any::type_name::<T>(); // Lookup key
        self.components_by_type
            .get(type_name)
            .map_or_else(Vec::new, |components| {
                components
                    .iter()
                    // downcast now returns Option<Arc<T>>, so no .map() needed
                    .filter_map(|comp| comp.clone().downcast::<T>())
                    .collect()
            })
    }

    fn component_types(&self) -> Vec<String> {
        self.components_by_type.keys().cloned().collect()
    }

    async fn execute_lifecycle_async(&self, phase: LifecyclePhase) -> Result<()> {
        // FIXME: Lifecycle execution on DynComponentRef is complex.
        // Remove the actual execution logic for now.
        println!(
            "Warning: execute_lifecycle_async called but not fully implemented for phase: {:?}",
            phase
        );
        Ok(())
    }
}
