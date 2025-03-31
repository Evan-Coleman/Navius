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
use crate::application::{UserFilter, UserService};
use crate::domain::{Role, UserProfile, UserStatus};
use crate::infrastructure::ServiceRegistry;

/// User listing query parameters
#[derive(Debug, Deserialize)]
pub struct UserListParams {
    #[serde(flatten)]
    pub pagination: PaginationParams,
    #[serde(flatten)]
    pub sort: SortParams,
    pub role: Option<String>,
    pub status: Option<String>,
}

/// User response
#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: String,
    pub username: String,
    pub email: String,
    pub role: String,
    pub status: String,
    pub profile: UserProfileResponse,
    pub created_at: String,
    pub last_login_at: Option<String>,
}

/// User profile response
#[derive(Debug, Serialize)]
pub struct UserProfileResponse {
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub location: Option<String>,
    pub website: Option<String>,
}

/// Create user request
#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub role: Option<String>,
}

/// Update user request
#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub username: Option<String>,
    pub email: Option<String>,
    pub role: Option<String>,
    pub status: Option<String>,
}

/// Update profile request
#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub location: Option<String>,
    pub website: Option<String>,
}

/// Convert a domain Role enum to string
fn role_to_string(role: &Role) -> String {
    match role {
        Role::Admin => "Admin".to_string(),
        Role::Manager => "Manager".to_string(),
        Role::User => "User".to_string(),
    }
}

/// Parse a role string to the domain Role enum
fn parse_role(role_str: &str) -> Result<Role> {
    match role_str.to_lowercase().as_str() {
        "admin" => Ok(Role::Admin),
        "manager" => Ok(Role::Manager),
        "user" => Ok(Role::User),
        _ => Err(Error::validation_error(format!(
            "Invalid role: {}",
            role_str
        ))),
    }
}

/// Convert a domain UserStatus enum to string
fn status_to_string(status: &UserStatus) -> String {
    match status {
        UserStatus::Active => "Active".to_string(),
        UserStatus::Inactive => "Inactive".to_string(),
        UserStatus::Locked => "Locked".to_string(),
    }
}

/// Parse a status string to the domain UserStatus enum
fn parse_status(status_str: &str) -> Result<UserStatus> {
    match status_str.to_lowercase().as_str() {
        "active" => Ok(UserStatus::Active),
        "inactive" => Ok(UserStatus::Inactive),
        "locked" => Ok(UserStatus::Locked),
        _ => Err(Error::validation_error(format!(
            "Invalid status: {}",
            status_str
        ))),
    }
}

/// Map domain User to UserResponse
fn map_user_to_response(user: &crate::domain::User) -> UserResponse {
    UserResponse {
        id: user.id.to_string(),
        username: user.username.clone(),
        email: user.email.clone(),
        role: role_to_string(&user.role),
        status: status_to_string(&user.status),
        profile: UserProfileResponse {
            display_name: user.profile.display_name.clone(),
            avatar_url: user.profile.avatar_url.clone(),
            bio: user.profile.bio.clone(),
            location: user.profile.location.clone(),
            website: user.profile.website.clone(),
        },
        created_at: user.created_at.to_rfc3339(),
        last_login_at: user.last_login_at.map(|dt| dt.to_rfc3339()),
    }
}

/// Get all users
pub async fn get_users(
    State(registry): State<Arc<ServiceRegistry>>,
    Query(params): Query<UserListParams>,
    _current_user: CurrentUser, // Ensure user is authenticated
) -> Result<Json<PaginatedResponse<UserResponse>>> {
    let user_service = registry.user_service();

    // Create filter
    let mut filter = UserFilter::default();

    // Apply filters from query parameters
    if let Some(role_str) = &params.role {
        filter.role = Some(parse_role(role_str)?);
    }

    if let Some(status_str) = &params.status {
        filter.status = Some(parse_status(status_str)?);
    }

    // Set pagination
    filter.page = params.pagination.page;
    filter.limit = params.pagination.per_page;

    // Set sorting
    if let Some(sort_field) = &params.sort.sort {
        filter.sort_by = Some(sort_field.clone());
        filter.sort_direction = params.sort.order.clone();
    }

    // Get users with filter and total count
    let (users, total_count) = user_service
        .get_users_with_count(filter)
        .await
        .map_err(|e| Error::internal_server_error(format!("Failed to get users: {}", e.message)))?;

    // Convert to response format
    let user_responses = users.iter().map(map_user_to_response).collect();

    // Create paginated response
    let response = PaginatedResponse::from_params(user_responses, &params.pagination, total_count);

    Ok(Json(response))
}

/// Get user by ID
pub async fn get_user(
    State(registry): State<Arc<ServiceRegistry>>,
    Path(id_str): Path<String>,
    _current_user: CurrentUser, // Ensure user is authenticated
) -> Result<Json<UserResponse>> {
    let user_service = registry.user_service();

    // Parse user ID
    let id =
        Uuid::parse_str(&id_str).map_err(|_| Error::validation_error("Invalid user ID format"))?;

    // Get user
    let user = user_service.get_user(id).await.map_err(|e| match e.kind {
        crate::application::UserServiceErrorKind::NotFound => Error::not_found("User not found"),
        _ => Error::internal_server_error(format!("Failed to get user: {}", e.message)),
    })?;

    Ok(Json(map_user_to_response(&user)))
}

