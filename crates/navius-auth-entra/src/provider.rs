use crate::config::{EntraConfig, EntraConfigBuilder};
use crate::error::{EntraError, EntraResult};
use crate::jwks::JwksClient;
use crate::token::{EntraTokenClaims, EntraTokenValidator};
use crate::user::EntraUser;
use async_trait::async_trait;
use navius_auth::{AuthProvider, AuthResult, AuthenticationInfo, AuthorizationContext};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, instrument, warn};

/// Microsoft Entra ID authentication provider implementation
pub struct EntraProvider {
    /// Configuration for the Entra ID provider
    config: EntraConfig,

    /// Token validator for JWT validation
    validator: Arc<EntraTokenValidator>,

    /// JWKS client for retrieving signing keys
    jwks_client: Arc<RwLock<JwksClient>>,
}

impl EntraProvider {
    /// Create a new EntraProvider with the given configuration
    pub fn new(config: EntraConfig) -> EntraResult<Self> {
        // Validate the configuration
        config.validate()?;

        // Create JWKS client for key retrieval
        let jwks_client = Arc::new(RwLock::new(JwksClient::new(
            config.jwks_uri.clone(),
            config.jwks_cache_duration,
            config.jwks_refresh_ahead_duration,
        )));

        // Create token validator
        let validator = Arc::new(EntraTokenValidator::new(
            config.client_id.clone(),
            config.tenant_id.clone(),
            config.issuer.clone(),
            config.audience.clone(),
            config.clock_skew,
            jwks_client.clone(),
        ));

        Ok(Self {
            config,
            validator,
            jwks_client,
        })
    }

    /// Create a new EntraProvider using the builder pattern
    pub fn builder() -> EntraConfigBuilder {
        EntraConfigBuilder::new()
    }
}

#[async_trait]
impl AuthProvider for EntraProvider {
    #[instrument(skip(self, token), level = "debug")]
    async fn authenticate(&self, token: &str) -> AuthResult<AuthenticationInfo> {
        debug!("Authenticating token with Entra provider");

        // Validate the token
        let claims = self.validator.validate_token(token).await.map_err(|e| {
            error!("Token validation failed: {:?}", e);
            navius_auth::AuthError::InvalidToken(format!("Entra token validation failed: {}", e))
        })?;

        // Create user from claims
        let user = EntraUser::from_claims(&claims).map_err(|e| {
            error!("Failed to create user from claims: {:?}", e);
            navius_auth::AuthError::UserProfileError(format!(
                "Failed to create user profile: {}",
                e
            ))
        })?;

        // Create authentication info
        let auth_info = AuthenticationInfo {
            user_id: user.id.clone(),
            username: user
                .preferred_username
                .clone()
                .unwrap_or_else(|| user.id.clone()),
            display_name: user.display_name.clone(),
            email: user.email.clone(),
            roles: user.roles.clone(),
            provider: "entra".to_string(),
            metadata: serde_json::to_value(&user).unwrap_or_default(),
            expires_at: claims.exp,
        };

        info!("Successfully authenticated user: {}", auth_info.user_id);
        Ok(auth_info)
    }

    #[instrument(skip(self, ctx), level = "debug")]
    async fn authorize(&self, ctx: &AuthorizationContext) -> AuthResult<bool> {
        debug!(
            "Authorizing request for user: {:?}, roles: {:?}",
            ctx.user_id, ctx.required_roles
        );

        // If no roles required, authorization succeeds
        if ctx.required_roles.is_empty() {
            debug!("No roles required, authorization successful");
            return Ok(true);
        }

        // Check if user has any of the required roles
        let has_required_role = ctx
            .user_roles
            .iter()
            .any(|role| ctx.required_roles.contains(role));

        if has_required_role {
            debug!("User has required role, authorization successful");
            Ok(true)
        } else {
            warn!("User lacks required roles, authorization failed");
            Ok(false)
        }
    }

    fn provider_name(&self) -> &str {
        "entra"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, SystemTime};

    // Helper function to create a test configuration
    fn create_test_config() -> EntraConfig {
        EntraConfigBuilder::new()
            .client_id("test-client-id")
            .tenant_id("test-tenant-id")
            .issuer("https://login.microsoftonline.com/test-tenant-id/v2.0")
            .jwks_uri("https://login.microsoftonline.com/test-tenant-id/discovery/v2.0/keys")
            .build()
            .unwrap()
    }

    #[test]
    fn test_provider_creation() {
        let config = create_test_config();
        let provider = EntraProvider::new(config);
        assert!(provider.is_ok());
    }

    #[tokio::test]
    async fn test_authorize_no_roles() {
        let config = create_test_config();
        let provider = EntraProvider::new(config).unwrap();

        let ctx = AuthorizationContext {
            user_id: "test-user".to_string(),
            user_roles: vec![],
            required_roles: vec![],
            resource: None,
            action: None,
        };

        let result = provider.authorize(&ctx).await;
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[tokio::test]
    async fn test_authorize_with_roles() {
        let config = create_test_config();
        let provider = EntraProvider::new(config).unwrap();

        let ctx = AuthorizationContext {
            user_id: "test-user".to_string(),
            user_roles: vec!["User".to_string(), "Admin".to_string()],
            required_roles: vec!["Admin".to_string()],
            resource: None,
            action: None,
        };

        let result = provider.authorize(&ctx).await;
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[tokio::test]
    async fn test_authorize_missing_roles() {
        let config = create_test_config();
        let provider = EntraProvider::new(config).unwrap();

        let ctx = AuthorizationContext {
            user_id: "test-user".to_string(),
            user_roles: vec!["User".to_string()],
            required_roles: vec!["Admin".to_string()],
            resource: None,
            action: None,
        };

        let result = provider.authorize(&ctx).await;
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }
}
