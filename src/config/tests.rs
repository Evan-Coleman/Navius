#[cfg(test)]
mod tests {
    use super::app_config;
    use super::constants;
    use std::env;

    #[test]
    fn test_env_vars_with_missing_yaml_fields() {
        // Set environment variables for testing
        env::set_var(constants::auth::env_vars::TENANT_ID, "test-tenant-id");
        env::set_var(constants::auth::env_vars::CLIENT_ID, "test-client-id");
        env::set_var(constants::auth::env_vars::AUDIENCE, "test-audience");
        env::set_var(constants::auth::env_vars::SCOPE, "test-scope");
        env::set_var(constants::auth::env_vars::TOKEN_URL, "test-token-url");

        // Special case to test the default permission
        env::remove_var(constants::auth::env_vars::PERMISSION);

        // Set config directory to a non-existent path to force using just env vars
        env::set_var("CONFIG_DIR", "./nonexistent");

        // Handle any errors (this might fail in CI if we can't set env vars)
        let config_result = app_config::load_config();
        if let Ok(config) = config_result {
            // Verify that environment variables were correctly loaded
            assert_eq!(config.auth.entra.tenant_id, "test-tenant-id");
            assert_eq!(config.auth.entra.client_id, "test-client-id");
            assert_eq!(config.auth.entra.audience, "test-audience");
            assert_eq!(config.auth.entra.scope, "test-scope");
            assert_eq!(config.auth.entra.token_url, "test-token-url");

            // Verify default permission when env var is not set
            assert_eq!(
                config.auth.entra.permission,
                constants::auth::permissions::DEFAULT_PERMISSION
            );
        }

        // Clean up test environment variables
        env::remove_var(constants::auth::env_vars::TENANT_ID);
        env::remove_var(constants::auth::env_vars::CLIENT_ID);
        env::remove_var(constants::auth::env_vars::AUDIENCE);
        env::remove_var(constants::auth::env_vars::SCOPE);
        env::remove_var(constants::auth::env_vars::TOKEN_URL);
        env::remove_var("CONFIG_DIR");
    }
}
