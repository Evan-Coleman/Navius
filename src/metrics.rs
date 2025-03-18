pub fn init_metrics() -> PrometheusHandle {
    // Create registry with custom metrics prefix
    let registry = Registry::new();

    // Register metrics
    registry.register_counter("pet_cache_hits_total");
    registry.register_counter("pet_cache_misses_total");
    registry.register_counter("cache_entries_created");
    registry.register_gauge("pet_cache_size");
    registry.register_gauge("pet_cache_hit_count");
    registry.register_gauge("pet_cache_miss_count");
    registry.register_gauge("app_uptime_seconds");

    // Build recorder
    let builder = PrometheusBuilder::new();
    let recorder = builder
        .with_registry(registry)
        .build()
        .expect("Failed to build Prometheus recorder");

    // Install global recorder
    metrics::set_global_recorder(recorder).expect("Failed to set global recorder");

    // Get handle for rendering
    metrics_exporter_prometheus::handle()
}
