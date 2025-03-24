//! Repository data models
//!
//! This module defines the data models used by repositories.
//! These models represent database entities.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// User entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// Unique identifier
    #[serde(with = "uuid_serde")]
    pub id: Uuid,

    /// Username (unique)
    pub username: String,

    /// Email address (unique)
    pub email: String,

    /// User's full name
    pub full_name: Option<String>,

    /// Whether the user is active
    pub is_active: bool,

    /// User's role (admin, user, etc.)
    pub role: UserRole,

    /// When the user was created
    pub created_at: DateTime<Utc>,

    /// When the user was last updated
    pub updated_at: DateTime<Utc>,
}

impl User {
    /// Create a new user
    pub fn new(username: String, email: String, full_name: Option<String>, role: UserRole) -> Self {
        Self {
            id: Uuid::new_v4(),
            username,
            email,
            full_name,
            is_active: true,
            role,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    /// Update the user's last updated timestamp
    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }
}

/// User role
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum UserRole {
    /// Administrator with full access
    Admin,

    /// Regular user with limited access
    #[default]
    User,

    /// Read-only user
    ReadOnly,
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserRole::Admin => write!(f, "admin"),
            UserRole::User => write!(f, "user"),
            UserRole::ReadOnly => write!(f, "readonly"),
        }
    }
}

impl UserRole {
    /// Convert the role to a string
    pub fn as_str(&self) -> &'static str {
        match self {
            UserRole::Admin => "admin",
            UserRole::User => "user",
            UserRole::ReadOnly => "readonly",
        }
    }
}

// Helper module for Uuid serialization
mod uuid_serde {
    use serde::{self, Deserialize, Deserializer, Serializer};
    use uuid::Uuid;

    pub fn serialize<S>(uuid: &Uuid, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&uuid.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Uuid, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Uuid::parse_str(&s).map_err(serde::de::Error::custom)
    }
}
