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
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
use reqwest::Client;
use serde::{Deserialize, Serialize};
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
    jwks: JwksResponse,
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
    /// Whether to validate the token (disable for debugging)
    pub validate_token: bool,
    /// Required roles for authorization
    pub required_roles: RoleRequirement,
    /// HTTP client
    pub client: Client,
    /// JWKS URI for token validation
    pub jwks_uri: String,
    /// JWKS cache
    pub jwks_cache: Arc<Mutex<Option<JwksCacheEntry>>>,
    /// Debug mode - skips signature validation
    pub debug_validation: bool,
}

impl Default for EntraAuthConfig {
    fn default() -> Self {
        let tenant_id = std::env::var("RUST_BACKEND_TENANT_ID").unwrap_or_default();
        let client_id = std::env::var("RUST_BACKEND_CLIENT_ID").unwrap_or_default();
        let debug_validation = std::env::var("DEBUG_AUTH")
            .unwrap_or_else(|_| "false".to_string())
            .parse::<bool>()
            .unwrap_or(false);
        let audience = std::env::var("RUST_BACKEND_AUDIENCE")
            .unwrap_or_else(|_| format!("api://{}", client_id));

        Self {
            tenant_id: tenant_id.clone(),
            client_id: client_id.clone(),
            audience,
            validate_token: true,
            required_roles: RoleRequirement::None,
            client: Client::new(),
            jwks_uri: format!(
                "https://login.microsoftonline.com/{}/discovery/v2.0/keys",
                tenant_id
            ),
            jwks_cache: Arc::new(Mutex::new(None)),
            debug_validation,
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

    let token = header_str[7..].trim().to_string();

    // Basic JWT format validation - should have 3 parts separated by dots
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Err(AuthError::ValidationFailed(
            "Invalid JWT format: token must have three parts (header.payload.signature)"
                .to_string(),
        ));
    }

    // Each part should be non-empty and contain valid Base64URL characters
    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() {
            return Err(AuthError::ValidationFailed(format!(
                "JWT token part {} is empty",
                i + 1
            )));
        }

        // Very basic Base64URL validation - more thorough validation happens later
        for c in part.chars() {
            if !(c.is_alphanumeric() || c == '_' || c == '-' || c == '=') {
                return Err(AuthError::ValidationFailed(format!(
                    "Invalid JWT token: contains non-base64url characters"
                )));
            }
        }
    }

    Ok(token)
}

/// Fetch and cache JWKS from the Microsoft endpoint
async fn fetch_jwks(config: &EntraAuthConfig) -> Result<JwksResponse, AuthError> {
    // Check if we have a cached JWKS that's still valid
    {
        let cache = config.jwks_cache.lock().unwrap();
        if let Some(cache_entry) = &*cache {
            if cache_entry.expires_at > SystemTime::now() {
                return Ok(cache_entry.jwks.clone());
            }
        }
    }

    // If not, fetch a new JWKS
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

    let jwks = response
        .json::<JwksResponse>()
        .await
        .map_err(|e| AuthError::InternalError(format!("Failed to parse JWKS: {}", e)))?;

    // Cache the JWKS for 1 hour
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
fn find_jwk<'a>(jwks: &'a JwksResponse, kid: &str) -> Result<&'a Jwk, AuthError> {
    jwks.keys
        .iter()
        .find(|key| key.key_id == kid)
        .ok_or_else(|| {
            AuthError::ValidationFailed(format!("No matching key found for kid: {}", kid))
        })
}

