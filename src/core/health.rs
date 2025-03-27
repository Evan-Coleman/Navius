use crate::core::auth::providers::ProviderRegistry;

pub async fn health_handler(
    Extension(providers): Extension<ProviderRegistry>,
) -> impl IntoResponse {
    let auth_health = providers.check_health().await;

    Json(json!({
        "status": "UP",
        "components": {
            "auth_providers": auth_health
        }
    }))
}
