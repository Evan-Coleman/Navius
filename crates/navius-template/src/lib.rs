//! # Navius Template
//!
//! A flexible, template rendering system for the Navius framework with support for multiple
//! template engines through a common interface.
//!
//! ## Features
//!
//! - Multiple template engines: Support for Handlebars, Tera, and Liquid template engines
//! - Unified interface: Common API across all template engines
//! - Async rendering: First-class async/await support
//! - Template caching: Efficient caching of parsed templates
//! - Partial templates: Support for template partials and includes
//! - Localization-ready: Design with internationalization in mind
//! - Type-safe contexts: Strongly typed context variables with Serde
//! - Comprehensive error handling: Detailed error types and helpful messages
//! - Extensible: Easy to add support for additional template engines
//!
//! ## Quick Start
//!
//! ```no_run
//! use navius_template::error::TemplateResult;
//! use navius_template::engine::{TemplateEngine, TemplateEngineFactory};
//! # #[cfg(feature = "handlebars")]
//! use navius_template::handlebars::HandlebarsTemplateEngineFactory;
//! use serde_json::json;
//!
//! # #[cfg(feature = "handlebars")]
//! # async fn example() -> TemplateResult<()> {
//!     // Create a template engine
//!     let factory = HandlebarsTemplateEngineFactory::new();
//!     let mut engine = factory.create_engine().await?;
//!     
//!     // Register a template
//!     engine.register_template_string("greeting", "Hello, {{name}}!").await?;
//!     
//!     // Render the template with data
//!     let context = json!({
//!         "name": "World"
//!     });
//!     
//!     let output = engine.render("greeting", &context).await?;
//!     println!("{}", output); // Outputs: Hello, World!
//!     
//!     Ok(())
//! # }
//! ```

pub mod cache;
pub mod engine;
pub mod error;
pub mod registry;

// Template engine implementations are behind feature flags
#[cfg(feature = "handlebars")]
pub mod handlebars;

#[cfg(feature = "tera")]
pub mod tera;

#[cfg(feature = "liquid")]
pub mod liquid;

// Re-export main types
pub use engine::{
    MetricsTemplateEngine, TemplateEngine, TemplateEngineConfig, TemplateEngineFactory,
    TemplateMetrics,
};
pub use error::{TemplateError, TemplateResult};
pub use registry::TemplateEngineRegistry;

/// Provides access to the version number of this crate at runtime
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Creates a builder for the template system
pub fn builder() -> registry::TemplateEngineRegistryBuilder {
    registry::TemplateEngineRegistryBuilder::new()
}
