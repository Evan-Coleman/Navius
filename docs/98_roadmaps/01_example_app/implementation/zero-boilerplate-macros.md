# Zero Boilerplate Macro System

This document outlines the macro system design for the Zero Boilerplate Initiative.

## Core Macros

### `#[navius_app]`

The primary macro that transforms a simple async main function into a fully configured Navius application.

#### Implementation Details

```rust
#[proc_macro_attribute]
pub fn navius_app(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the function this is applied to
    let input = parse_macro_input!(item as ItemFn);
    
    // Ensure the function is async and named "main"
    if input.sig.ident != "main" {
        return Error::new_spanned(
            input.sig.ident,
            "the #[navius_app] attribute can only be applied to a function named 'main'",
        )
        .to_compile_error()
        .into();
    }

    if input.sig.asyncness.is_none() {
        return Error::new_spanned(
            input.sig,
            "the main function must be async",
        )
        .to_compile_error()
        .into();
    }

    // Generate the application bootstrap code
    let expanded = quote! {
        #[tokio::main]
        async fn main() -> ::navius_core::error::Result<()> {
            // Initialize tracing
            ::tracing_subscriber::fmt()
                .with_max_level(::tracing::Level::DEBUG)
                .init();

            // Build application registry from route handlers
            let registry = ::navius_core::di::ComponentRegistry::build_from_routes();
            
            // Load configuration
            let config = ::navius_core::config::load_config("config").unwrap_or_default();
            
            // Create the router with all annotated routes
            let router = ::navius_core::router::build_router_from_annotations();
            
            // Create the web server with settings from config
            let host = config.get_string("server.host").unwrap_or_else(|_| "127.0.0.1".to_string());
            let port = config.get_int("server.port").unwrap_or(3000) as u16;
            
            // Build the application
            let app = ::navius_core::app::App::new()
                .with_registry(registry)
                .with_web_server(host, port, router);
                
            // Execute user-defined main function
            let user_main = async #input;
            user_main.await;
            
            // Run the application
            app.run().await
        }
    };

    expanded.into()
}
```

### `#[route]`

A family of macros for defining HTTP routes.

#### Route Macro Variants

```rust
// GET route
#[proc_macro_attribute]
pub fn get(attr: TokenStream, item: TokenStream) -> TokenStream {
    route_macro(attr, item, "GET")
}

// POST route
#[proc_macro_attribute]
pub fn post(attr: TokenStream, item: TokenStream) -> TokenStream {
    route_macro(attr, item, "POST")
}

// PUT route
#[proc_macro_attribute]
pub fn put(attr: TokenStream, item: TokenStream) -> TokenStream {
    route_macro(attr, item, "PUT")
}

// DELETE route
#[proc_macro_attribute]
pub fn delete(attr: TokenStream, item: TokenStream) -> TokenStream {
    route_macro(attr, item, "DELETE")
}
```

#### Route Macro Implementation

```rust
fn route_macro(attr: TokenStream, item: TokenStream, method: &str) -> TokenStream {
    let path = parse_macro_input!(attr as LitStr);
    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_name = &input_fn.sig.ident;
    
    // Register the function in a static collection for later router generation
    let expanded = quote! {
        #input_fn
        
        #[::navius_core::paste::paste]
        #[::core::compiler_builtins::linkage::linkage]
        #[doc(hidden)]
        #[allow(non_upper_case_globals)]
        pub static [<__NAVIUS_ROUTE_ #fn_name>]: ::navius_core::router::RouteRegistration = 
            ::navius_core::router::RouteRegistration {
                path: #path,
                method: #method,
                handler_name: stringify!(#fn_name),
                handler: |app: &::navius_core::app::App| {
                    Box::new(#fn_name)
                },
            };
    };
    
    expanded.into()
}
```

### `#[inject]`

A macro for dependency injection.

#### Implementation Details

```rust
#[proc_macro_attribute]
pub fn inject(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attrs = parse_macro_input!(attr with Punctuated::<Path, Token![,]>::parse_terminated);
    let input = parse_macro_input!(item as ItemStruct);
    let struct_name = &input.ident;
    
    // Generate implementation code
    let expanded = quote! {
        #input
        
        #[automatically_derived]
        impl ::navius_core::di::Injectable for #struct_name {
            fn inject(registry: &::navius_core::di::ComponentRegistry) -> Self {
                Self {
                    #(
                        #attrs: registry.resolve::<#attrs>().expect(&format!(
                            "Failed to resolve dependency: {}", stringify!(#attrs)
                        )),
                    )*
                }
            }
        }
    };
    
    expanded.into()
}
```

## Framework Integration

These macros will interact with a new core framework module: `navius_core::router::annotations`. This module will:

1. Maintain static registries of annotated handlers
2. Provide functions to build routers from these registrations
3. Generate optimal routing trees based on path patterns

## Example Usage

### Simple API Service

```rust
use navius_core::prelude::*;

#[navius_app]
async fn main() {
    // Configuration automatically loaded from files and environment
    // App automatically runs with optimal defaults
}

#[get("/api/users")]
async fn list_users() -> Json<Vec<User>> {
    // Implementation
}

#[post("/api/users")]
async fn create_user(Json(payload): Json<UserCreate>) -> impl IntoResponse {
    // Implementation  
}

#[get("/api/users/:id")]
async fn get_user(Path(id): Path<String>) -> impl IntoResponse {
    // Implementation
}
```

### With Dependency Injection

```rust
use navius_core::prelude::*;

#[navius_app]
async fn main() {
    // Empty main function - framework handles everything
}

#[inject(UserService)]
struct UserController {
    user_service: UserService,
}

#[get("/api/users")]
async fn list_users(State(state): State<AppState>) -> impl IntoResponse {
    let controller = state.app.registry().resolve::<UserController>().unwrap();
    controller.user_service.list_users().await
}
```

## Implementation Timeline

1. **Phase 1 (Week 1-2)**: Implement basic `#[navius_app]` and route annotation macros
2. **Phase 2 (Week 3-4)**: Add dependency injection macros and registry integration
3. **Phase 3 (Week 5-6)**: Implement configuration integration and advanced routing features
4. **Phase 4 (Week 7-8)**: Testing, documentation, and examples 