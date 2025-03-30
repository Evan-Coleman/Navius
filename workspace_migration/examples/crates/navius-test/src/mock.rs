use crate::error::{TestError, TestResult};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};

/// The internal state of the mock registry
pub struct MockRegistryState {
    /// Map of registered mocks by interface type ID
    mocks: HashMap<TypeId, Box<dyn Any + Send + Sync>>,

    /// Map of interface type IDs to implementation type IDs
    implementation_map: HashMap<TypeId, TypeId>,

    /// Map of interface names to registered expectations
    expectations: HashMap<String, Vec<Expectation>>,
}

impl MockRegistryState {
    /// Create a new mock registry state
    pub fn new() -> Self {
        Self {
            mocks: HashMap::new(),
            implementation_map: HashMap::new(),
            expectations: HashMap::new(),
        }
    }
}

/// An expectation on a mock method
#[derive(Debug)]
pub struct Expectation {
    /// The method name
    pub method_name: String,

    /// Whether the expectation has been satisfied
    pub satisfied: bool,

    /// The number of times the method was called
    pub call_count: usize,

    /// The minimum number of times the method should be called
    pub min_calls: usize,

    /// The maximum number of times the method should be called
    pub max_calls: Option<usize>,
}

impl Expectation {
    /// Create a new expectation
    pub fn new(method_name: &str) -> Self {
        Self {
            method_name: method_name.to_string(),
            satisfied: false,
            call_count: 0,
            min_calls: 1,
            max_calls: None,
        }
    }

    /// Set the minimum number of calls
    pub fn with_min_calls(mut self, min: usize) -> Self {
        self.min_calls = min;
        self
    }

    /// Set the maximum number of calls
    pub fn with_max_calls(mut self, max: usize) -> Self {
        self.max_calls = Some(max);
        self
    }

    /// Record a call to the method
    pub fn record_call(&mut self) {
        self.call_count += 1;

        // Update satisfied status
        self.satisfied = self.call_count >= self.min_calls
            && (self.max_calls.is_none() || self.call_count <= self.max_calls.unwrap());
    }

    /// Check if the expectation is satisfied
    pub fn is_satisfied(&self) -> bool {
        self.call_count >= self.min_calls
            && (self.max_calls.is_none() || self.call_count <= self.max_calls.unwrap())
    }
}

/// A registry for mock implementations
#[derive(Clone)]
pub struct MockRegistry {
    /// The internal state of the registry
    state: Arc<Mutex<MockRegistryState>>,
}

impl MockRegistry {
    /// Create a new mock registry
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(MockRegistryState::new())),
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
            .insert(interface_id, implementation_id);

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
                expectations: HashMap::new(),
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

        state.mocks.remove(&interface_id);
        state.implementation_map.remove(&interface_id);

        Ok(())
    }

    /// Add an expectation for an interface method
    pub fn expect<Interface>(&self, method_name: &str) -> TestResult<&Self>
    where
        Interface: Any + Send + Sync + ?Sized,
    {
        let interface_name = std::any::type_name::<Interface>();

        let mut state = self.state.lock().map_err(|e| {
            TestError::setup_error(format!("Failed to lock mock registry state: {}", e))
        })?;

        let expectations = state
            .expectations
            .entry(interface_name.to_string())
            .or_insert_with(Vec::new);
        expectations.push(Expectation::new(method_name));

        Ok(self)
    }

    /// Record a call to an interface method
    pub fn record_call<Interface>(&self, method_name: &str) -> TestResult<()>
    where
        Interface: Any + Send + Sync + ?Sized,
    {
        let interface_name = std::any::type_name::<Interface>();

        let mut state = self.state.lock().map_err(|e| {
            TestError::setup_error(format!("Failed to lock mock registry state: {}", e))
        })?;

        let expectations = state.expectations.get_mut(&interface_name.to_string());

        if let Some(expectations) = expectations {
            for expectation in expectations.iter_mut() {
                if expectation.method_name == method_name {
                    expectation.record_call();
                }
            }
        }

        Ok(())
    }

    /// Verify that all expectations have been met
    pub fn verify(&self) -> TestResult<()> {
        let state = self.state.lock().map_err(|e| {
            TestError::setup_error(format!("Failed to lock mock registry state: {}", e))
        })?;

        let mut unsatisfied = Vec::new();

        for (interface_name, expectations) in &state.expectations {
            for expectation in expectations {
                if !expectation.is_satisfied() {
                    unsatisfied.push(format!(
                        "{}.{}: expected {} calls, got {}",
                        interface_name,
                        expectation.method_name,
                        expectation.min_calls,
                        expectation.call_count
                    ));
                }
            }
        }

        if unsatisfied.is_empty() {
            Ok(())
        } else {
            Err(TestError::assertion_error(format!(
                "Unsatisfied expectations:\n{}",
                unsatisfied.join("\n")
            )))
        }
    }

    /// Reset all expectations
    pub fn reset(&self) -> TestResult<()> {
        let mut state = self.state.lock().map_err(|e| {
            TestError::setup_error(format!("Failed to lock mock registry state: {}", e))
        })?;

        state.expectations.clear();

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

/// A helper for creating mock expectations
pub struct ExpectationBuilder<'a> {
    /// The mock registry
    registry: &'a MockRegistry,

    /// The interface name
    interface_name: String,

    /// The method name
    method_name: String,

    /// The minimum number of calls
    min_calls: usize,

    /// The maximum number of calls
    max_calls: Option<usize>,
}

