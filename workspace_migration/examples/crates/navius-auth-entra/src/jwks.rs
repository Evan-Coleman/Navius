use crate::config::EntraConfig;
use crate::error::{EntraError, EntraResult};
use chrono::{DateTime, Utc};
use jsonwebtoken::{Algorithm, DecodingKey};
use lru::LruCache;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::sync::{Arc, RwLock};
use std::time::Duration;

/// JSON Web Key (JWK) structure as defined in RFC 7517
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonWebKey {
    /// Key type
    #[serde(rename = "kty")]
    pub key_type: String,

    /// Key ID
    #[serde(rename = "kid")]
    pub key_id: String,

    /// Key use (sig for signature, enc for encryption)
    #[serde(rename = "use", default)]
    pub key_use: Option<String>,

    /// Key operations
    #[serde(rename = "key_ops", default)]
    pub key_operations: Option<Vec<String>>,

    /// Algorithm
    #[serde(rename = "alg", default)]
    pub algorithm: Option<String>,

    /// X.509 certificate chain (base64url-encoded DER)
    #[serde(rename = "x5c", default)]
    pub x509_chain: Option<Vec<String>>,

    /// X.509 certificate URL
    #[serde(rename = "x5u", default)]
    pub x509_url: Option<String>,

    /// X.509 certificate SHA-1 thumbprint (base64url-encoded)
    #[serde(rename = "x5t", default)]
    pub x509_thumbprint: Option<String>,

    /// X.509 certificate SHA-256 thumbprint (base64url-encoded)
    #[serde(rename = "x5t#S256", default)]
    pub x509_thumbprint_sha256: Option<String>,

    /// RSA modulus (base64url-encoded)
    #[serde(rename = "n", default)]
    pub modulus: Option<String>,

    /// RSA exponent (base64url-encoded)
    #[serde(rename = "e", default)]
    pub exponent: Option<String>,

    /// EC curve
    #[serde(rename = "crv", default)]
    pub curve: Option<String>,

    /// EC x coordinate (base64url-encoded)
    #[serde(rename = "x", default)]
    pub x_coordinate: Option<String>,

    /// EC y coordinate (base64url-encoded)
    #[serde(rename = "y", default)]
    pub y_coordinate: Option<String>,

    /// Additional properties
    #[serde(flatten)]
    pub additional_properties: HashMap<String, serde_json::Value>,
}

impl JsonWebKey {
    /// Convert the JWK to a DecodingKey for JWT validation
    pub fn to_decoding_key(&self) -> EntraResult<DecodingKey> {
        match self.key_type.as_str() {
            "RSA" => {
                // Try to create from x509 certificate first
                if let Some(x509_chain) = &self.x509_chain {
                    if let Some(cert) = x509_chain.first() {
                        return DecodingKey::from_rsa_pem(cert.as_bytes()).map_err(|e| {
                            EntraError::jwks(format!(
                                "Failed to create RSA decoding key from certificate: {}",
                                e
                            ))
                        });
                    }
                }

                // Otherwise, try to create from modulus and exponent
                if let (Some(n), Some(e)) = (&self.modulus, &self.exponent) {
                    return DecodingKey::from_rsa_components(n, e).map_err(|e| {
                        EntraError::jwks(format!(
                            "Failed to create RSA decoding key from components: {}",
                            e
                        ))
                    });
                }

                Err(EntraError::jwks("RSA key missing required components"))
            }
            "EC" => {
                if let (Some(x), Some(y), Some(crv)) =
                    (&self.x_coordinate, &self.y_coordinate, &self.curve)
                {
                    let curve = match crv.as_str() {
                        "P-256" => jsonwebtoken::jwk::EllipticCurve::P256,
                        "P-384" => jsonwebtoken::jwk::EllipticCurve::P384,
                        "P-521" => jsonwebtoken::jwk::EllipticCurve::P521,
                        _ => {
                            return Err(EntraError::jwks(format!("Unsupported EC curve: {}", crv)));
                        }
                    };

                    return DecodingKey::from_ec_components(x, y, curve).map_err(|e| {
                        EntraError::jwks(format!("Failed to create EC decoding key: {}", e))
                    });
                }

                Err(EntraError::jwks("EC key missing required components"))
            }
            "oct" => {
                // Symmetric keys are not supported for JWT validation with Entra ID
                Err(EntraError::jwks(
                    "Symmetric keys are not supported for JWT validation",
                ))
            }
            _ => Err(EntraError::jwks(format!(
                "Unsupported key type: {}",
                self.key_type
            ))),
        }
    }

