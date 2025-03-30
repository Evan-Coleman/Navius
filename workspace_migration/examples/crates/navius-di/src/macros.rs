// Macros for Navius Dependency Injection
//
// This module provides macros for defining components, services,
// and dependency injection annotations.

/// Macro to define a component with dependency injection
///
/// # Examples
///
/// ```
/// # use navius_di::component;
/// # use std::sync::Arc;
/// #
/// #[component]
/// struct UserService {
///     #[inject]
///     db_service: Arc<DatabaseService>,
///     
///     #[inject(qualifier = "main")]
///     config: Arc<Config>,
/// }
/// ```
#[macro_export]
macro_rules! component {
    ($vis:vis struct $name:ident {
        $(
            $(#[$attr:meta])*
            $field_vis:vis $field_name:ident: $field_type:ty
        ),* $(,)?
    }) => {
        $vis struct $name {
            $(
                $(#[$attr])*
                $field_vis $field_name: $field_type,
            )*
        }

        impl $crate::registry::Component for $name {
            fn create(registry: &$crate::registry::ComponentRegistry) -> $crate::error::Result<Self> {
                Ok(Self {
                    $(
                        $field_name: {
                            // Field-specific creation logic would go here
                            // In a real implementation, this would be generated based on field attributes
                            unimplemented!("This is just a placeholder; use #[component] attribute macro instead")
                        },
                    )*
                })
            }
        }
    };
}

/// Marks a struct field for dependency injection
///
/// # Examples
///
/// ```
/// # use navius_di::inject;
/// # use std::sync::Arc;
/// #
/// struct UserService {
///     #[inject]
///     db_service: Arc<DatabaseService>,
///     
///     #[inject(qualifier = "main")]
///     config: Arc<Config>,
/// }
/// ```
#[macro_export]
macro_rules! inject {
    () => {
        // This is just a placeholder for documentation
        // The actual implementation would be in a proc macro
    };
}

/// Marks a configuration struct for auto-binding
///
/// # Examples
///
/// ```
/// # use navius_di::config;
/// # use serde::Deserialize;
/// #
/// #[config(prefix = "app")]
/// #[derive(Debug, Deserialize)]
/// struct AppConfig {
///     name: String,
///     version: String,
/// }
/// ```
#[macro_export]
macro_rules! config {
    (prefix = $prefix:expr) => {
        // This is just a placeholder for documentation
        // The actual implementation would be in a proc macro
    };
}

/// Defines an application module that registers components
///
/// # Examples
///
/// ```
/// # use navius_di::module;
/// #
/// #[module]
/// pub struct AppModule;
///
/// impl AppModule {
///     #[bean]
///     pub fn database_service(config: &Config) -> DatabaseService {
///         DatabaseService::new(config.db_url.clone())
///     }
///
///     #[bean(qualifier = "primary")]
///     pub fn user_service(db: &DatabaseService) -> UserService {
///         UserService::new(db.clone())
///     }
/// }
/// ```
#[macro_export]
macro_rules! module {
    () => {
        // This is just a placeholder for documentation
        // The actual implementation would be in a proc macro
    };
}

/// Defines a factory method for creating a component
///
/// # Examples
///
/// ```
/// # use navius_di::bean;
/// #
/// pub struct AppConfig;
///
/// impl AppConfig {
///     #[bean]
///     pub fn database_service(&self) -> DatabaseService {
///         DatabaseService::new("jdbc:postgresql://localhost:5432/mydb")
///     }
///
///     #[bean(scope = "prototype")]
///     pub fn user_service(&self, db: &DatabaseService) -> UserService {
///         UserService::new(db.clone())
///     }
/// }
/// ```
#[macro_export]
macro_rules! bean {
    () => {
        // This is just a placeholder for documentation
        // The actual implementation would be in a proc macro
    };
}

/// Trait for objects that can be created from a registry
pub trait Component: Sized {
    /// Create an instance of this component from a registry
    fn create(registry: &crate::registry::ComponentRegistry) -> crate::error::Result<Self>;
}

/// Creates a component from the registry
#[macro_export]
macro_rules! autowire {
    ($registry:expr, $component:ty) => {
        <$component as $crate::macros::Component>::create($registry)
    };
}
