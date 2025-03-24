# Codebase Cleanup Guide

## Introduction

This guide outlines the process for cleaning up the codebase after implementing the Pet API Database Integration. It addresses build errors, test failures, and documentation updates in a systematic way.

## Prerequisites

- Access to the codebase repository
- Rust development environment
- PostgreSQL database for testing
- Understanding of the Pet API implementation

## Step 1: Error Analysis and Categorization

### Analyzing Build Errors

```bash
# Get detailed build errors
cargo build -v > build_errors.log

# Count the number of errors
grep -c "error:" build_errors.log
```

Use the error tracking template to categorize build errors:

1. Import/module errors
2. Type/trait implementation errors
3. Database integration errors
4. Other build errors

### Analyzing Test Errors

```bash
# Get detailed test failures
cargo test -v > test_errors.log

# Count the number of failing tests
grep -c "test result: FAILED" test_errors.log
```

Use the error tracking template to categorize test errors:

1. Unit test failures
2. Integration test failures 
3. Database test failures

## Step 2: Fixing Build Errors

### Module Structure Fixes

1. Check for missing re-exports in mod.rs files
2. Update import paths to reflect the new module structure
3. Ensure proper visibility (pub, pub(crate), etc.)

```rust
// Example: Fixing a module structure issue in mod.rs
pub mod models;
pub mod repositories;
pub mod services;

pub use models::Pet;
pub use repositories::PetRepository;
pub use services::PetService;
```

### Implementation Fixes

1. Update trait implementations to match their definitions
2. Fix method signatures and return types
3. Implement missing methods

```rust
// Example: Fixing a trait implementation
impl EntityRepository<Pet> for PetRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Pet>, DatabaseError> {
        // Implementation
    }
    
    // Add other required methods...
}
```

### Database Integration Fixes

1. Update SQL queries to match current schema
2. Fix connection pool configuration
3. Update migration scripts if needed

## Step 3: Fixing Test Errors

### Unit Test Fixes

1. Update test expectations to match new implementations
2. Fix test data to reflect model changes
3. Update mock implementations

```rust
// Example: Updating a unit test
#[tokio::test]
async fn test_pet_service_get_by_id() {
    // Setup
    let mock_repo = MockPetRepository::new();
    mock_repo.expect_find_by_id()
        .returning(|_| Ok(Some(Pet { 
            id: Uuid::new_v4(),
            name: "Fluffy".to_string(),
            species: "Cat".to_string(),
            age: 3,
            created_at: Utc::now(),
            updated_at: Utc::now()
        })));
    
    let service = PetService::new(Box::new(mock_repo));
    
    // Execute
    let result = service.get_by_id(Uuid::new_v4()).await;
    
    // Verify
    assert!(result.is_ok());
    let pet = result.unwrap().unwrap();
    assert_eq!(pet.name, "Fluffy");
    assert_eq!(pet.species, "Cat");
}
```

### Integration Test Fixes

1. Update API endpoint tests
2. Fix request/response formats
3. Update authentication setup

### Database Test Fixes

1. Ensure proper test isolation
2. Update test fixtures
3. Fix transaction handling

## Step 4: Updating Documentation

### API Documentation

1. Update endpoint documentation
2. Update request/response examples
3. Update parameter descriptions

### Architecture Documentation

1. Update core vs. app layer documentation
2. Document repository pattern implementation
3. Update dependency diagrams

### Developer Guides

1. Update getting started guides
2. Update database integration guides
3. Create example implementations

## Verification Process

### Build Verification

```bash
# Check if build succeeds
cargo build

# Check for warnings
cargo build -v | grep warning
```

### Test Verification

```bash
# Run all tests
cargo test

# Run specific module tests
cargo test app::database::repositories::
```

### Documentation Verification

1. Review updated documentation
2. Ensure examples work as expected
3. Check for outdated information

## Progress Tracking

Use the roadmap document to track progress:

1. Update completion status for each task
2. Update "Last Updated" date using: `date "+%B %d, %Y"`
3. Document any issues or challenges encountered

## Troubleshooting Common Issues

### Database Connectivity Issues

If tests fail due to database connectivity:

1. Check database configuration
2. Ensure PostgreSQL is running
3. Verify connection string format

### Missing Dependencies

If build fails due to missing dependencies:

1. Run `cargo update`
2. Check for version conflicts
3. Update Cargo.toml if needed

### Inconsistent Test Results

If tests pass intermittently:

1. Check for race conditions
2. Ensure proper test isolation
3. Add timeouts or retry logic if needed 