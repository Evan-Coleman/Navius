# navius-http API Inventory

**Version:** 0.1.0  
**Documentation Coverage:** 100%  
**Status:** ✅ Good

## Dependencies

- navius-core
- serde
- serde_json
- tokio
- tracing
- async-trait
- thiserror
- axum
- tower
- tower-http
- reqwest
- bytes
- futures
- futures-util

## Public Structs

| Name | File | Line | Documentation |
|------|------|------|---------------|
| TimeoutConfig | timeout.rs | 21 | ⚠️ Partial |
| TimeoutLayer | timeout.rs | 91 | ⚠️ Partial |
| TimeoutService<S> | timeout.rs | 135 | ⚠️ Partial |
| CorsConfig | cors.rs | 12 | ⚠️ Partial |
| CorsLayer | cors.rs | 143 | ⚠️ Partial |
| LoggingConfig | logging.rs | 22 | ⚠️ Partial |
| LoggingLayer | logging.rs | 162 | ⚠️ Partial |
| LoggingService<S> | logging.rs | 193 | ⚠️ Partial |
| RequestId<S> | request_id.rs | 20 | ⚠️ Partial |
| RequestIdLayer; | request_id.rs | 88 | ⚠️ Partial |
| HttpClient | client.rs | 10 | ⚠️ Partial |
| HttpClientBuilder | client.rs | 124 | ⚠️ Partial |
| RequestBuilder | client.rs | 234 | ⚠️ Partial |
| Version; | lib.rs | 59 | ⚠️ Partial |
| ShutdownSender(broadcast::Sender<()>); | server.rs | 15 | ⚠️ Partial |
| ShutdownReceiver(broadcast::Receiver<()>); | server.rs | 19 | ⚠️ Partial |
| HttpServer | server.rs | 23 | ⚠️ Partial |
| HttpServerHandle | server.rs | 152 | ⚠️ Partial |
| RouterBuilder | server.rs | 173 | ⚠️ Partial |

## Public Enums

| Name | File | Line | Documentation |
|------|------|------|---------------|
| Method | util.rs | 52 | ⚠️ Partial |
| Error | error.rs | 11 | ⚠️ Partial |

## Public Functions

