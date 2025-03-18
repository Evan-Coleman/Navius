use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use oauth2::{AuthUrl, ClientId, ClientSecret, Scope, TokenResponse, TokenUrl, basic::BasicClient};
use reqwest::Client;
use tracing::{debug, error, info};

use crate::config::app_config::AppConfig;
use crate::config::constants;

/// Token cache entry
struct TokenCacheEntry {
    /// The access token
    access_token: String,
    /// When the token expires
    expires_at: SystemTime,
}

/// Entra token client for acquiring tokens for downstream services
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
        let auth_url_format = crate::config::app_config::default_authorize_url_format();
        let token_url_format = crate::config::app_config::default_token_url_format();

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
        let tenant_id = &config.auth.entra.tenant_id;
        let client_id = &config.auth.entra.client_id;
        let client_secret =
            std::env::var(constants::auth::env_vars::CLIENT_SECRET).unwrap_or_default();

        // Use URL formats from config
        let auth_url_format = &config.auth.entra.authorize_url_format;
        let token_url_format = &config.auth.entra.token_url_format;

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
        let auth_url_format = crate::config::app_config::default_authorize_url_format();
        let token_url_format = crate::config::app_config::default_token_url_format();

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
