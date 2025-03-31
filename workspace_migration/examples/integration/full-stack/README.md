# Full Stack Integration Example

This example demonstrates a complete integration of the Navius framework components, showcasing a task management application with authentication, task CRUD operations, comments, notifications, and more.

## Features

- **Authentication**: JWT-based authentication with registration, login, and token refresh
- **User Management**: Create, update, and retrieve user profiles
- **Task Management**: Create, read, update, delete tasks with support for:
  - Priority levels
  - Status tracking
  - Due dates
  - Categories
  - Assignees
- **Comments**: Add, update, and delete comments on tasks
- **Notifications**: Receive notifications for task assignments and updates
- **Role-Based Access Control**: Different permissions for regular users and managers

## Architecture

The example follows clean architecture principles with:

- **Domain Layer**: Core business entities and rules
- **Application Layer**: Use cases and business logic
- **Infrastructure Layer**: Technical implementations (repositories, services)
- **API Layer**: HTTP interface with controllers and routes

## Getting Started

### Prerequisites

- Rust 1.70 or later
- Cargo

### Running the Example

```bash
cd workspace_migration/examples/integration/full-stack
cargo run
```

The server will start on `http://127.0.0.1:3000` by default.

### API Routes

- **Auth**: `/api/auth/*` - Authentication endpoints
- **Users**: `/api/users/*` - User management
- **Tasks**: `/api/tasks/*` - Task management
- **Categories**: `/api/categories/*` - Category management
- **Notifications**: `/api/notifications/*` - Notification endpoints
- **Health**: `/api/health` - Service health check

### Testing

Run the integration tests to verify the functionality:

```bash
cargo test --test integration_test
```

## Implementation Notes

- Uses in-memory repositories for demonstration purposes
- Implements JWT authentication with role-based middleware
- Showcases proper error handling and consistent response formats
- Demonstrates service registration and dependency injection

## Next Steps

This example can be extended with:

- Database integration (PostgreSQL, MongoDB)
- Redis for caching
- Frontend integration
- Deployment examples 