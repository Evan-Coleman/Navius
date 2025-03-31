use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use chrono::Utc;
use navius_core::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::api::middleware::CurrentUser;
use crate::infrastructure::ServiceRegistry;

/// Category listing query parameters
#[derive(Debug, Deserialize)]
pub struct CategoryListParams {
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

/// Category response
#[derive(Debug, Serialize)]
pub struct CategoryResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Create category request
#[derive(Debug, Deserialize)]
pub struct CreateCategoryRequest {
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
}

/// Update category request
#[derive(Debug, Deserialize)]
pub struct UpdateCategoryRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub color: Option<String>,
}

/// Get all categories
pub async fn get_categories(
    State(registry): State<Arc<ServiceRegistry>>,
    Query(params): Query<CategoryListParams>,
    _current_user: CurrentUser, // Ensure user is authenticated
) -> Result<Json<Vec<CategoryResponse>>> {
    // Implementation will be added later
    Ok(Json(vec![
        CategoryResponse {
            id: Uuid::new_v4().to_string(),
            name: "Work".to_string(),
            description: Some("Work-related tasks".to_string()),
            color: Some("#FF5733".to_string()),
            created_at: Utc::now().to_rfc3339(),
            updated_at: Utc::now().to_rfc3339(),
        },
        CategoryResponse {
            id: Uuid::new_v4().to_string(),
            name: "Personal".to_string(),
            description: Some("Personal tasks".to_string()),
            color: Some("#33FF57".to_string()),
            created_at: Utc::now().to_rfc3339(),
            updated_at: Utc::now().to_rfc3339(),
        },
    ]))
}

/// Get category by ID
pub async fn get_category(
    State(registry): State<Arc<ServiceRegistry>>,
    Path(id_str): Path<String>,
    _current_user: CurrentUser, // Ensure user is authenticated
) -> Result<Json<CategoryResponse>> {
    // Parse category ID
    let _id = Uuid::parse_str(&id_str)
        .map_err(|_| Error::validation_error("Invalid category ID format"))?;

    // Implementation will be added later
    Ok(Json(CategoryResponse {
        id: id_str,
        name: "Sample Category".to_string(),
        description: Some("Sample category description".to_string()),
        color: Some("#3357FF".to_string()),
        created_at: Utc::now().to_rfc3339(),
        updated_at: Utc::now().to_rfc3339(),
    }))
}

/// Create category
pub async fn create_category(
    State(registry): State<Arc<ServiceRegistry>>,
    Json(request): Json<CreateCategoryRequest>,
    current_user: CurrentUser, // Get current user from authentication
) -> Result<Json<CategoryResponse>> {
    // Implementation will be added later
    let id = Uuid::new_v4().to_string();
    Ok(Json(CategoryResponse {
        id,
        name: request.name,
        description: request.description,
        color: request.color,
        created_at: Utc::now().to_rfc3339(),
        updated_at: Utc::now().to_rfc3339(),
    }))
}

/// Update category
pub async fn update_category(
    State(registry): State<Arc<ServiceRegistry>>,
    Path(id_str): Path<String>,
    Json(request): Json<UpdateCategoryRequest>,
    current_user: CurrentUser, // Get current user from authentication
) -> Result<Json<CategoryResponse>> {
    // Parse category ID
    let _id = Uuid::parse_str(&id_str)
        .map_err(|_| Error::validation_error("Invalid category ID format"))?;

    // Implementation will be added later
    Ok(Json(CategoryResponse {
        id: id_str,
        name: request
            .name
            .unwrap_or_else(|| "Sample Category".to_string()),
        description: request.description,
        color: request.color,
        created_at: Utc::now().to_rfc3339(),
        updated_at: Utc::now().to_rfc3339(),
    }))
}

/// Delete category
pub async fn delete_category(
    State(registry): State<Arc<ServiceRegistry>>,
    Path(id_str): Path<String>,
    current_user: CurrentUser, // Get current user from authentication
) -> Result<StatusCode> {
    // Parse category ID
    let _id = Uuid::parse_str(&id_str)
        .map_err(|_| Error::validation_error("Invalid category ID format"))?;

    // Implementation will be added later
    Ok(StatusCode::NO_CONTENT)
}

/// Get tasks by category
pub async fn get_tasks_by_category(
    State(registry): State<Arc<ServiceRegistry>>,
    Path(id_str): Path<String>,
    _current_user: CurrentUser, // Ensure user is authenticated
) -> Result<Json<Vec<serde_json::Value>>> {
    // Parse category ID
    let _id = Uuid::parse_str(&id_str)
        .map_err(|_| Error::validation_error("Invalid category ID format"))?;

    // Implementation will be added later
    // This will use the tasks service to get tasks by category
    Ok(Json(vec![]))
}
