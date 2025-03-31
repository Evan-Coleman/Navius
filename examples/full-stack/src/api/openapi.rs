use utoipa::OpenApi;
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa_swagger_ui::SwaggerUi;
use axum::Router;
use std::sync::Arc;

use crate::api::controllers::{
    auth, users, tasks, categories, notifications, health
};
use crate::api::models::{
    error::{ErrorResponse, ErrorDetails, ErrorCode, FieldError},
    response::{ApiResponse, ResponseMeta},
    pagination::{PaginatedResponse, PaginationMeta, PaginationParams, SortParams}
};
use crate::infrastructure::ServiceRegistry;

/// Represents the OpenAPI schema for the application API
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Navius Full Stack Integration API",
        version = env!("CARGO_PKG_VERSION"),
        description = "API documentation for the Navius Full Stack Integration Example",
        contact(
            name = "Navius Framework Team",
            email = "support@example.com"
        ),
        license(
            name = "MIT OR Apache-2.0"
        )
    ),
    servers(
        (url = "/api", description = "API server")
    ),
    paths(
        // Auth controller paths
        auth::login,
        auth::refresh_token,
        auth::logout,
        
        // User controller paths
        users::get_users,
        users::get_user,
        users::create_user,
        users::update_user,
        users::delete_user,
        users::update_profile,
        
        // Task controller paths
        tasks::get_tasks,
        tasks::get_task,
        tasks::create_task,
        tasks::update_task,
        tasks::delete_task,
        tasks::assign_task,
        tasks::unassign_task,
        tasks::get_task_comments,
        tasks::add_comment,
        tasks::update_comment,
        tasks::delete_comment,
        
        // Category controller paths
        categories::get_categories,
        categories::get_category,
        categories::create_category,
        categories::update_category,
        categories::delete_category,
        categories::get_subcategories,
        
        // Notification controller paths
        notifications::get_notifications,
        notifications::get_notification,
        notifications::mark_notification_as_read,
        notifications::mark_all_as_read,
        notifications::delete_notification,
        notifications::get_unread_count,
        
        // Health controller paths
        health::health_check,
        health::readiness_check,
        health::liveness_check,
    ),
    components(
        schemas(
            // Auth schemas
            auth::LoginRequest,
            auth::LoginResponse,
            auth::RefreshRequest,
            auth::TokenResponse,

            // User schemas
            users::UserListParams,
            users::UserResponse,
            users::UserProfileResponse,
            users::CreateUserRequest,
            users::UpdateUserRequest,
            users::UpdateProfileRequest,

            // Task schemas
            tasks::TaskListParams,
            tasks::TaskResponse,
            tasks::CommentResponse,
            tasks::CreateTaskRequest,
            tasks::UpdateTaskRequest,
            tasks::AddCommentRequest,
            tasks::UpdateCommentRequest,

            // Category schemas
            categories::CategoryListParams,
            categories::CategoryResponse,
            categories::CreateCategoryRequest,
            categories::UpdateCategoryRequest,

            // Notification schemas
            notifications::NotificationListParams,
            notifications::NotificationResponse,
            notifications::CreateNotificationRequest,
            notifications::NotificationType,

            // Health schemas
            health::HealthResponse,
            health::ComponentStatus,

            // Common schemas
            ErrorResponse,
            ErrorDetails,
            ErrorCode,
            FieldError,
            ApiResponse<String>, // Generic placeholder 
            ResponseMeta,
            PaginatedResponse<String>, // Generic placeholder
            PaginationMeta,
            PaginationParams,
            SortParams,
        ),
        security_schemes(
            ("bearer_auth" = SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .description(Some("JWT token authentication"))
                    .build()
            )),
        )
    ),
    tags(
        (name = "auth", description = "Authentication endpoints"),
        (name = "users", description = "User management endpoints"),
        (name = "tasks", description = "Task management endpoints"),
        (name = "categories", description = "Category management endpoints"),
        (name = "notifications", description = "Notification management endpoints"),
        (name = "health", description = "Health and monitoring endpoints")
    ),
    external_docs(
        url = "https://github.com/example/navius",
        description = "Find more information about Navius Framework on GitHub"
    ),
)]
pub struct ApiDoc;

/// Configures the OpenAPI routes and Swagger UI
pub fn configure_openapi() -> SwaggerUi {
    SwaggerUi::new("/swagger-ui")
        .url("/api-docs/openapi.json", ApiDoc::openapi())
}

/// Adds OpenAPI routes to the application router
pub fn configure_openapi_routes(router: Router<Arc<ServiceRegistry>>) -> Router<Arc<ServiceRegistry>> {
    router.merge(configure_openapi())
} 