    /// Get the algorithm associated with this key
    pub fn get_algorithm(&self) -> EntraResult<Algorithm> {
        // Try to get from the alg field first
        if let Some(alg) = &self.algorithm {
            return match alg.as_str() {
                "RS256" => Ok(Algorithm::RS256),
                "RS384" => Ok(Algorithm::RS384),
                "RS512" => Ok(Algorithm::RS512),
                "ES256" => Ok(Algorithm::ES256),
                "ES384" => Ok(Algorithm::ES384),
                "HS256" => Ok(Algorithm::HS256),
                "HS384" => Ok(Algorithm::HS384),
                "HS512" => Ok(Algorithm::HS512),
                _ => Err(EntraError::jwks(format!("Unsupported algorithm: {}", alg))),
            };
        }

        // If alg is not specified, infer from key type
        match self.key_type.as_str() {
            "RSA" => Ok(Algorithm::RS256), // Default for RSA keys in Entra ID
            "EC" => {
                // Infer from curve if available
                if let Some(crv) = &self.curve {
                    match crv.as_str() {
                        "P-256" => Ok(Algorithm::ES256),
                        "P-384" => Ok(Algorithm::ES384),
                        _ => Err(EntraError::jwks(format!("Unsupported EC curve: {}", crv))),
                    }
                } else {
                    Ok(Algorithm::ES256) // Default for EC keys
                }
            }
            "oct" => Ok(Algorithm::HS256), // Default for symmetric keys
            _ => Err(EntraError::jwks(format!(
                "Cannot infer algorithm for key type: {}",
                self.key_type
            ))),
        }
    }
}

/// JSON Web Key Set (JWKS) structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonWebKeySet {
    /// Set of JSON Web Keys
    pub keys: Vec<JsonWebKey>,
}

impl JsonWebKeySet {
    /// Find a key by key ID
    pub fn find_key(&self, kid: &str) -> Option<&JsonWebKey> {
        self.keys.iter().find(|key| key.key_id == kid)
    }

    /// Find a key by key ID and algorithm
    pub fn find_key_for_algorithm(&self, kid: &str, alg: Algorithm) -> Option<&JsonWebKey> {
        self.keys
            .iter()
            .find(|key| key.key_id == kid && key.get_algorithm().map(|a| a == alg).unwrap_or(false))
    }
}

/// Cached JWKS entry
struct CachedJwks {
    /// The JWKS data
    jwks: JsonWebKeySet,
    /// Expiration time
    expires_at: DateTime<Utc>,
}

/// JWKS client for retrieving and caching JSON Web Key Sets
pub struct JwksClient {
    /// HTTP client for making requests
    http_client: Client,
    /// Configuration
    config: Arc<EntraConfig>,
    /// Cache of JWKS data
    cache: RwLock<LruCache<String, CachedJwks>>,
}

impl JwksClient {
    /// Create a new JWKS client
    pub fn new(config: Arc<EntraConfig>) -> Self {
        let http_client = Client::builder()
            .timeout(config.http_timeout())
            .user_agent(&config.http_client.user_agent)
            .build()
            .unwrap_or_default();

        let cache_size = NonZeroUsize::new(config.jwks_cache.max_size)
            .unwrap_or_else(|| NonZeroUsize::new(100).unwrap());

        Self {
            http_client,
            config,
            cache: RwLock::new(LruCache::new(cache_size)),
        }
    }

