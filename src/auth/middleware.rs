use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use std::time::{Duration, SystemTime};

use axum::{
    body::Body,
    extract::Request,
    http::{HeaderMap, HeaderValue, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use futures::future::BoxFuture;
use jsonwebtoken::{Algorithm, DecodingKey, TokenData, Validation, decode, decode_header};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tower::{Layer, Service};
use tracing::{debug, error, info};

use crate::app::AppState;

/// JWKS (JSON Web Key Set) response
#[derive(Debug, Clone, Deserialize)]
struct JwksResponse {
    keys: Vec<Jwk>,
}

/// JSON Web Key
#[derive(Debug, Clone, Deserialize)]
struct Jwk {
    #[serde(rename = "kid")]
    key_id: String,
    #[serde(rename = "x5c")]
    x509_chain: Option<Vec<String>>,
    #[serde(rename = "n")]
    modulus: Option<String>,
    #[serde(rename = "e")]
    exponent: Option<String>,
    kty: String,
}

/// JWKS cache entry
#[derive(Debug, Clone)]
struct JwksCacheEntry {
    /// The JWKS data
    jwks: JwksResponse,
    /// When the cache entry expires
    expires_at: SystemTime,
}

/// Claims from the JWT token we validate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntraClaims {
    /// Subject (user/client ID)
    pub sub: String,
    /// Audience (who is this token for)
    pub aud: String,
    /// Issuer (who issued this token)
    pub iss: String,
    /// Expiration time
    pub exp: usize,
    /// Not before time
    pub nbf: usize,
    /// Issued at time
    pub iat: usize,
    /// Roles assigned to the client/user
    #[serde(default)]
    pub roles: Vec<String>,
    /// App ID of the client
    pub appid: Option<String>,
    /// App ID URI of the client
    #[serde(rename = "azp")]
    pub app_id_uri: Option<String>,
    /// Scope in access token
    pub scp: Option<String>,
}

/// Role requirements for authorization
#[derive(Debug, Clone)]
pub enum RoleRequirement {
    /// Any of the listed roles is sufficient
    Any(Vec<String>),
    /// All of the listed roles are required
    All(Vec<String>),
    /// No roles required (authentication only)
    None,
}

/// Configuration for Entra authentication middleware
#[derive(Debug, Clone)]
pub struct EntraAuthConfig {
    /// Entra tenant ID
    pub tenant_id: String,
    /// Client ID of our application
    pub client_id: String,
    /// Expected audience value
    pub audience: String,
    /// JWKS URI for token validation
    pub jwks_uri: String,
    /// Whether to validate the token (disable for debugging)
    pub validate_token: bool,
    /// Required roles for authorization
    pub required_roles: RoleRequirement,
    /// Cache for JWKS
    pub jwks_cache: Arc<Mutex<Option<JwksCacheEntry>>>,
    /// HTTP client
    pub client: Client,
}

impl Default for EntraAuthConfig {
    fn default() -> Self {
        let tenant_id = std::env::var("RUST_BACKEND_TENANT_ID").unwrap_or_default();
        let client_id = std::env::var("RUST_BACKEND_CLIENT_ID").unwrap_or_default();

        Self {
            tenant_id: tenant_id.clone(),
            client_id: client_id.clone(),
            audience: format!("api://{}", client_id),
            jwks_uri: format!(
                "https://login.microsoftonline.com/{}/discovery/v2.0/keys",
                tenant_id
            ),
            validate_token: true,
            required_roles: RoleRequirement::None,
            jwks_cache: Arc::new(Mutex::new(None)),
            client: Client::new(),
        }
    }
}

/// Error response for authentication failures
#[derive(Debug)]
pub enum AuthError {
    /// Missing Authorization header
    MissingToken,
    /// Invalid token format
    InvalidTokenFormat,
    /// Token validation failed
    ValidationFailed(String),
    /// Backend error
    InternalError(String),
    /// Authorization failed
    AccessDenied(String),
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthError::MissingToken => (
                StatusCode::UNAUTHORIZED,
                "Missing authorization token".to_string(),
            ),
            AuthError::InvalidTokenFormat => {
                (StatusCode::UNAUTHORIZED, "Invalid token format".to_string())
            }
            AuthError::ValidationFailed(msg) => (StatusCode::UNAUTHORIZED, msg),
            AuthError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AuthError::AccessDenied(msg) => (StatusCode::FORBIDDEN, msg),
        };

        (
            status,
            [(
                axum::http::header::CONTENT_TYPE,
                HeaderValue::from_static("text/plain"),
            )],
            error_message,
        )
            .into_response()
    }
}

