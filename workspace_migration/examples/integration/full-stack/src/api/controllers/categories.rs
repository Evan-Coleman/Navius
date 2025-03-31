use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use chrono::Utc;
use navius_core::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::api::middleware::CurrentUser;
use crate::api::models::{PaginatedResponse, PaginationParams, SortParams};
use crate::application::CategoryService;
use crate::infrastructure::ServiceRegistry;

/// Category listing query parameters
///
/// Used for filtering, sorting, and paginating category list requests.
#[derive(Debug, Deserialize)]
pub struct CategoryListParams {
    /// Pagination parameters (page number and items per page)
    #[serde(flatten)]
    pub pagination: PaginationParams,
    /// Sorting parameters (field to sort by and sort order)
    #[serde(flatten)]
    pub sort: SortParams,
    /// Filter by parent category ID (optional)
    pub parent_id: Option<String>,
    /// Filter by creator user ID
    pub created_by: Option<String>,
    /// Filter active/inactive categories (true/false)
    pub active: Option<bool>,
}

/// Category response data
///
/// Contains all category information returned by the API.
#[derive(Debug, Serialize)]
pub struct CategoryResponse {
    /// Unique category identifier
    pub id: String,
    /// Category name
    pub name: String,
    /// Detailed category description (optional)
    pub description: Option<String>,
    /// Parent category ID (optional)
    pub parent_id: Option<String>,
    /// User ID of person who created the category
    pub created_by: String,
    /// Whether the category is active (true) or inactive (false)
    pub active: bool,
    /// Category color (optional)
    pub color: Option<String>,
    /// Category icon (optional)
    pub icon: Option<String>,
    /// Category creation timestamp in RFC3339 format
    pub created_at: String,
    /// Category last update timestamp in RFC3339 format
    pub updated_at: String,
}

/// Create category request
///
/// Contains information needed to create a new category.
#[derive(Debug, Deserialize)]
pub struct CreateCategoryRequest {
    /// Category name
    pub name: String,
    /// Detailed category description (optional)
    pub description: Option<String>,
    /// Parent category ID (optional)
    pub parent_id: Option<String>,
    /// Whether the category is active (defaults to true if not specified)
    pub active: Option<bool>,
    /// Category color (optional)
    pub color: Option<String>,
    /// Category icon (optional)
    pub icon: Option<String>,
}

/// Update category request
///
/// Contains information that can be updated for an existing category.
#[derive(Debug, Deserialize)]
pub struct UpdateCategoryRequest {
    /// New category name (optional)
    pub name: Option<String>,
    /// New category description (optional)
    pub description: Option<String>,
    /// New parent category ID (optional)
    pub parent_id: Option<String>,
    /// New active status (optional)
    pub active: Option<bool>,
    /// New category color (optional)
    pub color: Option<String>,
    /// New category icon (optional)
    pub icon: Option<String>,
}

/// Convert a domain Category to a CategoryResponse
///
/// Converts an internal Category domain object to the API response format.
fn map_category_to_response(category: &crate::domain::Category) -> CategoryResponse {
    CategoryResponse {
        id: category.id.to_string(),
        name: category.name.clone(),
        description: category.description.clone(),
        parent_id: category.parent_id.map(|id| id.to_string()),
        created_by: category.created_by.to_string(),
        active: category.active,
        color: category.color.clone(),
        icon: category.icon.clone(),
        created_at: category.created_at.to_rfc3339(),
        updated_at: category.updated_at.to_rfc3339(),
    }
}

/// Get all categories with filtering and pagination
///
/// Returns a paginated list of categories based on the provided filters.
/// Requires authentication.
///
/// # Errors
/// - Returns `ValidationError` if filter parameters are invalid
/// - Returns `InternalServerError` if there's an issue retrieving categories
pub async fn get_categories(
    State(registry): State<Arc<ServiceRegistry>>,
    Query(params): Query<CategoryListParams>,
    _current_user: CurrentUser, // Ensure user is authenticated
) -> Result<Json<PaginatedResponse<CategoryResponse>>> {
    let category_service = registry.category_service();

    // Parse parent_id if provided
    let parent_id = match &params.parent_id {
        Some(id_str) => Some(
            Uuid::parse_str(id_str)
                .map_err(|_| Error::validation_error("Invalid UUID for parent_id"))?,
        ),
        None => None,
    };

    // Parse created_by if provided
    let created_by = match &params.created_by {
        Some(id_str) => Some(
            Uuid::parse_str(id_str)
                .map_err(|_| Error::validation_error("Invalid UUID for created_by"))?,
        ),
        None => None,
    };

    // Get categories with filter
    let (categories, total_count) = category_service
        .get_categories_with_filter(
            parent_id,
            created_by,
            params.active,
            params.pagination.page,
            params.pagination.per_page,
            params.sort.sort,
            params.sort.order,
        )
        .await
        .map_err(|e| {
            Error::internal_server_error(format!("Failed to get categories: {}", e.message))
        })?;

    // Map to response format
    let category_responses = categories.iter().map(map_category_to_response).collect();

    // Create paginated response
    let response =
        PaginatedResponse::from_params(category_responses, &params.pagination, total_count);

    Ok(Json(response))
}

