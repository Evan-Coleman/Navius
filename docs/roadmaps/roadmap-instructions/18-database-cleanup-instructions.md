# Database Cleanup Instructions

## Overview
These instructions guide the process of completely removing all database-related code, particularly Pet-related functionality, to stabilize the server. The goal is to eliminate all database code without documenting or preserving it for future reference.

## Important Note
The files identified in the initial assessment are only examples and not a comprehensive list. A thorough search must be performed to identify all database-related files before beginning the removal process.

These are the files identified:


/Users/goblin/dev/git/navius/src/core/config/app_config.rs
/Users/goblin/dev/git/navius/src/core/database/models/mod.rs
/Users/goblin/dev/git/navius/src/core/database/models/pet.rs
/Users/goblin/dev/git/navius/src/core/database/repositories/mod.rs
/Users/goblin/dev/git/navius/src/core/database/repositories/pet_repository.rs
/Users/goblin/dev/git/navius/src/core/router/app_router.rs
/Users/goblin/dev/git/navius/src/core/services/mod.rs
/Users/goblin/dev/git/navius/src/core/services/pet.rs


/Users/goblin/dev/git/navius/src/app/api/pet_db.rs
/Users/goblin/dev/git/navius/src/app/api/pet_db_test.rs
/Users/goblin/dev/git/navius/src/app/database/migrations/01_create_pets_table.sql
/Users/goblin/dev/git/navius/src/app/database/models/pet.rs
/Users/goblin/dev/git/navius/src/app/database/repositories/mock.rs
/Users/goblin/dev/git/navius/src/core/database/models/pet.rs
/Users/goblin/dev/git/navius/src/core/database/repositories/mod.rs
/Users/goblin/dev/git/navius/src/core/database/repositories/pet_repository.rs

## Comprehensive File Search

Before beginning the removal process, conduct a thorough search to identify all database-related files:

```bash
# Search for all files related to 'Pet' or 'pet'
grep -r "Pet\|pet" --include="*.rs" src/
grep -r "Pet\|pet" --include="*.sql" src/

# Search for database-related code
grep -r "PgPool\|sqlx\|repository\|database" --include="*.rs" src/

# Search for specific database-related structs and traits
grep -r "struct.*Repository\|trait.*Repository" --include="*.rs" src/
```

Create a complete list of all files that need to be modified or removed. Add these to the appropriate phases in the implementation plan.

## Priority Levels
- **Critical**: Must be completed first; blocks server startup
- **High**: Required for basic functionality
- **Medium**: Improves stability but not blocking
- **Low**: Cleanup tasks that can be deferred

## Step-by-Step Implementation Guide

### Phase 1: Core Database Removal (Critical Priority)

#### 1.1 Remove Core Database Models
1. **Delete Pet Model**:
   ```bash
   # Delete the file
   rm src/core/database/models/pet.rs
   ```

2. **Update Model Module Exports**:
   Edit `src/core/database/models/mod.rs`:
   ```rust
   // BEFORE
   mod pet;
   pub use pet::Pet;
   
   // AFTER
   // Pet model removed for stability
   ```

3. **Remove Any Additional Model Files**:
   Delete any additional model files identified in the comprehensive search.

#### 1.2 Remove Core Database Repositories
1. **Delete Pet Repository**:
   ```bash
   # Delete the file
   rm src/core/database/repositories/pet_repository.rs
   ```

2. **Update Repository Module Exports**:
   Edit `src/core/database/repositories/mod.rs`:
   ```rust
   // BEFORE
   mod pet_repository;
   pub use pet_repository::{PetRepository, PgPetRepository};
   
   // AFTER
   // Pet repository removed for stability
   ```

3. **Remove Any Additional Repository Files**:
   Delete any additional repository files identified in the comprehensive search.

#### 1.3 Remove Core Services
1. **Delete Pet Service**:
   ```bash
   # Delete the file
   rm src/core/services/pet.rs
   ```

2. **Update Service Module Exports**:
   Edit `src/core/services/mod.rs`:
   ```rust
   // BEFORE
   pub mod pet;
   pub use pet::PetService;
   
   // AFTER
   // Pet service removed for stability
   ```

3. **Remove Any Additional Service Files**:
   Delete any additional service files identified in the comprehensive search.

#### 1.4 Update Core Router
Edit `src/core/router/app_router.rs`:
```rust
// BEFORE
pub fn pet_routes() -> Router<AppState> {
    Router::new()
        .route("/pets", get(list_pets).post(create_pet))
        .route("/pets/:id", get(get_pet).put(update_pet).delete(delete_pet))
}

// AFTER
// Pet routes removed for stability
pub fn app_router() -> Router<AppState> {
    // Return router without pet routes
    Router::new()
        .route("/health", get(health_check))
        // Other non-database routes
}
```

Update any additional router files identified in the comprehensive search.

