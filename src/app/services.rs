//! # Services module
//!
//! This module contains service implementations
//! that can be customized by users.

/// Example user service module
pub mod example_user_service;

// Make example types available but with prefixes
pub use example_user_service::CreateUserInput as ExampleCreateUserInput;
pub use example_user_service::UpdateUserInput as ExampleUpdateUserInput;
pub use example_user_service::UserOutput as ExampleUserOutput;
pub use example_user_service::UserService as ExampleUserService;
