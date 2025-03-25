#[tokio::test]
async fn test_provider_registry() {
    let config = load_test_config();
    let registry = ProviderRegistry::from_app_config(&config).unwrap();

    assert!(registry.get_provider("entra").is_some());
    assert!(registry.get_provider("google").is_none());

    let default = registry.default_provider();
    assert_eq!(default.name(), "entra");
}

#[tokio::test]
async fn test_middleware_layer() {
    let config = load_test_config();
    let registry = ProviderRegistry::from_app_config(&config).unwrap();
    let layer = AuthLayer::new(registry);

    let app = Router::new()
        .route("/", get(|| async { "OK" }))
        .layer(layer.default_provider().layer());

    // Test authenticated request
    let response = app
        .oneshot(
            Request::builder()
                .uri("/")
                .header("Authorization", "Bearer valid-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_jwks_refresh() {
    let config = load_test_config();
    let registry = ProviderRegistry::from_app_config(&config).unwrap();

    let handle = registry.start_jwks_refresh(1); // 1 second interval

    // Wait for first refresh
    tokio::time::sleep(Duration::from_secs(2)).await;

    let status = registry.check_health().await;
    assert!(status["entra"].ready);

    handle.abort();
}

#[tokio::test]
async fn test_health_check_integration() {
    let config = load_test_config();
    let registry = ProviderRegistry::from_app_config(&config).unwrap();

    let response = health_handler(Extension(registry)).await.into_response();
    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["status"], "UP");
    assert!(
        json["components"]["auth_providers"]["entra"]["ready"]
            .as_bool()
            .unwrap()
    );
}

#[tokio::test]
async fn test_rate_limiting() {
    let config = load_test_config();
    let registry = ProviderRegistry::from_app_config(&config).unwrap();
    let provider = registry.get_provider("entra").unwrap();

    // Test rate limiting
    for _ in 0..5 {
        assert!(provider.refresh_jwks().await.is_ok());
    }

    // Sixth request should be rate limited
    assert!(matches!(
        provider.refresh_jwks().await,
        Err(AuthError::RateLimited)
    ));
}

#[tokio::test]
async fn test_metrics_integration() {
    let config = load_test_config();
    let registry = ProviderRegistry::from_app_config(&config).unwrap();

    // Trigger some operations
    registry.check_health().await;
    let provider = registry.default_provider();
    let _ = provider.validate_token("test").await;

    // Verify metrics were recorded
    let metrics = metrics::capture_metrics();
    assert!(metrics.iter().any(|m| m.name() == "auth_tokens_validated"));
    assert!(metrics.iter().any(|m| m.name() == "auth_provider_ready"));
}

#[tokio::test]
async fn test_circuit_breaker() {
    let config = load_test_config();
    let registry = ProviderRegistry::from_app_config(&config).unwrap();
    let provider = registry.get_provider("entra").unwrap();

    // Force circuit to open
    for _ in 0..5 {
        let _ = provider.validate_token("invalid").await;
    }

    // Next request should fail fast
    assert!(matches!(
        provider.validate_token("valid").await,
        Err(AuthError::CircuitOpen)
    ));

    // Wait for reset timeout
    tokio::time::sleep(Duration::from_secs(60)).await;

    // Should be in half-open state
    let status = provider.health_check().await;
    assert_eq!(status.circuit_state, "half-open");
}

#[tokio::test]
async fn test_error_conversion() {
    let jwt_error = jsonwebtoken::errors::ErrorKind::InvalidToken.into_error();
    let auth_error: AuthError = jwt_error.into();
    assert!(matches!(auth_error, AuthError::ValidationFailed(_)));

    let reqwest_error = reqwest::Error::new(reqwest::ErrorKind::Request);
    let auth_error: AuthError = reqwest_error.into();
    assert!(matches!(auth_error, AuthError::NetworkError(_)));
}
