pub fn create_router(config: AppConfig) -> Router {
    let auth_registry =
        ProviderRegistry::from_app_config(&config).expect("Failed to initialize auth providers");

    Router::new().route("/secure", get(secure_handler)).layer(
        AuthLayer::new(auth_registry)
            .default_provider()
            .with_roles(RoleRequirement::Admin)
            .layer(),
    )
    // ... other routes
}
