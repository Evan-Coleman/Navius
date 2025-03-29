# Workspace Migration Plan

## Current State

The Navius project currently uses feature flags to conditionally compile different functionality, which has led to:
- Complex conditional compilation directives throughout the codebase
- Tight coupling between modules that should be independent
- Larger binary sizes even when features aren't needed
- Challenges in testing isolated components

## Proposed Workspace Structure

We will restructure the project into a Rust workspace with the following crates:

```
navius/
├── Cargo.toml (workspace)
├── crates/
│   ├── navius-core/         # Essential, mandatory functionality
│   ├── navius-api/          # Main application that ties everything together
│   ├── navius-auth/         # Authentication functionality
│   ├── navius-metrics/      # Metrics and monitoring capabilities
│   │   ├── navius-metrics-prometheus/  # Prometheus integration
│   │   ├── navius-metrics-dynatrace/   # Dynatrace integration
│   ├── navius-database/     # Database functionality
│   │   ├── navius-database-postgres/   # PostgreSQL support
│   ├── navius-cache/        # Caching functionality
│   │   ├── navius-cache-redis/        # Redis support
│   ├── navius-cli/          # CLI tools
│   └── navius-test-utils/   # Testing utilities
└── examples/                # Example applications
```

## Benefits

1. **Clear Boundaries**: Each crate has a well-defined API and responsibility
2. **Reduced Binary Size**: Applications only include the crates they need
3. **Focused Testing**: Each crate can be tested in isolation
4. **Improved Build Times**: Cargo can parallelize building independent crates
5. **Better Dependency Management**: Each crate specifies only what it needs
6. **Simpler Code**: Minimal need for conditional compilation
7. **Independent Versioning**: Each crate can evolve at its own pace

## Migration Strategy

### Phase 1: Setup Workspace Structure (1-2 weeks)

1. **Create Workspace Configuration**
   - Update root Cargo.toml with workspace configuration
   - Define shared dependencies in workspace

2. **Create Core Crate**
   - Implement navius-core with essential functionality
   - Move common code, error types, and configuration here

3. **Setup Test Infrastructure**
   - Create navius-test-utils crate
   - Ensure tests can run across the workspace

### Phase 2: Module Extraction (2-4 weeks)

For each major feature module:

1. **Analyze Dependencies**
   - Identify all code related to the module
   - Map external dependencies required

2. **Extract Module to Crate**
   - Create new crate structure
   - Move relevant code to new crate
   - Add proper error handling and conversion
   - Ensure crate has clear public API

3. **Update References**
   - Update imports in main application
   - Fix any breaking changes

4. **Implement Tests**
   - Ensure module tests work in isolation
   - Add integration tests as needed

### Phase 3: Application Integration (1-2 weeks)

1. **Create Main Application Crate**
   - Implement navius-api crate
   - Wire up all the modules with dependency injection

2. **Simplify Feature Configuration**
   - Update build scripts and configuration
   - Document new approach to selecting features

3. **Update Examples and Documentation**
   - Provide examples of using different combinations of crates
   - Update all documentation to reflect new structure

### Phase 4: Cleanup and Optimization (1 week)

1. **Remove Unused Code**
   - Delete old feature flag annotations
   - Clean up any redundant code

2. **Optimize Build Process**
   - Update CI/CD pipeline for workspace
   - Add optimized release profiles

3. **Performance Testing**
   - Verify that the new structure doesn't impact performance
   - Measure binary size improvements

## Migration Order (Priority)

1. **navius-core**: Essential types and utilities
2. **navius-auth**: Authentication functionality
3. **navius-metrics**: Metrics capabilities
4. **navius-database**: Database functionality
5. **navius-cache**: Caching functionality
6. **navius-api**: Main application
7. **navius-cli**: Command-line tools
8. **navius-test-utils**: Testing utilities

## Success Criteria

- All functionality preserved with same or better performance
- Binary size reduced by at least 30% for minimal builds
- Build times improved by at least 20%
- Clear, well-documented APIs between crates
- All tests passing across the workspace
- Documentation updated to reflect new structure

## Risks and Mitigations

| Risk | Mitigation |
|------|------------|
| Breaking API changes | Provide detailed migration guide for users |
| Increased complexity for simple use cases | Create convenience crates that bundle common combinations |
| Longer initial build times | Use CI caching and optimize workspace configuration |
| Regression in functionality | Comprehensive test coverage before and after migration |
| Incomplete extraction of features | Thorough dependency analysis before starting each extraction | 