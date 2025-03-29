---
title: "Code Example Issue Templates"
description: "Templates for common issues found in Rust code examples during documentation verification"
category: reference
tags:
  - documentation
  - code-examples
  - verification
  - rust
related:
  - ../30_documentation-reorganization-roadmap.md
  - ./phase2-completion-plan.md
last_updated: March 27, 2025
version: 1.0
---

# Code Example Issue Templates

## Overview

This document provides templates for common issues found in Rust code examples during the documentation verification process. These templates help standardize the approach to fixing issues and ensure consistency across all documentation.

## Issue Categories

Each issue template includes:
- **Issue Type**: The category of the issue
- **Description**: A brief explanation of the problem
- **Impact**: The effect this has on users trying to follow the example
- **Detection Method**: How to identify this issue
- **Fix Template**: A standardized approach to fixing the issue
- **Example**: Before and after code snippets

## Missing Imports Issue

### Issue Type
Missing imports for used types, traits, or functions

### Description
The code example uses types, traits, or functions without importing them. This makes the example incomplete and won't compile if copied directly.

### Impact
Users will receive compilation errors when trying to use the example. They'll need to figure out the correct imports themselves, which can be frustrating, especially for beginners.

### Detection Method
Look for types, traits, or functions that are used but not declared in the example and don't have a clear namespace path (e.g., `std::`, `crate::`).

### Fix Template
```rust
// Add missing imports at the top of the example
use crate::path::to::Type;
use crate::path::to::trait_name;
use std::collections::HashMap;
// ... original code example ...
```

### Example

**Before:**
```rust
fn process_data(data: Value) -> Result<Output, Error> {
    let client = Client::new();
    let response = client.send_request(data)?;
    Ok(response.into())
}
```

**After:**
```rust
use crate::client::Client;
use crate::types::{Value, Output, Error};

fn process_data(data: Value) -> Result<Output, Error> {
    let client = Client::new();
    let response = client.send_request(data)?;
    Ok(response.into())
}
```

## Outdated API Usage Issue

### Issue Type
Usage of deprecated or outdated API functions, methods, or patterns

### Description
The code example uses API elements that have been deprecated or changed in newer versions of the codebase.

### Impact
Users will either receive deprecation warnings or runtime errors when using the example with the current version of the codebase.

### Detection Method
Compare the API usage in the example with the current API documentation. Look for functions or methods that have been renamed, changed signatures, or been replaced entirely.

### Fix Template
```rust
// Before: Outdated API usage
// let result = old_function(param1, param2);

// After: Updated to current API
let result = new_function(param1, param2);
```

### Example

**Before:**
```rust
// Using deprecated API
let config = AppConfig::create("app_name", config_path);
let app = App::from_config(config);
app.start_server();
```

**After:**
```rust
// Using current API
let config = Config::builder()
    .with_name("app_name")
    .with_path(config_path)
    .build()?;
    
let app = Application::new(config);
app.run();
```

## Incorrect Error Handling Issue

### Issue Type
Improper or missing error handling

### Description
The code example doesn't properly handle errors, either by ignoring them completely, handling them incorrectly, or using outdated error handling patterns.

### Impact
Users may not understand how to properly handle errors in their code, leading to potential runtime crashes or incorrect behavior.

### Detection Method
Look for `.unwrap()` or `.expect()` calls in production code examples, missing error propagation, or ignored error cases.

### Fix Template
```rust
// Before: Poor error handling with unwrap
// let data = get_data().unwrap();

// After: Proper error handling with ? operator or match
let data = get_data()?;

// Or with match:
let data = match get_data() {
    Ok(d) => d,
    Err(e) => return Err(e.into()),
};
```

### Example

**Before:**
```rust
fn process_file(path: &str) {
    let content = std::fs::read_to_string(path).unwrap();
    let data = parse_content(&content).unwrap();
    save_result(data).unwrap();
}
```

**After:**
```rust
fn process_file(path: &str) -> Result<(), AppError> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| AppError::io_error(e))?;
        
    let data = parse_content(&content)?;
    save_result(data)?;
    
    Ok(())
}
```

## Incomplete Example Issue

### Issue Type
Code example that's too fragmentary to be useful

