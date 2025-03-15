use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Pet {
    pub id: i64,
    pub name: String,
    pub status: Option<String>,
}