/// Get category by ID
///
/// Returns a specific category by its ID.
/// Requires authentication.
///
/// # Errors
/// - Returns `ValidationError` if the category ID is invalid
/// - Returns `NotFound` if the category doesn't exist
/// - Returns `InternalServerError` if there's an issue retrieving the category
pub async fn get_category(
    State(registry): State<Arc<ServiceRegistry>>,
    Path(id_str): Path<String>,
    _current_user: CurrentUser, // Ensure user is authenticated
) -> Result<Json<CategoryResponse>> {
    let category_service = registry.category_service();

    // Parse category ID
    let id = Uuid::parse_str(&id_str)
        .map_err(|_| Error::validation_error("Invalid category ID format"))?;

    // Get category
    let category = category_service
        .get_category(id)
        .await
        .map_err(|e| match e.kind {
            crate::application::CategoryServiceErrorKind::NotFound => {
                Error::not_found("Category not found")
            }
            _ => Error::internal_server_error(format!("Failed to get category: {}", e.message)),
        })?;

    Ok(Json(map_category_to_response(&category)))
}

/// Create a new category
///
/// Creates a new category with the provided details.
/// Requires authentication. The authenticated user becomes the category creator.
/// Only managers can create categories.
///
/// # Errors
/// - Returns `ValidationError` if the request data is invalid
/// - Returns `Forbidden` if the user doesn't have permission to create categories
/// - Returns `InternalServerError` if there's an issue creating the category
pub async fn create_category(
    State(registry): State<Arc<ServiceRegistry>>,
    Json(request): Json<CreateCategoryRequest>,
    current_user: CurrentUser, // Get current user from authentication
) -> Result<Json<CategoryResponse>> {
    let category_service = registry.category_service();

    // Get current user ID from the authentication
    let current_user_id = current_user.0;

    // Parse parent_id if provided
    let parent_id = match &request.parent_id {
        Some(id_str) => Some(
            Uuid::parse_str(id_str)
                .map_err(|_| Error::validation_error("Invalid UUID for parent_id"))?,
        ),
        None => None,
    };

    // Create category
    let category = category_service
        .create_category(
            request.name,
            request.description,
            parent_id,
            request.active.unwrap_or(true),
            request.color,
            request.icon,
            current_user_id,
        )
        .await
        .map_err(|e| match e.kind {
            crate::application::CategoryServiceErrorKind::PermissionDenied => {
                Error::forbidden(e.message)
            }
            crate::application::CategoryServiceErrorKind::ParentNotFound => {
                Error::not_found("Parent category not found")
            }
            crate::application::CategoryServiceErrorKind::ValidationError => {
                Error::validation_error(e.message)
            }
            _ => Error::internal_server_error(format!("Failed to create category: {}", e.message)),
        })?;

    Ok(Json(map_category_to_response(&category)))
}