    /// Fetch JWKS from the Microsoft Entra endpoint
    pub async fn fetch_jwks(&self) -> EntraResult<JsonWebKeySet> {
        let jwks_url = self.config.jwks_endpoint()?;

        // Attempt to get from cache first
        if self.config.jwks_cache.enabled {
            let cache_hit = {
                let cache = self
                    .cache
                    .read()
                    .map_err(|e| EntraError::internal(format!("Cache lock error: {}", e)))?;

                if let Some(cached) = cache.peek(&jwks_url) {
                    if cached.expires_at > Utc::now() {
                        Some(cached.jwks.clone())
                    } else {
                        None
                    }
                } else {
                    None
                }
            };

            if let Some(jwks) = cache_hit {
                return Ok(jwks);
            }
        }

        // Cache miss or disabled, fetch from server
        let response = self
            .http_client
            .get(&jwks_url)
            .send()
            .await
            .map_err(|e| EntraError::jwks(format!("Failed to fetch JWKS: {}", e)))?;

        if !response.status().is_success() {
            return Err(EntraError::jwks(format!(
                "JWKS endpoint returned error status: {}",
                response.status()
            )));
        }

        let jwks: JsonWebKeySet = response
            .json()
            .await
            .map_err(|e| EntraError::jwks(format!("Failed to parse JWKS response: {}", e)))?;

        // Update cache
        if self.config.jwks_cache.enabled {
            let expires_at =
                Utc::now() + chrono::Duration::seconds(self.config.jwks_cache.ttl_seconds as i64);

            let mut cache = self
                .cache
                .write()
                .map_err(|e| EntraError::internal(format!("Cache lock error: {}", e)))?;

            cache.put(
                jwks_url,
                CachedJwks {
                    jwks: jwks.clone(),
                    expires_at,
                },
            );
        }

        Ok(jwks)
    }

    /// Get the decoding key for the specified key ID
    pub async fn get_key_for_kid(&self, kid: &str) -> EntraResult<DecodingKey> {
        let jwks = self.fetch_jwks().await?;

        let key = jwks
            .find_key(kid)
            .ok_or_else(|| EntraError::jwks(format!("Key with ID '{}' not found in JWKS", kid)))?;

        key.to_decoding_key()
    }

    /// Get the decoding key for the specified key ID and algorithm
    pub async fn get_key_for_kid_and_alg(
        &self,
        kid: &str,
        alg: Algorithm,
    ) -> EntraResult<DecodingKey> {
        let jwks = self.fetch_jwks().await?;

        let key = jwks.find_key_for_algorithm(kid, alg).ok_or_else(|| {
            EntraError::jwks(format!(
                "Key with ID '{}' and algorithm {:?} not found in JWKS",
                kid, alg
            ))
        })?;

        key.to_decoding_key()
    }

    /// Clear the JWKS cache
    pub fn clear_cache(&self) -> EntraResult<()> {
        let mut cache = self
            .cache
            .write()
            .map_err(|e| EntraError::internal(format!("Cache lock error: {}", e)))?;
        cache.clear();
        Ok(())
    }

    /// Get the current cache size
    pub fn cache_size(&self) -> EntraResult<usize> {
        let cache = self
            .cache
            .read()
            .map_err(|e| EntraError::internal(format!("Cache lock error: {}", e)))?;
        Ok(cache.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_key_by_id() {
        let jwks = JsonWebKeySet {
            keys: vec![
                JsonWebKey {
                    key_type: "RSA".to_string(),
                    key_id: "key1".to_string(),
                    key_use: Some("sig".to_string()),
                    key_operations: None,
                    algorithm: Some("RS256".to_string()),
                    x509_chain: None,
                    x509_url: None,
                    x509_thumbprint: None,
                    x509_thumbprint_sha256: None,
                    modulus: Some("n_value".to_string()),
                    exponent: Some("e_value".to_string()),
                    curve: None,
                    x_coordinate: None,
                    y_coordinate: None,
                    additional_properties: HashMap::new(),
                },
                JsonWebKey {
                    key_type: "EC".to_string(),
                    key_id: "key2".to_string(),
                    key_use: Some("sig".to_string()),
                    key_operations: None,
                    algorithm: Some("ES256".to_string()),
                    x509_chain: None,
                    x509_url: None,
                    x509_thumbprint: None,
                    x509_thumbprint_sha256: None,
                    modulus: None,
                    exponent: None,
                    curve: Some("P-256".to_string()),
                    x_coordinate: Some("x_value".to_string()),
                    y_coordinate: Some("y_value".to_string()),
                    additional_properties: HashMap::new(),
                },
            ],
        };

        assert!(jwks.find_key("key1").is_some());
        assert!(jwks.find_key("key2").is_some());
        assert!(jwks.find_key("key3").is_none());

        assert!(
            jwks.find_key_for_algorithm("key1", Algorithm::RS256)
                .is_some()
        );
        assert!(
            jwks.find_key_for_algorithm("key1", Algorithm::RS384)
                .is_none()
        );
        assert!(
            jwks.find_key_for_algorithm("key2", Algorithm::ES256)
                .is_some()
        );
        assert!(
            jwks.find_key_for_algorithm("key2", Algorithm::ES384)
                .is_none()
        );
    }
}
