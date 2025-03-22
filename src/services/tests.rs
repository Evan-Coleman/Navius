//! Tests for the services module
//!
//! This module provides tests for the service implementations.

use std::sync::Arc;
use uuid::Uuid;

use crate::core::config::app_config::DatabaseConfig;
use crate::core::database::connection::MockDatabaseConnection;
use crate::repository::{User, UserRepository, models::UserRole};
use crate::services::user::{CreateUserDto, UpdateUserDto};
use crate::services::{ServiceError, UserService};

/// Create a test user repository
fn create_test_user_repository() -> Arc<UserRepository> {
    let mut config = DatabaseConfig::default();
    config.enabled = true;

    let conn = MockDatabaseConnection::new(&config);
    let pool = Arc::new(Box::new(conn) as Box<dyn crate::core::database::PgPool>);

    Arc::new(UserRepository::new(pool))
}

#[tokio::test]
async fn test_create_user() {
    // Create dependencies
    let repo = create_test_user_repository();
    let service = UserService::new(repo);

    // Create user DTO
    let create_dto = CreateUserDto {
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
        full_name: Some("Test User".to_string()),
        role: None,
    };

    // Create user
    let user = service.create_user(create_dto).await.unwrap();

    // Verify user was created correctly
    assert_eq!(user.username, "testuser");
    assert_eq!(user.email, "test@example.com");
    assert_eq!(user.full_name, Some("Test User".to_string()));
    assert_eq!(user.role, UserRole::User);
    assert!(user.is_active);

    // Verify count
    let count = service.count_users().await.unwrap();
    assert_eq!(count, 1);
}

#[tokio::test]
async fn test_create_user_validation() {
    // Create dependencies
    let repo = create_test_user_repository();
    let service = UserService::new(repo);

    // Try to create user with invalid username
    let create_dto = CreateUserDto {
        username: "ab".to_string(), // Too short - should be at least 3 chars
        email: "test@example.com".to_string(),
        full_name: None,
        role: None,
    };

    let result = service.create_user(create_dto).await;
    assert!(result.is_err());
    match result {
        Err(ServiceError::Validation(msg)) => {
            assert!(msg.contains("at least 3 characters"));
        }
        _ => panic!("Expected validation error"),
    }

    // Try to create user with invalid email
    let create_dto = CreateUserDto {
        username: "validuser".to_string(),
        email: "invalid-email".to_string(), // Missing @ and domain
        full_name: None,
        role: None,
    };

    let result = service.create_user(create_dto).await;
    assert!(result.is_err());
    match result {
        Err(ServiceError::Validation(msg)) => {
            assert!(msg.contains("Invalid email format"));
        }
        _ => panic!("Expected validation error"),
    }
}

#[tokio::test]
async fn test_duplicate_username() {
    // Create dependencies
    let repo = create_test_user_repository();
    let service = UserService::new(repo);

    // Create first user
    let create_dto = CreateUserDto {
        username: "testuser".to_string(),
        email: "test1@example.com".to_string(),
        full_name: None,
        role: None,
    };

    service.create_user(create_dto).await.unwrap();

    // Try to create second user with same username
    let create_dto = CreateUserDto {
        username: "testuser".to_string(),       // Same username
        email: "test2@example.com".to_string(), // Different email
        full_name: None,
        role: None,
    };

    let result = service.create_user(create_dto).await;
    assert!(result.is_err());
    match result {
        Err(ServiceError::UsernameExists) => {}
        _ => panic!("Expected username exists error"),
    }
}

#[tokio::test]
async fn test_duplicate_email() {
    // Create dependencies
    let repo = create_test_user_repository();
    let service = UserService::new(repo);

    // Create first user
    let create_dto = CreateUserDto {
        username: "user1".to_string(),
        email: "same@example.com".to_string(),
        full_name: None,
        role: None,
    };

    service.create_user(create_dto).await.unwrap();

    // Try to create second user with same email
    let create_dto = CreateUserDto {
        username: "user2".to_string(),         // Different username
        email: "same@example.com".to_string(), // Same email
        full_name: None,
        role: None,
    };

    let result = service.create_user(create_dto).await;
    assert!(result.is_err());
    match result {
        Err(ServiceError::EmailExists) => {}
        _ => panic!("Expected email exists error"),
    }
}

