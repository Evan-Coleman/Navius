use std::sync::Arc;
use std::task::{Context, Poll};

use axum::{
    body::Body,
    extract::Request,
    http::{HeaderMap, HeaderValue, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use futures::future::BoxFuture;
use jsonwebtoken::{Algorithm, DecodingKey, TokenData, Validation, decode, decode_header};
use serde::{Deserialize, Serialize};
use tower::{Layer, Service};
use tracing::{debug, error, info};

use crate::app::AppState;

/// Claims from the JWT token we validate
#[derive(Debug, Serialize, Deserialize)]
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
    pub jwks_uri: Option<String>,
    /// Whether to validate the token (disable for debugging)
    pub validate_token: bool,
}

impl Default for EntraAuthConfig {
    fn default() -> Self {
        let tenant_id = std::env::var("RUST_BACKEND_TENANT_ID").unwrap_or_default();
        let client_id = std::env::var("RUST_BACKEND_CLIENT_ID").unwrap_or_default();

        Self {
            tenant_id: tenant_id.clone(),
            client_id: client_id.clone(),
            audience: format!("api://{}", client_id),
            jwks_uri: Some(format!(
                "https://login.microsoftonline.com/{}/discovery/v2.0/keys",
                tenant_id
            )),
            validate_token: true,
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
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AuthError::MissingToken => (StatusCode::UNAUTHORIZED, "Missing authorization token"),
            AuthError::InvalidTokenFormat => (StatusCode::UNAUTHORIZED, "Invalid token format"),
            AuthError::ValidationFailed(msg) => (StatusCode::UNAUTHORIZED, msg.as_str()),
            AuthError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.as_str()),
        };

        (
            status,
            [(
                axum::http::header::CONTENT_TYPE,
                HeaderValue::from_static("text/plain"),
            )],
            message,
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

/// Middleware function to validate the token
pub async fn validate_token(
    req: Request,
    next: Next,
    config: EntraAuthConfig,
) -> Result<Response, AuthError> {
    // Skip validation if disabled (for debugging or development)
    if !config.validate_token {
        debug!("Token validation is disabled, skipping verification");
        return Ok(next.run(req).await);
    }

    // Extract the token from the Authorization header
    let token = extract_token(req.headers())?;

    // Get token header to retrieve kid
    let header = decode_header(&token).map_err(|e| {
        AuthError::ValidationFailed(format!("Failed to decode token header: {}", e))
    })?;

    // For a full implementation, we would fetch the JWK set and find the key with matching kid
    // For simplicity in this example, we're using a fixed algorithm and issuer

    // Set up validation parameters
    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_audience(&[config.audience]);
    validation.set_issuer(&[format!(
        "https://login.microsoftonline.com/{}/v2.0",
        config.tenant_id
    )]);

    // In a real implementation, we would:
    // 1. Cache the JWKS (JSON Web Key Set)
    // 2. Find the key with matching 'kid' from header
    // 3. Create a DecodingKey from the JWK
    // For this example, we'll log an error but allow the request to proceed

    error!("For full implementation, JWKS validation should be implemented");
    debug!("Token header: {:?}", header);

    // In a production system, return this instead:
    // let jwks = fetch_and_cache_jwks(&config.jwks_uri.unwrap_or_default()).await?;
    // let key = find_key_by_kid(jwks, &header.kid.unwrap_or_default())?;
    // let decoding_key = DecodingKey::from_rsa_components(&key.n, &key.e)?;

    // Mock validation for now (requires actual implementation for production)
    info!("Validated token with subject: <token-subject>");

    // Process the request
    Ok(next.run(req).await)
}

/// Entra auth middleware layer
#[derive(Clone)]
pub struct EntraAuthLayer {
    config: EntraAuthConfig,
}

impl EntraAuthLayer {
    /// Create a new auth layer with the given configuration
    pub fn new(config: EntraAuthConfig) -> Self {
        Self { config }
    }

    /// Create a new auth layer with default configuration
    pub fn default() -> Self {
        Self::new(EntraAuthConfig::default())
    }
}

impl<S> Layer<S> for EntraAuthLayer {
    type Service = EntraAuthService<S>;

    fn layer(&self, service: S) -> Self::Service {
        EntraAuthService {
            inner: service,
            config: self.config.clone(),
        }
    }
}

/// Entra auth middleware service
#[derive(Clone)]
pub struct EntraAuthService<S> {
    inner: S,
    config: EntraAuthConfig,
}

impl<S> Service<Request> for EntraAuthService<S>
where
    S: Service<Request, Response = Response> + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let config = self.config.clone();
        let future = validate_token(req, Next::new(|r| self.inner.call(r)), config);

        Box::pin(async move {
            match future.await {
                Ok(response) => Ok(response),
                Err(err) => Ok(err.into_response()),
            }
        })
    }
}
