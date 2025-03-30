use thiserror::Error;

/// Dependency Injection Error
#[derive(Error, Debug)]
pub enum Error {
    /// A component was not found in the registry
    #[error("Component not found: {name}")]
    ComponentNotFound { name: String },

    /// A qualifier was not found in the registry
    #[error("Qualifier not found: {qualifier}")]
    QualifierNotFound { qualifier: String },

    /// A type mismatch occurred between expected and found types
    #[error("Type mismatch: expected {expected}, found {found}")]
    TypeMismatch { expected: String, found: String },

    /// A circular dependency was detected during component initialization
    #[error("Circular dependency detected: {path}")]
    CircularDependency { path: String },

    /// A component lifecycle error occurred
    #[error("Lifecycle error: {message}")]
    LifecycleError { message: String },

    /// An autowiring error occurred
    #[error("Autowiring error: {message}")]
    AutowiringError { message: String },

    /// Multiple matching components were found for an autowired dependency
    #[error("Multiple matching components found for dependency {dependency}")]
    MultipleMatchingComponents { dependency: String },

    /// A component with the same qualifier already exists
    #[error("A component with qualifier '{qualifier}' already exists")]
    DuplicateQualifier { qualifier: String },

    /// An operation timed out
    #[error("Operation timed out: {operation}")]
    Timeout { operation: String },

    /// A scope error occurred
    #[error("Scope error: {message}")]
    ScopeError { message: String },

    /// Any other error
    #[error("Error: {0}")]
    Other(String),
}

impl Error {
    /// Create a new error with a message
    pub fn new(message: &str) -> Self {
        Self::Other(message.to_string())
    }
}

/// Result type for dependency injection operations
pub type Result<T> = std::result::Result<T, Error>;
