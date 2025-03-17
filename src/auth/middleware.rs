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
use crate::config::app_config;
use crate::config::constants;

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
pub struct JwksCacheEntry {
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
    /// Scope in access token (can be string or array)
    #[serde(default)]
    pub scp: Option<String>,
}

impl EntraClaims {
    /// Get scopes as a Vec<String>
    pub fn get_scopes(&self) -> Vec<String> {
        let mut permissions = Vec::new();

        // Check for explicit scopes (delegated permissions)
        if let Some(scope_str) = &self.scp {
            permissions.extend(scope_str.split(' ').map(String::from));
        }

        // Include roles (application permissions) as well
        permissions.extend(self.roles.clone());

        permissions
    }
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

/// Permission (scope) requirements for authorization
#[derive(Debug, Clone)]
pub enum PermissionRequirement {
    /// Any of the listed permissions is sufficient
    Any(Vec<String>),
    /// All of the listed permissions are required
    All(Vec<String>),
    /// No permissions required (authentication only)
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
    /// Required permissions (scopes) for authorization
    pub required_permissions: PermissionRequirement,
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
        // Get configuration from environment variables
        let tenant_id = std::env::var(constants::auth::env_vars::TENANT_ID).unwrap_or_default();
        let client_id = std::env::var(constants::auth::env_vars::CLIENT_ID).unwrap_or_default();
        let debug_validation = std::env::var(constants::auth::env_vars::DEBUG_AUTH)
            .unwrap_or_else(|_| "false".to_string())
            .parse::<bool>()
            .unwrap_or(false);
        let audience = std::env::var(constants::auth::env_vars::AUDIENCE).unwrap_or_else(|_| {
            format!(
                "{}",
                constants::auth::urls::DEFAULT_AUDIENCE_FORMAT.replace("{}", &client_id)
            )
        });

        Self {
            tenant_id: tenant_id.clone(),
            client_id: client_id.clone(),
            audience,
            validate_token: true,
            required_roles: RoleRequirement::None,
            required_permissions: PermissionRequirement::None,
            client: Client::new(),
            jwks_uri: format!(
                "{}",
                constants::auth::urls::ENTRA_JWKS_URI_FORMAT.replace("{}", &tenant_id)
            ),
            jwks_cache: Arc::new(Mutex::new(None)),
            debug_validation,
        }
    }
}

impl EntraAuthConfig {
    /// Create a new EntraAuthConfig from AppConfig
    pub fn from_app_config(config: &app_config::AppConfig) -> Self {
        let tenant_id = config.auth.entra.tenant_id.clone();
        let client_id = config.auth.entra.client_id.clone();
        let debug_validation = config.auth.debug;
        let audience = if config.auth.entra.audience.is_empty() {
            format!(
                "{}",
                constants::auth::urls::DEFAULT_AUDIENCE_FORMAT.replace("{}", &client_id)
            )
        } else {
            config.auth.entra.audience.clone()
        };

        Self {
            tenant_id: tenant_id.clone(),
            client_id,
            audience,
            validate_token: true,
            required_roles: RoleRequirement::None,
            required_permissions: PermissionRequirement::None,
            client: Client::new(),
            jwks_uri: format!(
                "{}",
                constants::auth::urls::ENTRA_JWKS_URI_FORMAT.replace("{}", &tenant_id)
            ),
            jwks_cache: Arc::new(Mutex::new(None)),
            debug_validation,
        }
    }

    /// Build on existing config to modify specific fields
    pub fn with_role_requirement(mut self, role_requirement: RoleRequirement) -> Self {
        self.required_roles = role_requirement;
        self
    }

