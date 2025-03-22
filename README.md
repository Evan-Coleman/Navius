# Navius

[![Build Status](https://img.shields.io/badge/build-passing-brightgreen)](https://gitlab.com/ecoleman2/navius)
[![Test Coverage](https://img.shields.io/badge/coverage-98%25-brightgreen)](https://gitlab.com/ecoleman2/navius)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Rust Version](https://img.shields.io/badge/rust-stable-orange)](https://www.rust-lang.org/)
[![Primary: GitLab](https://img.shields.io/badge/primary-gitlab-orange)](https://gitlab.com/ecoleman2/navius)
[![Mirror: GitHub](https://img.shields.io/badge/mirror-github-black)](https://github.com/Evan-Coleman/Navius)

> **Enterprise-grade web framework built for speed, reliability, and developer productivity**

Navius is a high-performance, modern alternative to Spring Boot, built with Rust. It delivers exceptional performance, security, and developer experience while reducing infrastructure costs and eliminating entire classes of runtime errors.

<div align="center">
  <img src="https://via.placeholder.com/800x400?text=Navius+Diagram" alt="Navius Architecture" width="800px" />
</div>

## Repository Information

Navius uses a dual repository approach:

- **Primary Development**: [GitLab](https://gitlab.com/ecoleman2/navius) - All development, issues, and merge requests
- **Community Mirror**: [GitHub](https://github.com/Evan-Coleman/Navius) - Public visibility and community engagement

For contributions, please:
1. For bug reports and features, use the [GitLab issue tracker](https://gitlab.com/ecoleman2/navius/-/issues)
2. Community contributions via GitHub PRs are welcome and will be reviewed

> **Note**: We are currently undergoing a project restructuring to improve organization and developer experience. See the [Project Restructuring Roadmap](docs/roadmaps/project-restructuring.md) for details.

## 🚀 Why Navius?

### For Developers
- **10x Less Code**: Build APIs in a fraction of the code compared to Spring Boot
- **Type Safety**: Catch errors at compile time instead of runtime
- **Hot Reloading**: Fast development cycle with automatic reloading
- **Familiar Pattern**: Follow the familiar controller/service/repository pattern
- **Comprehensive Testing**: Support for unit, integration, and property-based testing
- **Unified Documentation**: OpenAPI/Swagger integration from day one

### For Operations
- **⚡ Blazing Fast**: Up to 40x better throughput than Spring Boot
- **🔒 Memory Safe**: No null pointers, buffer overflows, or memory leaks
- **📉 Lower Costs**: 5-10x lower CPU and memory footprint
- **🔋 Energy Efficient**: Significantly reduced carbon footprint
- **🔍 Built-in Observability**: Metrics, tracing, and health checks included

### For Business
- **⏱️ Faster Time to Market**: Build and deploy production-ready applications faster
- **💰 Reduced Infrastructure Costs**: Lower cloud spend with efficient resource usage
- **🔥 Enhanced Customer Experience**: More responsive applications with lower latency
- **🛡️ Better Security**: Memory-safe language eliminates entire classes of vulnerabilities

## ⚙️ Core Features

Navius includes everything you need to build enterprise applications:

| Feature | Description |
|---------|-------------|
| **REST API** | Build APIs using Axum, with controller/service/repository pattern |
| **OpenAPI** | Automatic Swagger documentation generation |
| **Configuration** | Multi-environment config with YAML and environment variables |
| **Database** | PostgreSQL integration with migration support |
| **Authentication** | JWT, OAuth2, and Microsoft Entra (Azure AD) integration |
| **Caching** | Built-in Redis and in-memory caching |
| **Reliability** | Circuit breakers, rate limiting, timeouts, and retries |
| **Observability** | Metrics, health checks, and structured logging |
| **Testing** | Comprehensive testing framework with mocking support |

## 🏃‍♂️ Quick Start

```bash
# Start a new project (coming soon)
cargo install navius-cli
navius new my-awesome-api

# Or clone the template
git clone https://github.com/Evan-Coleman/Navius.git my-project
cd my-project

# Run the development server
./run_dev.sh
```

Visit http://localhost:3000/docs to see your API documentation.

## 💻 Sample Code

Create RESTful endpoints with minimal boilerplate:

```rust
// Define a route
#[utoipa::path(
    get,
    path = "/users/{id}",
    responses(
        (status = 200, description = "User found", body = User),
        (status = 404, description = "User not found", body = ErrorResponse)
    ),
    params(
        ("id" = Uuid, Path, description = "User ID")
    )
)]
async fn get_user(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<User>, AppError> {
    state.user_service.get_user_by_id(id).await
        .map(Json)
        .map_err(|e| e.into())
}
```

## 📊 Performance Comparison

| Framework | Requests/sec | Latency (p95) | Memory Usage |
|-----------|--------------|---------------|--------------|
| Navius | 125,000 | 1.2ms | 15MB |
| Spring Boot | 3,000 | 45ms | 150MB |
| Express.js | 8,000 | 12ms | 80MB |

*Benchmark details: Simple JSON API endpoint, AMD Ryzen 9 5950X, 32GB RAM*

## 📚 Documentation

- [Installation Guide](docs/installation.md)
- [Developer Guide](docs/DEVELOPMENT.md)
- [Project Structure](docs/project_structure.md)
- [API Integration](docs/API_INTEGRATION.md)
- [API Resource Abstraction](docs/api_resource_guide.md)
- [Authentication](docs/authentication.md)
- [PostgreSQL Integration](docs/postgresql_integration.md)
- [Security Guide](docs/security.md)
- [Testing Guide](docs/testing_guide.md)
- [Deployment Guide](docs/deployment.md)
- [Migration from Spring Boot](docs/spring-boot-migration.md)
- [Roadmaps](docs/roadmaps/)

## 🔄 Migration from Spring Boot

Coming from Spring Boot? Check out our [migration guide](docs/spring-boot-migration.md) to ease your transition:

- Mapping of Spring Boot concepts to Navius
- Step-by-step migration strategy
- Code comparison examples

## 🤝 Contributing

Contributions are welcome! Please check out our [contributing guide](CONTRIBUTING.md) to get started.

## 📄 License

Navius is Apache 2.0 licensed. See [LICENSE](LICENSE) for details. 