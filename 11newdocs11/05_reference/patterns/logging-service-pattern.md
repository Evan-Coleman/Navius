# Logging Service Pattern

This document describes the Logging Service Pattern implemented in the Navius framework, which provides a clean abstraction over logging functionality through a provider-based approach.

## Overview

The Logging Service Pattern follows these key principles:

1. **Separation of Interface and Implementation**: Logging operations are defined by a core interface, with multiple implementations provided via the provider pattern.
2. **Pluggable Providers**: Different logging implementations can be swapped in and out based on configuration.
3. **Structured Logging**: All logging is done with structured data rather than raw strings, making logs more searchable and meaningful.
4. **Context Propagation**: Logging context can be inherited across components via child loggers.
5. **Configuration-driven**: Behavior is controlled through configuration rather than code changes.

## Core Components

The pattern consists of the following components:

### LoggingOperations Interface

The `LoggingOperations` trait defines the core functionality exposed to application code:

```rust
pub trait LoggingOperations: Send + Sync + 'static {
    fn log(&self, level: LogLevel, info: LogInfo) -> Result<(), LoggingError>;
    fn log_structured(&self, record: StructuredLog) -> Result<(), LoggingError>;
    // Additional methods...
}
```

### LoggingProvider Interface

The `LoggingProvider` trait defines how logging implementations are created:

```rust
pub trait LoggingProvider: Send + Sync + 'static {
    async fn create_logger(&self, config: &LoggingConfig) -> Result<Arc<dyn LoggingOperations>, LoggingError>;
    fn name(&self) -> &'static str;
    fn supports(&self, config: &LoggingConfig) -> bool;
}
```

### LoggingProviderRegistry

A registry that manages available providers and creates loggers based on configuration:

```rust
pub struct LoggingProviderRegistry {
    providers: Mutex<HashMap<String, Arc<dyn LoggingProvider>>>,
    default_provider_name: Mutex<String>,
}
```

## Implementation Steps

To implement this pattern in your own code:

1. Define the logging operations interface
2. Create a provider interface for instantiating loggers
3. Implement a provider registry for managing available providers
4. Create concrete implementations of the logging operations
5. Implement the provider for each concrete implementation

## Benefits

- **Testability**: Logging can be easily mocked for testing purposes
- **Extensibility**: New logging backends can be added without changing application code
- **Consistency**: All logs follow the same structured format
- **Configuration**: Behavior can be changed through configuration
- **Runtime Selection**: Logging implementation can be selected at runtime

## Example Usage

See the [Logging Service Example](../../examples/logging-service-example.md) for detailed usage examples.

## Related Patterns

- **Provider Pattern**: For creating implementations of an interface
- **Registry Pattern**: For managing and accessing providers
- **Factory Method Pattern**: For creating logger instances
- **Decorator Pattern**: For adding functionality to loggers (e.g., child loggers)
- **Strategy Pattern**: For selecting logging implementation at runtime

## Recommended Implementation

The Navius framework provides a complete implementation of this pattern in the `core::logger` module. This implementation includes:

1. `TracingLoggerProvider`: A provider for tracing-based logging
2. `ConsoleLoggerProvider`: A provider for colorized console output
3. `LoggingProviderRegistry`: A registry for managing providers
4. Factory methods for creating loggers based on configuration

Refer to the framework implementation for a complete reference implementation of this pattern.

## Considerations

- Ensure thread safety when implementing loggers
- Consider performance implications, especially for high-volume logging
- Provide extension points for advanced features like filtering and sampling
- Plan for error handling when logging fails
- Consider how to handle buffering and asynchronous logging 