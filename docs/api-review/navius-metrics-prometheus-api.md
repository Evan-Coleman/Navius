# navius-metrics-prometheus API Inventory

**Version:** 0.1.0  
**Documentation Coverage:** 100%  
**Status:** ✅ Good

## Dependencies

- tokio
- tracing
- serde
- thiserror
- async-trait
- navius-core
- navius-metrics
- metrics-exporter-prometheus
- axum
- tower
- tower-http

## Public Structs

| Name | File | Line | Documentation |
|------|------|------|---------------|
| PrometheusCollector | lib.rs | 11 | ⚠️ Partial |
| PrometheusExporter | lib.rs | 59 | ⚠️ Partial |
| PrometheusMetricsBuilder | lib.rs | 85 | ⚠️ Partial |

## Public Functions

| Name | File | Line | Documentation |
|------|------|------|---------------|
| new(handle: | lib.rs | 18 | ⚠️ Partial |
| new(handle: | lib.rs | 65 | ⚠️ Partial |
| new(service_name: | lib.rs | 94 | ⚠️ Partial |
| with_namespace(mut | lib.rs | 104 | ⚠️ Partial |
| with_label(mut | lib.rs | 110 | ⚠️ Partial |
| with_http_listener(mut | lib.rs | 116 | ⚠️ Partial |
| build(self) | lib.rs | 122 | ⚠️ Partial |

