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