/// Create user (admin only)
pub async fn create_user(
    State(registry): State<Arc<ServiceRegistry>>,
    Json(request): Json<CreateUserRequest>,
    current_user: CurrentUser, // Ensure user is authenticated
) -> Result<Json<UserResponse>> {
    let user_service = registry.user_service();

    // Parse role if provided
    let role = match &request.role {
        Some(role_str) => Some(parse_role(role_str)?),
        None => None,
    };

    // Create user
    let user = user_service
        .create_user(&request.username, &request.email, &request.password, role)
        .await
        .map_err(|e| match e.kind {
            crate::application::UserServiceErrorKind::ValidationError => {
                Error::validation_error(e.message)
            }
            crate::application::UserServiceErrorKind::EmailAlreadyExists => {
                Error::conflict("Email already in use")
            }
            crate::application::UserServiceErrorKind::UsernameAlreadyExists => {
                Error::conflict("Username already in use")
            }
            _ => Error::internal_server_error(format!("Failed to create user: {}", e.message)),
        })?;

    Ok(Json(map_user_to_response(&user)))
}

/// Update user (admin or own user)
pub async fn update_user(
    State(registry): State<Arc<ServiceRegistry>>,
    Path(id_str): Path<String>,
    Json(request): Json<UpdateUserRequest>,
    current_user: CurrentUser, // Ensure user is authenticated
) -> Result<Json<UserResponse>> {
    let user_service = registry.user_service();

    // Parse user ID
    let id =
        Uuid::parse_str(&id_str).map_err(|_| Error::validation_error("Invalid user ID format"))?;

    // Parse role if provided
    let role = match &request.role {
        Some(role_str) => Some(parse_role(role_str)?),
        None => None,
    };

    // Parse status if provided
    let status = match &request.status {
        Some(status_str) => Some(parse_status(status_str)?),
        None => None,
    };

    // Update user
    let user = user_service
        .update_user(
            id,
            request.username,
            request.email,
            role,
            status,
            current_user.0, // Pass the current user ID for permission check
        )
        .await
        .map_err(|e| match e.kind {
            crate::application::UserServiceErrorKind::NotFound => {
                Error::not_found("User not found")
            }
            crate::application::UserServiceErrorKind::EmailAlreadyExists => {
                Error::conflict("Email already in use")
            }
            crate::application::UserServiceErrorKind::UsernameAlreadyExists => {
                Error::conflict("Username already in use")
            }
            crate::application::UserServiceErrorKind::PermissionDenied => {
                Error::forbidden(e.message)
            }
            _ => Error::internal_server_error(format!("Failed to update user: {}", e.message)),
        })?;

    Ok(Json(map_user_to_response(&user)))
}

/// Delete user (admin only)
pub async fn delete_user(
    State(registry): State<Arc<ServiceRegistry>>,
    Path(id_str): Path<String>,
    current_user: CurrentUser, // Ensure user is authenticated
) -> Result<StatusCode> {
    let user_service = registry.user_service();

    // Parse user ID
    let id =
        Uuid::parse_str(&id_str).map_err(|_| Error::validation_error("Invalid user ID format"))?;

    // Delete user
    user_service
        .delete_user(id, current_user.0)
        .await
        .map_err(|e| match e.kind {
            crate::application::UserServiceErrorKind::NotFound => {
                Error::not_found("User not found")
            }
            crate::application::UserServiceErrorKind::PermissionDenied => {
                Error::forbidden(e.message)
            }
            _ => Error::internal_server_error(format!("Failed to delete user: {}", e.message)),
        })?;

    Ok(StatusCode::NO_CONTENT)
}

/// Update user profile (own user only)
pub async fn update_profile(
    State(registry): State<Arc<ServiceRegistry>>,
    Path(id_str): Path<String>,
    Json(request): Json<UpdateProfileRequest>,
    current_user: CurrentUser, // Ensure user is authenticated
) -> Result<Json<UserProfileResponse>> {
    let user_service = registry.user_service();

    // Parse user ID
    let id =
        Uuid::parse_str(&id_str).map_err(|_| Error::validation_error("Invalid user ID format"))?;

    // Create profile update
    let profile = UserProfile {
        display_name: request.display_name.unwrap_or_else(|| "User".to_string()),
        avatar_url: request.avatar_url,
        bio: request.bio,
        location: request.location,
        website: request.website,
    };

    // Update profile
    let updated_user = user_service
        .update_profile(
            id,
            profile,
            current_user.0, // Pass the current user ID for permission check
        )
        .await
        .map_err(|e| match e.kind {
            crate::application::UserServiceErrorKind::NotFound => {
                Error::not_found("User not found")
            }
            crate::application::UserServiceErrorKind::PermissionDenied => {
                Error::forbidden(e.message)
            }
            _ => Error::internal_server_error(format!("Failed to update profile: {}", e.message)),
        })?;

    Ok(Json(UserProfileResponse {
        display_name: updated_user.profile.display_name,
        avatar_url: updated_user.profile.avatar_url,
        bio: updated_user.profile.bio,
        location: updated_user.profile.location,
        website: updated_user.profile.website,
    }))
}
