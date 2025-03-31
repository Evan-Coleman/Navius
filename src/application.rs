// Application services and business logic go here
// This module will contain the core business logic of the application

use std::sync::Arc;
use navius_core::error::AppError;
use crate::config::AppConfig;
use crate::infrastructure::ServiceRegistry;

/// Initialize the application services
pub fn init() {
    // Initialize any global application state here
    tracing::info!("Initializing application services");
}

/// Initialize authentication service
#[cfg(feature = "entra-auth")]
pub async fn init_auth_service(
    config: &AppConfig,
    registry: &mut ServiceRegistry,
) -> Result<(), AppError> {
    use navius_auth::AuthService;
    use navius_auth_entra::EntraAuthProvider;
    
    if let Some(auth_config) = &config.auth {
        if !auth_config.enabled {
            tracing::info!("Authentication is disabled in configuration");
            return Ok(());
        }
        
        if auth_config.provider != "entra" {
            return Err(AppError::configuration_error(
                format!("Unsupported auth provider: {}", auth_config.provider),
            ));
        }
        
        if let Some(entra_config) = &auth_config.entra {
            let auth_provider = EntraAuthProvider::new()
                .with_tenant_id(&entra_config.tenant_id)
                .with_client_id(&entra_config.client_id)
                .with_client_secret(&entra_config.client_secret)
                .with_issuer(&entra_config.issuer)
                .with_jwks_uri(&entra_config.jwks_uri)
                .build()
                .map_err(|e| AppError::authentication_error(
                    format!("Failed to initialize Entra auth provider: {}", e),
                ))?;
            
            let auth_service = AuthService::new(Arc::new(auth_provider));
            registry.register(auth_service);
            
            tracing::info!("Initialized Entra authentication service");
        } else {
            return Err(AppError::configuration_error("Entra configuration is missing"));
        }
    }
    
    Ok(())
}

/// Initialize plugin system
#[cfg(feature = "plugin")]
pub async fn init_plugin_system(
    registry: &mut ServiceRegistry,
) -> Result<(), AppError> {
    use navius_plugin::{PluginManager, PluginRegistry};
    
    let plugin_registry = PluginRegistry::new();
    let plugin_manager = PluginManager::new(plugin_registry);
    
    registry.register(plugin_manager);
    
    tracing::info!("Initialized plugin system");
    
    Ok(())
}

/// Initialize event system
#[cfg(feature = "event")]
pub async fn init_event_system(
    registry: &mut ServiceRegistry,
) -> Result<(), AppError> {
    use navius_event::{EventBus, EventRegistry};
    
    let event_registry = EventRegistry::new();
    let event_bus = EventBus::new(event_registry);
    
    registry.register(event_bus);
    
    tracing::info!("Initialized event system");
    
    Ok(())
}

/// Initialize metrics
#[cfg(feature = "metrics")]
pub async fn init_metrics(
    registry: &mut ServiceRegistry,
) -> Result<(), AppError> {
    use navius_metrics::MetricsRegistry;
    use navius_metrics_prometheus::PrometheusExporter;
    
    let metrics_registry = MetricsRegistry::new();
    let prometheus_exporter = PrometheusExporter::new();
    
    registry.register(metrics_registry);
    registry.register(prometheus_exporter);
    
    tracing::info!("Initialized metrics");
    
    Ok(())
}
