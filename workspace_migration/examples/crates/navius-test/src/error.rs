use std::error::Error as StdError;
use std::fmt;
use std::result;

/// Result type for test operations
pub type TestResult<T> = Result<T, TestError>;

/// Errors that can occur in the navius-test crate
#[derive(Debug)]
pub enum TestError {
    /// An error occurred while registering a mock object
    RegistryError(String),

    /// An expected mock object was not found
    MockNotFound(String),

    /// The mock object's type does not match the expected type
    MockTypeMismatch(String),

    /// An expectation was not met
    ExpectationNotMet(String),

    /// An error occurred while setting up a test
    SetupError(String),

    /// An error occurred while tearing down a test
    TeardownError(String),

    /// An assertion failed
    AssertionFailed(String),

    /// An error occurred in a test fixture
    FixtureError(String),

    /// An error occurred in a mock object
    MockError(String),

    /// An IO error occurred
    IoError(std::io::Error),

    /// A generic error occurred
    Other(Box<dyn std::error::Error + Send + Sync>),

    /// Create a validation error with a message
    ValidationError(String),

    /// Create an assertion error with a message
    AssertionError(String),

    /// Setup error with message
    ExecutionError(String),

    /// Teardown error with message
    MissingComponent(String),

    /// Missing resource error
    MissingResource(String),

    /// Invalid configuration error
    InvalidConfiguration(String),

    /// Conversion error
    ConversionError(String),

    /// Configuration error
    ConfigurationError(String),

    /// Concurrency error
    ConcurrencyError(String),

    /// Timeout error
    TimeoutError(String),

    /// Mock expectation error
    MockExpectationError(String),

    /// Injected error
    InjectedError(String),

    /// Error propagation error
    ErrorPropagationError(String),

    /// Error context error
    ErrorContextError(String),

    /// Mock not registered error
    MockNotRegistered(String),

    /// IO error
    IoError(std::io::Error),
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
    pub fn mock_not_registered<S: Into<String>>(message: S) -> Self {
        TestError::MockNotRegistered(message.into())
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

    /// Create a validation error with a message
    pub fn validation_error<S: Into<String>>(message: S) -> Self {
        Self::ValidationError(message.into())
    }

    /// Create an assertion error with a message
    pub fn assertion_error<S: Into<String>>(message: S) -> Self {
        Self::AssertionError(message.into())
    }

    /// Create a conversion error with a message
    pub fn conversion_error<S: Into<String>>(message: S) -> Self {
        Self::ConversionError(message.into())
    }

    /// Create a configuration error with a message
    pub fn configuration_error<S: Into<String>>(message: S) -> Self {
        Self::ConfigurationError(message.into())
    }

    /// Create a concurrency error with a message
    pub fn concurrency_error<S: Into<String>>(message: S) -> Self {
        Self::ConcurrencyError(message.into())
    }

    /// Create a timeout error with a message
    pub fn timeout_error<S: Into<String>>(message: S) -> Self {
        Self::TimeoutError(message.into())
    }

    /// Create an IO error with a message
    pub fn io_error<E: Into<std::io::Error>>(error: E) -> Self {
        TestError::IoError(error.into())
    }
}

impl fmt::Display for TestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TestError::RegistryError(msg) => write!(f, "Registry error: {}", msg),
            TestError::MockNotFound(msg) => write!(f, "Mock not found: {}", msg),
            TestError::MockTypeMismatch(msg) => write!(f, "Mock type mismatch: {}", msg),
            TestError::ExpectationNotMet(msg) => write!(f, "Expectation not met: {}", msg),
            TestError::SetupError(msg) => write!(f, "Setup error: {}", msg),
            TestError::TeardownError(msg) => write!(f, "Teardown error: {}", msg),
            TestError::AssertionFailed(msg) => write!(f, "Assertion failed: {}", msg),
            TestError::FixtureError(msg) => write!(f, "Fixture error: {}", msg),
            TestError::MockError(msg) => write!(f, "Mock error: {}", msg),
            TestError::IoError(err) => write!(f, "IO error: {}", err),
            TestError::Other(err) => write!(f, "Error: {}", err),
            TestError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            TestError::AssertionError(msg) => write!(f, "Assertion error: {}", msg),
            TestError::ExecutionError(msg) => write!(f, "Execution error: {}", msg),
            TestError::MissingComponent(msg) => write!(f, "Missing component error: {}", msg),
            TestError::MissingResource(msg) => write!(f, "Missing resource error: {}", msg),
            TestError::InvalidConfiguration(msg) => {
                write!(f, "Invalid configuration error: {}", msg)
            }
            TestError::ConversionError(msg) => write!(f, "Conversion error: {}", msg),
            TestError::ConfigurationError(msg) => write!(f, "Configuration error: {}", msg),
            TestError::ConcurrencyError(msg) => write!(f, "Concurrency error: {}", msg),
            TestError::TimeoutError(msg) => write!(f, "Timeout error: {}", msg),
            TestError::MockExpectationError(msg) => write!(f, "Mock expectation error: {}", msg),
            TestError::InjectedError(msg) => write!(f, "Injected error: {}", msg),
            TestError::ErrorPropagationError(msg) => write!(f, "Error propagation error: {}", msg),
            TestError::ErrorContextError(msg) => write!(f, "Error context error: {}", msg),
            TestError::MockNotRegistered(msg) => write!(f, "Mock not registered: {}", msg),
        }
    }
}

