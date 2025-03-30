use std::error::Error as StdError;
use std::fmt;

/// Result type for test operations
pub type TestResult<T> = Result<T, TestError>;

/// Error type for test operations
#[derive(Debug)]
pub enum TestError {
    /// Error during test setup
    SetupError(String),

    /// Error during test execution
    ExecutionError(String),

    /// Error during test teardown
    TeardownError(String),

    /// Missing component in test fixture
    MissingComponent(String),

    /// Mock implementation not registered
    MockNotRegistered(String),

    /// Error in mock expectation
    MockExpectationError(String),

    /// Error when injecting failures
    InjectedError(String),

    /// Error when verifying error propagation
    ErrorPropagationError(String),

    /// Error when verifying error context
    ErrorContextError(String),
}

impl TestError {
    /// Create a new setup error
    pub fn setup_error<S: Into<String>>(message: S) -> Self {
        TestError::SetupError(message.into())
    }

    /// Create a new execution error
    pub fn execution_error<S: Into<String>>(message: S) -> Self {
        TestError::ExecutionError(message.into())
    }

    /// Create a new teardown error
    pub fn teardown_error<S: Into<String>>(message: S) -> Self {
        TestError::TeardownError(message.into())
    }

    /// Create a new missing component error
    pub fn missing_component<S: Into<String>>(component_type: S) -> Self {
        TestError::MissingComponent(format!(
            "Missing component of type: {}",
            component_type.into()
        ))
    }

    /// Create a new mock not registered error
    pub fn mock_not_registered<S: Into<String>>(interface_type: S) -> Self {
        TestError::MockNotRegistered(format!(
            "Mock not registered for interface: {}",
            interface_type.into()
        ))
    }

    /// Create a new mock expectation error
    pub fn mock_expectation_error<S: Into<String>>(message: S) -> Self {
        TestError::MockExpectationError(message.into())
    }

    /// Create a new injected error
    pub fn injected_error<S: Into<String>>(message: S) -> Self {
        TestError::InjectedError(message.into())
    }

    /// Create a new error propagation error
    pub fn error_propagation_error<S: Into<String>>(message: S) -> Self {
        TestError::ErrorPropagationError(message.into())
    }

    /// Create a new error context error
    pub fn error_context_error<S: Into<String>>(message: S) -> Self {
        TestError::ErrorContextError(message.into())
    }
}

impl fmt::Display for TestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TestError::SetupError(msg) => write!(f, "Test setup error: {}", msg),
            TestError::ExecutionError(msg) => write!(f, "Test execution error: {}", msg),
            TestError::TeardownError(msg) => write!(f, "Test teardown error: {}", msg),
            TestError::MissingComponent(msg) => write!(f, "Missing component: {}", msg),
            TestError::MockNotRegistered(msg) => write!(f, "Mock not registered: {}", msg),
            TestError::MockExpectationError(msg) => write!(f, "Mock expectation error: {}", msg),
            TestError::InjectedError(msg) => write!(f, "Injected error: {}", msg),
            TestError::ErrorPropagationError(msg) => write!(f, "Error propagation error: {}", msg),
            TestError::ErrorContextError(msg) => write!(f, "Error context error: {}", msg),
        }
    }
}

impl StdError for TestError {}

/// Error injection point for testing error handling
#[derive(Debug, Clone)]
pub struct ErrorInjection {
    /// The name of the error injection point
    name: String,

    /// Whether the error should be injected
    should_inject: bool,

    /// The error message to use when injecting
    error_message: String,

    /// The number of times to skip before injecting
    skip_count: usize,

    /// The current invocation count
    invocation_count: usize,
}

