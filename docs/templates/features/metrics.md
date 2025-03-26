---
title: "Metrics System"
version: "1.0"
features: ["metrics"]
variables:
  port: "8081"
---
# Metrics System

The metrics system provides real-time monitoring of {{project_name}} performance.

## Configuration

Metrics are exposed on port {{port}} by default and can be changed in the configuration:

```yaml
metrics:
  enabled: true
  port: {{port}}
  path: "/metrics"
```

## Available Metrics

{{#if advanced_metrics}}
### Advanced Metrics

With the advanced metrics feature enabled, the following additional metrics are available:

- `request_duration_seconds`: Histogram of request durations
- `request_size_bytes`: Histogram of request sizes
- `response_size_bytes`: Histogram of response sizes
{{/if}}

### Standard Metrics

The following metrics are always available:

- `requests_total`: Counter of total requests
- `errors_total`: Counter of total errors