impl StdError for TestError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            TestError::IoError(err) => Some(err),
            TestError::Other(err) => Some(err.as_ref()),
            _ => None,
        }
    }
}

impl From<std::io::Error> for TestError {
    fn from(err: std::io::Error) -> Self {
        TestError::IoError(err)
    }
}

impl From<String> for TestError {
    fn from(err: String) -> Self {
        TestError::Other(Box::new(SimpleError(err)))
    }
}

impl From<&str> for TestError {
    fn from(err: &str) -> Self {
        TestError::Other(Box::new(SimpleError(err.to_string())))
    }
}

/// A simple error that can be created from a string
#[derive(Debug)]
struct SimpleError(String);

impl fmt::Display for SimpleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl StdError for SimpleError {}

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

/// Assertion macros
#[macro_export]
macro_rules! assert_mock_call {
    ($mock:expr, $method:expr) => {
        $crate::assert_mock_call_internal($mock, $method, Vec::<String>::new(), file!(), line!())
    };

    ($mock:expr, $method:expr, $($arg:expr),*) => {
        $crate::assert_mock_call_internal(
            $mock,
            $method,
            vec![$($arg.to_string()),*],
            file!(),
            line!(),
        )
    };
}

/// Internal function for assert_mock_call
#[doc(hidden)]
pub fn assert_mock_call_internal<T>(
    _mock: &T,
    method: &str,
    args: Vec<String>,
    file: &str,
    line: u32,
) -> TestResult<()> {
    // This would be implemented to check if the mock was called with the given method and args
    // For now, it's a placeholder
    Ok(())
}

/// Create an error for an unimplemented feature
pub fn unimplemented<S: Into<String>>(feature: S) -> TestError {
    TestError::Other(Box::new(SimpleError(format!(
        "Feature not implemented: {}",
        feature.into()
    ))))
}

/// Create an error for a failed assertion
pub fn assertion_failed<S: Into<String>>(message: S) -> TestError {
    TestError::AssertionFailed(message.into())
}

/// Verify that a condition is true
pub fn assert_true(condition: bool, message: &str) -> TestResult<()> {
    if condition {
        Ok(())
    } else {
        Err(assertion_failed(message))
    }
}

/// Verify that a condition is false
pub fn assert_false(condition: bool, message: &str) -> TestResult<()> {
    if !condition {
        Ok(())
    } else {
        Err(assertion_failed(message))
    }
}

/// Verify that two values are equal
pub fn assert_eq<T: PartialEq + fmt::Debug>(
    actual: T,
    expected: T,
    message: &str,
) -> TestResult<()> {
    if actual == expected {
        Ok(())
    } else {
        Err(assertion_failed(format!(
            "{}: expected {:?}, got {:?}",
            message, expected, actual
        )))
    }
}

