---
title: "Health Service Example"
description: "Examples of using the generic health service with custom indicators and providers"
category: examples
tags:
  - health
  - service
  - monitoring
  - indicators
  - health-check
related:
  - roadmaps/25-generic-service-implementations.md
  - reference/patterns/health-check-pattern.md
  - reference/api/health-api.md
last_updated: March 26, 2024
version: 1.0
---

# Health Service Example

This example demonstrates how to use the generic health service implementation, including defining custom health indicators, registering providers, and building health dashboards.

## Overview

The Health Service implementation follows a provider-based architecture that enables:

- Dynamic health indicators that can be registered at runtime
- Pluggable health providers for different subsystems
- Automatic discovery of health indicators
- Detailed health reporting with component status history
- Customizable health check aggregation

## Core Components

The health service consists of several key components:

1. **HealthIndicator Trait**: Defines interface for individual component health checks
2. **HealthProvider Trait**: Defines interface for providers that create health indicators
3. **HealthDiscoveryService**: Discovers and registers health indicators dynamically
4. **HealthService**: Manages health checks and aggregates results
5. **HealthDashboard**: Tracks health history and provides detailed reporting

## Basic Usage

### Accessing the Health Service

The health service is accessible through the application's service registry:

```rust
use crate::core::services::ServiceRegistry;
use crate::core::services::health::HealthService;

// Get the service from service registry
let health_service = service_registry.get::<HealthService>();

// Get the health status
let health_status = health_service.check_health().await?;
println!("System health: {}", health_status.status);
```

### Implementing a Custom Health Indicator

Create a custom health indicator by implementing the `HealthIndicator` trait:

```rust
use crate::core::services::health::{HealthIndicator, DependencyStatus};
use std::sync::Arc;
use crate::core::router::AppState;

pub struct DatabaseHealthIndicator {
    db_connection_string: String,
}

impl DatabaseHealthIndicator {
    pub fn new(connection_string: &str) -> Self {
        Self {
            db_connection_string: connection_string.to_string(),
        }
    }
}

impl HealthIndicator for DatabaseHealthIndicator {
    fn name(&self) -> String {
        "database".to_string()
    }
    
    fn check_health(&self, _state: &Arc<AppState>) -> DependencyStatus {
        // Check database connection
        if let Ok(_) = check_database_connection(&self.db_connection_string) {
            DependencyStatus::up()
        } else {
            DependencyStatus::down()
                .with_detail("reason", "Could not connect to database")
                .with_detail("connection", &self.db_connection_string)
        }
    }
    
    // Optional: provide metadata about this indicator
    fn metadata(&self) -> std::collections::HashMap<String, String> {
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("type".to_string(), "database".to_string());
        metadata.insert("version".to_string(), "1.0".to_string());
        metadata
    }
    
    // Optional: set indicator priority (lower runs first)
    fn order(&self) -> i32 {
        10
    }
    
    // Optional: mark as critical (failure means system is down)
    fn is_critical(&self) -> bool {
        true
    }
}

// Helper function to check database connection
fn check_database_connection(connection_string: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Actual implementation would connect to database
    Ok(())
}
```

### Creating a Health Provider

Create a provider that generates health indicators:

```rust
use crate::core::services::health::{HealthIndicator, HealthProvider};
use crate::core::config::AppConfig;

pub struct InfrastructureHealthProvider;

impl HealthProvider for InfrastructureHealthProvider {
    fn create_indicators(&self) -> Vec<Box<dyn HealthIndicator>> {
        let mut indicators = Vec::new();
        
        // Add database health indicator
        indicators.push(Box::new(DatabaseHealthIndicator::new(
            "postgres://localhost/app"
        )));
        
        // Add disk space indicator
        indicators.push(Box::new(DiskSpaceHealthIndicator::new("/data")));
        
        // Add other infrastructure indicators
        indicators.push(Box::new(MemoryHealthIndicator::new(90)));
        
        indicators
    }
    
    fn is_enabled(&self, config: &AppConfig) -> bool {
        // Check if this provider should be enabled
        config.get_bool("health.infrastructure_checks_enabled").unwrap_or(true)
    }
}
```

### Registering Health Indicators and Providers

Register custom health indicators and providers:

```rust
use crate::core::services::health::{HealthService, HealthIndicator, HealthProvider};

// Setup health service with indicators
async fn setup_health_service() -> HealthService {
    // Create a new health service
    let mut health_service = HealthService::new();
    
    // Register individual indicators
    health_service.register_indicator(Box::new(DatabaseHealthIndicator::new(
        "postgres://localhost/app"
    )));
    
    // Register a provider
    health_service.register_provider(Box::new(InfrastructureHealthProvider));
    
    // Initialize service
    health_service.init().await.unwrap();
    
    health_service
}
```

### Using the Health Discovery Service

