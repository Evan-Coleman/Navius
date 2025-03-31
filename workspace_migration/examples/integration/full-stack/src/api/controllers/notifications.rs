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
use crate::application::NotificationService;
use crate::infrastructure::ServiceRegistry;

/// Notification listing query parameters
///
/// Used for filtering, sorting, and paginating notification list requests.
#[derive(Debug, Deserialize)]
pub struct NotificationListParams {
    /// Pagination parameters (page number and items per page)
    #[serde(flatten)]
    pub pagination: PaginationParams,
    /// Sorting parameters (field to sort by and sort order)
    #[serde(flatten)]
    pub sort: SortParams,
    /// Filter for read/unread notifications
    pub read: Option<bool>,
    /// Filter by notification type
    pub notification_type: Option<String>,
}

/// Notification type enum
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationType {
    TaskAssigned,
    TaskUpdated,
    CommentAdded,
    UserMention,
    SystemAlert,
}

/// Notification response data
///
/// Contains all notification information returned by the API.
#[derive(Debug, Serialize)]
pub struct NotificationResponse {
    /// Unique notification identifier
    pub id: String,
    /// User ID this notification belongs to
    pub user_id: String,
    /// Type of notification (task_assigned, task_updated, comment_added, etc.)
    pub notification_type: String,
    /// Notification title
    pub title: String,
    /// Detailed notification message
    pub message: String,
    /// Whether the notification has been read by the user
    pub read: bool,
    /// Resource ID that this notification relates to (e.g., task ID)
    pub resource_id: Option<String>,
    /// Resource type that this notification relates to (e.g., "task", "comment")
    pub resource_type: Option<String>,
    /// Additional metadata as a JSON object
    pub metadata: Option<serde_json::Value>,
    /// Notification creation timestamp in RFC3339 format
    pub created_at: String,
    /// Notification last update timestamp in RFC3339 format
    pub updated_at: String,
}

/// Create notification request
///
/// Contains information needed to create a new notification.
/// Typically used internally by the system rather than through the API.
#[derive(Debug, Deserialize)]
pub struct CreateNotificationRequest {
    /// User ID this notification belongs to
    pub user_id: String,
    /// Type of notification (task_assigned, task_updated, comment_added, etc.)
    pub notification_type: String,
    /// Notification title
    pub title: String,
    /// Detailed notification message
    pub message: String,
    /// Resource ID that this notification relates to (e.g., task ID)
    pub resource_id: Option<String>,
    /// Resource type that this notification relates to (e.g., "task", "comment")
    pub resource_type: Option<String>,
    /// Additional metadata as a JSON object
    pub metadata: Option<serde_json::Value>,
}

/// Convert a domain Notification to a NotificationResponse
///
/// Converts an internal Notification domain object to the API response format.
fn map_notification_to_response(
    notification: &crate::domain::Notification,
) -> NotificationResponse {
    NotificationResponse {
        id: notification.id.to_string(),
        user_id: notification.user_id.to_string(),
        notification_type: notification.notification_type.clone(),
        title: notification.title.clone(),
        message: notification.message.clone(),
        read: notification.read,
        resource_id: notification.resource_id.map(|id| id.to_string()),
        resource_type: notification.resource_type.clone(),
        metadata: notification.metadata.clone(),
        created_at: notification.created_at.to_rfc3339(),
        updated_at: notification.updated_at.to_rfc3339(),
    }
}