    /// Build on existing config to add permission requirements
    pub fn with_permission_requirement(
        mut self,
        permission_requirement: PermissionRequirement,
    ) -> Self {
        self.required_permissions = permission_requirement;
        self
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
        let (status, message) = match self {
            AuthError::MissingToken => (
                StatusCode::UNAUTHORIZED,
                "Authentication required: Missing authorization token".to_string(),
            ),
            AuthError::InvalidTokenFormat => (
                StatusCode::UNAUTHORIZED,
                "Authentication failed: Invalid token format".to_string(),
            ),
            AuthError::ValidationFailed(reason) => (
                StatusCode::UNAUTHORIZED,
                format!("Authentication failed: {}", reason),
            ),
            AuthError::InternalError(reason) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Server error during authentication: {}", reason),
            ),
            AuthError::AccessDenied(reason) => (StatusCode::FORBIDDEN, reason),
        };

        let body = axum::Json(serde_json::json!({
            "status": status.as_u16(),
            "error": status.to_string(),
            "message": message,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }));

        (status, body).into_response()
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

    // For debugging, log the token (only in debug mode to avoid security issues)
    debug!("Received token: {}", token);
    debug!(
        "Token parts: Header={} / Payload={} / Signature={}",
        parts[0], parts[1], "..."
    );

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

/// Validate permissions in the token
fn validate_permissions(claims: &EntraClaims, config: &EntraAuthConfig) -> Result<(), AuthError> {
    let scopes = claims.get_scopes();

    debug!("🔍 TOKEN ANALYSIS 🔍");
    debug!("🔑 Subject: {}", claims.sub);
    debug!("🔑 App ID: {:?}", claims.appid);
    debug!("🔑 App URI: {:?}", claims.app_id_uri);
    debug!("🔑 Raw roles in token: {:?}", claims.roles);
    debug!("🔑 Raw scopes in token: {:?}", claims.scp);
    debug!("🔑 All extracted permissions: {:?}", scopes);

    // Authorization based on permissions (scopes)
    match &config.required_permissions {
        PermissionRequirement::Any(required_permissions) => {
            debug!(
                "🛡️ ACCESS CHECK: Endpoint requires ANY of these permissions: {:?}",
                required_permissions
            );

            if required_permissions.is_empty() {
                debug!("✅ No specific permissions required, allowing access");
                return Ok(());
            }

            for permission in required_permissions {
                debug!("🔍 Checking for permission: '{}'", permission);
                if scopes.contains(permission) {
                    debug!(
                        "✅ PERMISSION MATCH: Found required permission '{}' in token",
                        permission
                    );
                    return Ok(());
                }
            }

            error!(
                "❌ ACCESS DENIED: Token for subject '{}' does not have any of the required permissions: {:?}",
                claims.sub, required_permissions
            );
            error!("⚠️ TOKEN PERMISSIONS AVAILABLE: {:?}", scopes);
            error!(
                "⚠️ POTENTIAL FIX: Update the endpoint to require one of these permissions OR update the app registration in Entra to include '{}' in the API permissions",
                required_permissions
                    .get(0)
                    .unwrap_or(&"unknown".to_string())
            );

            Err(AuthError::AccessDenied(format!(
                "Access denied: Token for '{}' does not have any of the required permissions: {:?}. Available permissions: {:?}",
                claims.sub, required_permissions, scopes
            )))
        }
        PermissionRequirement::All(required_permissions) => {
            if required_permissions.is_empty() {
                return Ok(());
            }

            for permission in required_permissions {
                if !scopes.contains(permission) {
                    return Err(AuthError::AccessDenied(format!(
                        "Access denied: Token missing required permission: {}",
                        permission
                    )));
                }
            }

            Ok(())
        }
        PermissionRequirement::None => Ok(()),
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

    /// Create a new EntraAuthLayer from AppConfig
    pub fn from_app_config(config: &crate::config::app_config::AppConfig) -> Self {
        Self {
            config: EntraAuthConfig::from_app_config(config),
        }
    }

    /// Create a new EntraAuthLayer with custom configuration
    pub fn new(config: EntraAuthConfig) -> Self {
        Self { config }
    }

    /// Create a new auth layer with role requirements
    pub fn with_roles(roles: RoleRequirement) -> Self {
        Self::new(EntraAuthConfig::default().with_role_requirement(roles))
    }

    /// Create a new EntraAuthLayer that requires any of the given roles
    pub fn require_any_role(roles: Vec<String>) -> Self {
        Self::with_roles(RoleRequirement::Any(roles))
    }

    /// Create a new auth layer that requires all of the specified roles
    pub fn require_all_roles(roles: Vec<String>) -> Self {
        Self::with_roles(RoleRequirement::All(roles))
    }

    /// Create a new auth layer with permission requirements
    pub fn with_permissions(permissions: PermissionRequirement) -> Self {
        Self::new(EntraAuthConfig::default().with_permission_requirement(permissions))
    }

    /// Create a new auth layer that requires any of the given permissions
    pub fn require_any_permission(permissions: Vec<String>) -> Self {
        Self::with_permissions(PermissionRequirement::Any(permissions))
    }

    /// Create a new auth layer that requires all of the specified permissions
    pub fn require_all_permissions(permissions: Vec<String>) -> Self {
        Self::with_permissions(PermissionRequirement::All(permissions))
    }

    /// Create a new auth layer from app config with added role requirements
    pub fn from_app_config_with_roles(
        config: &crate::config::app_config::AppConfig,
        roles: RoleRequirement,
    ) -> Self {
        Self::new(EntraAuthConfig::from_app_config(config).with_role_requirement(roles))
    }

    /// Create a new auth layer from app config requiring any of the given roles
    pub fn from_app_config_require_any_role(
        config: &crate::config::app_config::AppConfig,
        roles: Vec<String>,
    ) -> Self {
        Self::from_app_config_with_roles(config, RoleRequirement::Any(roles))
    }

    /// Create a new auth layer from app config requiring all of the specified roles
    pub fn from_app_config_require_all_roles(
        config: &crate::config::app_config::AppConfig,
        roles: Vec<String>,
    ) -> Self {
        Self::from_app_config_with_roles(config, RoleRequirement::All(roles))
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
    // Extract the token
    let headers = req.headers();
    let token = extract_token(headers)?;

    let header = decode_header(&token).map_err(|e| {
        AuthError::ValidationFailed(format!("Failed to decode token header: {}", e))
    })?;

    info!(
        "🔐 AUTHENTICATION: Processing token for request: {} {}",
        req.method(),
        req.uri().path()
    );
    debug!(
        "🔐 Token validation - Debug mode: {}, Audience: {}, Token: {}",
        config.debug_validation, config.audience, token
    );

    if config.debug_validation {
        info!(
            "⚠️ DEBUG MODE ACTIVE: Using simplified token validation (no signature verification)"
        );

        // In debug mode, let's completely bypass JWT validation
        // and just accept any token format
        let mut req = req;
        let claims = EntraClaims {
            aud: config.audience.clone(),
            exp: constants::timestamps::YEAR_2100, // Year 2100
            iss: "debug_issuer".to_string(),
            nbf: 0,
            iat: constants::timestamps::JAN_1_2021, // 2021-01-01
            sub: "debug_subject".to_string(),
            roles: vec!["admin".to_string(), "pet-manager".to_string()],
            appid: Some("debug_app_id".to_string()),
            app_id_uri: Some("debug_app_id_uri".to_string()),
            scp: Some(constants::auth::permissions::DEFAULT_PERMISSION.to_string()),
        };

        info!("✅ DEBUG MODE: Using dummy claims instead of token validation");
        req.extensions_mut().insert(claims);
        return Ok(req);
    }

    // Skip validation if disabled (for debugging or development)
    if !config.validate_token {
        debug!("Token validation is disabled, skipping verification");
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

    // Permission-based authorization check
    validate_permissions(&token_data.claims, config)?;

    info!(
        "Successfully validated token for subject: {}",
        token_data.claims.sub
    );
    if !token_data.claims.roles.is_empty() {
        debug!("User roles: {:?}", token_data.claims.roles);
    }
    if token_data.claims.scp.is_some() {
        debug!("User scopes from scp: {:?}", token_data.claims.scp);
    }
    debug!("Combined permissions: {:?}", token_data.claims.get_scopes());

    // Store claims in request extensions for handlers to access
    req.extensions_mut().insert(token_data.claims);

    Ok(req)
}
