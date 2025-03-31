use std::io;
use thiserror::Error;

/// Result type for template operations
pub type TemplateResult<T> = Result<T, TemplateError>;

/// Represents errors that can occur in the template system
#[derive(Debug, Error)]
pub enum TemplateError {
    /// Error parsing a template
    #[error("Failed to parse template: {0}")]
    ParseError(String),

    /// Error registering a template
    #[error("Failed to register template '{0}': {1}")]
    RegistrationError(String, String),

    /// Error rendering a template
    #[error("Failed to render template '{0}': {1}")]
    RenderError(String, String),

    /// Template not found
    #[error("Template '{0}' not found")]
    TemplateNotFound(String),

    /// Template engine not found
    #[error("Template engine '{0}' not found")]
    EngineNotFound(String),

    /// Template context serialization error
    #[error("Failed to serialize template context: {0}")]
    SerializationError(String),

    /// Template engine configuration error
    #[error("Template engine configuration error: {0}")]
    ConfigurationError(String),

    /// Directory operation error
    #[error("Directory operation error: {0}")]
    DirectoryError(String),

    /// Template cache error
    #[error("Template cache error: {0}")]
    CacheError(String),

    /// Template partial resolution error
    #[error("Failed to resolve template partial '{0}': {1}")]
    PartialResolutionError(String, String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),

    /// Error with specific template engine
    #[error("Engine error ({0}): {1}")]
    EngineError(String, String),

    /// Generic error
    #[error("Template error: {0}")]
    Other(String),
}

impl TemplateError {
    /// Creates a new parse error
    pub fn parse_error(message: impl Into<String>) -> Self {
        Self::ParseError(message.into())
    }

    /// Creates a new registration error
    pub fn registration_error(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self::RegistrationError(name.into(), message.into())
    }

    /// Creates a new render error
    pub fn render_error(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self::RenderError(name.into(), message.into())
    }

    /// Creates a new template not found error
    pub fn template_not_found(name: impl Into<String>) -> Self {
        Self::TemplateNotFound(name.into())
    }

    /// Creates a new engine not found error
    pub fn engine_not_found(name: impl Into<String>) -> Self {
        Self::EngineNotFound(name.into())
    }

    /// Creates a new serialization error
    pub fn serialization_error(message: impl Into<String>) -> Self {
        Self::SerializationError(message.into())
    }

    /// Creates a new configuration error
    pub fn configuration_error(message: impl Into<String>) -> Self {
        Self::ConfigurationError(message.into())
    }

    /// Creates a new directory error
    pub fn directory_error(message: impl Into<String>) -> Self {
        Self::DirectoryError(message.into())
    }

    /// Creates a new cache error
    pub fn cache_error(message: impl Into<String>) -> Self {
        Self::CacheError(message.into())
    }

    /// Creates a new partial resolution error
    pub fn partial_resolution_error(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self::PartialResolutionError(name.into(), message.into())
    }

    /// Creates a new engine error
    pub fn engine_error(engine: impl Into<String>, message: impl Into<String>) -> Self {
        Self::EngineError(engine.into(), message.into())
    }

    /// Creates a new generic error
    pub fn other(message: impl Into<String>) -> Self {
        Self::Other(message.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = TemplateError::ParseError("invalid syntax".to_string());
        assert_eq!(err.to_string(), "Failed to parse template: invalid syntax");

        let err = TemplateError::TemplateNotFound("greeting".to_string());
        assert_eq!(err.to_string(), "Template 'greeting' not found");

        let err =
            TemplateError::RenderError("greeting".to_string(), "missing variable".to_string());
        assert_eq!(
            err.to_string(),
            "Failed to render template 'greeting': missing variable"
        );
    }

    #[test]
    fn test_error_creation_methods() {
        let err = TemplateError::parse_error("invalid syntax");
        assert!(matches!(err, TemplateError::ParseError(_)));

        let err = TemplateError::template_not_found("greeting");
        assert!(matches!(err, TemplateError::TemplateNotFound(_)));

        let err = TemplateError::render_error("greeting", "missing variable");
        assert!(matches!(err, TemplateError::RenderError(_, _)));
    }
}
