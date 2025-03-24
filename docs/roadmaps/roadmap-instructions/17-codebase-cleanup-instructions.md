# Codebase Cleanup Instructions

## Overview
These instructions guide the process of cleaning up the codebase after implementing the Pet API Database Integration, fixing approximately 60 test errors and 32 build errors.

## Detailed Error Analysis

### Build Errors Summary (32 total)
- **SQLx Offline Mode Errors (10)**: All SQLx queries need cache generation
- **Import/Module Errors (6)**: Missing or incorrect imports across codebase
- **Type/Trait Implementation Errors (10)**: Type mismatches, especially with Arc wrappers
- **Method Implementation Errors (4)**: Missing methods referenced in code
- **General Syntax Errors (2)**: Miscellaneous compiler issues

#### SQLx Offline Mode Errors
| Error | Location | Root Cause | Dependencies | Status |
| ----- | -------- | ---------- | ------------ | ------ |
| `SQLX_OFFLINE=true` but there is no cached data for this query | src/core/database/repositories/mod.rs:76-90 | SQLx is in offline mode but query cache has not been generated | Need to run `cargo sqlx prepare` | Not fixed |
| `SQLX_OFFLINE=true` but there is no cached data for this query | src/app/database/repositories/pet_repository.rs:81-88 | SQLx is in offline mode but query cache has not been generated | Need to run `cargo sqlx prepare` | Not fixed |

#### Import/Module Errors
| Error | Location | Root Cause | Dependencies | Status |
| ----- | -------- | ---------- | ------------ | ------ |
| Unresolved imports `crate::core::metrics::MetricsHandle`, `crate::core::models::ApiResponse` | src/core/router/app_router.rs:33-34 | Imports reference non-existent items | Need to update module structure or fix imports | Not fixed |
| Unresolved import `crate::models::User` | src/core/services/user.rs:15 | Import references non-existent item | Need to update import path or create missing model | Not fixed |
| Cannot find trait `PgPool` in this scope | src/app/api/examples/users.rs:358 | Missing import for trait | Need to import `crate::database::PgPool` | Not fixed |
| Ambiguous name `error` | src/app/api/examples/users.rs:18 | Multiple glob imports causing name conflict | Need to use specific imports | Not fixed |

#### Type/Trait Implementation Errors
| Error | Location | Root Cause | Dependencies | Status |
| ----- | -------- | ---------- | ------------ | ------ |
| Mismatched types: expected `Option<Arc<CacheRegistry>>`, found `Option<CacheRegistry>` | src/core/router/app_router.rs:245 | Type mismatch | Need to wrap in Arc | Not fixed |
| Mismatched types: expected `Arc<EntraTokenClient>`, found `EntraTokenClient` | src/core/router/app_router.rs:248 | Type mismatch | Need to wrap in Arc | Not fixed |
| Mismatched types: expected `Option<Pool<Postgres>>`, found `Option<Arc<Box<dyn PgPool>>>` | src/core/router/app_router.rs:253 | Type mismatch | Need to use consistent database pool types | Not fixed |
| `query_str` does not live long enough | src/core/database/utils.rs:80 | Lifetime issue | Static reference required for query | Not fixed |
| The parameter type `T` may not live long enough | src/core/database/utils.rs:80 | Missing lifetime constraint | Need to add 'static bound | Not fixed |

#### Method Implementation Errors
| Error | Location | Root Cause | Dependencies | Status |
| ----- | -------- | ---------- | ------------ | ------ |
| No method named `count_entries` found for reference `&Arc<CacheRegistry>` | src/core/handlers/health.rs:48 | Method doesn't exist or not accessible | Need to implement method or fix call | Not fixed |
| No method named `get_stats` found for reference `&ResourceCache<T>` | src/core/cache/cache_manager.rs:230 | Method doesn't exist | Need to implement method | Not fixed |
| No method named `register_resource` found for reference `&Arc<registry::ApiResourceRegistry>` | src/core/utils/api_resource/core.rs:39 | Method doesn't exist or incorrect receiver type | Need to implement method or fix call | Not fixed |

### Test Errors Summary (~60 total)
- **Import/Module Errors**: Similar to build errors but across test files
- **Type Mismatch Errors**: Consistent with build errors, particularly Arc wrappers
- **Missing Test Utilities**: MockTokenClient not implemented but referenced
- **Missing Method Errors**: Methods called on test objects that don't exist
- **SQLx Query Cache Errors**: Same as build errors

