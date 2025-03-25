# Core Stability Roadmap Instructions

This document provides detailed instructions for implementing the Core Stability Roadmap. Follow these steps sequentially to ensure a smooth implementation.

## Phase 1: Fix Build Errors

### 1. Router Module Fixes

#### Step 1.1: Create app_router.rs file
1. Create a new file at `src/core/router/app_router.rs`
2. Define the `AppState` struct with the following fields:
   - `config`: App configuration
   - `start_time`: For tracking uptime
   - `client`: HTTP client (Optional)
   - `cache_registry`: For caching (Optional)
   - `token_client`: For authentication (Optional)
   - `metrics_handle`: For metrics (Optional)
   - `resource_registry`: For API resources (Optional)
   - `service_registry`: For services
3. Implement `Default` for `AppState`
4. Add functions for router creation

#### Step 1.2: Fix AppState references
1. Update all imports from `crate::core::router::AppState` to use the correct path
2. Fix any method calls on `AppState` to match the new implementation
3. Ensure `AppState` is properly re-exported from the `app` module

### 2. Module Structure Cleanup

#### Step 2.1: Fix missing examples module
1. Create a basic structure at `src/app/api/examples.rs` or
2. Remove references to it from `src/app/api/mod.rs`

#### Step 2.2: Fix module declarations
1. Review all module declarations in `.rs` files
2. Ensure all referenced modules exist
3. Fix any incorrect re-exports

## Phase 2: Endpoint Implementation

### 1. Health Endpoint Enhancement

#### Step 1.1: Update health model
1. Modify `DetailedHealthResponse` in `src/core/models/` to match the desired format:
   ```json
   {
     "status": "healthy",
     "version": "0.1.0",
     "uptime_seconds": 14,
     "environment": "development",
     "dependencies": [...]
   }
   ```

#### Step 1.2: Enhance dependency reporting
1. Update `DependencyStatus` to include all required fields
2. Implement proper status reporting for all dependencies
3. Add details for each dependency

#### Step 1.3: Implement health handler
1. Update the `detailed_health_handler` in `src/core/handlers/health.rs`
2. Include environment information from config
3. Calculate uptime in seconds
4. Set appropriate overall status

### 2. Info Endpoint Enhancement

#### Step 2.1: Update info model
1. Modify the info response model in `src/core/models/`
2. Add any missing fields

#### Step 2.2: Enhance info handler
1. Update the `info` handler in `src/core/handlers/actuator.rs`
2. Add additional system information
3. Include environment details

## Phase 3: Testing and Documentation

### 1. Test Coverage

#### Step 1.1: Health endpoint tests
1. Create unit tests for the health handler
2. Test with various dependency states
3. Verify correct status codes

#### Step 1.2: Info endpoint tests
1. Create unit tests for the info handler
2. Verify all fields are properly populated

#### Step 1.3: Integration tests
1. Create an integration test for the actuator routes
2. Test the full request/response cycle

### 2. Documentation

#### Step 2.1: API documentation
1. Add OpenAPI/Swagger documentation for the health endpoint
2. Add documentation for the info endpoint

#### Step 2.2: Usage examples
1. Add examples of how to use the endpoints
2. Include sample responses

## Implementation Notes

### Dependencies Status Reporting
- Database: Should report "up" if connected, "down" if failed, "disabled" if not configured
- Cache: Should report "up" if working, "down" if failed
- Authentication: Should report "enabled"/"disabled" based on configuration

### Status Codes
- 200 OK: All dependencies are up or disabled (intentionally)
- 503 Service Unavailable: One or more critical dependencies are down

### Uptime Calculation
- Use the `start_time` field in `AppState` to calculate seconds since server start
- Format as an integer value for `uptime_seconds`

## Common Issues and Solutions

### Circular Dependencies
- If you encounter circular dependency issues, consider breaking the dependency chain
- Use trait objects or type erasure to resolve circular references

### Missing Modules
- Always check that the module exists before importing it
- Create empty modules with TODO comments if needed for compilation

### AppState Access
- Ensure AppState is passed to all handlers that need it
- Use the State extractor in your route handlers

## Testing Tips

- Use the `MockClient` for testing HTTP requests
- Set up different test fixtures for various dependency states
- Test both success and failure paths for each endpoint
- Use integration tests to verify the full request/response lifecycle 