The Health Discovery Service automatically finds and registers health indicators:

```rust
use crate::core::services::health_discovery::HealthDiscoveryService;
use crate::core::services::health::HealthService;

async fn setup_with_discovery() -> HealthService {
    // Create services
    let mut health_service = HealthService::new();
    let discovery_service = HealthDiscoveryService::new();
    
    // Discover and register health indicators
    let indicators = discovery_service.discover_indicators().await;
    for indicator in indicators {
        health_service.register_indicator(indicator);
    }
    
    // Initialize service
    health_service.init().await.unwrap();
    
    health_service
}
```

## Health Dashboard

The Health Dashboard provides detailed health history:

```rust
use crate::core::services::health_dashboard::HealthDashboard;
use crate::core::services::health::HealthService;
use std::sync::Arc;

async fn setup_dashboard(health_service: Arc<HealthService>) -> HealthDashboard {
    // Create a dashboard with history tracking
    let mut dashboard = HealthDashboard::new()
        .with_history_size(100)  // Keep last 100 status checks
        .with_health_service(health_service);
    
    // Start background monitoring
    dashboard.start_monitoring(std::time::Duration::from_secs(60)).await;
    
    dashboard
}
```

## Complete Example

Here's a complete example showing how to set up and use the health service:

```rust
use crate::core::services::health::{
    HealthService, HealthIndicator, DependencyStatus
};
use crate::core::services::health_dashboard::HealthDashboard;
use std::sync::Arc;
use std::collections::HashMap;

// Define a custom health indicator
struct ApiHealthIndicator {
    api_url: String,
}

impl ApiHealthIndicator {
    fn new(url: &str) -> Self {
        Self { api_url: url.to_string() }
    }
}

impl HealthIndicator for ApiHealthIndicator {
    fn name(&self) -> String {
        "external-api".to_string()
    }
    
    fn check_health(&self, _state: &Arc<AppState>) -> DependencyStatus {
        // In a real implementation, check API availability
        if self.api_url.starts_with("https") {
            DependencyStatus::up()
        } else {
            DependencyStatus::down()
                .with_detail("error", "Insecure URL")
                .with_detail("url", &self.api_url)
        }
    }
    
    fn metadata(&self) -> HashMap<String, String> {
        let mut metadata = HashMap::new();
        metadata.insert("type".to_string(), "external-api".to_string());
        metadata
    }
}

async fn setup_health_system() {
    // Create a health service
    let mut health_service = HealthService::new();
    
    // Register health indicators
    health_service.register_indicator(Box::new(ApiHealthIndicator::new(
        "https://api.example.com/status"
    )));
    
    // Initialize the service
    health_service.init().await.unwrap();
    let health_service = Arc::new(health_service);
    
    // Create health dashboard
    let dashboard = HealthDashboard::new()
        .with_health_service(Arc::clone(&health_service))
        .with_history_size(50);
    
    // Check health
    let health = health_service.check_health().await.unwrap();
    println!("System health: {}", health.status);
    
    // List components
    for component in health.components {
        println!("{}: {}", component.name, component.status);
        for (key, value) in component.details {
            println!("  {}: {}", key, value);
        }
    }
    
    // Get dashboard history
    let history = dashboard.get_component_history("external-api").await;
    println!("API health history: {} entries", history.len());
    
    // Clear dashboard history
    dashboard.clear_history().await;
}
```

## Health API Endpoints

The health service automatically exposes API endpoints:

- `/actuator/health` - Basic health check (UP/DOWN)
- `/actuator/health/detail` - Detailed health information
- `/actuator/dashboard` - Health dashboard with history

Example response:

```json
{
  "status": "UP",
  "timestamp": "2025-03-26T12:34:56.789Z",
  "components": [
    {
      "name": "database",
      "status": "UP",
      "details": {
        "type": "postgres",
        "version": "14.5"
      }
    },
    {
      "name": "external-api",
      "status": "DOWN",
      "details": {
        "error": "Connection timeout",
        "url": "https://api.example.com/status"
      }
    }
  ]
}
```

## Best Practices

1. **Critical Components**: Mark critical health indicators that should fail the entire system
2. **Dependency Order**: Set the order of health checks to check dependencies first
3. **Metadata**: Include useful metadata in health indicators
4. **Dashboard History**: Configure appropriate history size based on monitoring needs
5. **Performance**: Ensure health checks are lightweight and don't impact system performance
6. **Security**: Don't expose sensitive information in health details
7. **Timeouts**: Set appropriate timeouts for health checks
8. **Discovery**: Use the discovery service to automatically find health indicators

## Related Documentation

- [Generic Service Implementations Roadmap](../roadmaps/25-generic-service-implementations.md)
- [Health API Reference](../reference/api/health-api.md)
- [Health Check Pattern](../reference/patterns/health-check-pattern.md) 