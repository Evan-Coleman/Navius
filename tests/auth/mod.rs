#[tokio::test]
async fn full_auth_workflow() {
    let config = load_test_config();
    let registry = ProviderRegistry::from_app_config(&config).unwrap();

    // Test valid token flow
    let valid_token = generate_valid_token();
    let claims = registry
        .default_provider()
        .validate_token(&valid_token)
        .await
        .unwrap();
    assert_eq!(claims.sub, "test-user");

    // Test invalid token
    assert!(matches!(
        registry.default_provider().validate_token("invalid").await,
        Err(AuthError::ValidationFailed(_))
    ));

    // Test rate limiting
    for _ in 0..5 {
        assert!(registry.default_provider().refresh_jwks().await.is_ok());
    }
    assert!(matches!(
        registry.default_provider().refresh_jwks().await,
        Err(AuthError::RateLimited)
    ));

    // Test circuit breaker
    for _ in 0..5 {
        let _ = registry.default_provider().validate_token("invalid").await;
    }
    assert!(matches!(
        registry.default_provider().validate_token("valid").await,
        Err(AuthError::CircuitOpen)
    ));
}