/// Get all notifications for the current user with filtering and pagination
///
/// Returns a paginated list of notifications for the authenticated user based on the provided filters.
/// Users can only access their own notifications.
///
/// # Errors
/// - Returns `ValidationError` if filter parameters are invalid
/// - Returns `InternalServerError` if there's an issue retrieving notifications
pub async fn get_notifications(
    State(registry): State<Arc<ServiceRegistry>>,
    Query(params): Query<NotificationListParams>,
    current_user: CurrentUser, // Get current user from authentication
) -> Result<Json<PaginatedResponse<NotificationResponse>>> {
    let notification_service = registry.notification_service();

    // Get current user ID from the authentication
    let current_user_id = current_user.0;

    // Get notifications with filter
    let (notifications, total_count) = notification_service
        .get_notifications_with_filter(
            current_user_id,
            params.read,
            params.notification_type,
            params.pagination.page,
            params.pagination.per_page,
            params.sort.sort,
            params.sort.order,
        )
        .await
        .map_err(|e| {
            Error::internal_server_error(format!("Failed to get notifications: {}", e.message))
        })?;

    // Map to response format
    let notification_responses = notifications
        .iter()
        .map(map_notification_to_response)
        .collect();

    // Create paginated response
    let response =
        PaginatedResponse::from_params(notification_responses, &params.pagination, total_count);

    Ok(Json(response))
}

/// Get notification by ID
///
/// Returns a specific notification by its ID.
/// Users can only access their own notifications.
///
/// # Errors
/// - Returns `ValidationError` if the notification ID is invalid
/// - Returns `NotFound` if the notification doesn't exist
/// - Returns `Forbidden` if the current user doesn't have permission to access the notification
/// - Returns `InternalServerError` if there's an issue retrieving the notification
pub async fn get_notification(
    State(registry): State<Arc<ServiceRegistry>>,
    Path(id_str): Path<String>,
    current_user: CurrentUser, // Get current user from authentication
) -> Result<Json<NotificationResponse>> {
    let notification_service = registry.notification_service();

    // Get current user ID from the authentication
    let current_user_id = current_user.0;

    // Parse notification ID
    let id = Uuid::parse_str(&id_str)
        .map_err(|_| Error::validation_error("Invalid notification ID format"))?;

    // Get notification
    let notification = notification_service
        .get_notification(id, current_user_id)
        .await
        .map_err(|e| match e.kind {
            crate::application::NotificationServiceErrorKind::NotFound => {
                Error::not_found("Notification not found")
            }
            crate::application::NotificationServiceErrorKind::PermissionDenied => {
                Error::forbidden("You don't have permission to access this notification")
            }
            _ => Error::internal_server_error(format!("Failed to get notification: {}", e.message)),
        })?;

    Ok(Json(map_notification_to_response(&notification)))
}

/// Mark notification as read
///
/// Updates a specific notification to mark it as read.
/// Users can only mark their own notifications as read.
///
/// # Errors
/// - Returns `ValidationError` if the notification ID is invalid
/// - Returns `NotFound` if the notification doesn't exist
/// - Returns `Forbidden` if the current user doesn't have permission to access the notification
/// - Returns `InternalServerError` if there's an issue updating the notification
pub async fn mark_notification_as_read(
    State(registry): State<Arc<ServiceRegistry>>,
    Path(id_str): Path<String>,
    current_user: CurrentUser, // Get current user from authentication
) -> Result<Json<NotificationResponse>> {
    let notification_service = registry.notification_service();

    // Get current user ID from the authentication
    let current_user_id = current_user.0;

    // Parse notification ID
    let id = Uuid::parse_str(&id_str)
        .map_err(|_| Error::validation_error("Invalid notification ID format"))?;

    // Mark notification as read
    let notification = notification_service
        .mark_as_read(id, current_user_id)
        .await
        .map_err(|e| match e.kind {
            crate::application::NotificationServiceErrorKind::NotFound => {
                Error::not_found("Notification not found")
            }
            crate::application::NotificationServiceErrorKind::PermissionDenied => {
                Error::forbidden("You don't have permission to access this notification")
            }
            _ => Error::internal_server_error(format!(
                "Failed to mark notification as read: {}",
                e.message
            )),
        })?;

    Ok(Json(map_notification_to_response(&notification)))
}

