// Export base types for mocking
pub mod config;
pub mod events;
pub mod expect;

// Define the Expectation type and other key types
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::error::{TestError, TestResult};

// Core traits and structures for mocking
pub trait MockProvider {
    fn register(&self, registry: &MockRegistry) -> Arc<Self>;
}

pub trait MockVerify {
    fn verify(&self) -> TestResult<()>;
}

#[derive(Debug)]
pub struct MockRegistry {
    mocks: RwLock<HashMap<TypeId, Box<dyn Any + Send + Sync>>>,
}

impl MockRegistry {
    pub fn new() -> Self {
        MockRegistry {
            mocks: RwLock::new(HashMap::new()),
        }
    }

    /// Register a mock component
    pub fn register<T: ?Sized + 'static, M: 'static + Send + Sync>(
        &self,
        mock: Arc<M>,
    ) -> TestResult<()> {
        let type_id = TypeId::of::<T>();
        let boxed_mock = Box::new(mock);
        let dyn_mock = boxed_mock as Box<dyn Any + Send + Sync>;

        let mut mocks = self.mocks.write().unwrap();
        mocks.insert(type_id, dyn_mock);

        Ok(())
    }

    /// Register a mock component without the additional parameters
    pub fn register_mock<T: ?Sized + 'static>(&self, mock: Arc<impl Any + Send + Sync + 'static>) {
        let type_id = TypeId::of::<T>();
        let boxed_mock = Box::new(mock);
        let dyn_mock = boxed_mock as Box<dyn Any + Send + Sync>;

        let mut mocks = self.mocks.write().unwrap();
        mocks.insert(type_id, dyn_mock);
    }

    /// Get a mock component
    pub fn get<T: ?Sized + 'static>(&self) -> TestResult<Arc<dyn Any + Send + Sync>> {
        let type_id = TypeId::of::<T>();
        let mocks = self.mocks.read().unwrap();

        if let Some(mock) = mocks.get(&type_id) {
            // SAFETY: We're downcasting to the type we registered with
            let arc_any = mock.downcast_ref::<Arc<dyn Any + Send + Sync>>().unwrap();
            Ok(arc_any.clone())
        } else {
            Err(TestError::missing_component(format!(
                "Mock for {:?} not found",
                type_id
            )))
        }
    }

    /// Verify all mocks
    pub fn verify(&self) -> TestResult<()> {
        let mocks = self.mocks.read().unwrap();

        let mut errors = Vec::new();

        for (_, mock) in mocks.iter() {
            // Try to downcast to MockVerify
            if let Some(verifiable) = mock.downcast_ref::<Arc<dyn MockVerify + Send + Sync>>() {
                if let Err(e) = verifiable.verify() {
                    errors.push(e);
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(TestError::mock_expectation_error(format!(
                "Mock verification failed: {:?}",
                errors
            )))
        }
    }
}

// Re-export from the config, events, and expect modules
pub use self::config::*;
pub use self::events::*;
pub use self::expect::*;
