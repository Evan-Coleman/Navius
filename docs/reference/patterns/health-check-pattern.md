---
title: "Health Check Pattern"
description: "Design and implementation of the health check pattern with pluggable health indicators"
category: patterns
tags:
  - patterns
  - health
  - monitoring
  - architecture
related:
  - reference/patterns/repository-pattern.md
  - reference/api/health-api.md
  - examples/health-service-example.md
last_updated: March 26, 2025
version: 1.0
---

# Health Check Pattern

## Overview

The Health Check Pattern provides a standardized way to assess the operational status of an application and its dependencies. It enables monitoring systems to detect issues and facilitates automated recovery procedures.

## Problem Statement

Modern applications have numerous dependencies (databases, external services, caches, etc.) that can fail independently. Applications need to:

- Report their own operational status
- Check the status of all dependencies
- Provide detailed diagnostics for troubleshooting
- Support both simple availability checks and detailed health information
- Allow easy extension for new components

## Solution: Health Check Pattern with Pluggable Indicators

The Health Check Pattern in Navius uses a provider-based architecture with these components:

1. **HealthIndicator Trait**: Interface for individual component health checks
2. **HealthProvider Trait**: Interface for components that provide health indicators
3. **HealthDiscoveryService**: Automatically discovers and registers health indicators
4. **HealthService**: Orchestrates health checks and aggregates results
5. **HealthDashboard**: Tracks health history and provides detailed reporting

### Pattern Structure

```
┌─────────────────┐          ┌───────────────────┐
│   HealthService │◄─────────┤HealthIndicator(s) │
└────────┬────────┘          └───────────────────┘
         │                            ▲
         │                            │ implements
         │                   ┌────────┴────────┐
         │                   │Component-specific│
         │                   │HealthIndicators │
         ▼                   └─────────────────┘
┌─────────────────┐
│HealthController │
└─────────────────┘
```

### Implementation

#### 1. Health Indicator Interface

The `HealthIndicator` trait defines the contract for all health checks:

```rust
pub trait HealthIndicator: Send + Sync {
    /// Get the name of this health indicator
    fn name(&self) -> String;
    
    /// Check the health of this component
    fn check_health(&self, state: &Arc<AppState>) -> DependencyStatus;
    
    /// Optional metadata about this indicator
    fn metadata(&self) -> HashMap<String, String> {
        HashMap::new()
    }
    
    /// Order in which this indicator should run (lower values run first)
    fn order(&self) -> i32 {
        0
    }
    
    /// Whether this indicator is critical (system is DOWN if it fails)
    fn is_critical(&self) -> bool {
        false
    }
}
```

#### 2. Health Provider Interface

The `HealthProvider` trait enables components to provide their own health indicators:

```rust
pub trait HealthProvider: Send + Sync {
    /// Create health indicators for the application
    fn create_indicators(&self) -> Vec<Box<dyn HealthIndicator>>;
    
    /// Whether this provider is enabled
    fn is_enabled(&self, config: &AppConfig) -> bool;
}
```

#### 3. Health Service

The `HealthService` aggregates and manages health indicators:

```rust
pub struct HealthService {
    indicators: Vec<Box<dyn HealthIndicator>>,
    providers: Vec<Box<dyn HealthProvider>>,
}

impl HealthService {
    pub fn new() -> Self { /* ... */ }
    
    pub fn register_indicator(&mut self, indicator: Box<dyn HealthIndicator>) { /* ... */ }
    
    pub fn register_provider(&mut self, provider: Box<dyn HealthProvider>) { /* ... */ }
    
    pub async fn check_health(&self) -> Result<HealthStatus, ServiceError> { /* ... */ }
}
```

#### 4. Health Discovery

The `HealthDiscoveryService` automatically discovers health indicators:

```rust
pub struct HealthDiscoveryService;

impl HealthDiscoveryService {
    pub fn new() -> Self { /* ... */ }
    
    pub async fn discover_indicators(&self) -> Vec<Box<dyn HealthIndicator>> { /* ... */ }
}
```

## Benefits

1. **Standardization**: Consistent approach to health monitoring across components
2. **Extensibility**: Easy to add health checks for new components
3. **Automation**: Facilitates automated monitoring and recovery
4. **Detailed Diagnostics**: Provides rich health information for troubleshooting
5. **Dynamic Discovery**: Automatically detects new health indicators
6. **Priority Execution**: Checks dependencies in correct order

## Implementation Considerations

### 1. Defining Health Status

Health status should be simple but descriptive:
- UP: Component is functioning normally
- DOWN: Component is not functioning
- DEGRADED: Component is functioning with reduced capabilities
- UNKNOWN: Component status cannot be determined

### 2. Health Check Categories

Organize health checks into categories:
- **Critical Infrastructure**: Database, cache, file system
- **External Dependencies**: APIs, third-party services
- **Internal Components**: Message queues, background tasks
- **Environment**: Disk space, memory, CPU

### 3. Health Check Response

The health API should support multiple response formats:
- Simple UP/DOWN for load balancers and basic monitoring
- Detailed response with component-specific health for diagnostics
- Historical data for trend analysis

### 4. Security Considerations

Health endpoints contain sensitive information:
- Secure detailed health endpoints with authentication
- Limit information in public health endpoints
- Don't expose connection strings or credentials

## API Endpoints

The health service exposes these standard endpoints:
- `/actuator/health`: Basic health status (UP/DOWN)
- `/actuator/health/detail`: Detailed component health
- `/actuator/dashboard`: Health history dashboard

## Example Implementation

### Basic Health Indicator

```rust
pub struct DatabaseHealthIndicator {
    connection_string: String,
}

impl HealthIndicator for DatabaseHealthIndicator {
    fn name(&self) -> String {
        "database".to_string()
    }
    
    fn check_health(&self, _state: &Arc<AppState>) -> DependencyStatus {
        match check_database_connection(&self.connection_string) {
            Ok(_) => DependencyStatus::up(),
            Err(e) => DependencyStatus::down()
                .with_detail("error", e.to_string())
                .with_detail("connection", &self.connection_string)
        }
    }
    
    fn is_critical(&self) -> bool {
        true
    }
    
    fn order(&self) -> i32 {
        10  // Run early since other components may depend on DB
    }
}
```

### Health Response Format

```json
{
  "status": "UP",
  "timestamp": "2024-03-26T12:34:56.789Z",
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
      "name": "redis-cache",
      "status": "UP",
      "details": {
        "used_memory": "1.2GB",
        "uptime": "3d"
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

## Related Patterns

- **Circuit Breaker Pattern**: Used with health checks to prevent cascading failures
- **Bulkhead Pattern**: Isolates components to prevent system-wide failures
- **Observer Pattern**: Health indicators observe component status
- **Repository Pattern**: Often used with health checks for data access
- **Strategy Pattern**: Different health check strategies can be implemented

## References

- [Health Check API - Spring Boot Actuator](https://docs.spring.io/spring-boot/docs/current/reference/html/actuator.html#actuator.endpoints.health)
- [Health Check Pattern - Cloud Design Patterns](https://docs.microsoft.com/en-us/azure/architecture/patterns/health-endpoint-monitoring)
- [Kubernetes Liveness and Readiness Probes](https://kubernetes.io/docs/tasks/configure-pod-container/configure-liveness-readiness-startup-probes/) 