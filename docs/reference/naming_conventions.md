# Naming Conventions

This document outlines the naming conventions used in the Navius project after the project restructuring.

## File and Directory Naming

All Rust source files and directories should follow the snake_case convention:

```
src/
├── app/
│   ├── api/
│   │   └── user_api.rs  ✅ Good: snake_case
│   └── services/
│       └── user_service.rs  ✅ Good: snake_case
└── core/
    ├── repository/
    │   └── user_repository.rs  ✅ Good: snake_case
    └── utils/
        └── string_utils.rs  ✅ Good: snake_case

// ❌ Bad examples:
userApi.rs
UserService.rs
String-Utils.rs
```

## Module Declarations

Module declarations should match the file names:

```rust
// In src/core/utils/mod.rs
pub mod string_utils;  // ✅ Matches file name string_utils.rs
pub mod date_format;   // ✅ Matches file name date_format.rs

// ❌ Bad examples:
pub mod StringUtils;
pub mod Date_Format;
```

## Structure and Enum Naming

- **Structures and Enums**: Use PascalCase (UpperCamelCase)
- **Traits**: Use PascalCase (UpperCamelCase)

```rust
// Structures - PascalCase
pub struct UserRepository { /* ... */ }
pub struct ApiResource { /* ... */ }

// Enums - PascalCase
pub enum UserRole {
    Admin,
    User,
    Guest,
}

// Traits - PascalCase
pub trait Repository { /* ... */ }
pub trait CacheProvider { /* ... */ }
```

## Function and Method Naming

Functions and methods should use snake_case:

```rust
// Functions - snake_case
pub fn create_user() { /* ... */ }
pub fn validate_input() { /* ... */ }

// Methods - snake_case
impl UserService {
    pub fn find_by_id(&self, id: &str) { /* ... */ }
    pub fn update_profile(&self, user: &User) { /* ... */ }
}
```

## Variable and Parameter Naming

Variables and parameters should use snake_case:

```rust
// Variables and parameters - snake_case
let user_id = "123";
let connection_string = "postgres://...";

fn process_request(request_body: &str, user_context: &Context) { /* ... */ }
```

## Constants and Static Variables

Constants and static variables should use SCREAMING_SNAKE_CASE:

```rust
// Constants - SCREAMING_SNAKE_CASE
const MAX_CONNECTIONS: u32 = 100;
const DEFAULT_TIMEOUT_MS: u64 = 5000;

// Static variables - SCREAMING_SNAKE_CASE
static API_VERSION: &str = "v1";
```

## Type Aliases

Type aliases should use PascalCase:

```rust
// Type aliases - PascalCase
type ConnectionPool = Pool<Connection>;
type Result<T> = std::result::Result<T, AppError>;
```

## Consistent Naming Across Files

Related components should have consistent naming:

```rust
// Related components
mod user_repository;  // File: user_repository.rs
mod user_service;     // File: user_service.rs
mod user_api;         // File: user_api.rs

// Structures within files
pub struct UserRepository { /* ... */ }  // In user_repository.rs
pub struct UserService { /* ... */ }      // In user_service.rs
```

## Automated Checking

We've implemented a script to help identify files that don't follow the naming conventions:

```bash
./.devtools/scripts/fix-imports-naming.sh
```

This script identifies files with uppercase characters in their names, which may indicate non-compliance with the snake_case convention.

## Benefits of Consistent Naming

1. **Readability** - Makes code more readable and predictable
2. **Consistency** - Ensures all developers follow the same patterns
3. **Idiomatic** - Follows Rust's recommended naming conventions
4. **Tooling** - Better integration with Rust tools and IDE features 