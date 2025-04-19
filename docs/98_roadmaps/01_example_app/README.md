# Example App Roadmap

## Overview
This roadmap outlines the process of creating an example application from the ground up using Navius crates. The focus is on a Test-Driven Development (TDD) approach, implementing features incrementally with verification at each step.

## Goals
- Create a fully functional example app that demonstrates Navius capabilities
- Implement features using TDD with high test coverage
- Identify and address improvements needed in Navius crates
- Document best practices for using Navius ecosystem
- **[HIGHEST PRIORITY]** Reduce boilerplate in user code by moving complexity to the Navius crates

## Current Status
- Status: In Progress
- Progress: 15%
- Updated at: May 30, 2025
- **Current Scope**: Zero Boilerplate Initiative - Moving infrastructure code from user space to Navius crates

## Folder Structure
```
01_example_app/
├── README.md                # This file - main entry point with overview
├── roadmap/                 # Contains the main roadmap documents
│   ├── 01-example-app.md    # Main roadmap file with overall progress
│   ├── sub-process/         # Contains detailed sub-processes
│   │   ├── setup.md         # Initial project setup process
│   │   ├── core-features.md # Core feature implementation
│   │   ├── api-endpoints.md # API endpoint implementation
│   │   ├── testing.md       # Testing strategy and implementation
│   │   └── crate-enhancements.md # Navius crate enhancement tracking
│   └── example-app-plan.md  # Detailed implementation plan
├── docs/                    # Documentation files like ADRs
│   └── architectural-decisions/
│       └── 001-app-structure.md # Initial architectural decisions
├── reports/                 # Date-stamped progress reports
│   └── progress_2024-05-30.md # Initial progress report
└── progress.md              # Current consolidated progress tracking
```

## Key Files
- [01-example-app.md](roadmap/01-example-app.md) - The main roadmap with milestones and timeline
- [progress.md](progress.md) - Current consolidated progress tracking
- [example-app-plan.md](roadmap/example-app-plan.md) - Detailed implementation plan
- [crate-enhancements.md](roadmap/sub-process/crate-enhancements.md) - **Critical file tracking current Zero Boilerplate Initiative**

## Zero Boilerplate Initiative (Highest Priority)

Before implementing the actual example app features, our focus is on reducing the amount of boilerplate code users need to write. Currently, users need to implement infrastructure components that should be provided by the framework:

1. **Infrastructure Code Moving to Framework**:
   - `web_configurator.rs` → navius-config crate
   - `web_plugin.rs` → navius-http crate
   - `app.rs` → navius-core crate
   - Simplification of `main.rs` via macros

2. **Key Principles**:
   - The `/src` folder should contain only application-specific code
   - Framework setup and configuration should be minimal
   - Users should focus on business logic, not infrastructure

3. **Target Main.rs Example**:
```rust
#[auto_config]
#[routes]
#[tokio::main]
async fn main() {
    App::new().run().await;
}

#[get("/api/v1/hello")]
async fn hello_world() -> impl IntoResponse {
    "Hello, world!"
}
```

See [crate-enhancements.md](roadmap/sub-process/crate-enhancements.md) for detailed tracking of this initiative.

## Implementation Strategy
The example app will be built following these principles:
1. **Test-Driven Development** - Write tests before implementation
2. **Incremental Feature Development** - Small, verified steps
3. **Continuous Verification** - Ensure no errors/warnings at each step
4. **Crate Enhancement** - Improve Navius crates as needed
5. **Zero Boilerplate** - Move infrastructure code from user space to Navius crates

## Technologies
- **Framework**: Axum
- **Crates**: Navius ecosystem (core, http, auth, db, cache, etc.)
- **Database**: PostgreSQL
- **Authentication**: Microsoft Entra (via navius-auth-entra)
- **Metrics**: Prometheus (via navius-metrics-prometheus)

## Next Steps
1. Complete the Zero Boilerplate Initiative
2. Create the initial project structure with minimal user code
3. Develop the core application features
4. Implement API endpoints
5. Add comprehensive tests
6. Document findings and best practices

## Related Documents
- Main Project README: [/README.md](/README.md)
- Roadmaps Overview: [/docs/98_roadmaps/README.md](/docs/98_roadmaps/README.md) 