/// Mark all notifications as read
///
/// Updates all notifications for the current user to mark them as read.
/// This operation can be filtered by notification type.
///
/// # Errors
/// - Returns `InternalServerError` if there's an issue updating the notifications
pub async fn mark_all_as_read(
    State(registry): State<Arc<ServiceRegistry>>,
    Query(params): Query<NotificationListParams>,
    current_user: CurrentUser, // Get current user from authentication
) -> Result<StatusCode> {
    let notification_service = registry.notification_service();

    // Get current user ID from the authentication
    let current_user_id = current_user.0;

    // Mark all as read, optionally filtering by type
    notification_service
        .mark_all_as_read(current_user_id, params.notification_type)
        .await
        .map_err(|e| {
            Error::internal_server_error(format!(
                "Failed to mark notifications as read: {}",
                e.message
            ))
        })?;

    Ok(StatusCode::OK)
}

/// Delete notification
///
/// Deletes a specific notification.
/// Users can only delete their own notifications.
///
/// # Errors
/// - Returns `ValidationError` if the notification ID is invalid
/// - Returns `NotFound` if the notification doesn't exist
/// - Returns `Forbidden` if the current user doesn't have permission to access the notification
/// - Returns `InternalServerError` if there's an issue deleting the notification
pub async fn delete_notification(
    State(registry): State<Arc<ServiceRegistry>>,
    Path(id_str): Path<String>,
    current_user: CurrentUser, // Get current user from authentication
) -> Result<StatusCode> {
    let notification_service = registry.notification_service();

    // Get current user ID from the authentication
    let current_user_id = current_user.0;

    // Parse notification ID
    let id = Uuid::parse_str(&id_str)
        .map_err(|_| Error::validation_error("Invalid notification ID format"))?;

    // Delete notification
    notification_service
        .delete_notification(id, current_user_id)
        .await
        .map_err(|e| match e.kind {
            crate::application::NotificationServiceErrorKind::NotFound => {
                Error::not_found("Notification not found")
            }
            crate::application::NotificationServiceErrorKind::PermissionDenied => {
                Error::forbidden("You don't have permission to delete this notification")
            }
            _ => Error::internal_server_error(format!(
                "Failed to delete notification: {}",
                e.message
            )),
        })?;

    Ok(StatusCode::NO_CONTENT)
}

/// Get unread notification count
///
/// Returns the count of unread notifications for the current user.
/// This can be filtered by notification type.
///
/// # Errors
/// - Returns `InternalServerError` if there's an issue retrieving the count
pub async fn get_unread_count(
    State(registry): State<Arc<ServiceRegistry>>,
    Query(params): Query<NotificationListParams>,
    current_user: CurrentUser, // Get current user from authentication
) -> Result<Json<serde_json::Value>> {
    let notification_service = registry.notification_service();

    // Get current user ID from the authentication
    let current_user_id = current_user.0;

    // Get unread count
    let count = notification_service
        .get_unread_count(current_user_id, params.notification_type)
        .await
        .map_err(|e| {
            Error::internal_server_error(format!(
                "Failed to get unread notification count: {}",
                e.message
            ))
        })?;

    Ok(Json(serde_json::json!({ "count": count })))
}

/// Send notification (admin only)
pub async fn send_notification(
    State(registry): State<Arc<ServiceRegistry>>,
    Json(request): Json<CreateNotificationRequest>,
    current_user: CurrentUser, // Get current user from authentication
) -> Result<Json<NotificationResponse>> {
    // Get current user ID from the authentication
    let _current_user_id = current_user.0;

    // Parse user ID
    let _user_id = Uuid::parse_str(&request.user_id)
        .map_err(|_| Error::validation_error("Invalid user ID format"))?;

    // Implementation will be added later
    let id = Uuid::new_v4().to_string();
    Ok(Json(NotificationResponse {
        id,
        notification_type: request.notification_type,
        title: request.title,
        message: request.message,
        is_read: false,
        link: request.link,
        created_at: Utc::now().to_rfc3339(),
    }))
}