#[tokio::test]
async fn test_get_user() {
    // Create dependencies
    let repo = create_test_user_repository();
    let service = UserService::new(repo);

    // Create user
    let create_dto = CreateUserDto {
        username: "getuser".to_string(),
        email: "get@example.com".to_string(),
        full_name: Some("Get User".to_string()),
        role: Some(UserRole::Admin),
    };

    let user = service.create_user(create_dto).await.unwrap();
    let user_id = user.id;

    // Get user by ID
    let found = service.get_user_by_id(user_id).await.unwrap();
    assert_eq!(found.id, user_id);
    assert_eq!(found.username, "getuser");
    assert_eq!(found.role, UserRole::Admin);

    // Get user by username
    let found = service.get_user_by_username("getuser").await.unwrap();
    assert_eq!(found.id, user_id);
    assert_eq!(found.email, "get@example.com");

    // Try to get non-existent user
    let result = service.get_user_by_id(Uuid::new_v4()).await;
    assert!(result.is_err());
    match result {
        Err(ServiceError::UserNotFound) => {}
        _ => panic!("Expected user not found error"),
    }
}

#[tokio::test]
async fn test_update_user() {
    // Create dependencies
    let repo = create_test_user_repository();
    let service = UserService::new(repo);

    // Create user
    let create_dto = CreateUserDto {
        username: "updateuser".to_string(),
        email: "update@example.com".to_string(),
        full_name: Some("Original Name".to_string()),
        role: Some(UserRole::User),
    };

    let user = service.create_user(create_dto).await.unwrap();
    let user_id = user.id;

    // Update the user
    let update_dto = UpdateUserDto {
        email: Some("newemail@example.com".to_string()),
        full_name: Some("Updated Name".to_string()),
        is_active: Some(false),
        role: Some(UserRole::ReadOnly),
    };

    let updated = service.update_user(user_id, update_dto).await.unwrap();

    // Verify updates
    assert_eq!(updated.id, user_id);
    assert_eq!(updated.email, "newemail@example.com");
    assert_eq!(updated.full_name, Some("Updated Name".to_string()));
    assert_eq!(updated.is_active, false);
    assert_eq!(updated.role, UserRole::ReadOnly);

    // Username should not change
    assert_eq!(updated.username, "updateuser");

    // Get the user again to verify persistence
    let found = service.get_user_by_id(user_id).await.unwrap();
    assert_eq!(found.email, "newemail@example.com");
    assert_eq!(found.is_active, false);
}

#[tokio::test]
async fn test_delete_user() {
    // Create dependencies
    let repo = create_test_user_repository();
    let service = UserService::new(repo);

    // Create user
    let create_dto = CreateUserDto {
        username: "deleteuser".to_string(),
        email: "delete@example.com".to_string(),
        full_name: None,
        role: None,
    };

    let user = service.create_user(create_dto).await.unwrap();
    let user_id = user.id;

    // Delete the user
    service.delete_user(user_id).await.unwrap();

    // Verify user was deleted
    let result = service.get_user_by_id(user_id).await;
    assert!(result.is_err());

    // Verify count is zero
    let count = service.count_users().await.unwrap();
    assert_eq!(count, 0);

    // Try to delete non-existent user
    let result = service.delete_user(Uuid::new_v4()).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_all_users() {
    // Create dependencies
    let repo = create_test_user_repository();
    let service = UserService::new(repo);

    // Initially, no users
    let all_users = service.get_all_users().await.unwrap();
    assert_eq!(all_users.len(), 0);

    // Create several users
    for i in 1..=3 {
        let create_dto = CreateUserDto {
            username: format!("user{}", i),
            email: format!("user{}@example.com", i),
            full_name: None,
            role: None,
        };

        service.create_user(create_dto).await.unwrap();
    }

    // Verify all users are returned
    let all_users = service.get_all_users().await.unwrap();
    assert_eq!(all_users.len(), 3);

    // Verify user count
    let count = service.count_users().await.unwrap();
    assert_eq!(count, 3);
}
