//! Tests for the repository module
//!
//! This module provides tests for repository implementations.

use std::sync::Arc;
use uuid::Uuid;

use crate::core::config::app_config::DatabaseConfig;
use crate::core::database::connection::MockDatabaseConnection;
use crate::repository::{Repository, User, UserRepository, models::UserRole};

/// Create a test database pool
fn create_test_db_pool() -> Arc<Box<dyn crate::core::database::PgPool>> {
    let mut config = DatabaseConfig::default();
    config.enabled = true;

    let conn = MockDatabaseConnection::new(&config);
    Arc::new(Box::new(conn) as Box<dyn crate::core::database::PgPool>)
}

#[tokio::test]
async fn test_user_repository_crud() {
    // Create a test DB pool
    let pool = create_test_db_pool();

    // Create a user repository
    let repo = UserRepository::new(pool);

    // Initially, no users exist
    let all_users = repo.find_all().await.unwrap();
    assert_eq!(all_users.len(), 0);

    let count = repo.count().await.unwrap();
    assert_eq!(count, 0);

    // Create a new user
    let mut user = User::new("testuser".to_string(), "test@example.com".to_string());
    user.full_name = Some("Test User".to_string());

    // Save the user
    let saved_user = repo.save(user.clone()).await.unwrap();
    assert_eq!(saved_user.id, user.id);
    assert_eq!(saved_user.username, "testuser");
    assert_eq!(saved_user.email, "test@example.com");
    assert_eq!(saved_user.full_name, Some("Test User".to_string()));
    assert_eq!(saved_user.role, UserRole::User);
    assert!(saved_user.is_active);

    // Verify count increased
    let count = repo.count().await.unwrap();
    assert_eq!(count, 1);

    // Find the user by ID
    let found_user = repo.find_by_id(user.id).await.unwrap();
    assert!(found_user.is_some());
    let found_user = found_user.unwrap();
    assert_eq!(found_user.id, user.id);
    assert_eq!(found_user.username, "testuser");

    // Find the user by username
    let found_by_username = repo.find_by_username("testuser").await.unwrap();
    assert!(found_by_username.is_some());
    assert_eq!(found_by_username.unwrap().id, user.id);

    // Find the user by email
    let found_by_email = repo.find_by_email("test@example.com").await.unwrap();
    assert!(found_by_email.is_some());
    assert_eq!(found_by_email.unwrap().id, user.id);

    // Update the user
    let mut user_to_update = found_user.clone();
    user_to_update.full_name = Some("Updated User".to_string());
    user_to_update.role = UserRole::Admin;

    let updated_user = repo.save(user_to_update).await.unwrap();
    assert_eq!(updated_user.id, user.id);
    assert_eq!(updated_user.full_name, Some("Updated User".to_string()));
    assert_eq!(updated_user.role, UserRole::Admin);

    // Verify the update was saved
    let found_again = repo.find_by_id(user.id).await.unwrap();
    assert!(found_again.is_some());
    let found_again = found_again.unwrap();
    assert_eq!(found_again.full_name, Some("Updated User".to_string()));
    assert_eq!(found_again.role, UserRole::Admin);

    // Delete the user
    let deleted = repo.delete(user.id).await.unwrap();
    assert!(deleted);

    // Verify the user is gone
    let not_found = repo.find_by_id(user.id).await.unwrap();
    assert!(not_found.is_none());

    // Verify count decreased
    let count = repo.count().await.unwrap();
    assert_eq!(count, 0);
}

#[tokio::test]
async fn test_user_repository_multiple_users() {
    // Create a test DB pool
    let pool = create_test_db_pool();

    // Create a user repository
    let repo = UserRepository::new(pool);

    // Create several users
    for i in 1..=5 {
        let user = User::new(format!("user{}", i), format!("user{}@example.com", i));

        repo.save(user).await.unwrap();
    }

    // Verify count
    let count = repo.count().await.unwrap();
    assert_eq!(count, 5);

    // Get all users
    let all_users = repo.find_all().await.unwrap();
    assert_eq!(all_users.len(), 5);

    // Verify usernames
    let usernames: Vec<String> = all_users.iter().map(|u| u.username.clone()).collect();

    assert!(usernames.contains(&"user1".to_string()));
    assert!(usernames.contains(&"user2".to_string()));
    assert!(usernames.contains(&"user3".to_string()));
    assert!(usernames.contains(&"user4".to_string()));
    assert!(usernames.contains(&"user5".to_string()));

    // Find a specific user
    let user3 = repo.find_by_username("user3").await.unwrap();
    assert!(user3.is_some());
    let user3 = user3.unwrap();
    assert_eq!(user3.username, "user3");
    assert_eq!(user3.email, "user3@example.com");

    // Delete all users
    for user in all_users {
        repo.delete(user.id).await.unwrap();
    }

    // Verify all users are gone
    let final_count = repo.count().await.unwrap();
    assert_eq!(final_count, 0);
}

#[tokio::test]
async fn test_user_role_serialization() {
    assert_eq!(UserRole::Admin.to_string(), "admin");
    assert_eq!(UserRole::User.to_string(), "user");
    assert_eq!(UserRole::ReadOnly.to_string(), "readonly");

    let default_role = UserRole::default();
    assert_eq!(default_role, UserRole::User);
}

#[tokio::test]
async fn test_user_creation() {
    let user = User::new("newuser".to_string(), "newuser@example.com".to_string());

    assert_eq!(user.username, "newuser");
    assert_eq!(user.email, "newuser@example.com");
    assert_eq!(user.full_name, None);
    assert!(user.is_active);
    assert_eq!(user.role, UserRole::User);

    // UUID should be valid
    assert!(!user.id.is_nil());

    // Created and updated timestamps should be the same initially
    assert_eq!(user.created_at, user.updated_at);
}

#[tokio::test]
async fn test_user_touch() {
    let mut user = User::new("touchuser".to_string(), "touch@example.com".to_string());

    let initial_updated_at = user.updated_at;

    // Wait a moment to ensure timestamp will be different
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;

    // Touch the user
    user.touch();

    // updated_at should have changed
    assert!(user.updated_at > initial_updated_at);

    // created_at should remain the same
    assert_eq!(user.created_at, initial_updated_at);
}