impl<'a> ExpectationBuilder<'a> {
    /// Create a new expectation builder
    pub fn new<Interface>(registry: &'a MockRegistry, method_name: &str) -> Self
    where
        Interface: Any + Send + Sync + ?Sized,
    {
        Self {
            registry,
            interface_name: std::any::type_name::<Interface>().to_string(),
            method_name: method_name.to_string(),
            min_calls: 1,
            max_calls: None,
        }
    }

    /// Set the minimum number of calls
    pub fn min_calls(mut self, min: usize) -> Self {
        self.min_calls = min;
        self
    }

    /// Set the maximum number of calls
    pub fn max_calls(mut self, max: usize) -> Self {
        self.max_calls = Some(max);
        self
    }

    /// Set the exact number of calls
    pub fn times(mut self, count: usize) -> Self {
        self.min_calls = count;
        self.max_calls = Some(count);
        self
    }

    /// Build the expectation
    pub fn build(self) -> TestResult<()> {
        let mut state = self.registry.state.lock().map_err(|e| {
            TestError::setup_error(format!("Failed to lock mock registry state: {}", e))
        })?;

        let expectations = state
            .expectations
            .entry(self.interface_name)
            .or_insert_with(Vec::new);

        let mut expectation = Expectation::new(&self.method_name);
        expectation.min_calls = self.min_calls;
        expectation.max_calls = self.max_calls;

        expectations.push(expectation);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Define a simple trait for testing
    trait TestService: Send + Sync {
        fn get_value(&self) -> i32;
        fn set_value(&mut self, value: i32);
    }

    // Define a mock implementation
    #[derive(Clone)]
    struct MockTestService {
        value: i32,
    }

    impl MockTestService {
        fn new() -> Self {
            Self { value: 0 }
        }
    }

    impl TestService for MockTestService {
        fn get_value(&self) -> i32 {
            self.value
        }

        fn set_value(&mut self, value: i32) {
            self.value = value;
        }
    }

    #[test]
    fn test_register_and_get_mock() {
        let registry = MockRegistry::new();

        // Register a mock
        registry
            .register::<dyn TestService, MockTestService>(MockTestService::new())
            .unwrap();

        // Get the mock
        let mock: MockTestService = registry.get::<dyn TestService, MockTestService>().unwrap();

        // Verify we got the mock
        assert_eq!(mock.get_value(), 0);
    }

    #[test]
    fn test_expectations() {
        let registry = MockRegistry::new();

        // Register a mock
        registry
            .register::<dyn TestService, MockTestService>(MockTestService::new())
            .unwrap();

        // Set up expectations
        registry.expect::<dyn TestService>("get_value").unwrap();

        // Record calls
        registry
            .record_call::<dyn TestService>("get_value")
            .unwrap();

        // Verify expectations
        registry.verify().unwrap();
    }

    #[test]
    fn test_expectation_builder() {
        let registry = MockRegistry::new();

        // Create expectation with the builder
        ExpectationBuilder::<dyn TestService>::new(&registry, "get_value")
            .times(2)
            .build()
            .unwrap();

        // Record one call
        registry
            .record_call::<dyn TestService>("get_value")
            .unwrap();

        // Verification should fail (only one call made)
        assert!(registry.verify().is_err());

        // Record another call
        registry
            .record_call::<dyn TestService>("get_value")
            .unwrap();

        // Now verification should succeed
        registry.verify().unwrap();
    }
}