/// Extract the bearer token from the Authorization header
fn extract_token(headers: &HeaderMap) -> Result<String, AuthError> {
    let header = headers
        .get(axum::http::header::AUTHORIZATION)
        .ok_or(AuthError::MissingToken)?;

    let header_str = header.to_str().map_err(|_| AuthError::InvalidTokenFormat)?;

    if !header_str.starts_with("Bearer ") {
        return Err(AuthError::InvalidTokenFormat);
    }

    Ok(header_str[7..].to_string())
}

/// Fetch and cache JWKS from the JWKS URI
async fn fetch_and_cache_jwks(config: &EntraAuthConfig) -> Result<JwksResponse, AuthError> {
    // Check cache first
    {
        let cache = config.jwks_cache.lock().unwrap();
        if let Some(entry) = &*cache {
            // Check if cache is still valid (with 1 hour TTL)
            let now = SystemTime::now();
            if entry.expires_at > now {
                debug!("Using cached JWKS");
                return Ok(entry.jwks.clone());
            }
        }
    }

    // Cache is stale or doesn't exist, fetch new JWKS
    info!("Fetching JWKS from {}", config.jwks_uri);
    let response = config
        .client
        .get(&config.jwks_uri)
        .send()
        .await
        .map_err(|e| AuthError::InternalError(format!("Failed to fetch JWKS: {}", e)))?;

    if !response.status().is_success() {
        return Err(AuthError::InternalError(format!(
            "Failed to fetch JWKS, status: {}",
            response.status()
        )));
    }

    let jwks: JwksResponse = response
        .json()
        .await
        .map_err(|e| AuthError::InternalError(format!("Failed to parse JWKS: {}", e)))?;

    // Cache the JWKS with 1 hour TTL
    let expires_at = SystemTime::now() + Duration::from_secs(3600);
    {
        let mut cache = config.jwks_cache.lock().unwrap();
        *cache = Some(JwksCacheEntry {
            jwks: jwks.clone(),
            expires_at,
        });
    }

    Ok(jwks)
}

/// Find a JWK by its key ID
fn find_jwk_by_kid(jwks: &JwksResponse, kid: &str) -> Result<Jwk, AuthError> {
    jwks.keys
        .iter()
        .find(|key| key.key_id == kid)
        .cloned()
        .ok_or_else(|| AuthError::ValidationFailed(format!("JWK with kid '{}' not found", kid)))
}

/// Create a decoding key from a JWK
fn create_decoding_key(jwk: &Jwk) -> Result<DecodingKey, AuthError> {
    // Try using x509 certificate chain first
    if let Some(x509_chain) = &jwk.x509_chain {
        if let Some(cert) = x509_chain.first() {
            return DecodingKey::from_rsa_pem(
                format!(
                    "-----BEGIN CERTIFICATE-----\n{}\n-----END CERTIFICATE-----",
                    cert
                )
                .as_bytes(),
            )
            .map_err(|e| {
                AuthError::ValidationFailed(format!(
                    "Failed to create decoding key from x509: {}",
                    e
                ))
            });
        }
    }

    // Fall back to modulus and exponent
    if let (Some(n), Some(e)) = (&jwk.modulus, &jwk.exponent) {
        return DecodingKey::from_rsa_components(n, e).map_err(|e| {
            AuthError::ValidationFailed(format!(
                "Failed to create decoding key from components: {}",
                e
            ))
        });
    }

    Err(AuthError::ValidationFailed(
        "JWK doesn't contain required key material".to_string(),
    ))
}

/// Validate claims in the token
fn validate_claims(claims: &EntraClaims, config: &EntraAuthConfig) -> Result<(), AuthError> {
    // Authorization based on roles
    match &config.required_roles {
        RoleRequirement::Any(required_roles) => {
            if required_roles.is_empty() {
                return Ok(());
            }

            for role in required_roles {
                if claims.roles.contains(role) {
                    return Ok(());
                }
            }

            Err(AuthError::AccessDenied(format!(
                "Access denied: User does not have any of the required roles: {:?}",
                required_roles
            )))
        }
        RoleRequirement::All(required_roles) => {
            if required_roles.is_empty() {
                return Ok(());
            }

            for role in required_roles {
                if !claims.roles.contains(role) {
                    return Err(AuthError::AccessDenied(format!(
                        "Access denied: User missing required role: {}",
                        role
                    )));
                }
            }

            Ok(())
        }
        RoleRequirement::None => Ok(()),
    }
}

