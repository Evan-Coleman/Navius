//! # navius-auth-entra
//!
//! Microsoft Entra ID authentication provider for the Navius framework.
//!
//! This crate provides an implementation of the `AuthProvider` trait from `navius-auth`
//! that can be used to authenticate users using Microsoft Entra ID (formerly Azure AD).

pub mod config;
pub mod error;
pub mod jwks;
pub mod provider;
pub mod token;
pub mod user;

pub use crate::config::EntraConfig;
pub use crate::error::{EntraError, EntraResult};
pub use crate::provider::EntraProvider;
pub use crate::token::EntraTokenClaims;
pub use crate::user::EntraUser;

// Re-export required traits from navius-auth
pub use navius_auth::{AuthProvider, ProviderType, Result as AuthResult};

/// Version of the crate
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