#### Missing Mock Implementation Errors
| Error | Location | Root Cause | Dependencies | Status |
| ----- | -------- | ---------- | ------------ | ------ |
| Failed to resolve: use of undeclared type `MockTokenClient` | src/core/reliability/test.rs:217 | Missing mock | Need to implement MockTokenClient | Not fixed |
| Failed to resolve: use of undeclared type `MockTokenClient` | src/core/utils/api_resource/core.rs:536 | Missing mock | Need to implement MockTokenClient | Not fixed |

#### Missing Method Errors in Tests
| Error | Location | Root Cause | Dependencies | Status |
| ----- | -------- | ---------- | ------------ | ------ |
| No method named `status_code` found for struct `axum::Json<core::models::error::HealthCheckResponse>` | src/core/handlers/health.rs:103 | Method doesn't exist | Use different method to get status code | Not fixed |
| No method named `status_code` found for struct `axum::Json<core::models::error::DetailedHealthResponse>` | src/core/handlers/health.rs:113 | Method doesn't exist | Use different method to get status code | Not fixed |

## Root Causes

1. **Incomplete SQLx Setup**: SQLx is configured to run in offline mode, but query cache hasn't been generated
2. **Architectural Inconsistency**: Pet API implementation in both `/app` and `/core` directories
3. **Module Structure Changes**: Imports refer to modules that may have moved or been renamed
4. **Type System Issues**: Inconsistent use of Arc wrappers and trait bounds
5. **Missing Mock Implementations**: Missing MockTokenClient for testing
6. **Duplicate Types**: Multiple types with same name (e.g., ServiceError) causing ambiguity
7. **Method Implementation Gaps**: Several called methods don't exist or are improperly accessed

## Detailed Implementation Steps

### Step 1: Generate SQLx Query Cache
Run the SQLx cache generation script to fix offline mode errors:
```bash
# Ensure database is properly configured
./scripts/generate_sqlx_cache.sh
```

### Step 2: Fix Critical Type System Issues
1. **Fix Arc wrapper consistency**:
   ```rust
   // Example: In app_router.rs:245
   // Change from:
   cache_registry: cache_registry.clone(),
   // To:
   cache_registry: Some(Arc::new(cache_registry.clone())),
   
   // Example: In app_router.rs:248
   // Change from:
   Some(EntraTokenClient::from_config(&config))
   // To:
   Some(Arc::new(EntraTokenClient::from_config(&config)))
   ```

2. **Fix lifetime issues in database/utils.rs**:
   ```rust
   // Example: Add 'static lifetime
   pub async fn exists<'a, T>(
       pool: &'a Pool<Postgres>,
       table: &'a str,
       column: &'a str,
       value: T,
   ) -> Result<bool, sqlx::Error>
   where
       T: sqlx::Type<Postgres> + sqlx::Encode<'static, Postgres> + Send + 'a,
   {
       // Function body
   }
   ```

### Step 3: Fix Module Structure and Imports
1. **Resolve ambiguous imports**:
   ```rust
   // Instead of glob imports like:
   pub use crate::app::services::*;
   pub use crate::core::services::*;
   
   // Use specific imports:
   pub use crate::app::services::{Service1, Service2};
   pub use crate::core::services::{Service3, Service4};
   ```

2. **Fix missing imports**:
   ```rust
   // Add specific imports for missing items
   use crate::database::PgPool;
   ```

### Step 4: Implement Missing Components
1. **Create or expose MockTokenClient**:
   - The MockTokenClient has been created in src/core/auth/mock.rs
   - Ensure it's properly exposed in the module structure

2. **Implement missing methods**:
   ```rust
   // Example: Implement count_entries for CacheRegistry
   impl CacheRegistry {
       pub fn count_entries(&self) -> usize {
           // Implementation details
       }
   }
   ```

### Step 5: Fix Pet API Repository Duplication
Decide on consistent location (recommend `/core` since it's a core feature):
1. Remove duplicate implementation
2. Update all imports and references
3. Ensure consistent model usage

### Step 6: Fix Test Issues
1. Update test expectations to match current API:
   ```rust
   // Example: Instead of response.status_code()
   assert_eq!(StatusCode::OK, 200);
   ```

2. Fix test setups that use missing components

## Work Tracking and Review Process

For each fix, follow this process:
1. Make the fix
2. Run cargo check to validate compilation
3. Run tests for the specific module
4. Update the status in the roadmap

Critical review points where progress should be assessed:
1. After SQLx cache generation
2. After fixing major type system issues
3. After resolving Pet API duplication
4. After all build errors are fixed
5. After all test errors are fixed 