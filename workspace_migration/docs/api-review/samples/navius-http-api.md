# navius-http API Inventory

**Generated:** March 29, 2025  
**Crate Version:** 0.4.0-dev  
**Documentation Coverage:** 83%

## Overview

`navius-http` provides HTTP client and server capabilities for the Navius framework, with support for routing, middleware, and request/response handling.

## Public API Summary

- **Total Public Items:** 64
- **Structs:** 18
- **Enums:** 5
- **Traits:** 7
- **Functions:** 34

## Public Structs

### HttpClient

**Location:** `src/client.rs:24`  
**Documentation:** Complete  
**Methods:** 8  
**Visibility:** Public  

Provides a versatile HTTP client for making requests to external services.

```rust
pub struct HttpClient {
    // ...private fields...
}

impl HttpClient {
    pub fn new() -> Self { ... }
    pub fn with_config(config: HttpClientConfig) -> Self { ... }
    pub fn get(&self, url: &str) -> RequestBuilder { ... }
    pub fn post(&self, url: &str) -> RequestBuilder { ... }
    pub fn put(&self, url: &str) -> RequestBuilder { ... }
    pub fn delete(&self, url: &str) -> RequestBuilder { ... }
    pub fn request(&self, method: Method, url: &str) -> RequestBuilder { ... }
    pub fn execute(&self, request: Request) -> Result<Response, HttpError> { ... }
}
```

**Issues:**
- None

### RequestBuilder

**Location:** `src/request.rs:42`  
**Documentation:** Incomplete  
**Methods:** 12  
**Visibility:** Public  

Builder for constructing HTTP requests with various options.

```rust
pub struct RequestBuilder {
    // ...private fields...
}

impl RequestBuilder {
    pub fn header(self, name: &str, value: &str) -> Self { ... }
    pub fn headers(self, headers: Headers) -> Self { ... }
    pub fn query<T: Serialize + ?Sized>(self, query: &T) -> Self { ... }
    pub fn body<T: Into<Body>>(self, body: T) -> Self { ... }
    pub fn json<T: Serialize + ?Sized>(self, json: &T) -> Self { ... }
    pub fn form<T: Serialize + ?Sized>(self, form: &T) -> Self { ... }
    pub fn timeout(self, timeout: Duration) -> Self { ... }
    pub fn connect_timeout(self, timeout: Duration) -> Self { ... }
    pub fn follow_redirects(self, follow: bool) -> Self { ... }
    pub fn max_redirects(self, max: usize) -> Self { ... }
    pub fn build(self) -> Result<Request, HttpError> { ... }
    pub fn send(self) -> Result<Response, HttpError> { ... }
}
```

**Issues:**
- Missing documentation for `query`, `body`, and `json` methods
- Inconsistent naming: `follow_redirects` vs. `max_redirects` (should be `set_max_redirects`)

### Response

**Location:** `src/response.rs:18`  
**Documentation:** Complete  
**Methods:** 10  
**Visibility:** Public  

Represents an HTTP response with status, headers, and body.

```rust
pub struct Response {
    // ...private fields...
}

impl Response {
    pub fn status(&self) -> StatusCode { ... }
    pub fn headers(&self) -> &Headers { ... }
    pub fn body(&self) -> &Body { ... }
    pub fn text(self) -> Result<String, HttpError> { ... }
    pub fn json<T: DeserializeOwned>(self) -> Result<T, HttpError> { ... }
    pub fn bytes(self) -> Result<Vec<u8>, HttpError> { ... }
    pub fn is_success(&self) -> bool { ... }
    pub fn is_redirect(&self) -> bool { ... }
    pub fn is_client_error(&self) -> bool { ... }
    pub fn is_server_error(&self) -> bool { ... }
}
```

**Issues:**
- None

## Public Traits

### HttpHandler

**Location:** `src/handler.rs:18`  
**Documentation:** Complete  
**Methods:** 3  
**Visibility:** Public  

Trait for components that can handle HTTP requests and produce responses.

```rust
pub trait HttpHandler: Send + Sync + 'static {
    fn handle(&self, request: Request) -> Result<Response, HttpError>;
    fn handle_async(&self, request: Request) -> BoxFuture<'static, Result<Response, HttpError>> {
        Box::pin(async move { self.handle(request) })
    }
    fn clone_box(&self) -> Box<dyn HttpHandler>;
}
```

**Issues:**
- None

