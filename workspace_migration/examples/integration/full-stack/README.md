# Full Stack Integration Example

This example demonstrates a complete application built with the Navius framework, showcasing how all major components work together. It implements a Task Management System with user authentication, task tracking, notifications, and reporting.

## Features

- **User Management**
  - Registration and authentication
  - Role-based permissions
  - User profiles

- **Task Management**
  - Create, read, update, delete tasks
  - Assign tasks to users
  - Categorize and prioritize tasks

- **Notification System**
  - Real-time notifications
  - Email notifications (simulated)
  - Notification history

- **Reporting**
  - Task statistics
  - User activity metrics
  - Performance dashboard

## Architecture

This example follows a clean architecture approach with:

- **Domain Layer**: Core business entities and logic
- **Application Layer**: Use cases and orchestration
- **Infrastructure Layer**: External systems integration
- **API Layer**: HTTP endpoints and controllers

### Component Integration

The example demonstrates integration between all major Navius components:

- **navius-core**: Application setup, configuration, error handling
- **navius-http**: HTTP server, routing, middleware
- **navius-auth**: Authentication, authorization, role management
- **navius-db/navius-db-postgres**: Database access and transactions
- **navius-cache/navius-cache-redis**: Caching and session management
- **navius-event**: Event dispatching and subscription
- **navius-plugin**: Plugin registration and lifecycle management
- **navius-di**: Dependency injection and component registry

## Getting Started

### Prerequisites

- Rust 1.75 or later
- Docker and Docker Compose
- Cargo 1.75 or later

### Running the Example

1. Start the infrastructure:

```bash
cd workspace_migration/examples/integration/full-stack
docker-compose up -d
```

2. Run database migrations:

```bash
cargo run --bin migrations
```

3. Start the application:

```bash
cargo run
```

4. Access the application:
   - API: http://localhost:8080/api
   - Swagger: http://localhost:8080/swagger
   - Metrics: http://localhost:8080/metrics
   - Grafana Dashboard: http://localhost:3000 (admin/admin)

### Sample API Requests

#### Register a new user:

```bash
curl -X POST http://localhost:8080/api/users \
  -H "Content-Type: application/json" \
  -d '{"username":"testuser","email":"test@example.com","password":"Password123!"}'
```

#### Login:

```bash
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"Password123!"}'
```

#### Create a task:

```bash
curl -X POST http://localhost:8080/api/tasks \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer {token}" \
  -d '{"title":"Test Task","description":"This is a test task","due_date":"2025-04-15T12:00:00Z","priority":"HIGH"}'
```

## Project Structure

```
full-stack/
├── src/
│   ├── main.rs                 # Application entry point
│   ├── config.rs               # Configuration
│   ├── domain/                 # Domain entities and rules
│   │   ├── user.rs             # User entity
│   │   ├── task.rs             # Task entity
│   │   └── notification.rs     # Notification entity
│   ├── application/            # Application services
│   │   ├── user_service.rs     # User management
│   │   ├── task_service.rs     # Task management
│   │   └── notification_service.rs # Notification handling
│   ├── infrastructure/         # External integrations
│   │   ├── repositories/       # Database repositories
│   │   ├── cache/              # Cache implementations
│   │   └── email/              # Email service
│   ├── api/                    # HTTP API
│   │   ├── routes.rs           # Route definitions
│   │   ├── controllers/        # Request handlers
│   │   └── middleware/         # HTTP middleware
│   └── plugins/                # Plugin implementations
│       ├── metrics_plugin.rs   # Metrics collection
│       └── reporting_plugin.rs # Reporting functionality
├── tests/                      # Integration tests
├── examples/                   # Additional examples
├── data/                       # Configuration files for services
│   ├── init.sql                # Database initialization
│   └── prometheus.yml          # Prometheus configuration
├── docs/                       # Documentation
├── Cargo.toml                  # Project manifest
└── docker-compose.yml          # Infrastructure setup
```

## Testing

Run the tests with:

```bash
cargo test
```

## Documentation

- [API Documentation](./docs/api.md)
- [Architecture Overview](./docs/architecture.md)
- [Plugin Development](./docs/plugins.md)

## Performance Monitoring

The example includes Prometheus and Grafana for monitoring:

- **Prometheus**: http://localhost:9090
- **Grafana**: http://localhost:3000

## License

MIT OR Apache-2.0 