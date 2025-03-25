mod entra;
mod google;

pub use entra::EntraProvider;
pub use google::GoogleProvider;

#[async_trait]
pub trait OAuthProvider: Send + Sync {
    // Trait implementation...
}