/// Verify that two values are not equal
pub fn assert_ne<T: PartialEq + fmt::Debug>(
    actual: T,
    expected: T,
    message: &str,
) -> TestResult<()> {
    if actual != expected {
        Ok(())
    } else {
        Err(assertion_failed(format!(
            "{}: expected not {:?}, got {:?}",
            message, expected, actual
        )))
    }
}

/// Verify that a value is Some and matches the expected value
pub fn assert_some<T: PartialEq + fmt::Debug>(
    actual: Option<T>,
    expected: T,
    message: &str,
) -> TestResult<()> {
    match actual {
        Some(value) if value == expected => Ok(()),
        Some(value) => Err(assertion_failed(format!(
            "{}: expected Some({:?}), got Some({:?})",
            message, expected, value
        ))),
        None => Err(assertion_failed(format!(
            "{}: expected Some({:?}), got None",
            message, expected
        ))),
    }
}

/// Verify that a value is None
pub fn assert_none<T: fmt::Debug>(actual: Option<T>, message: &str) -> TestResult<()> {
    match actual {
        None => Ok(()),
        Some(value) => Err(assertion_failed(format!(
            "{}: expected None, got Some({:?})",
            message, value
        ))),
    }
}

/// Verify that a result is Ok
pub fn assert_ok<T: fmt::Debug, E: fmt::Debug>(
    actual: Result<T, E>,
    message: &str,
) -> TestResult<T> {
    match actual {
        Ok(value) => Ok(value),
        Err(err) => Err(assertion_failed(format!(
            "{}: expected Ok(_), got Err({:?})",
            message, err
        ))),
    }
}

/// Verify that a result is Err
pub fn assert_err<T: fmt::Debug, E: fmt::Debug>(
    actual: Result<T, E>,
    message: &str,
) -> TestResult<E> {
    match actual {
        Ok(value) => Err(assertion_failed(format!(
            "{}: expected Err(_), got Ok({:?})",
            message, value
        ))),
        Err(err) => Ok(err),
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

    #[test]
    fn test_error_display() {
        let err = TestError::RegistryError("test error".to_string());
        assert_eq!(format!("{}", err), "Registry error: test error");

        let err = TestError::MockNotFound("test error".to_string());
        assert_eq!(format!("{}", err), "Mock not found: test error");

        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = TestError::IoError(io_err);
        assert_eq!(format!("{}", err), "IO error: file not found");
    }

    #[test]
    fn test_error_from() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = TestError::from(io_err);
        match err {
            TestError::IoError(_) => {}
            _ => panic!("Expected TestError::IoError"),
        }

        let err = TestError::from("test error");
        match err {
            TestError::Other(_) => {}
            _ => panic!("Expected TestError::Other"),
        }

        let err = TestError::from("test error".to_string());
        match err {
            TestError::Other(_) => {}
            _ => panic!("Expected TestError::Other"),
        }
    }

    #[test]
    fn test_assert_functions() {
        assert!(assert_true(true, "should be true").is_ok());
        assert!(assert_true(false, "should be true").is_err());

        assert!(assert_false(false, "should be false").is_ok());
        assert!(assert_false(true, "should be false").is_err());

        assert!(assert_eq(1, 1, "should be equal").is_ok());
        assert!(assert_eq(1, 2, "should be equal").is_err());

        assert!(assert_ne(1, 2, "should not be equal").is_ok());
        assert!(assert_ne(1, 1, "should not be equal").is_err());

        assert!(assert_some(Some(1), 1, "should be Some(1)").is_ok());
        assert!(assert_some(Some(2), 1, "should be Some(1)").is_err());
        assert!(assert_some(None, 1, "should be Some(1)").is_err());

        assert!(assert_none(None::<i32>, "should be None").is_ok());
        assert!(assert_none(Some(1), "should be None").is_err());

        assert!(assert_ok(Ok::<_, ()>(1), "should be Ok").is_ok());
        assert!(assert_ok(Err::<i32, _>(1), "should be Ok").is_err());

        assert!(assert_err(Err::<(), _>(1), "should be Err").is_ok());
        assert!(assert_err(Ok::<_, i32>(1), "should be Err").is_err());
    }
}