impl ErrorInjection {
    /// Create a new error injection point
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            should_inject: false,
            error_message: "Injected error".to_string(),
            skip_count: 0,
            invocation_count: 0,
        }
    }

    /// Inject an error at this point
    pub fn inject(mut self) -> Self {
        self.should_inject = true;
        self
    }

    /// Set the error message
    pub fn with_message<S: Into<String>>(mut self, message: S) -> Self {
        self.error_message = message.into();
        self
    }

    /// Skip a number of invocations before injecting
    pub fn skip(mut self, count: usize) -> Self {
        self.skip_count = count;
        self
    }

    /// Check if an error should be injected and generate the error if needed
    pub fn check<T, E, F>(&mut self, f: F) -> Result<T, E>
    where
        F: FnOnce() -> Result<T, E>,
        E: From<String>,
    {
        self.invocation_count += 1;

        if self.should_inject && self.invocation_count > self.skip_count {
            return Err(self.error_message.clone().into());
        }

        f()
    }
}

/// Error context for tracking error propagation
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// The name of the component that generated the error
    pub component: String,

    /// The operation that failed
    pub operation: String,

    /// Additional context information
    pub context: Vec<(String, String)>,
}

impl ErrorContext {
    /// Create a new error context
    pub fn new<S1: Into<String>, S2: Into<String>>(component: S1, operation: S2) -> Self {
        Self {
            component: component.into(),
            operation: operation.into(),
            context: Vec::new(),
        }
    }

    /// Add context information
    pub fn add_context<S1: Into<String>, S2: Into<String>>(
        &mut self,
        key: S1,
        value: S2,
    ) -> &mut Self {
        self.context.push((key.into(), value.into()));
        self
    }

    /// Convert to a string representation
    pub fn to_string(&self) -> String {
        let mut result = format!(
            "Component: {}, Operation: {}",
            self.component, self.operation
        );

        for (key, value) in &self.context {
            result.push_str(&format!(", {}: {}", key, value));
        }

        result
    }
}

/// Error propagation tracker for testing error flows
#[derive(Debug, Clone)]
pub struct ErrorPropagationTracker {
    /// The path of components the error passed through
    pub path: Vec<String>,

    /// Contexts added at each step
    pub contexts: Vec<ErrorContext>,
}

impl ErrorPropagationTracker {
    /// Create a new error propagation tracker
    pub fn new() -> Self {
        Self {
            path: Vec::new(),
            contexts: Vec::new(),
        }
    }

    /// Add a component to the path
    pub fn add_component<S: Into<String>>(&mut self, component: S) -> &mut Self {
        self.path.push(component.into());
        self
    }

    /// Add a context to the tracker
    pub fn add_context(&mut self, context: ErrorContext) -> &mut Self {
        self.contexts.push(context);
        self
    }

    /// Check if the error passed through a specific component
    pub fn passed_through<S: AsRef<str>>(&self, component: S) -> bool {
        self.path.iter().any(|c| c == component.as_ref())
    }

    /// Check if a specific context key was added
    pub fn has_context<S: AsRef<str>>(&self, key: S) -> bool {
        self.contexts
            .iter()
            .any(|ctx| ctx.context.iter().any(|(k, _)| k == key.as_ref()))
    }

    /// Get the value for a specific context key
    pub fn get_context<S: AsRef<str>>(&self, key: S) -> Option<String> {
        for ctx in &self.contexts {
            for (k, v) in &ctx.context {
                if k == key.as_ref() {
                    return Some(v.clone());
                }
            }
        }

        None
    }
}

/// Error verification for testing error handling
#[derive(Debug)]
pub struct ErrorVerifier {
    /// Expected error message
    expected_message: Option<String>,

    /// Expected components in the error propagation path
    expected_path: Vec<String>,

    /// Expected context keys
    expected_context_keys: Vec<String>,
}

impl ErrorVerifier {
    /// Create a new error verifier
    pub fn new() -> Self {
        Self {
            expected_message: None,
            expected_path: Vec::new(),
            expected_context_keys: Vec::new(),
        }
    }

    /// Set the expected error message
    pub fn expect_message<S: Into<String>>(mut self, message: S) -> Self {
        self.expected_message = Some(message.into());
        self
    }

    /// Add an expected component to the path
    pub fn expect_component<S: Into<String>>(mut self, component: S) -> Self {
        self.expected_path.push(component.into());
        self
    }

    /// Add an expected context key
    pub fn expect_context<S: Into<String>>(mut self, key: S) -> Self {
        self.expected_context_keys.push(key.into());
        self
    }

