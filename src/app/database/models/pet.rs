use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pet {
    pub id: Uuid,
    pub name: String,
    pub pet_type: String,
    pub breed: Option<String>,
    pub age: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Pet {
    pub fn new(name: String, pet_type: String, breed: Option<String>, age: Option<i32>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            pet_type,
            breed,
            age,
            created_at: now,
            updated_at: now,
        }
    }
}
