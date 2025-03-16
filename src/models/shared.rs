use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Basic Pet model to replace the petstore_api model during transition
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Pet {
    pub id: i64,
    pub name: String,
    pub status: Option<String>,
    pub category: Option<Category>,
    pub photo_urls: Vec<String>,
    pub tags: Vec<Tag>,
}

/// Category model
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Category {
    pub id: i64,
    pub name: String,
}

/// Tag model
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Tag {
    pub id: i64,
    pub name: String,
}