/// Update an existing category
///
/// Updates the specified category with the provided details.
/// All fields are optional - only the provided fields will be updated.
/// Requires authentication and manager role to modify categories.
///
/// # Errors
/// - Returns `ValidationError` if the request data is invalid
/// - Returns `NotFound` if the category doesn't exist
/// - Returns `Forbidden` if the current user doesn't have permission
/// - Returns `InternalServerError` if there's an issue updating the category
pub async fn update_category(
    State(registry): State<Arc<ServiceRegistry>>,
    Path(id_str): Path<String>,
    Json(request): Json<UpdateCategoryRequest>,
    current_user: CurrentUser, // Get current user from authentication
) -> Result<Json<CategoryResponse>> {
    let category_service = registry.category_service();

    // Get current user ID from the authentication
    let current_user_id = current_user.0;

    // Parse category ID
    let id = Uuid::parse_str(&id_str)
        .map_err(|_| Error::validation_error("Invalid category ID format"))?;

    // Parse parent_id if provided
    let parent_id = match &request.parent_id {
        Some(id_str) => Some(
            Uuid::parse_str(id_str)
                .map_err(|_| Error::validation_error("Invalid UUID for parent_id"))?,
        ),
        None => None,
    };

    // Update category
    let category = category_service
        .update_category(
            id,
            request.name,
            request.description,
            parent_id,
            request.active,
            request.color,
            request.icon,
            current_user_id,
        )
        .await
        .map_err(|e| match e.kind {
            crate::application::CategoryServiceErrorKind::NotFound => {
                Error::not_found("Category not found")
            }
            crate::application::CategoryServiceErrorKind::PermissionDenied => {
                Error::forbidden(e.message)
            }
            crate::application::CategoryServiceErrorKind::ParentNotFound => {
                Error::not_found("Parent category not found")
            }
            crate::application::CategoryServiceErrorKind::ValidationError => {
                Error::validation_error(e.message)
            }
            _ => Error::internal_server_error(format!("Failed to update category: {}", e.message)),
        })?;

    Ok(Json(map_category_to_response(&category)))
}

/// Delete a category
///
/// Deletes the specified category.
/// Only managers can delete categories.
/// Categories with subcategories or associated tasks cannot be deleted.
///
/// # Errors
/// - Returns `ValidationError` if the category ID is invalid
/// - Returns `NotFound` if the category doesn't exist
/// - Returns `Forbidden` if the current user doesn't have permission
/// - Returns `BadRequest` if the category has subcategories or tasks
/// - Returns `InternalServerError` if there's an issue deleting the category
pub async fn delete_category(
    State(registry): State<Arc<ServiceRegistry>>,
    Path(id_str): Path<String>,
    current_user: CurrentUser, // Get current user from authentication
) -> Result<StatusCode> {
    let category_service = registry.category_service();

    // Get current user ID from the authentication
    let current_user_id = current_user.0;

    // Parse category ID
    let id = Uuid::parse_str(&id_str)
        .map_err(|_| Error::validation_error("Invalid category ID format"))?;

    // Delete category
    category_service
        .delete_category(id, current_user_id)
        .await
        .map_err(|e| match e.kind {
            crate::application::CategoryServiceErrorKind::NotFound => {
                Error::not_found("Category not found")
            }
            crate::application::CategoryServiceErrorKind::PermissionDenied => {
                Error::forbidden(e.message)
            }
            crate::application::CategoryServiceErrorKind::CategoryHasChildren => {
                Error::bad_request("Cannot delete category with subcategories")
            }
            crate::application::CategoryServiceErrorKind::CategoryHasTasks => {
                Error::bad_request("Cannot delete category with associated tasks")
            }
            _ => Error::internal_server_error(format!("Failed to delete category: {}", e.message)),
        })?;

    Ok(StatusCode::NO_CONTENT)
}

/// Get subcategories for a category
///
/// Returns a list of child categories for the specified parent category.
/// Requires authentication.
///
/// # Errors
/// - Returns `ValidationError` if the category ID is invalid
/// - Returns `NotFound` if the parent category doesn't exist
/// - Returns `InternalServerError` if there's an issue retrieving the subcategories
pub async fn get_subcategories(
    State(registry): State<Arc<ServiceRegistry>>,
    Path(id_str): Path<String>,
    _current_user: CurrentUser, // Ensure user is authenticated
) -> Result<Json<Vec<CategoryResponse>>> {
    let category_service = registry.category_service();

    // Parse category ID
    let id = Uuid::parse_str(&id_str)
        .map_err(|_| Error::validation_error("Invalid category ID format"))?;

    // Get category to ensure it exists
    category_service
        .get_category(id)
        .await
        .map_err(|e| match e.kind {
            crate::application::CategoryServiceErrorKind::NotFound => {
                Error::not_found("Category not found")
            }
            _ => Error::internal_server_error(format!("Failed to get category: {}", e.message)),
        })?;

    // Get subcategories
    let subcategories = category_service.get_subcategories(id).await.map_err(|e| {
        Error::internal_server_error(format!("Failed to get subcategories: {}", e.message))
    })?;

    // Map to response format
    let subcategory_responses = subcategories.iter().map(map_category_to_response).collect();

    Ok(Json(subcategory_responses))
}
