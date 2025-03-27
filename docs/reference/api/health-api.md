---
title: "Health API Reference"
description: "API documentation for Navius health monitoring endpoints"
category: api
tags:
  - api
  - health
  - monitoring
  - actuator
related:
  - reference/patterns/health-check-pattern.md
  - examples/health-service-example.md
  - reference/api/api-resource.md
last_updated: March 26, 2024
version: 1.0
---

# Health API Reference

## Overview

The Health API provides endpoints for monitoring the operational status of the application and its dependencies. It offers different levels of detail for different consumers, from simple UP/DOWN status for load balancers to detailed component status for administrators.

## Base URL

All health endpoints are accessible under the `/actuator` path prefix.

## Authentication

- Basic health status (`/actuator/health`) is publicly accessible by default
- Detailed health information (`/actuator/health/detail`) requires authentication with ADMIN role
- Health dashboard (`/actuator/dashboard`) requires authentication with ADMIN role

Authentication requirements can be configured in `config/default.yaml` under the `api.security.endpoints` section.

## Endpoints

### Health Status

```http
GET /actuator/health
```

Returns the overall health status of the application.

#### Response Format

```json
{
  "status": "UP",
  "timestamp": "2024-03-26T12:34:56.789Z"
}
```

#### Status Values

- `UP`: The application is functioning normally
- `DOWN`: The application is not functioning
- `DEGRADED`: The application is functioning with reduced capabilities
- `UNKNOWN`: The application status cannot be determined

#### Response Codes

- `200 OK`: The application is UP or DEGRADED
- `503 Service Unavailable`: The application is DOWN
- `500 Internal Server Error`: An error occurred checking health

### Detailed Health Status

```http
GET /actuator/health/detail
```

Returns detailed health information for all components.

#### Response Format

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
        "version": "14.5",
        "connection_pool": "10/20",
        "response_time_ms": 5
      }
    },
    {
      "name": "redis-cache",
      "status": "UP",
      "details": {
        "used_memory": "1.2GB",
        "uptime": "3d",
        "clients_connected": 5
      }
    },
    {
      "name": "disk-space",
      "status": "UP",
      "details": {
        "total": "100GB",
        "free": "75GB",
        "threshold": "10GB"
      }
    },
    {
      "name": "external-api",
      "status": "DOWN",
      "details": {
        "error": "Connection timeout",
        "url": "https://api.example.com/status",
        "last_successful_check": "2024-03-26T10:15:30.000Z"
      }
    }
  ]
}
```

#### Response Codes

- `200 OK`: Health information retrieved successfully
- `401 Unauthorized`: Authentication required
- `403 Forbidden`: Insufficient permissions
- `500 Internal Server Error`: An error occurred checking health

### Component Health

```http
GET /actuator/health/component/{component-name}
```

Returns detailed health information for a specific component.

#### Parameters

- `component-name`: Name of the component to check

#### Response Format

```json
{
  "name": "database",
  "status": "UP",
  "timestamp": "2024-03-26T12:34:56.789Z",
  "details": {
    "type": "postgres",
    "version": "14.5",
    "connection_pool": "10/20",
    "response_time_ms": 5
  }
}
```

#### Response Codes

- `200 OK`: Component health retrieved successfully
- `401 Unauthorized`: Authentication required
- `403 Forbidden`: Insufficient permissions
- `404 Not Found`: Component not found
- `500 Internal Server Error`: An error occurred checking health

### Health Dashboard

```http
GET /actuator/dashboard
```

Returns health history and trend data.

#### Query Parameters

- `duration`: Time period to show history for (default: `1h`, options: `5m`, `15m`, `1h`, `6h`, `1d`, `7d`)
- `component`: Optional component name to filter by

#### Response Format

```json
{
  "current_status": "UP",
  "timestamp": "2024-03-26T12:34:56.789Z",
  "uptime_percentage": 99.8,
  "last_outage": "2024-03-25T08:15:20.000Z",
  "history": [
    {
      "timestamp": "2024-03-26T12:30:00.000Z",
      "status": "UP",
      "components": {
        "database": "UP",
        "redis-cache": "UP",
        "disk-space": "UP",
        "external-api": "DOWN"
      }
    },
    {
      "timestamp": "2024-03-26T12:25:00.000Z",
      "status": "UP",
      "components": {
        "database": "UP",
        "redis-cache": "UP",
        "disk-space": "UP",
        "external-api": "DOWN"
      }
    }
    // Additional history entries...
  ],
  "components": [
    {
      "name": "database",
      "current_status": "UP",
      "uptime_percentage": 100.0,
      "last_outage": null
    },
    {
      "name": "redis-cache",
      "current_status": "UP",
      "uptime_percentage": 100.0,
      "last_outage": null
    },
    {
      "name": "disk-space",
      "current_status": "UP",
      "uptime_percentage": 100.0,
      "last_outage": null
    },
    {
      "name": "external-api",
      "current_status": "DOWN",
      "uptime_percentage": 82.5,
      "last_outage": "2024-03-26T10:15:00.000Z"
    }
  ]
}
```

#### Response Codes

- `200 OK`: Dashboard data retrieved successfully
- `400 Bad Request`: Invalid parameters
- `401 Unauthorized`: Authentication required
- `403 Forbidden`: Insufficient permissions
- `500 Internal Server Error`: An error occurred retrieving dashboard data

### Clear Dashboard History

```http
POST /actuator/dashboard/clear
```

Clears the health dashboard history.

#### Response Format

```json
{
  "message": "Health history cleared successfully",
  "timestamp": "2024-03-26T12:34:56.789Z"
}
```

#### Response Codes

- `200 OK`: History cleared successfully
- `401 Unauthorized`: Authentication required
- `403 Forbidden`: Insufficient permissions
- `500 Internal Server Error`: An error occurred clearing history

## Data Models

### Health Status

| Field     | Type   | Description                           |
|-----------|--------|---------------------------------------|
| status    | string | Overall health status (UP, DOWN, etc.)|
| timestamp | string | ISO-8601 timestamp of the health check|

### Component Health

| Field     | Type   | Description                                |
|-----------|--------|--------------------------------------------|
| name      | string | Name of the component                      |
| status    | string | Component health status (UP, DOWN, etc.)   |
| details   | object | Component-specific health details          |

### Health History Entry

| Field      | Type   | Description                             |
|------------|--------|-----------------------------------------|
| timestamp  | string | ISO-8601 timestamp of the history entry |
| status     | string | Overall health status at that time      |
| components | object | Status of each component at that time   |

## Error Responses

### Standard Error Format

```json
{
  "error": {
    "code": "HEALTH_CHECK_FAILED",
    "message": "Failed to check health of component: database",
    "details": {
      "component": "database",
      "reason": "Connection timeout"
    }
  }
}
```

### Common Error Codes

| Code                  | Description                                      |
|-----------------------|--------------------------------------------------|
| HEALTH_CHECK_FAILED   | Failed to check health of one or more components |
| COMPONENT_NOT_FOUND   | Requested component does not exist               |
| INVALID_PARAMETER     | Invalid parameter provided                       |
| INSUFFICIENT_PERMISSIONS | User does not have required permissions       |

## Usage Examples

### Check Basic Health Status

```shell
curl -X GET http://localhost:3000/actuator/health
```

### Check Detailed Health Status (Authenticated)

```shell
curl -X GET http://localhost:3000/actuator/health/detail \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

