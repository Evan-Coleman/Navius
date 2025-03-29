---
title: "Testing Prompt"
description: "Documentation about Testing Prompt"
category: contributing
tags:
  - documentation
  - integration
  - security
  - testing
last_updated: March 27, 2025
version: 1.0
---
Please implement a comprehensive unit testing suite for my Navius. Following these specific instructions, continue building upon our existing progress:

BEFORE STARTING READ THIS WEBSITE FOR REFERENCE: https://doc.rust-lang.org/book/ch11-03-test-organization.html

1. DIRECTORY STRUCTURE:
   - ✅ We've already KEPT the /test/resources folder with Bruno tests
   - ✅ We've already restructured the tests directory following Rust conventions
   - Current structure:
     * Unit tests inside `#[cfg(test)]` modules in /src/core files
     * Integration tests in /tests directory
     * Common test utilities in /tests/common/mod.rs

2. TEST IMPLEMENTATION APPROACH:
   - Focus ONLY on testing /src/core functionality (NOT other parts of /src)
   - For each implementation file in /src/core:
     * Make tests thorough, but concise - one clear assertion per test where possible
     * Ensure test names clearly describe what they're testing and expected behavior
     * Use table-driven tests for similar test cases (avoid repetitive test code)
   - Use the standard Rust test framework and tokio for async tests
   - Refer to the testing roadmap in docs/testing-roadmap.md and update it as you make progress

3. SPECIFIC MODULES TO TEST (Prioritized):
   - Complete testing for these core modules in priority order:
     1. /src/core/utils (start here - foundation for other modules)
     2. /src/core/error (partially done with error_types.rs - complete error handling tests)
     3. /src/core/router (partially done with core_router.rs - add more tests for routes)
     4. /src/core/auth (critical for security)
     5. /src/core/handlers (build on previous modules)
   - Aim for at least 80% test coverage for each module, with 100% for critical paths
   - Use `cargo test --doc` to ensure documentation examples are also tested

4. TEST TYPES TO IMPLEMENT:
   - Unit tests:
     * Pure function tests (test inputs/outputs without mocks)
     * Boundary tests (empty collections, max values, etc.)
     * Error cases (ensure proper error handling)
   - Tests with dependencies:
     * Use mocking libraries like mockall for complex dependencies
     * Create simple mock implementations for straightforward dependencies
     * Test with both happy path and error path scenarios
   - Integration tests:
     * Focus on module interactions
     * Test representative end-to-end flows

5. COMMON TEST UTILITIES:
   - ✅ We've created a foundation in /tests/common/mod.rs
   - Extend this with:
     * More specialized mock implementations for different services
     * Fixtures for different test scenarios (auth scenarios, error cases, etc.)
     * Utility functions to reduce boilerplate in test files

6. EXAMPLE TEST PATTERN:
   For each module, follow this pattern:
   ```rust
   // At the bottom of each implementation file
   #[cfg(test)]
   mod tests {
       use super::*;
       // Import test utilities
       
       // Group tests logically by function or feature
       
       // For simple functionality
       #[test]
       fn test_function_name_expected_behavior() {
           // Arrange: Set up test data and expectations
           let input = "test_input";
           let expected = "expected_result";
           
           // Act: Call the function being tested
           let result = function_name(input);
           
           // Assert: Verify the results
           assert_eq!(result, expected);
       }
       
       // For async functionality
       #[tokio::test]
       async fn test_async_function_expected_behavior() {
           // Similar pattern with async/await
       }
       
       // For error cases
       #[test]
       fn test_function_name_error_case() {
           // Test behavior when errors occur
       }
   }
   ```

7. INTEGRATION TEST PATTERN:
   ```rust
   // In /tests/module_name_tests.rs
   use navius::core::module_name::{Function, Type};
   use navius::core::related_module::{RelatedFunction};
   mod common;
   
   #[tokio::test]
   async fn test_integration_scenario() {
       // Arrange: Set up test environment and data
       let mock = common::create_mock_service();
       
       // Act: Perform the integration action that crosses module boundaries
       let result = Function::process_with_dependencies(mock).await;
       
       // Assert: Verify expected integration behavior
       assert!(result.is_ok());
       assert_eq!(result.unwrap().status, "success");
   }
   ```

8. DOCUMENTATION AND CI:
   - Update the README.md with:
     * Clear instructions for running different test categories
     * Example: `cargo test` for all tests, `cargo test module_name` for specific modules
     * Explanation of test structure and organization
   - Configure GitHub Actions or GitLab CI with:
     * Unit test jobs
     * Integration test jobs
     * Code coverage reporting with minimum thresholds
     * Consider adding cargo-tarpaulin for coverage calculation

Please implement this testing plan step by step, starting with the priority modules and updating the testing roadmap as you progress. Remember to use existing code patterns and test examples as a reference. Also, limit editing of this prompt and the testing-roadmap.md file to only update progress (unless you have a really good reason to. In which case ask me about it).

## Related Documents
- [Contributing Guide](./) - How to contribute to the project
- [Development Setup](./) - Setting up your development environment
