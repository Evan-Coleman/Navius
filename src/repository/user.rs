//! User repository implementation
//!
//! This module provides an implementation of the Repository trait for User entities.

use async_trait::async_trait;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::sync::RwLock;
use uuid::Uuid;

use super::{BaseRepository, Repository, User};
use crate::core::database::PgPool;
use crate::core::error::AppError;

const USER_DATA_FILE: &str = "user_data.json";

/// User repository for storing and retrieving users
pub struct UserRepository {
    base: BaseRepository,

    // In-memory storage for users until real database is implemented
    // This would be replaced by actual database queries in a real implementation
    users: Arc<RwLock<HashMap<Uuid, User>>>,
}

impl UserRepository {
    /// Create a new user repository
    pub fn new(db_pool: Arc<Box<dyn PgPool>>) -> Self {
        // Try to load users from the data file if it exists
        let users = Self::load_users_from_file().unwrap_or_else(|_| HashMap::new());

        let repo = Self {
            base: BaseRepository::new(db_pool),
            users: Arc::new(RwLock::new(users)),
        };

        repo
    }

    /// Load users from file
    fn load_users_from_file() -> Result<HashMap<Uuid, User>, AppError> {
        if !Path::new(USER_DATA_FILE).exists() {
            return Ok(HashMap::new());
        }

        let data = fs::read_to_string(USER_DATA_FILE).map_err(|e| {
            AppError::DatabaseError(format!("Failed to read user data file: {}", e))
        })?;

        let users: HashMap<Uuid, User> = serde_json::from_str(&data)
            .map_err(|e| AppError::DatabaseError(format!("Failed to parse user data: {}", e)))?;

        Ok(users)
    }

    /// Save users to file
    fn save_users_to_file(&self) -> Result<(), AppError> {
        let users = self
            .users
            .read()
            .map_err(|e| AppError::DatabaseError(format!("Failed to read users: {}", e)))?;

        let data = serde_json::to_string(&*users).map_err(|e| {
            AppError::DatabaseError(format!("Failed to serialize user data: {}", e))
        })?;

        fs::write(USER_DATA_FILE, data).map_err(|e| {
            AppError::DatabaseError(format!("Failed to write user data file: {}", e))
        })?;

        Ok(())
    }

    /// Find a user by username
    pub async fn find_by_username(&self, username: &str) -> Result<Option<User>, AppError> {
        // In a real implementation, this would be a query like:
        // SELECT * FROM users WHERE username = $1

        let users = self
            .users
            .read()
            .map_err(|e| AppError::DatabaseError(format!("Failed to read users: {}", e)))?;

        let user = users.values().find(|u| u.username == username).cloned();

        Ok(user)
    }

    /// Find a user by email
    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        // In a real implementation, this would be a query like:
        // SELECT * FROM users WHERE email = $1

        let users = self
            .users
            .read()
            .map_err(|e| AppError::DatabaseError(format!("Failed to read users: {}", e)))?;

        let user = users.values().find(|u| u.email == email).cloned();

        Ok(user)
    }

    /// Get the number of users
    pub async fn count(&self) -> Result<usize, AppError> {
        let users = self
            .users
            .read()
            .map_err(|e| AppError::DatabaseError(format!("Failed to read users: {}", e)))?;

        Ok(users.len())
    }
}

#[async_trait]
impl Repository<User, Uuid> for UserRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, AppError> {
        // In a real implementation, this would be a query like:
        // SELECT * FROM users WHERE id = $1

        let users = self
            .users
            .read()
            .map_err(|e| AppError::DatabaseError(format!("Failed to read users: {}", e)))?;

        let user = users.get(&id).cloned();
        Ok(user)
    }

    async fn save(&self, mut entity: User) -> Result<User, AppError> {
        // In a real implementation, this would be:
        // INSERT INTO users VALUES (...) ON CONFLICT (id) DO UPDATE SET ...

        // Update the updated_at timestamp
        entity.touch();

        let mut users = self
            .users
            .write()
            .map_err(|e| AppError::DatabaseError(format!("Failed to write users: {}", e)))?;

        users.insert(entity.id, entity.clone());

        // Persist to file
        drop(users); // Release the write lock before saving to file
        self.save_users_to_file()?;

        Ok(entity)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, AppError> {
        // In a real implementation, this would be:
        // DELETE FROM users WHERE id = $1

        let mut users = self
            .users
            .write()
            .map_err(|e| AppError::DatabaseError(format!("Failed to write users: {}", e)))?;

        let removed = users.remove(&id).is_some();

        // Persist to file if something was removed
        if removed {
            drop(users); // Release the write lock before saving to file
            self.save_users_to_file()?;
        }

        Ok(removed)
    }

    async fn find_all(&self) -> Result<Vec<User>, AppError> {
        // In a real implementation, this would be:
        // SELECT * FROM users

        let users = self
            .users
            .read()
            .map_err(|e| AppError::DatabaseError(format!("Failed to read users: {}", e)))?;

        let all_users = users.values().cloned().collect();

        Ok(all_users)
    }
}
