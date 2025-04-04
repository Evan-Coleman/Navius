# ADR-001: Example App Structure

## Status
Proposed

## Date
May 30, 2024

## Context
We need to establish a clear architectural structure for the example app that demonstrates the Navius crate ecosystem while following best practices. The app will be built using TDD and requires a structure that facilitates incremental development, testing, and enhancements to the Navius crates.

## Decision
We will adopt a clean architecture approach with clear separation of concerns, using the following structure:

```
example-app/
├── src/
│   ├── main.rs           # Application entry point
│   ├── lib.rs            # Library exports
│   ├── core/             # Core domain logic
│   │   ├── models/       # Domain entities and value objects
│   │   ├── services/     # Business logic services
│   │   └── errors/       # Domain error types
│   ├── api/              # API layer
│   │   ├── handlers/     # Request handlers
│   │   ├── middleware/   # API middleware
│   │   ├── routes/       # Route definitions
│   │   └── responses/    # Response types
│   ├── data/             # Data access layer
│   │   ├── repositories/ # Repository implementations
│   │   ├── migrations/   # Database migrations
│   │   └── entities/     # Database entity models
│   ├── config/           # Configuration management
│   │   ├── settings.rs   # Application settings
│   │   └── env.rs        # Environment variable handling
│   ├── utils/            # Utility functions and helpers
│   │   ├── logging.rs    # Logging utilities
│   │   └── validation.rs # Validation helpers
│   └── bootstrap/        # Application bootstrap code
│       ├── app.rs        # Application setup
│       └── server.rs     # Server configuration
├── tests/                # Integration tests
│   ├── api/              # API tests
│   ├── data/             # Data layer tests
│   └── common/           # Test utilities
├── Cargo.toml            # Dependency management
├── .env.example          # Environment template
├── docker-compose.yml    # Docker services
└── README.md             # Project documentation
```

## Key Architectural Principles

1. **Clean Architecture**
   - Core domain logic independent of external concerns
   - Dependencies point inward (data → core ← api)
   - Clear boundaries between layers

2. **Dependency Injection**
   - Use navius-di for dependency management
   - Constructor injection for services and repositories
   - Testable components with mockable dependencies

3. **Error Handling**
   - Domain-specific error types
   - Proper error mapping between layers
   - Consistent error responses

4. **Configuration Management**
   - Environment-based configuration
   - Sensible defaults with override capabilities
   - Validation of configuration at startup

5. **Testability**
   - TDD approach for all components
   - High test coverage (90%+)
   - Separation of unit and integration tests

## Navius Crate Integration

The example app will integrate the following Navius crates:

- **navius-core**: For core domain functionality
- **navius-http**: For API layer and routing
- **navius-auth**: For authentication and authorization
- **navius-db**: For database access
- **navius-cache**: For caching frequently accessed data
- **navius-job**: For background job processing
- **navius-event**: For domain events
- **navius-messaging**: For messaging between components
- **navius-metrics**: For application metrics
- **navius-test**: For testing utilities

## Consequences

### Positive
- Clear separation of concerns facilitates TDD
- Structure supports incremental development
- Modular design allows for focused testing
- Architecture highlights Navius crate capabilities
- Easy to identify enhancement opportunities

### Negative
- More initial structure than a minimal example
- May require some boilerplate code
- Added complexity for a small application

## Alternatives Considered

1. **Minimal Single-File Approach**
   - Would be simpler to start with
   - Less initial structure
   - Limited ability to demonstrate all Navius features
   - Would become unwieldy as features are added

2. **Feature-Based Structure**
   - Organize by feature rather than layer
   - Could better represent domain boundaries
   - More difficult to demonstrate clean separation
   - Potential for duplicated code

## References
- [Clean Architecture by Robert C. Martin](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Axum Framework Documentation](https://docs.rs/axum/latest/axum/)
- [Navius Crate Documentation](#) 