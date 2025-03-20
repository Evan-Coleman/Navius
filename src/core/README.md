# Core Framework Functionality

This directory contains core functionality of the framework that should not be modified by users. The core functionality includes:

- Essential routes for health checks, API documentation, and system information
- Core middleware for authentication, logging, and observability
- Infrastructure components that power the framework

## Directory Structure

- `router/`: Core routing functionality that should not be modified
- `handlers/`: Essential handlers for framework features

## How to Extend

Rather than modifying the core functionality, users should:

1. Use the `src/app/user_router.rs` to add custom routes and handlers
2. Create custom handlers in `src/handlers/` 
3. Create custom middleware in appropriate directories

The framework is designed so that users can create their own routes and handlers without modifying the core functionality, ensuring stability and maintainability.

## Core Routes

The core framework provides these essential routes:

- Public Routes:
  - `/health` - Basic health check

- Actuator Routes (protected by admin authentication when enabled):
  - `/actuator/health` - Detailed health check
  - `/actuator/info` - System information
  - `/actuator/docs` - API documentation (Swagger UI)
  - `/actuator/docs/{*file}` - API documentation resources

Users should avoid creating routes with the same paths to prevent conflicts. 