/// Middleware layer for Entra ID authentication
#[derive(Clone, Debug)]
pub struct EntraAuthLayer {
    config: EntraAuthConfig,
}

impl EntraAuthLayer {
    /// Create a new EntraAuthLayer with default configuration
    pub fn default() -> Self {
        Self {
            config: EntraAuthConfig::default(),
        }
    }

    /// Create a new EntraAuthLayer with the given configuration
    pub fn new(config: EntraAuthConfig) -> Self {
        Self { config }
    }

    /// Create a new auth layer with role requirements
    pub fn with_roles(roles: RoleRequirement) -> Self {
        let mut config = EntraAuthConfig::default();
        config.required_roles = roles;
        Self::new(config)
    }

    /// Create a new EntraAuthLayer that requires any of the given roles
    pub fn require_any_role(roles: Vec<String>) -> Self {
        Self::with_roles(RoleRequirement::Any(roles))
    }

    /// Create a new auth layer that requires all of the specified roles
    pub fn require_all_roles(roles: Vec<String>) -> Self {
        Self::with_roles(RoleRequirement::All(roles))
    }
}

impl<S> Layer<S> for EntraAuthLayer {
    type Service = EntraAuthMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        EntraAuthMiddleware {
            inner,
            config: self.config.clone(),
        }
    }
}

/// Middleware for Entra ID authentication
#[derive(Clone)]
pub struct EntraAuthMiddleware<S> {
    inner: S,
    config: EntraAuthConfig,
}

impl<S> EntraAuthMiddleware<S> {
    /// Create a new EntraAuthMiddleware with the given service and configuration
    pub fn new(inner: S, config: EntraAuthConfig) -> Self {
        Self { inner, config }
    }
}

// Implement the middleware using axum's middleware approach
impl<S> tower::Service<Request> for EntraAuthMiddleware<S>
where
    S: tower::Service<Request, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let config = self.config.clone();
        let inner = self.inner.clone();

        Box::pin(async move {
            let mut inner_svc = inner;

            // Handle the auth validation
            match validate_token_wrapper(req, &config).await {
                Ok(req) => inner_svc.call(req).await,
                Err(err) => Ok(err.into_response()),
            }
        })
    }
}

/// Wrapper function to validate token without using Next
async fn validate_token_wrapper(
    mut req: Request,
    config: &EntraAuthConfig,
) -> Result<Request, AuthError> {
    // Skip validation if disabled (for debugging or development)
    if !config.validate_token {
        debug!("Token validation is disabled, skipping verification");
        return Ok(req);
    }

    // Extract the token from the Authorization header
    let token = extract_token(req.headers())?;

    // Get token header to retrieve kid
    let header = decode_header(&token).map_err(|e| {
        AuthError::ValidationFailed(format!("Failed to decode token header: {}", e))
    })?;

    // Get the kid from the header
    let kid = header.kid.ok_or_else(|| {
        AuthError::ValidationFailed("Token header does not contain a 'kid'".to_string())
    })?;

    // Fetch and cache JWKS
    let jwks = fetch_and_cache_jwks(config).await?;

    // Find the JWK for this kid
    let jwk = find_jwk_by_kid(&jwks, &kid)?;

    // Create a decoding key from the JWK
    let decoding_key = create_decoding_key(&jwk)?;

    // Set up validation parameters
    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_audience(&[config.audience.clone()]);
    validation.set_issuer(&[format!(
        "https://login.microsoftonline.com/{}/v2.0",
        config.tenant_id
    )]);

    // Decode and validate the token
    let token_data = decode::<EntraClaims>(&token, &decoding_key, &validation)
        .map_err(|e| AuthError::ValidationFailed(format!("Token validation failed: {}", e)))?;

    // Additional role-based validation
    validate_claims(&token_data.claims, config)?;

    info!(
        "Successfully validated token for subject: {}",
        token_data.claims.sub
    );
    debug!("User roles: {:?}", token_data.claims.roles);

    // Store claims in request extensions for handlers to access
    req.extensions_mut().insert(token_data.claims);

    Ok(req)
}