#### 1.5 Update AppConfig
Edit `src/core/config/app_config.rs`:
```rust
// BEFORE
pub struct DatabaseConfig {
    pub connection_string: String,
    pub max_connections: u32,
    pub enable_migrations: bool,
}

// AFTER
// Database config removed for stability
pub struct AppConfig {
    // Keep non-database related configs
    pub server: ServerConfig,
    pub logging: LoggingConfig,
    // pub database: DatabaseConfig, // Removed
}
```

### Phase 2: App Database Removal (High Priority)

#### 2.1 Remove App Database Models and Migrations
1. **Delete Pet Model**:
   ```bash
   # Delete the file
   rm src/app/database/models/pet.rs
   ```

2. **Delete Migration Files**:
   ```bash
   # Delete the file
   rm src/app/database/migrations/01_create_pets_table.sql
   ```

3. **Remove Any Additional Model Files**:
   Delete any additional model files identified in the comprehensive search.

#### 2.2 Remove App Pet Handlers
1. **Delete Handler Files**:
   ```bash
   # Delete the files
   rm src/app/api/pet_db.rs
   rm src/app/api/pet_db_test.rs
   ```

2. **Update App Router**:
   Edit `src/app/router.rs`:
   ```rust
   // BEFORE
   pub fn app_routes() -> Router {
       Router::new()
           .nest("/api", api_routes())
           .nest("/pets", pet_routes())
   }
   
   // AFTER
   pub fn app_routes() -> Router {
       Router::new()
           .nest("/api", api_routes())
           // Pet routes removed for stability
   }
   ```

3. **Remove Any Additional Handler Files**:
   Delete any additional handler files identified in the comprehensive search.

#### 2.3 Clean Mock Repositories
Edit `src/app/database/repositories/mock.rs`:
```rust
// BEFORE
pub struct MockPetRepository {
    pets: Arc<Mutex<Vec<Pet>>>,
}

impl MockPetRepository {
    // Implementation details
}

// AFTER
// Mock repositories removed for stability
```

Remove any additional mock repository files identified in the comprehensive search.

### Phase 3: Testing and Verification (Medium Priority)

#### 3.1 Update Test Suite
1. **Identify Tests That Need Removal**:
   ```bash
   # Find all tests that reference Pet-related code
   grep -r "Pet" --include="*_test.rs" src/
   ```

2. **Remove or Comment Out Database Tests**:
   For each affected test file:
   ```rust
   // BEFORE
   #[tokio::test]
   async fn test_list_pets() {
       let repository = MockPetRepository::new(vec![create_test_pet()]);
       let service = PetService::new(Arc::new(repository));
       let response = service.get_all_pets().await.unwrap();
       assert_eq!(response.len(), 1);
   }
   
   // AFTER
   // Test removed as part of database cleanup
   /*
   #[tokio::test]
   async fn test_list_pets() {
       // Test code
   }
   */
   ```

#### 3.2 Verify Server Operation
1. **Run Server Without Database**:
   ```bash
   cargo run
   ```

2. **Test Health Endpoint**:
   ```bash
   curl http://localhost:3000/health
   ```

### Phase 4: Cleanup (Low Priority)

#### 4.1 Remove Unused Dependencies
Update `Cargo.toml`:
```toml
# BEFORE
[dependencies]
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres", "uuid", "chrono", "json"] }
tokio-postgres = "0.7"

# AFTER
# Database dependencies removed
# sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres", "uuid", "chrono", "json"] }
# tokio-postgres = "0.7"
```

#### 4.2 Update Documentation
1. **Update README.md**:
   Add a note about the database removal:
   ```markdown
   ## Important Note
   
   Database functionality has been removed for stability.
   ```

## Verification Steps

After completing all steps, verify the changes:

1. **Build Check**:
   ```bash
   cargo check
   ```

2. **Test Suite**:
   ```bash
   cargo test
   ```

3. **Run Server**:
   ```bash
   cargo run
   ```

4. **Health Check**:
   ```bash
   curl http://localhost:3000/health
   ```

## Alternative Approach If Strict Removal Fails

If removing all database code causes too many cascading errors:

1. **Comment Out Code Instead of Deleting**
   ```rust
   // Comment out entire files or problematic sections
   /*
   pub struct PetRepository {
       // ...
   }
   */
   ```

2. **Create Empty Stubs**
   ```rust
   // Create empty stubs that satisfy type requirements but do nothing
   pub struct PetRepository {}
   
   #[async_trait]
   impl PetRepository for PgPetRepository {
       async fn find_all(&self) -> Result<Vec<Pet>, AppError> {
           // Return empty result
           Ok(vec![])
       }
       // Other required methods...
   }
   ```

3. **Use Feature Flags**
   ```rust
   // Use feature flags to conditionally compile database code
   #[cfg(feature = "database")]
   pub fn init_database() {
       // Database initialization code
   }
   
   #[cfg(not(feature = "database"))]
   pub fn init_database() {
       // Empty function
   }
   ``` 