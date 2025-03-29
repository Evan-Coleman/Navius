# auth Module Optimization Plan

## Module Overview
- Files: 11
- Current feature usage: 0 files have feature flags
- Target feature: `auth`

## Files to Update
- /Users/goblin/dev/git/navius/src/app/auth/client.rs
- /Users/goblin/dev/git/navius/src/core/auth/claims.rs
- /Users/goblin/dev/git/navius/src/core/auth/client.rs
- /Users/goblin/dev/git/navius/src/core/auth/error.rs
- /Users/goblin/dev/git/navius/src/core/auth/interfaces.rs
- /Users/goblin/dev/git/navius/src/core/auth/middleware.rs
- /Users/goblin/dev/git/navius/src/core/auth/mock.rs
- /Users/goblin/dev/git/navius/src/core/auth/models.rs
- /Users/goblin/dev/git/navius/src/core/auth/providers.rs
- /Users/goblin/dev/git/navius/src/core/auth/providers/common.rs
- /Users/goblin/dev/git/navius/src/core/auth/providers/entra.rs

## Dependencies to Make Optional
*Identify these from Cargo.toml based on the output from Step 3*

## Required Code Changes

### 1. Update Cargo.toml
```toml
# Add or modify feature definition
[features]
auth = [] # Add any dependent features or optional dependencies here

# Make dependencies optional
[dependencies]
# example = { version = "1.0", optional = true }
# ...

# Map optional dependencies to the feature
[dependencies.example]
optional = true
```

### 2. Update Module Exports
```rust
// In the module's mod.rs or lib.rs
#[cfg(feature = "auth")]
pub mod submodule;

// Re-export key types conditionally
#[cfg(feature = "auth")]
pub use submodule::{Type1, Type2};
```

### 3. Conditional Service Registration
```rust
// In application startup code
#[cfg(feature = "auth")]
app.service(auth::service());
```

### 4. Add Feature Documentation
```rust
/// auth module
/// 
/// This module requires the `auth` feature to be enabled.
/// 
/// # Example
/// 
/// Enable the feature in Cargo.toml:
/// ```toml
/// [features]
/// auth = []
/// ```
/// 
/// Use the module:
/// ```rust
/// #[cfg(feature = "auth")]
/// use navius::core::auth;
/// ```
pub mod auth {
    // ...
}
```

## Testing Plan
1. Build with feature enabled: `cargo build --features auth`
2. Build without feature: `cargo build --no-default-features`
3. Verify all tests pass: `cargo test --features auth`
4. Compare binary sizes to measure improvement
