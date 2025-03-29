---
title: "Logging Service Example"
description: ""
category: "Documentation"
tags: []
last_updated: "March 28, 2025"
version: "1.0"
---

# Logging Service Generalization

This document explains how to use the generic logging interface in the Navius application. The logging service has been redesigned to use a provider-based approach with pluggable implementations.

## Core Concepts

### LoggingOperations

The `LoggingOperations` trait defines the core interface for all loggers:

```
pub trait LoggingOperations: Send + Sync + 'static {
    // Log a message at a specific level
    fn log(&self, level: LogLevel, info: LogInfo) -> Result<(), LoggingError>;
    
    // Convenience methods for different log levels
    fn trace(&self, info: LogInfo) -> Result<(), LoggingError>;
    fn debug(&self, info: LogInfo) -> Result<(), LoggingError>;
    fn info(&self, info: LogInfo) -> Result<(), LoggingError>;
    fn warn(&self, info: LogInfo) -> Result<(), LoggingError>;
    fn error(&self, info: LogInfo) -> Result<(), LoggingError>;
    
    // Log a structured record directly
    fn log_structured(&self, record: StructuredLog) -> Result<(), LoggingError>;
    
    // Set global context for all logs
    fn with_global_context(&self, key: &str, value: &str) -> Result<(), LoggingError>;
    
    // Control log levels
    fn set_level(&self, level: LogLevel) -> Result<(), LoggingError>;
    fn get_level(&self) -> LogLevel;
    
    // Flush any buffered logs
    async fn flush(&self) -> Result<(), LoggingError>;
    
    // Create a child logger with additional context
    fn child(&self, context: &str) -> Arc<dyn LoggingOperations>;
}
```

### LoggingProvider

The `LoggingProvider` trait allows different logging implementations to be created:

```
pub trait LoggingProvider: Send + Sync + 'static {
    // Create a new logger instance
    async fn create_logger(&self, config: &LoggingConfig) -> Result<Arc<dyn LoggingOperations>, LoggingError>;
    
    // Get provider name
    fn name(&self) -> &'static str;
    
    // Check if this provider supports the given configuration
    fn supports(&self, config: &LoggingConfig) -> bool;
}
```

## Using the Logging Service

### Initialization

Initialize the logging service using the factory method:

```
use navius::core::logger::{init, LoggingConfig};

async fn setup_logging() -> Result<Arc<dyn LoggingOperations>, LoggingError> {
    // Create a default configuration
    let config = LoggingConfig::default();
    
    // Initialize the logging system
    let logger = init(&config).await?;
    
    // Return the logger instance
    Ok(logger)
}
```

### Basic Logging

Log messages at different levels:

```
use navius::core::logger::{LogInfo, LogLevel};

// Log an info message
logger.info(LogInfo::new("Application started")).unwrap();

// Log a warning with context
logger.warn(
    LogInfo::new("Resource limit approaching")
        .with_context("memory-service")
        .with_field("current_usage", "85%")
).unwrap();

// Log an error with request tracking
logger.error(
    LogInfo::new("Authentication failed")
        .with_request_id("req-123456")
        .with_user_id("user@example.com")
).unwrap();
```

### Structured Logging

Create structured logs for consistent formatting:

```
use navius::core::logger::{LogInfo, LogLevel, StructuredLog};
use std::collections::HashMap;

// Create structured log fields
let mut fields = HashMap::new();
fields.insert("operation".to_string(), "user-create".to_string());
fields.insert("duration_ms".to_string(), "42".to_string());

// Create log info
let log_info = LogInfo {
    message: "Operation completed".to_string(),
    context: Some("user-service".to_string()),
    module: Some("api".to_string()),
    request_id: Some("req-123".to_string()),
    user_id: None,
    timestamp: Some(chrono::Utc::now()),
    additional_fields: fields,
};

// Convert to structured log
let structured_log = StructuredLog::from((LogLevel::Info, log_info));

// Log the structured record
logger.log_structured(structured_log).unwrap();
```

