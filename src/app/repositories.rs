/// Example user repository implementation
pub mod example_user_repository;

// Make example repository available with prefix
pub use example_user_repository::UserRepository as ExampleUserRepository;
