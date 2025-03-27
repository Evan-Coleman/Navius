/// Test modules for Navius application
///
/// This module contains all tests for the application.
/// It is organized into submodules for different types of tests:
/// - integration: Integration tests that test components together
/// - common: Common test utilities and fixtures
/// - mocks: Mock implementations of traits for testing
// Re-export common test utilities
pub mod common;

// Include integration tests
pub mod integration;
