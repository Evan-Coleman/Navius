---
title: Navius Quickstart Guide
description: "A rapid introduction to get you up and running with Navius in minutes"
category: getting-started
tags:
  - quickstart
  - setup
  - installation
  - tutorial
  - beginners
related:
  - installation.md
  - development-setup.md
  - first-steps.md
  - hello-world.md
last_updated: April 8, 2025
version: 1.0
---

# Navius Quickstart Guide

This quickstart guide will help you build and run your first Navius application in just a few minutes. For more detailed information, see our comprehensive [installation guide](./installation.md) and [development setup](./development-setup.md).

## Prerequisites

Before you begin, ensure you have:

- [Rust](https://www.rust-lang.org/tools/install) (version 1.70 or later)
- [Git](https://git-scm.com/downloads)
- [Docker](https://www.docker.com/get-started) (optional, for database services)
- A code editor or IDE (VS Code or JetBrains CLion recommended)

Verify your Rust installation:

```bash
rustc --version
# Should show rustc 1.70.0 or later
```

## Step 1: Clone the Navius Template

The fastest way to get started is using our template project:

```bash
git clone https://github.com/navius/navius-template.git my-navius-app
cd my-navius-app
```

## Step 2: Launch the Development Environment

Start the development environment, which includes PostgreSQL and Redis:

```bash
# Start required services
docker-compose up -d

# Verify services are running
docker-compose ps
```

## Step 3: Configure Your Environment

The template includes a sample configuration file. Create a development environment file:

```bash
cp .env.example .env.development
```

Open `.env.development` and update the settings as needed:

```
DATABASE_URL=postgres://navius:navius@localhost:5432/navius_dev
REDIS_URL=redis://localhost:6379/0
LOG_LEVEL=debug
SERVER_PORT=3000
```

## Step 4: Build and Run

Build and run your Navius application:

```bash
# Build the application
cargo build

# Run in development mode
cargo run
```

You should see output similar to:

```
[INFO] Navius Framework v0.8.1
[INFO] Loading configuration from .env.development
[INFO] Initializing database connection
[INFO] Starting Navius server on http://127.0.0.1:3000
[INFO] Server started successfully. Press Ctrl+C to stop.
```

## Step 5: Explore Your Application

Your application is now running! Open a web browser and navigate to:

- API: http://localhost:3000/api
- API Documentation: http://localhost:3000/api/docs
- Health Check: http://localhost:3000/health

## Step 6: Make Your First Change

Let's modify the application to add a custom endpoint:

1. Open `src/routes/mod.rs` and add a new route:

```rust
// ... existing code ...

pub fn configure_routes(app: &mut ServiceBuilder) -> &mut ServiceBuilder {
    app
        .route("/", get(handlers::index))
        .route("/hello", get(hello_world)) // Add this line
        .route("/health", get(handlers::health_check))
        // ... other routes ...
}

// Add this function
async fn hello_world() -> impl IntoResponse {
    Json(json!({
        "message": "Hello from Navius!",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}
```

2. Save the file and restart the server:

```bash
# Stop the running server with Ctrl+C, then run again
cargo run
```

3. Visit your new endpoint at http://localhost:3000/hello

## Step 7: Next Steps

Congratulations! You've successfully:
- Set up a Navius development environment
- Run your first Navius application
- Added a custom endpoint

### What to Try Next

- Create a more complex [REST API](../04_guides/api/rest-api-development.md)
- Learn about [dependency injection](../04_guides/core/dependency-injection.md)
- Explore [database integration](../04_guides/data/postgresql-integration.md)
- Check out the [hello world tutorial](./hello-world.md) for a step-by-step project

## Common Issues

### Could not connect to database

**Problem**: The server fails to start with database connection errors.  
**Solution**: Ensure Docker is running and containers are up with `docker-compose ps`. Verify database credentials in `.env.development`.

### Port already in use

**Problem**: The server fails to start because port 3000 is already in use.  
**Solution**: Change the `SERVER_PORT` in `.env.development` or stop the other application using port 3000.

### Cargo build fails

**Problem**: The build process fails with dependency or compilation errors.  
**Solution**: Ensure you're using Rust 1.70+ with `rustc --version`. Run `cargo update` to update dependencies.

## Getting Help

If you encounter any issues:

- Check the [troubleshooting guide](../07_troubleshooting/common-issues.md)
- Visit our [community forum](https://forum.navius.dev)
- Join the [Discord server](https://discord.gg/navius) for real-time help
- Read the detailed documentation in this site

## Additional Resources

- [Development Workflow](../04_guides/development/development-workflow.md)
- [Testing Guide](../04_guides/development/testing-guide.md)
- [Debugging Guide](../04_guides/development/debugging-guide.md)
- [API Documentation](../05_reference/api/README.md)