### Advanced: Creating Child Loggers

Child loggers inherit settings but add additional context:

```
// Create a logger for a specific subsystem
let auth_logger = logger.child("auth-service");

// All logs from this logger will include the auth-service context
auth_logger.info(LogInfo::new("User login successful")).unwrap();

// Create nested child loggers
let oauth_logger = auth_logger.child("oauth");
oauth_logger.debug(LogInfo::new("Token validation")).unwrap();
```

## Implementing a Custom Logger

To create a custom logger implementation:

1. Create a struct that implements `LoggingOperations`
2. Create a provider that implements `LoggingProvider`
3. Register your provider with the registry

```
use navius::core::logger::{
    LoggingOperations, LoggingProvider, LoggingProviderRegistry,
    LogInfo, LogLevel, StructuredLog, LoggingConfig, LoggingError
};
use std::sync::Arc;
use async_trait::async_trait;

// Example custom logger implementation
struct CustomLogger;

impl LoggingOperations for CustomLogger {
    fn log(&self, level: LogLevel, info: LogInfo) -> Result<(), LoggingError> {
        // Implement your custom logging logic here
        println!("[{}] {}", level, info.message);
        Ok(())
    }
    
    // Implement other required methods...
}

// Custom provider implementation
struct CustomLoggerProvider;

#[async_trait]
impl LoggingProvider for CustomLoggerProvider {
    async fn create_logger(&self, _config: &LoggingConfig) -> Result<Arc<dyn LoggingOperations>, LoggingError> {
        Ok(Arc::new(CustomLogger))
    }
    
    fn name(&self) -> &'static str {
        "custom"
    }
    
    fn supports(&self, config: &LoggingConfig) -> bool {
        config.logger_type == "custom"
    }
}

// Register your provider
async fn register_custom_provider() -> Result<Arc<dyn LoggingOperations>, LoggingError> {
    let registry = Arc::new(LoggingProviderRegistry::new());
    registry.register_provider(Arc::new(CustomLoggerProvider))?;
    
    let config = LoggingConfig {
        logger_type: "custom".to_string(),
        ..Default::default()
    };
    
    registry.create_logger_from_config(&config).await
}
```

## Built-in Logger Implementations

### Tracing Logger

The default implementation is based on the `tracing` crate:

```
// Create a tracing-based logger
let config = LoggingConfig {
    logger_type: "tracing".to_string(),
    level: "debug".to_string(),
    format: "json".to_string(),
    ..Default::default()
};

let logger = init(&config).await.unwrap();
```

### Console Logger

A colorized console logger is also provided:

```
// Create a console logger with colors
let config = LoggingConfig {
    logger_type: "console".to_string(),
    level: "info".to_string(),
    colorize: true,
    ..Default::default()
};

let logger = init(&config).await.unwrap();
```

## Configuring Logging

The `LoggingConfig` struct controls logger behavior:

```
let config = LoggingConfig {
    // Logger implementation to use
    logger_type: "tracing".to_string(),
    
    // Minimum log level
    level: "info".to_string(),
    
    // Output format
    format: "json".to_string(),
    
    // Enable colorized output (for console loggers)
    colorize: true,
    
    // Include file path and line number
    include_file_info: true,
    
    // Global fields to add to all logs
    global_fields: {
        let mut fields = HashMap::new();
        fields.insert("app_name".to_string(), "navius".to_string());
        fields.insert("environment".to_string(), "development".to_string());
        fields
    },
    
    ..Default::default()
};
```

## Best Practices

1. **Use structured logging**: Prefer structured logs over raw strings for better searchability
2. **Include context**: Always add relevant context to logs
3. **Use child loggers**: Create child loggers for subsystems to maintain context
4. **Set appropriate log levels**: Use debug/trace for development and info/warn for production
5. **Add request IDs**: Include request IDs in logs for distributed request tracing 
