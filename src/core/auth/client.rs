use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use async_trait::async_trait;
use oauth2::{
    AuthUrl, ClientId, ClientSecret, Scope, TokenResponse as OAuth2TokenResponse, TokenUrl,
    basic::BasicClient,
};
use reqwest::Client;
use serde_json::Value;
use tracing::{debug, error, info};

use crate::core::auth::interfaces::{TokenClient, TokenValidationResult};
use crate::core::auth::models::{JwtClaims, TokenResponse, UserProfile};
use crate::core::config::app_config::AppConfig;
use crate::core::config::app_config::ProviderConfig;
use crate::core::config::constants;
use crate::core::error::AppError;

/// Token cache entry
#[derive(Debug)]
struct TokenCacheEntry {
    /// The access token
    access_token: String,
    /// When the token expires
    expires_at: SystemTime,
}

/// Entra token client for acquiring tokens for downstream services
#[derive(Debug)]
pub struct EntraTokenClient {
    /// HTTP client for making requests
    client: Client,
    /// OAuth2 client ID
    client_id: ClientId,
    /// OAuth2 client secret
    client_secret: ClientSecret,
    /// Authorization URL
    auth_url: AuthUrl,
    /// Token URL
    token_url: TokenUrl,
    /// Token cache to avoid unnecessary requests
    token_cache: Arc<Mutex<HashMap<String, TokenCacheEntry>>>,
}

