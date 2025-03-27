# User-Facing Services

This directory contains user-facing services that can be extended or customized by application developers.

## Purpose

The `app/services` directory is intended for:

- Custom service implementations
- Extending core service functionality
- Creating application-specific business logic

## Implementation Guidelines

When adding new services:

1. Create new files for logically grouped functionality
2. Follow the dependency injection pattern established in core services
3. Use proper error handling with the core error types
4. Write unit tests for your service implementations

## Example

```rust
// src/app/services/notification.rs
use async_trait::async_trait;
use crate::core::error::Result;
use crate::models::Notification;

#[async_trait]
pub trait NotificationService: Send + Sync {
    async fn send_notification(&self, notification: Notification) -> Result<()>;
}

pub struct EmailNotificationService {
    // Dependencies
}

impl EmailNotificationService {
    pub fn new(/* dependencies */) -> Self {
        Self {
            // Initialize
        }
    }
}

#[async_trait]
impl NotificationService for EmailNotificationService {
    async fn send_notification(&self, notification: Notification) -> Result<()> {
        // Implementation
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_send_notification() {
        // Test implementation
    }
}
```

Then use this service in your API handlers or other parts of the application. 

# Example Services

This directory contains example implementations of services that use the repository pattern in the Navius framework.

## File Naming Convention

Files in this directory use the `example_` prefix to indicate that they are reference implementations
that can be removed in production environments. A script can be created to remove all example code
by filtering for the `example_` prefix.

## Available Examples

- `example_user_service.rs`: Example service implementation for managing user entities using the repository pattern.

## Usage Guidelines

These examples demonstrate best practices for implementing services in the Navius framework:

1. Use repositories for data access
2. Implement the `Service` and `Lifecycle` traits from core
3. Create DTOs (Data Transfer Objects) for input and output
4. Implement proper business logic validation
5. Use appropriate error handling
6. Include comprehensive logging
7. Implement health checks

## Creating Your Own Services

When creating your own service implementations, follow these steps:

1. Create a new file for your service (without the `example_` prefix)
2. Create appropriate DTOs for input and output
3. Implement the `Service` and `Lifecycle` traits
4. Use repositories for data access
5. Add business logic and validation
6. Implement proper error handling
7. Write comprehensive tests

## Service Design Principles

Services should follow these design principles:

1. Single Responsibility: Each service should focus on a specific domain entity or business function
2. Separation of Concerns: Keep business logic in services and data access in repositories
3. Immutability: Use immutable DTOs for input and output
4. Validation: Validate all inputs before processing
5. Error Handling: Use descriptive error types and provide helpful error messages
6. Testability: Design services to be easily testable with mock repositories

## Service Lifecycle

Services implement the `Lifecycle` trait with these methods:

- `init()`: Initialize resources and dependencies
- `shutdown()`: Clean up resources and dependencies
- `health_check()`: Verify service health and dependencies 