### Description
The code example is missing critical context or code segments needed for it to be complete and usable.

### Impact
Users won't be able to understand or use the example effectively, leading to confusion and implementation errors.

### Detection Method
Check if the example on its own provides enough context to understand its purpose and usage. Look for missing function signatures, incomplete type definitions, or unexplained variables.

### Fix Template
```rust
// Complete example with proper context and structure
struct Context {
    // Relevant fields
}

// First, initialize the context
let context = Context { /* ... */ };

// Then, perform the operation being demonstrated
let result = perform_operation(&context);

// Finally, handle the result appropriately
process_result(result);
```

### Example

**Before:**
```rust
// Fragmentary example
settings.set_timeout(30);
client.configure(settings);
let response = client.send();
```

**After:**
```rust
// Complete example
use crate::client::{Client, Settings};
use crate::types::Response;

fn send_request() -> Result<Response, ClientError> {
    // Create and configure client
    let mut settings = Settings::default();
    settings.set_timeout(30);
    
    let mut client = Client::new();
    client.configure(settings);
    
    // Send request and return response
    let response = client.send()?;
    Ok(response)
}
```

## Missing Main Function Issue

### Issue Type
Executable example without a main function

### Description
The code example appears to be a complete program but lacks a `main()` function, making it non-executable as-is.

### Impact
Users who try to run the example directly will encounter compiler errors about missing the main function.

### Detection Method
Check if the example contains executable code that doesn't exist within a function. Look for examples that perform operations at the module level.

### Fix Template
```rust
// Wrap executable code in a main function
fn main() -> Result<(), Error> {
    // Original example code here
    
    Ok(())
}
```

### Example

**Before:**
```rust
let config = Config::from_file("config.json")?;
let app = Application::new(config);
app.run()?;
println!("Application completed successfully!");
```

**After:**
```rust
use std::error::Error;
use crate::config::Config;
use crate::app::Application;

fn main() -> Result<(), Box<dyn Error>> {
    let config = Config::from_file("config.json")?;
    let app = Application::new(config);
    app.run()?;
    println!("Application completed successfully!");
    
    Ok(())
}
```

## Type Mismatch Issue

### Issue Type
Incorrect types or mismatched types in function calls or assignments

### Description
The code example uses incorrect types or doesn't properly match types between function calls, assignments, or returns.

### Impact
Users will encounter compilation errors about type mismatches when trying to use the example.

### Detection Method
Look for places where types don't align, such as returning one type but declaring another, or passing incorrect types to functions.

### Fix Template
```rust
// Correct type annotations and ensure type consistency
fn function_name() -> CorrectReturnType {
    // Function implementation
    correct_return_value
}
```

### Example

**Before:**
```rust
fn get_user_count(users: Vec<User>) -> i32 {
    users.len()
}
```

**After:**
```rust
fn get_user_count(users: &[User]) -> usize {
    users.len()
}
```

## Compilation Helper

For quick batch fixes of common issues, the following checklist can help ensure examples are compilable:

1. **Add missing imports**:
   ```rust
   use std::collections::{HashMap, BTreeMap};
   use std::sync::{Arc, Mutex};
   use crate::types::{User, Role, Permission};
   use crate::errors::AppError;
   ```

2. **Add error handling boilerplate**:
   ```rust
   fn example_function() -> Result<(), Box<dyn std::error::Error>> {
       // Example code that uses ? operator
       Ok(())
   }
   ```

3. **Add function wrappers**:
   ```rust
   #[allow(unused_variables, dead_code)]
   fn example_wrapper() {
       // Example code that was at module level
   }
   ```

4. **Fix common Navius imports**:
   ```rust
   // Current standard imports for Navius
   use navius::core::config::Config;
   use navius::app::Application;
   use navius::app::state::AppState;
   use navius::core::error::AppError;
   ```

## Using the Templates

When reviewing code examples:

1. Identify the issues using the detection methods
2. Apply the appropriate fix template
3. Update the tracking spreadsheet with the issues found
4. Document any patterns for batch fixes
5. Verify the fixed examples compile correctly

## Related Documents

- [Phase 2 Completion Plan](./phase2-completion-plan.md)
- [Documentation Reorganization Roadmap](../30_documentation-reorganization-roadmap.md) 