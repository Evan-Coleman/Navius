use crate::app::models::user_entity::User;
use crate::core::services::repository_service::{GenericRepository, RepositoryService};
use tokio::test;

#[tokio::test]
async fn test_app_repository_service() {
    // Create repository service
    let repo_service = RepositoryService::new();
    repo_service.init().await.unwrap();

    // Create a user repository
    let user_repo = repo_service
        .create_typed_repository::<User>()
        .await
        .unwrap();

    // Create a test user
    let user = User::new(
        "testuser".to_string(),
        "test@example.com".to_string(),
        "Test User".to_string(),
    );

    // Save the user
    let saved_user = user_repo.save(&user).await.unwrap();
    assert_eq!(saved_user.username, "testuser");

    // Find the user
    let found_user = user_repo.find_by_id(user.id()).await.unwrap();
    assert!(found_user.is_some());
    let found_user = found_user.unwrap();
    assert_eq!(found_user.email, "test@example.com");

    // Count users
    let count = user_repo.count().await.unwrap();
    assert_eq!(count, 1);

    // Delete the user
    let deleted = user_repo.delete(user.id()).await.unwrap();
    assert!(deleted);
}

#[tokio::test]
async fn test_app_generic_repository() {
    // Create repository service
    let repo_service = RepositoryService::new();
    repo_service.init().await.unwrap();

    // Create a generic repository
    let user_repo = GenericRepository::<User>::with_service(&repo_service)
        .await
        .unwrap();

    // Create a test user
    let user = User::new(
        "genericuser".to_string(),
        "generic@example.com".to_string(),
        "Generic User".to_string(),
    );

    // Save the user
    let saved_user = user_repo.save(&user).await.unwrap();
    assert_eq!(saved_user.username, "genericuser");

    // Find the user
    let found_user = user_repo.find_by_id(user.id()).await.unwrap();
    assert!(found_user.is_some());
    let found_user = found_user.unwrap();
    assert_eq!(found_user.email, "generic@example.com");

    // Count users
    let count = user_repo.count().await.unwrap();
    assert_eq!(count, 1);

    // Delete the user
    let deleted = user_repo.delete(user.id()).await.unwrap();
    assert!(deleted);
}
