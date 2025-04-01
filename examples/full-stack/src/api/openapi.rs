use crate::infrastructure::ServiceRegistry;
use axum::{Router, response::Html};
use std::sync::Arc;

/// Configures the OpenAPI routes and Swagger UI
pub fn configure_openapi() -> Router<Arc<ServiceRegistry>> {
    let swagger_ui_html = Html(format!(
        r#"
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="utf-8" />
            <meta name="viewport" content="width=device-width, initial-scale=1" />
            <meta name="description" content="SwaggerUI" />
            <title>SwaggerUI</title>
            <link rel="stylesheet" href="https://unpkg.com/swagger-ui-dist@5.9.0/swagger-ui.css" />
        </head>
        <body>
            <div id="swagger-ui"></div>
            <script src="https://unpkg.com/swagger-ui-dist@5.9.0/swagger-ui-bundle.js" crossorigin></script>
            <script>
                window.onload = () => {{
                    window.ui = SwaggerUIBundle({{
                        url: '/api-docs/openapi.json',
                        dom_id: '#swagger-ui',
                        deepLinking: true,
                        presets: [
                            SwaggerUIBundle.presets.apis,
                            SwaggerUIBundle.SwaggerUIStandalonePreset
                        ],
                    }});
                }};
            </script>
        </body>
        </html>
    "#
    ));

    Router::new().route(
        "/swagger-ui",
        axum::routing::get(move || async { swagger_ui_html }),
    )
}

/// Adds OpenAPI routes to the application router
pub fn configure_openapi_routes(
    router: Router<Arc<ServiceRegistry>>,
) -> Router<Arc<ServiceRegistry>> {
    router.merge(configure_openapi())
}
