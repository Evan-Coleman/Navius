use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use oauth2::{AuthUrl, ClientId, ClientSecret, Scope, TokenResponse, TokenUrl, basic::BasicClient};
use reqwest::Client;
use tracing::{debug, error, info};

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
    /// OAuth2 client
    oauth_client: BasicClient,
    /// Client secret for client credentials flow
    client_secret: ClientSecret,
    /// Token cache to avoid unnecessary requests
    token_cache: Arc<Mutex<HashMap<String, TokenCacheEntry>>>,
}

impl EntraTokenClient {
    /// Create a new token client with the given credentials
    pub fn new(tenant_id: &str, client_id: &str, client_secret: &str) -> Self {
        let auth_url = format!(
            "https://login.microsoftonline.com/{}/oauth2/v2.0/authorize",
            tenant_id
        );
        let token_url = format!(
            "https://login.microsoftonline.com/{}/oauth2/v2.0/token",
            tenant_id
        );

        let oauth_client = BasicClient::new(
            ClientId::new(client_id.to_string()),
            None,
            AuthUrl::new(auth_url).unwrap(),
            Some(TokenUrl::new(token_url).unwrap()),
        );

        Self {
            client: Client::new(),
            oauth_client,
            client_secret: ClientSecret::new(client_secret.to_string()),
            token_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Create a new token client from environment variables
    pub fn from_env() -> Self {
        let tenant_id =
            std::env::var("RUST_BACKEND_TENANT_ID").expect("RUST_BACKEND_TENANT_ID not set");
        let client_id =
            std::env::var("RUST_BACKEND_CLIENT_ID").expect("RUST_BACKEND_CLIENT_ID not set");
        let client_secret =
            std::env::var("RUST_BACKEND_SECRET").expect("RUST_BACKEND_SECRET not set");

        Self::new(&tenant_id, &client_id, &client_secret)
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

        // Token not in cache or expired, get a new one
        let token_result = self
            .oauth_client
            .exchange_client_credentials()
            .add_scope(Scope::new(scope.to_string()))
            .set_client_secret(self.client_secret.clone())
            .request_async(oauth2::reqwest::async_http_client)
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