| Name | File | Line | Documentation |
|------|------|------|---------------|
| new() | timeout.rs | 42 | ⚠️ Partial |
| with_timeout(mut | timeout.rs | 47 | ⚠️ Partial |
| with_path_timeout(mut | timeout.rs | 53 | ⚠️ Partial |
| with_excluded_paths(mut | timeout.rs | 59 | ⚠️ Partial |
| exclude_path(mut | timeout.rs | 65 | ⚠️ Partial |
| get_timeout_for_path(&self, | timeout.rs | 71 | ⚠️ Partial |
| new() | timeout.rs | 97 | ⚠️ Partial |
| with_config(config: | timeout.rs | 104 | ⚠️ Partial |
| with_timeout(timeout: | timeout.rs | 109 | ⚠️ Partial |
| fn | timeout.rs | 235 | ⚠️ Partial |
| timeout_layer() | timeout.rs | 266 | ⚠️ Partial |
| timeout_layer_with_duration(timeout: | timeout.rs | 271 | ⚠️ Partial |
| with_timeout( | timeout.rs | 276 | ⚠️ Partial |
| new() | cors.rs | 76 | ⚠️ Partial |
| with_allowed_origins(mut | cors.rs | 81 | ⚠️ Partial |
| allow_origin(mut | cors.rs | 87 | ⚠️ Partial |
| with_allowed_methods(mut | cors.rs | 93 | ⚠️ Partial |
| allow_method(mut | cors.rs | 99 | ⚠️ Partial |
| with_allowed_headers(mut | cors.rs | 105 | ⚠️ Partial |
| allow_header(mut | cors.rs | 111 | ⚠️ Partial |
| with_credentials(mut | cors.rs | 117 | ⚠️ Partial |
| with_exposed_headers(mut | cors.rs | 123 | ⚠️ Partial |
| expose_header(mut | cors.rs | 129 | ⚠️ Partial |
| with_max_age(mut | cors.rs | 135 | ⚠️ Partial |
| new() | cors.rs | 149 | ⚠️ Partial |
| with_config(config: | cors.rs | 154 | ⚠️ Partial |
| cors_layer() | cors.rs | 201 | ⚠️ Partial |
| permissive_cors_layer() | cors.rs | 206 | ⚠️ Partial |
| new() | logging.rs | 66 | ⚠️ Partial |
| with_log_level(mut | logging.rs | 71 | ⚠️ Partial |
| with_headers(mut | logging.rs | 77 | ⚠️ Partial |
| with_body(mut | logging.rs | 83 | ⚠️ Partial |
| with_max_body_length(mut | logging.rs | 89 | ⚠️ Partial |
| with_excluded_paths(mut | logging.rs | 95 | ⚠️ Partial |
| exclude_path(mut | logging.rs | 101 | ⚠️ Partial |
| with_excluded_headers(mut | logging.rs | 107 | ⚠️ Partial |
| exclude_header(mut | logging.rs | 113 | ⚠️ Partial |
| new() | logging.rs | 168 | ⚠️ Partial |
| with_config(config: | logging.rs | 175 | ⚠️ Partial |
| fn | logging.rs | 316 | ⚠️ Partial |
| logging_layer() | logging.rs | 386 | ⚠️ Partial |
| detailed_logging_layer() | logging.rs | 391 | ⚠️ Partial |
| default_middleware() | mod.rs | 20 | ⚠️ Partial |
| new() | request_id.rs | 92 | ⚠️ Partial |
| request_id_layer() | request_id.rs | 106 | ⚠️ Partial |
| fn | request_id.rs | 111 | ⚠️ Partial |
| to_header_value<T: | util.rs | 10 | ⚠️ Partial |
| map_to_headers(map: | util.rs | 16 | ⚠️ Partial |
| parse_url(url: | util.rs | 29 | ⚠️ Partial |
| join_url(base: | util.rs | 34 | ⚠️ Partial |
| get_request_id(headers: | util.rs | 42 | ⚠️ Partial |
| new() | client.rs | 18 | ⚠️ Partial |
| with_client(client: | client.rs | 27 | ⚠️ Partial |
| builder() | client.rs | 36 | ⚠️ Partial |
| with_base_url(mut | client.rs | 41 | ⚠️ Partial |
| with_headers(mut | client.rs | 47 | ⚠️ Partial |
| with_header(mut | client.rs | 53 | ⚠️ Partial |
| get(&self, | client.rs | 65 | ⚠️ Partial |
| post(&self, | client.rs | 71 | ⚠️ Partial |
| put(&self, | client.rs | 77 | ⚠️ Partial |
| delete(&self, | client.rs | 83 | ⚠️ Partial |
| patch(&self, | client.rs | 89 | ⚠️ Partial |
| head(&self, | client.rs | 95 | ⚠️ Partial |
| new() | client.rs | 135 | ⚠️ Partial |
| with_timeout(mut | client.rs | 147 | ⚠️ Partial |
| with_user_agent(mut | client.rs | 153 | ⚠️ Partial |
| with_base_url(mut | client.rs | 159 | ⚠️ Partial |
| with_headers(mut | client.rs | 165 | ⚠️ Partial |
| with_header(mut | client.rs | 171 | ⚠️ Partial |
| follow_redirects(mut | client.rs | 183 | ⚠️ Partial |
| max_redirects(mut | client.rs | 189 | ⚠️ Partial |
| build(self) | client.rs | 195 | ⚠️ Partial |
| header(mut | client.rs | 256 | ⚠️ Partial |
| headers(mut | client.rs | 268 | ⚠️ Partial |
| query<T: | client.rs | 274 | ⚠️ Partial |
| json<T: | client.rs | 280 | ⚠️ Partial |
| form<T: | client.rs | 286 | ⚠️ Partial |
| body<T: | client.rs | 292 | ⚠️ Partial |
| timeout(mut | client.rs | 298 | ⚠️ Partial |
| fn | client.rs | 304 | ⚠️ Partial |
| fn | client.rs | 312 | ⚠️ Partial |
| fn | client.rs | 329 | ⚠️ Partial |
| fn | client.rs | 346 | ⚠️ Partial |
| internal<T: | error.rs | 60 | ⚠️ Partial |
| validation<T: | error.rs | 65 | ⚠️ Partial |
| configuration<T: | error.rs | 70 | ⚠️ Partial |
| http<T: | error.rs | 75 | ⚠️ Partial |
| request<T: | error.rs | 83 | ⚠️ Partial |
| response<T: | error.rs | 88 | ⚠️ Partial |
| client(err: | error.rs | 93 | ⚠️ Partial |
| status_code(&self) | error.rs | 98 | ⚠️ Partial |
| current() | lib.rs | 63 | ⚠️ Partial |
| semver() | lib.rs | 68 | ⚠️ Partial |
| init() | lib.rs | 74 | ⚠️ Partial |
| new() | server.rs | 32 | ⚠️ Partial |
| with_router(mut | server.rs | 42 | ⚠️ Partial |
| with_router_builder(self, | server.rs | 48 | ⚠️ Partial |
| with_address(mut | server.rs | 53 | ⚠️ Partial |
| with_host_and_port(mut | server.rs | 59 | ⚠️ Partial |
| with_timeout(mut | server.rs | 69 | ⚠️ Partial |
| with_shutdown(mut | server.rs | 75 | ⚠️ Partial |
| create_shutdown_channel() | server.rs | 81 | ⚠️ Partial |
| fn | server.rs | 88 | ⚠️ Partial |
| shutdown(&self) | server.rs | 159 | ⚠️ Partial |
| fn | server.rs | 164 | ⚠️ Partial |
| new() | server.rs | 179 | ⚠️ Partial |
| merge(mut | server.rs | 186 | ⚠️ Partial |
| nest(mut | server.rs | 192 | ⚠️ Partial |
| route(mut | server.rs | 198 | ⚠️ Partial |
| build(self) | server.rs | 204 | ⚠️ Partial |