### Middleware

**Location:** `src/middleware.rs:27`  
**Documentation:** Complete  
**Methods:** 2  
**Visibility:** Public  

Trait for HTTP middleware components that can modify requests and responses.

```rust
pub trait Middleware: Send + Sync + 'static {
    fn process(&self, request: Request, next: Next) -> Result<Response, HttpError>;
    fn process_async(&self, request: Request, next: Next) -> BoxFuture<'static, Result<Response, HttpError>> {
        Box::pin(async move { self.process(request, next) })
    }
}
```

**Issues:**
- None

## Public Enums

### Method

**Location:** `src/method.rs:12`  
**Documentation:** Complete  
**Variants:** 9  
**Visibility:** Public  

Represents HTTP methods for requests.

```rust
pub enum Method {
    Get,
    Post,
    Put,
    Delete,
    Head,
    Options,
    Connect,
    Patch,
    Trace,
}
```

**Issues:**
- None

### HttpErrorKind

**Location:** `src/error.rs:31`  
**Documentation:** Partial  
**Variants:** 7  
**Visibility:** Public  

Represents different kinds of HTTP errors.

```rust
pub enum HttpErrorKind {
    Request,
    Response,
    Timeout,
    Connection,
    Serialization,
    Validation,
    Internal,
}
```

**Issues:**
- Missing documentation for the `Validation` and `Internal` variants

## Public Functions

### create_default_client

**Location:** `src/client.rs:156`  
**Documentation:** Missing  
**Visibility:** Public  

Creates an HTTP client with default configuration.

```rust
pub fn create_default_client() -> HttpClient {
    HttpClient::new()
}
```

**Issues:**
- Missing documentation
- Redundant with `HttpClient::new()`

### configure_server

**Location:** `src/server.rs:87`  
**Documentation:** Complete  
**Visibility:** Public  

Configures an HTTP server with the provided options.

```rust
pub fn configure_server(options: ServerOptions) -> Server {
    // ...implementation...
}
```

**Issues:**
- None

## Dependency Analysis

### Internal Dependencies
- `navius-core`: Used for component registration and lifecycle management
- `navius-di`: Used for dependency injection in server handlers

### External Dependencies
- `hyper`: HTTP implementation
- `tokio`: Async runtime
- `serde`: Serialization/deserialization
- `http`: HTTP types

## Documentation Gap Analysis

| Item | Type | Issue |
|------|------|-------|
| `RequestBuilder::query` | Method | Missing documentation |
| `RequestBuilder::body` | Method | Missing documentation |
| `RequestBuilder::json` | Method | Missing documentation |
| `HttpErrorKind::Validation` | Enum Variant | Missing documentation |
| `HttpErrorKind::Internal` | Enum Variant | Missing documentation |
| `create_default_client` | Function | Missing documentation |
| `from_hyper_request` | Function | Missing documentation |
| `to_hyper_request` | Function | Missing documentation |
| `from_hyper_response` | Function | Missing documentation |
| `to_hyper_response` | Function | Missing documentation |
| `trace_request` | Function | Missing documentation |

## Consistency Analysis

### Naming Inconsistencies
- `RequestBuilder::follow_redirects` vs. `RequestBuilder::max_redirects` (should be `set_max_redirects`)
- `trace_request` vs. `log_response` (should be `trace_response`)

### Parameter Ordering Inconsistencies
- `HttpClient::with_config(config)` vs. `Server::with_options(address, options)` (inconsistent ordering)

### Error Handling Inconsistencies
- Some functions return `Result<T, HttpError>` while others return `Result<T, Error>` or `HttpResult<T>`

## Recommendations

### High Priority
1. Complete documentation for all missing items
2. Standardize error return types to consistently use `Result<T, HttpError>`
3. Address naming inconsistencies in the `RequestBuilder` methods

### Medium Priority
1. Consider deprecating `create_default_client` in favor of `HttpClient::new()`
2. Standardize parameter ordering across similar functions
3. Improve integration with `navius-core` logging facilities

### Low Priority
1. Add more comprehensive examples for complex types
2. Consider splitting the large `RequestBuilder` into smaller, more focused builders

## Next Steps

1. Review recommendations with the API design team
2. Prioritize changes based on impact and effort
3. Create GitHub issues for tracking implementation work
4. Schedule implementation for the April 22-May 5 implementation phase

---

*Generated by Navius API Inventory Tool v0.1.0* 