### Check Specific Component Health

```shell
curl -X GET http://localhost:3000/actuator/health/component/database \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

### Get Health Dashboard for Last Hour

```shell
curl -X GET http://localhost:3000/actuator/dashboard?duration=1h \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

### Clear Dashboard History

```shell
curl -X POST http://localhost:3000/actuator/dashboard/clear \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

## Configuration

The Health API can be configured in `config/default.yaml`:

```yaml
# Health monitoring configuration
health:
  # Enable/disable health monitoring
  enabled: true
  
  # Check interval in seconds
  check_interval_seconds: 60
  
  # Maximum history entries to keep
  max_history_size: 1000
  
  # Controls which components are enabled
  components:
    database: true
    redis: true
    disk_space: true
    external_apis: true
    
  # Security settings
  security:
    # Whether detailed health info requires authentication
    require_auth_for_detail: true
    # Required role for detailed health info
    detail_role: "ADMIN"
```

## Implementing Custom Health Indicators

You can implement custom health indicators by implementing the `HealthIndicator` trait:

```rust
use crate::core::services::health::{HealthIndicator, DependencyStatus};
use std::sync::Arc;
use std::collections::HashMap;
use crate::core::router::AppState;

pub struct CustomHealthIndicator {
    // Your custom fields
}

impl HealthIndicator for CustomHealthIndicator {
    fn name(&self) -> String {
        "custom-component".to_string()
    }
    
    fn check_health(&self, _state: &Arc<AppState>) -> DependencyStatus {
        // Implement your health check logic
        if check_passes() {
            DependencyStatus::up()
                .with_detail("version", "1.0")
                .with_detail("custom_metric", "value")
        } else {
            DependencyStatus::down()
                .with_detail("error", "Check failed")
        }
    }
    
    fn is_critical(&self) -> bool {
        false  // Set to true if this component is critical
    }
}
```

Register your custom indicator with the health service:

```rust
let mut health_service = service_registry.get_mut::<HealthService>();
health_service.register_indicator(Box::new(CustomHealthIndicator::new()));
```

## Related Documentation

- [Health Check Pattern](../patterns/health-check-pattern.md)
- [Health Service Example](../../examples/health-service-example.md)
- [API Resource Pattern](../patterns/api-resource-pattern.md) 