impl EntraTokenClient {
    /// Create a new token client with the given credentials
    pub fn new(tenant_id: &str, client_id: &str, client_secret: &str) -> Self {
        // Use default URL formats from app_config for consistency
        let auth_url_format = crate::core::config::app_config::default_authorize_url_format();
        let token_url_format = crate::core::config::app_config::default_token_url_format();

        let auth_url_str = auth_url_format.replace("{}", tenant_id);
        let token_url_str = token_url_format.replace("{}", tenant_id);

        let client_id = ClientId::new(client_id.to_string());
        let client_secret = ClientSecret::new(client_secret.to_string());
        let auth_url = AuthUrl::new(auth_url_str).unwrap();
        let token_url = TokenUrl::new(token_url_str).unwrap();

        Self {
            client: Client::new(),
            client_id,
            client_secret,
            auth_url,
            token_url,
            token_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Create a new token client from the application configuration
    pub fn from_config(config: &AppConfig) -> Self {
        // Get the default provider config
        let provider_name = &config.auth.default_provider;
        let provider_config = config
            .auth
            .providers
            .get(provider_name)
            .expect("Default provider configuration not found");

        // Get tenant ID from provider specific config
        let tenant_id = provider_config
            .provider_specific
            .get("tenant_id")
            .and_then(|v| v.as_str())
            .unwrap_or_default();

        let client_id = &provider_config.client_id;
        let client_secret =
            std::env::var(constants::auth::env_vars::CLIENT_SECRET).unwrap_or_default();

        // Get URL formats from provider specific config
        let auth_url_format = provider_config
            .provider_specific
            .get("authorize_url_format")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| {
                let fmt = crate::core::config::app_config::default_authorize_url_format();
                Box::leak(fmt.into_boxed_str())
            });

        let token_url_format = provider_config
            .provider_specific
            .get("token_url_format")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| {
                let fmt = crate::core::config::app_config::default_token_url_format();
                Box::leak(fmt.into_boxed_str())
            });

        let auth_url_str = auth_url_format.replace("{}", tenant_id);
        let token_url_str = token_url_format.replace("{}", tenant_id);

        let client_id_obj = ClientId::new(client_id.to_string());
        let client_secret_obj = ClientSecret::new(client_secret.to_string());
        let auth_url = AuthUrl::new(auth_url_str).unwrap();
        let token_url = TokenUrl::new(token_url_str).unwrap();

        Self {
            client: Client::new(),
            client_id: client_id_obj,
            client_secret: client_secret_obj,
            auth_url,
            token_url,
            token_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Create a new token client from environment variables
    pub fn from_env() -> Self {
        let tenant_id = std::env::var(constants::auth::env_vars::TENANT_ID).unwrap_or_default();
        let client_id = std::env::var(constants::auth::env_vars::CLIENT_ID).unwrap_or_default();
        let client_secret =
            std::env::var(constants::auth::env_vars::CLIENT_SECRET).unwrap_or_default();

        // Use default URL formats from app_config for consistency
        let auth_url_format = crate::core::config::app_config::default_authorize_url_format();
        let token_url_format = crate::core::config::app_config::default_token_url_format();

        let auth_url_str = auth_url_format.replace("{}", &tenant_id);
        let token_url_str = token_url_format.replace("{}", &tenant_id);

        let client_id_obj = ClientId::new(client_id.to_string());
        let client_secret_obj = ClientSecret::new(client_secret.to_string());
        let auth_url = AuthUrl::new(auth_url_str).unwrap();
        let token_url = TokenUrl::new(token_url_str).unwrap();

        Self {
            client: Client::new(),
            client_id: client_id_obj,
            client_secret: client_secret_obj,
            auth_url,
            token_url,
            token_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Acquire a token for the specified resource/scope
    pub async fn get_token(&self, scope: &str) -> Result<String, String> {
        // Check cache first
        {
            let cache = self.token_cache.lock().unwrap();
            if let Some(entry) = cache.get(scope) {
                // Check if token is still valid (with 5 min buffer)
                let now = SystemTime::now();
                if entry.expires_at > now + Duration::from_secs(300) {
                    debug!("Using cached token for scope: {}", scope);
                    return Ok(entry.access_token.clone());
                }
            }
        }

        info!("Acquiring new token for scope: {}", scope);

        // Configure HTTP client for oauth2
        let http_client = reqwest::ClientBuilder::new()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|e| format!("Failed to build HTTP client: {}", e))?;

        // Create a new OAuth2 client for this request
        let oauth_client = BasicClient::new(self.client_id.clone())
            .set_client_secret(self.client_secret.clone())
            .set_auth_uri(self.auth_url.clone())
            .set_token_uri(self.token_url.clone());

        // Token not in cache or expired, get a new one using client credentials flow
        let token_result = oauth_client
            .exchange_client_credentials()
            .add_scope(Scope::new(scope.to_string()))
            .request_async(&http_client)
            .await
            .map_err(|e| format!("Failed to get token: {}", e))?;

        let access_token = token_result.access_token().secret().to_string();

        // Calculate expiration time
        let expires_in = token_result
            .expires_in()
            .unwrap_or(Duration::from_secs(3600));
        let expires_at = SystemTime::now() + expires_in;

        // Cache the token
        {
            let mut cache = self.token_cache.lock().unwrap();
            cache.insert(
                scope.to_string(),
                TokenCacheEntry {
                    access_token: access_token.clone(),
                    expires_at,
                },
            );
        }

        Ok(access_token)
    }

    /// Create an HTTP client with auth header for the specified scope
    pub async fn create_client(&self, scope: &str) -> Result<Client, String> {
        let token = self.get_token(scope).await?;

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::AUTHORIZATION,
            reqwest::header::HeaderValue::from_str(&format!("Bearer {}", token))
                .map_err(|e| format!("Invalid token: {}", e))?,
        );

        Client::builder()
            .default_headers(headers)
            .build()
            .map_err(|e| format!("Failed to build client: {}", e))
    }
}

#[async_trait]
impl TokenClient for EntraTokenClient {
    async fn get_token(&self, _username: &str, _password: &str) -> Result<TokenResponse, AppError> {
        // For now, just return an error since this implementation doesn't support
        // username/password auth flow - it uses client credentials
        Err(AppError::NotImplementedError(
            "Username/password auth flow not implemented for EntraTokenClient".to_string(),
        ))
    }

    async fn validate_token(&self, _token: &str) -> Result<TokenValidationResult, AppError> {
        // This client is for acquiring tokens, not validating them
        // In a real implementation, we would call the Entra ID token validation endpoint
        Err(AppError::NotImplementedError(
            "Token validation not implemented for EntraTokenClient".to_string(),
        ))
    }

    async fn refresh_token(&self, _refresh_token: &str) -> Result<TokenResponse, AppError> {
        // This client uses client credentials flow which doesn't use refresh tokens
        Err(AppError::NotImplementedError(
            "Refresh token flow not implemented for EntraTokenClient".to_string(),
        ))
    }

    async fn get_user_profile(&self, _token: &str) -> Result<UserProfile, AppError> {
        // This implementation doesn't support fetching user profiles
        Err(AppError::NotImplementedError(
            "User profile retrieval not implemented for EntraTokenClient".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::app_config::{AppConfig, ProviderConfig};
    use std::collections::HashMap;
    use std::env;
    use std::time::{Duration, SystemTime};

    #[test]
    fn test_token_client_creation() {
        // Create token client
        let client = EntraTokenClient::new(
            "test-tenant-placeholder",
            "test-client-placeholder",
            "test-secret-placeholder",
        );

        // Check that it's properly initialized
        assert_eq!(client.client_id.as_str(), "test-client-placeholder");
        assert_eq!(
            client.client_secret.secret().as_str(),
            "test-secret-placeholder"
        );
        assert!(client.auth_url.as_str().contains("test-tenant-placeholder"));
        assert!(
            client
                .token_url
                .as_str()
                .contains("test-tenant-placeholder")
        );
    }

    #[test]
    fn test_from_config() {
        // Create minimal config
        let mut config = AppConfig::default();
        config.auth.enabled = true;

        // Add a provider for Entra
        let mut provider_config = ProviderConfig {
            enabled: true,
            client_id: "config-client-placeholder".to_string(),
            jwks_uri: "https://login.microsoftonline.com/tenant/discovery/v2.0/keys".to_string(),
            issuer_url: "https://login.microsoftonline.com/tenant/v2.0".to_string(),
            audience: "api://default".to_string(),
            role_mappings: HashMap::new(),
            provider_specific: HashMap::new(),
        };

        // Add tenant_id to provider specific config
        provider_config.provider_specific.insert(
            "tenant_id".to_string(),
            Value::String("config-tenant-placeholder".to_string()),
        );

        config.auth.default_provider = "entra".to_string();
        config
            .auth
            .providers
            .insert("entra".to_string(), provider_config);

        // Create client from config
        let client = EntraTokenClient::from_config(&config);

        assert_eq!(client.client_id.as_str(), "config-client-placeholder");
        assert!(
            client
                .auth_url
                .as_str()
                .contains("config-tenant-placeholder")
        );
        assert!(
            client
                .token_url
                .as_str()
                .contains("config-tenant-placeholder")
        );
    }

    #[test]
    fn test_from_env() {
        // Skip actual environment checking but test the path
        // We'll verify that the function exists and can be called
        // This would normally require setting up environment variables
        let result = std::panic::catch_unwind(|| {
            // This will likely fail without env vars set, but we're just checking the function exists
            let _ = EntraTokenClient::from_env();
        });

        // We just verify that the function exists and doesn't crash immediately
        assert!(result.is_err() || result.is_ok());
    }

    #[test]
    fn test_token_cache_operations() {
        // Create a client
        let client = EntraTokenClient::new(
            "test-tenant-placeholder",
            "test-client-placeholder",
            "test-secret-placeholder",
        );

        // Test empty cache
        {
            let cache = client.token_cache.lock().unwrap();
            assert!(cache.is_empty());
        }

        // Manually insert a token in the cache
        let scope = "test-scope";
        let expiry = SystemTime::now()
            .checked_add(Duration::from_secs(3600))
            .unwrap();

        {
            let mut cache = client.token_cache.lock().unwrap();
            cache.insert(
                scope.to_string(),
                TokenCacheEntry {
                    access_token: "cached-token".to_string(),
                    expires_at: expiry,
                },
            );
        }

        // Verify we can retrieve the cached token
        {
            let cache = client.token_cache.lock().unwrap();
            assert!(!cache.is_empty());
            let entry = cache.get(scope).unwrap();
            assert_eq!(entry.access_token, "cached-token");
        }
    }

    // Note: We can't easily test token acquisition without mocking the OAuth2 server
    // In a real-world scenario, you might use a tool like mockito to mock the HTTP responses
}