/// Create a decoding key from a JWK
fn create_decoding_key(jwk: &Jwk) -> Result<DecodingKey, AuthError> {
    // First try X.509 certificate chain
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
                AuthError::ValidationFailed(format!("Failed to create key from certificate: {}", e))
            });
        }
    }

    // Fall back to modulus and exponent
    if let (Some(n), Some(e)) = (&jwk.modulus, &jwk.exponent) {
        return DecodingKey::from_rsa_components(n, e).map_err(|e| {
            AuthError::ValidationFailed(format!("Failed to create key from components: {}", e))
        });
    }

    Err(AuthError::ValidationFailed(
        "JWK doesn't contain necessary key material".to_string(),
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
    debug!("Token extracted from header, starting validation");

    // Try to decode header - if this fails, the token is not a valid JWT
    let header = match decode_header(&token) {
        Ok(h) => h,
        Err(e) => {
            error!(
                "Token tampering detected: failed to decode token header: {}",
                e
            );
            // Provide a clearer error message for malformed tokens
            if e.to_string().contains("control character") {
                return Err(AuthError::ValidationFailed(
                    "Invalid token format: not a valid JWT token".to_string(),
                ));
            } else {
                return Err(AuthError::ValidationFailed(format!("Invalid token: {}", e)));
            }
        }
    };

    if config.debug_validation {
        // In debug mode, use simplified validation (not recommended for production)
        debug!("DEBUG MODE: Using simplified token validation without signature verification");

        // Parse token without validating signature
        let mut validation = Validation::new(Algorithm::RS256);
        validation.validate_exp = true;
        validation.validate_nbf = true;
        validation.set_audience(&[config.audience.clone()]);
        validation.insecure_disable_signature_validation();

        // Use an empty key since we're not validating signatures
        let dummy_key = DecodingKey::from_secret(&[]);

        // Try to decode the token with better error handling
        let token_data = match decode::<EntraClaims>(&token, &dummy_key, &validation) {
            Ok(data) => data,
            Err(e) => {
                error!("Token validation failed: {}", e);
                let detail = match e.kind() {
                    jsonwebtoken::errors::ErrorKind::InvalidToken => "Token format is invalid",
                    jsonwebtoken::errors::ErrorKind::InvalidSignature => "Invalid signature",
                    jsonwebtoken::errors::ErrorKind::Base64(_) => {
                        "Base64 decoding error - token may be malformed or corrupted"
                    }
                    jsonwebtoken::errors::ErrorKind::ExpiredSignature => "Token has expired",
                    jsonwebtoken::errors::ErrorKind::InvalidAudience => {
                        &format!("Invalid audience, expected: {}", config.audience)
                    }
                    jsonwebtoken::errors::ErrorKind::InvalidIssuer => "Invalid issuer",
                    _ => "Token validation failed",
                };
                return Err(AuthError::ValidationFailed(format!("{}: {}", detail, e)));
            }
        };

        // Role-based validation
        validate_claims(&token_data.claims, config)?;

        // Store claims in request extensions
        req.extensions_mut().insert(token_data.claims);

        info!("DEBUG MODE: Token accepted without signature verification");
        return Ok(req);
    }

    // Full validation with signature verification

    // Get the key ID
    let kid = header.kid.ok_or_else(|| {
        AuthError::ValidationFailed("Token header missing 'kid' claim".to_string())
    })?;

    // Fetch JWKS (JSON Web Key Set) from Microsoft
    let jwks = fetch_jwks(config).await?;

    // Find the key matching our token's kid
    let jwk = find_jwk(&jwks, &kid)?;

    // Create a decoding key
    let decoding_key = create_decoding_key(jwk)?;

    // Set up validation parameters
    let mut validation = Validation::new(Algorithm::RS256);
    validation.validate_exp = true;
    validation.validate_nbf = true;
    validation.set_audience(&[config.audience.clone()]);

    // Set valid issuers
    validation.set_issuer(&[
        format!(
            "https://login.microsoftonline.com/{}/v2.0",
            config.tenant_id
        ),
        format!(
            "https://login.microsoftonline.com/{}/v2.0/",
            config.tenant_id
        ),
        format!("https://sts.windows.net/{}", config.tenant_id),
        format!("https://sts.windows.net/{}/", config.tenant_id),
    ]);

    // Validate token with better error handling
    let token_data = match decode::<EntraClaims>(&token, &decoding_key, &validation) {
        Ok(data) => data,
        Err(e) => {
            error!("Token validation failed: {}", e);
            let detail = match e.kind() {
                jsonwebtoken::errors::ErrorKind::InvalidToken => "Token format is invalid",
                jsonwebtoken::errors::ErrorKind::InvalidSignature => "Invalid signature",
                jsonwebtoken::errors::ErrorKind::Base64(_) => {
                    "Base64 decoding error - token may be malformed or corrupted"
                }
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => "Token has expired",
                jsonwebtoken::errors::ErrorKind::InvalidAudience => {
                    &format!("Invalid audience, expected: {}", config.audience)
                }
                jsonwebtoken::errors::ErrorKind::InvalidIssuer => "Invalid issuer",
                _ => "Token validation failed",
            };
            return Err(AuthError::ValidationFailed(format!("{}: {}", detail, e)));
        }
    };

    // Role-based authorization check
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
