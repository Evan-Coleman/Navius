use crate::error::{TestError, TestResult};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};

/// A registry for mock implementations
#[derive(Clone)]
pub struct MockRegistry {
    /// The internal state of the registry
    state: Arc<Mutex<MockRegistryState>>,
}

/// The internal state of a mock registry
struct MockRegistryState {
    /// Map of registered mock implementations by interface type ID
    mocks: HashMap<TypeId, Box<dyn Any + Send + Sync>>,

    /// Map of implementation type IDs to interface type IDs
    implementation_map: HashMap<TypeId, TypeId>,
}

impl MockRegistry {
    /// Create a new mock registry
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(MockRegistryState {
                mocks: HashMap::new(),
                implementation_map: HashMap::new(),
            })),
        }
    }

    /// Register a mock implementation for an interface
    pub fn register<Interface, Implementation>(
        &self,
        implementation: Implementation,
    ) -> TestResult<()>
    where
        Interface: Any + Send + Sync + ?Sized,
        Implementation: Any + Send + Sync,
    {
        let interface_id = TypeId::of::<Interface>();
        let implementation_id = TypeId::of::<Implementation>();

        let mut state = self.state.lock().map_err(|e| {
            TestError::setup_error(format!("Failed to lock mock registry state: {}", e))
        })?;

        state.mocks.insert(interface_id, Box::new(implementation));
        state
            .implementation_map
            .insert(implementation_id, interface_id);

        Ok(())
    }

    /// Get a mock implementation for an interface
    pub fn get<Interface, Implementation>(&self) -> TestResult<Implementation>
    where
        Interface: Any + Send + Sync + ?Sized,
        Implementation: Any + Send + Sync + Clone,
    {
        let interface_id = TypeId::of::<Interface>();

        let state = self.state.lock().map_err(|e| {
            TestError::setup_error(format!("Failed to lock mock registry state: {}", e))
        })?;

        state
            .mocks
            .get(&interface_id)
            .and_then(|boxed| boxed.downcast_ref::<Implementation>())
            .map(|implementation| implementation.clone())
            .ok_or_else(|| TestError::mock_not_registered(std::any::type_name::<Interface>()))
    }

    /// Check if a mock implementation is registered for an interface
    pub fn has<Interface>(&self) -> bool
    where
        Interface: Any + Send + Sync + ?Sized,
    {
        let interface_id = TypeId::of::<Interface>();

        let state = self.state.lock().unwrap_or_else(|e| {
            // In case of error, log and return an empty state
            eprintln!("Failed to lock mock registry state: {}", e);
            MockRegistryState {
                mocks: HashMap::new(),
                implementation_map: HashMap::new(),
            }
        });

        state.mocks.contains_key(&interface_id)
    }

    /// Remove a mock implementation for an interface
    pub fn remove<Interface>(&self) -> TestResult<()>
    where
        Interface: Any + Send + Sync + ?Sized,
    {
        let interface_id = TypeId::of::<Interface>();

        let mut state = self.state.lock().map_err(|e| {
            TestError::setup_error(format!("Failed to lock mock registry state: {}", e))
        })?;

        // Find and remove the implementation ID associated with this interface
        let implementation_ids: Vec<_> = state
            .implementation_map
            .iter()
            .filter(|(_, interface)| **interface == interface_id)
            .map(|(impl_id, _)| *impl_id)
            .collect();

        for impl_id in implementation_ids {
            state.implementation_map.remove(&impl_id);
        }

        state.mocks.remove(&interface_id);

        Ok(())
    }

    /// Clear all mock implementations
    pub fn clear(&self) -> TestResult<()> {
        let mut state = self.state.lock().map_err(|e| {
            TestError::setup_error(format!("Failed to lock mock registry state: {}", e))
        })?;

        state.mocks.clear();
        state.implementation_map.clear();

        Ok(())
    }
}

impl fmt::Debug for MockRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.state.lock() {
            Ok(state) => {
                let mock_count = state.mocks.len();

                f.debug_struct("MockRegistry")
                    .field("mocks", &format!("{} registered", mock_count))
                    .finish()
            }
            Err(_) => write!(f, "MockRegistry(locked)"),
        }
    }
}

impl Default for MockRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for creating mock implementations
pub trait MockBuilder<T> {
    /// Create a mock implementation
    fn build_mock() -> T;
}

/// Helper trait for registering mock implementations
pub trait RegisterMock<Interface, Implementation> {
    /// Register a mock implementation
    fn register_mock(&self, implementation: Implementation) -> TestResult<()>;
}

impl<Interface, Implementation> RegisterMock<Interface, Implementation> for MockRegistry
where
    Interface: Any + Send + Sync + ?Sized,
    Implementation: Any + Send + Sync,
{
    fn register_mock(&self, implementation: Implementation) -> TestResult<()> {
        self.register::<Interface, Implementation>(implementation)
    }
}
