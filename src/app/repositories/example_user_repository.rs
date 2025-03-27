use std::error::Error;
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app::User;
use crate::app::UserRole;
use crate::core::models::{Entity, Repository, RepositoryConfig};
use crate::core::services::error::ServiceError;
use crate::core::services::memory_repository::InMemoryRepositoryProvider;
use crate::core::services::repository_service::RepositoryService;
use crate::core::services::service_traits::Lifecycle;

/// Repository for managing user entities
#[derive(Debug)]
pub struct UserRepository {
    /// Inner repository implementation
    inner: Arc<dyn Repository<User>>,
}

impl UserRepository {
    /// Create a new user repository
    pub async fn new(repository_service: &RepositoryService) -> Result<Self, ServiceError> {
        // Create default repository config for User entity
        let config = RepositoryConfig {
            provider: "memory".to_string(),
            collection_name: Some(User::collection_name()),
            ..Default::default()
        };

        // Use the InMemoryRepositoryProvider to create the repository with the updated method signature
        let boxed_repo = repository_service
            .create_repository::<User, InMemoryRepositoryProvider>(Some(config))
            .await?;

        // Wrap the raw Box<dyn Repository<User>> in an Arc
        Ok(Self {
            inner: Arc::from(boxed_repo),
        })
    }

    /// Find user by username
    pub async fn find_by_username(&self, username: &str) -> Result<Option<User>, ServiceError> {
        let all_users = self.inner.find_all().await?;
        Ok(all_users.into_iter().find(|u| u.username == username))
    }

    /// Find users by role
    pub async fn find_by_role(&self, role: UserRole) -> Result<Vec<User>, ServiceError> {
        let all_users = self.inner.find_all().await?;
        Ok(all_users.into_iter().filter(|u| u.role == role).collect())
    }

    /// Find active users
    pub async fn find_active_users(&self) -> Result<Vec<User>, ServiceError> {
        let all_users = self.inner.find_all().await?;
        Ok(all_users.into_iter().filter(|u| u.active).collect())
    }
}

// Delegate core repository operations to the inner repository
#[async_trait]
impl Repository<User> for UserRepository {
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<User>, ServiceError> {
        self.inner.find_by_id(id).await
    }

    async fn find_all(&self) -> Result<Vec<User>, ServiceError> {
        self.inner.find_all().await
    }

    async fn save(&self, entity: &User) -> Result<User, ServiceError> {
        self.inner.save(entity).await
    }

    async fn delete(&self, id: &Uuid) -> Result<bool, ServiceError> {
        self.inner.delete(id).await
    }

    async fn count(&self) -> Result<usize, ServiceError> {
        self.inner.count().await
    }

    async fn exists(&self, id: &Uuid) -> Result<bool, ServiceError> {
        self.inner.exists(id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::services::Lifecycle;
    use tokio::test;

    #[test]
    async fn test_user_repository() {
        // Create a repository service
        let repository_service = RepositoryService::new();
        repository_service.init().await.unwrap(); // Initialize to register the memory provider

        // Create a user repository
        let repo = UserRepository::new(&repository_service).await.unwrap();

        // Create a test user
        let user = User::new(
            "testrepo".to_string(),
            "repo@example.com".to_string(),
            "Test Repo User".to_string(),
        )
        .with_role(UserRole::Admin);

        // Save the user
        let saved_user = repo.save(&user).await.unwrap();

        // Find by ID
        let found_user = repo.find_by_id(&saved_user.id).await.unwrap();
        assert!(found_user.is_some());
        assert_eq!(found_user.unwrap().username, "testrepo");

        // Find by username
        let found_by_username = repo.find_by_username("testrepo").await.unwrap();
        assert!(found_by_username.is_some());

        // Find by role
        let admins = repo.find_by_role(UserRole::Admin).await.unwrap();
        assert_eq!(admins.len(), 1);
        assert_eq!(admins[0].username, "testrepo");

        // Find active users
        let active_users = repo.find_active_users().await.unwrap();
        assert_eq!(active_users.len(), 1);

        // Count users
        let count = repo.count().await.unwrap();
        assert_eq!(count, 1);

        // Delete user
        let deleted = repo.delete(&saved_user.id).await.unwrap();
        assert!(deleted);

        // Verify deleted
        let count_after_delete = repo.count().await.unwrap();
        assert_eq!(count_after_delete, 0);
    }
}