    /// Verify that an error meets the expectations
    pub fn verify<E: fmt::Display>(
        &self,
        error: &E,
        tracker: &ErrorPropagationTracker,
    ) -> TestResult<()> {
        // Check error message if set
        if let Some(ref expected_message) = self.expected_message {
            let error_str = error.to_string();
            if !error_str.contains(expected_message) {
                return Err(TestError::error_verification_error(format!(
                    "Expected error message '{}', got '{}'",
                    expected_message, error_str
                )));
            }
        }

        // Check error path
        for component in &self.expected_path {
            if !tracker.passed_through(component) {
                return Err(TestError::error_verification_error(format!(
                    "Expected error to pass through component '{}', but it didn't",
                    component
                )));
            }
        }

        // Check context keys
        for key in &self.expected_context_keys {
            if !tracker.has_context(key) {
                return Err(TestError::error_verification_error(format!(
                    "Expected error to have context key '{}', but it didn't",
                    key
                )));
            }
        }

        Ok(())
    }
}

impl TestError {
    /// Create a new error verification error
    pub fn error_verification_error<S: Into<String>>(message: S) -> Self {
        TestError::ExecutionError(message.into())
    }
}

/// Helper function to simulate errors for testing
pub fn simulate_error<T, E, F>(probability: f64, error_message: &str, f: F) -> Result<T, E>
where
    F: FnOnce() -> Result<T, E>,
    E: From<String>,
{
    let random = rand::random::<f64>();
    if random < probability {
        Err(error_message.to_string().into())
    } else {
        f()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_injection() {
        // Test error injection
        let mut injection = ErrorInjection::new("test_injection").inject();
        let result: Result<(), String> = injection.check(|| Ok(()));
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Injected error");

        // Test with custom message
        let mut injection = ErrorInjection::new("test_injection")
            .inject()
            .with_message("Custom error");
        let result: Result<(), String> = injection.check(|| Ok(()));
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Custom error");

        // Test with skip
        let mut injection = ErrorInjection::new("test_injection").inject().skip(1);
        let result1: Result<(), String> = injection.check(|| Ok(()));
        assert!(result1.is_ok());
        let result2: Result<(), String> = injection.check(|| Ok(()));
        assert!(result2.is_err());
    }

    #[test]
    fn test_error_context() {
        // Test error context
        let mut context = ErrorContext::new("database", "query");
        context
            .add_context("query", "SELECT * FROM users")
            .add_context("user_id", "123");

        assert_eq!(context.component, "database");
        assert_eq!(context.operation, "query");
        assert_eq!(context.context.len(), 2);
        assert_eq!(context.context[0].0, "query");
        assert_eq!(context.context[0].1, "SELECT * FROM users");
    }

    #[test]
    fn test_error_propagation_tracker() {
        // Test error propagation tracker
        let mut tracker = ErrorPropagationTracker::new();
        tracker
            .add_component("controller")
            .add_component("service")
            .add_component("repository");

        let mut context1 = ErrorContext::new("controller", "process_request");
        context1.add_context("user_id", "123");

        let mut context2 = ErrorContext::new("service", "get_user");
        context2.add_context("user_id", "123");

        tracker.add_context(context1).add_context(context2);

        assert!(tracker.passed_through("controller"));
        assert!(tracker.passed_through("service"));
        assert!(tracker.has_context("user_id"));
        assert_eq!(tracker.get_context("user_id"), Some("123".to_string()));
    }

    #[test]
    fn test_error_verifier() {
        // Test error verifier
        let mut tracker = ErrorPropagationTracker::new();
        tracker
            .add_component("controller")
            .add_component("service")
            .add_component("repository");

        let mut context = ErrorContext::new("controller", "process_request");
        context.add_context("user_id", "123");
        tracker.add_context(context);

        let verifier = ErrorVerifier::new()
            .expect_message("not found")
            .expect_component("service")
            .expect_context("user_id");

        let error = "User not found".to_string();
        let result = verifier.verify(&error, &tracker);
        assert!(result.is_ok());
    }
}
