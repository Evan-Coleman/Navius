use navius_http::server::HttpServer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

/// OpenAPI schema generator
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Navius API",
        version = env!("CARGO_PKG_VERSION"),
        description = "Modern, secure API framework built in Rust",
        contact(
            name = "Navius Team",
            url = "https://github.com/navius"
        ),
        license(
            name = "Apache-2.0",
            url = "https://github.com/navius/navius/blob/main/LICENSE"
        )
    ),
    servers(
        (url = "/", description = "Local development server")
    ),
    paths(
        crate::api::controllers::health::health_check
    ),
    components(
        schemas(
            crate::api::controllers::health::HealthResponse,
            crate::api::controllers::health::ComponentStatus,
            crate::api::models::ApiResponse<()>,
            crate::api::models::ResponseMetadata,
            crate::api::models::PaginationInfo,
            crate::api::models::ErrorResponse,
            crate::api::models::ErrorInfo,
            crate::api::models::PaginationParams,
            crate::api::models::SortDirection
        )
    ),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "system", description = "System endpoints")
    ),
    external_docs(url = "https://docs.navius.io", description = "Navius Documentation")
)]
struct ApiDoc;

/// Configure OpenAPI routes
pub fn configure_openapi_routes(server: HttpServer) -> HttpServer {
    server.merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
}
