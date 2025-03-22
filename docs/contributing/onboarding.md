# Developer Onboarding Guide

**Updated At:** March 22, 2025

Welcome to the Navius project! This guide will help you get started as a developer on the project.

## Getting Started

### Prerequisites

Before you begin, ensure you have the following installed:

- Rust (latest stable version)
- Cargo (comes with Rust)
- Git
- Docker and Docker Compose
- VS Code (recommended) or your preferred IDE

### Setting Up Your Development Environment

1. **Clone the repository**:

```bash
git clone https://gitlab.com/navius/navius.git
cd navius
```

2. **Set up environment variables**:

Create a `.env` file in the project root with the following variables:

```
RUST_LOG=debug
CONFIG_DIR=./config
RUN_ENV=development
```

3. **Install IDE extensions**:

For VS Code, set up the recommended extensions:

```bash
mkdir -p .vscode
cp .devtools/ide/vscode/* .vscode/
```

Then restart VS Code and install the recommended extensions when prompted.

4. **Build the project**:

```bash
cargo build
```

This will also generate the API clients from OpenAPI specifications.

5. **Run the tests**:

```bash
cargo test
```

## Project Structure

Navius follows a modular architecture with a clean separation of concerns. See the [Project Navigation Guide](../guides/project-navigation.md) for a detailed explanation of the codebase structure.

Key directories:

- `src/core/` - Core business logic and framework functionality
- `src/app/` - User-extensible application code
- `config/` - Configuration files
- `docs/` - Documentation
- `.devtools/` - Development tools and scripts

## Development Workflow

### Running the Server

To run the development server:

```bash
.devtools/scripts/run_dev.sh
```

### Adding a New Feature

1. **Create a feature branch**:

```bash
git checkout -b feature/your-feature-name
```

2. **Implement the feature**:

- Add routes in `src/app/router.rs`
- Implement handlers in `src/app/api/`
- Add business logic in `src/app/services/`
- Add tests for your feature

3. **Run tests**:

```bash
cargo test
```

4. **Verify code style**:

```bash
cargo clippy
cargo fmt --check
```

5. **Create a merge request**:

Push your changes and create a merge request on GitLab.

### Useful Development Scripts

The project includes several helper scripts in the `.devtools/scripts/` directory:

- `run_dev.sh` - Run the development server
- `regenerate_api.sh` - Regenerate API clients from OpenAPI specs
- `navigate.sh` - Help navigate the codebase
- `verify-structure.sh` - Verify the project structure

Example usage:

```bash
# Find files in the auth component
.devtools/scripts/navigate.sh component auth

# Trace a request flow
.devtools/scripts/navigate.sh flow "GET /users"

# Verify project structure
.devtools/scripts/verify-structure.sh
```

## Debugging

VS Code launch configurations are provided for debugging:

1. Open the "Run and Debug" panel in VS Code
2. Select "Debug Navius Server" to debug the server
3. Set breakpoints in your code
4. Start debugging (F5)

For debugging tests, use the "Debug Unit Tests" configuration.

## Architecture Overview

Navius follows clean architecture principles:

1. **Core Layer** (`src/core/`):
   - Contains the core business logic
   - Independent from external frameworks
   - Defines interfaces for external dependencies

2. **Application Layer** (`src/app/`):
   - User-extensible scaffolding
   - Uses core functionality
   - Provides extension points for customization

3. **Framework Integration**:
   - Uses Axum for web framework
   - SQLx for database access
   - Redis for caching

See the [Module Dependencies Diagram](../architecture/module-dependencies.md) for a visual representation of the architecture.

## Documentation

All features should be documented. The project uses the following documentation structure:

- `docs/guides/` - User guides and tutorials
- `docs/reference/` - API and technical reference
- `docs/architecture/` - Architecture documentation
- `docs/contributing/` - Contribution guidelines
- `docs/roadmaps/` - Development roadmaps

## Getting Help

If you need help with the codebase:

1. Consult the [Project Navigation Guide](../guides/project-navigation.md)
2. Use the navigation scripts to explore the codebase
3. Read the documentation in the `docs/` directory
4. Reach out to the team on the project's communication channels 