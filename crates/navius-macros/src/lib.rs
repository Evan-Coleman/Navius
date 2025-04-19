use darling::FromMeta;
use proc_macro::TokenStream;
use proc_macro_error::{abort, proc_macro_error};
use quote::quote;
use syn::{Ident, ItemFn, ItemMod, LitStr, parse_macro_input};

// Only keep one definition of RouteArgs - use darling for parsing
#[derive(Debug, FromMeta)]
struct RouteArgs {
    path: String,
    method: String,
}

/// Method value enum
enum MethodValue {
    Single(String),
    Multiple(Vec<String>),
}

/// Marks a function as an HTTP route handler.
///
/// This macro simplifies the creation of HTTP route handlers by allowing you to
/// specify the path and HTTP method directly on the function.
///
/// # Example
///
/// ```rust
/// use navius_macros::route;
/// use axum::Json;
/// use serde_json::json;
///
/// #[route(path = "/hello", method = "GET")]
/// async fn hello_world() -> Json<serde_json::Value> {
///     Json(json!({ "message": "Hello, world!" }))
/// }
///
/// // Multiple methods
/// #[route(path = "/users", method = ["GET", "POST"])]
/// async fn users() -> &'static str {
///     "Handles both GET and POST"
/// }
/// ```
#[proc_macro_error]
#[proc_macro_attribute]
pub fn route(args: TokenStream, input: TokenStream) -> TokenStream {
    let attr_args = match darling::ast::NestedMeta::parse_meta_list(args.into()) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(darling::Error::from(e).write_errors());
        }
    };

    let args = match RouteArgs::from_list(&attr_args) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(e.write_errors());
        }
    };

    let input_fn = parse_macro_input!(input as ItemFn);

    // Extract function information
    let fn_name = &input_fn.sig.ident;
    let fn_vis = &input_fn.vis;
    let fn_body = &input_fn.block;
    let fn_args = &input_fn.sig.inputs;
    let fn_output = &input_fn.sig.output;
    let fn_generics = &input_fn.sig.generics;
    let fn_asyncness = &input_fn.sig.asyncness;

    // Parse the method string
    let method_value = if args.method.starts_with('[') && args.method.ends_with(']') {
        // It's an array format - parse it
        let methods_str = args.method.trim_start_matches('[').trim_end_matches(']');
        let methods = methods_str
            .split(',')
            .map(|s| s.trim().trim_matches('"').to_string())
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>();
        MethodValue::Multiple(methods)
    } else {
        // It's a single method
        MethodValue::Single(args.method.clone())
    };

    // Generate method code based on parsed method value
    let method_code = match method_value {
        MethodValue::Single(method_str) => {
            let method_lit = LitStr::new(&method_str, proc_macro2::Span::call_site());
            quote! {
                vec![#method_lit.to_string()]
            }
        }
        MethodValue::Multiple(methods) => {
            let method_lits = methods
                .iter()
                .map(|m| LitStr::new(m, proc_macro2::Span::call_site()))
                .collect::<Vec<_>>();
            quote! {
                vec![#(#method_lits.to_string()),*]
            }
        }
    };

    // Path as a literal string
    let path_lit = LitStr::new(&args.path, proc_macro2::Span::call_site());

    // Register the route in the module's route registry
    let register_fn = Ident::new(
        &format!("__register_route_{}", fn_name),
        proc_macro2::Span::call_site(),
    );

    // Generate the output
    let output = quote! {
        // Original function
        #fn_vis #fn_asyncness fn #fn_name #fn_generics(#fn_args) #fn_output #fn_body

        // Registration function called by the module's registry
        #[doc(hidden)]
        pub fn #register_fn() -> navius_http::RouteRegistration {
            navius_http::RouteRegistration {
                path: #path_lit,
                methods: #method_code,
                handler_name: stringify!(#fn_name),
                handler: |router| {
                    use axum::routing::on;
                    let route_fn = on(
                        #method_code
                            .iter()
                            .map(|m| m.parse::<axum::http::Method>().unwrap())
                            .collect::<Vec<_>>(),
                        #fn_name,
                    );
                    router.route(#path_lit, route_fn)
                },
            }
        }
    };

    output.into()
}

// Only keep one definition of NestArgs - use darling for parsing
#[derive(Debug, FromMeta)]
struct NestArgs {
    prefix: String,
}

/// Marks a module as containing nested routes.
///
/// This macro simplifies the creation of nested routes by allowing you to
/// specify a prefix path for all routes in the module.
///
/// # Example
///
/// ```rust
/// use navius_macros::{nest, route};
/// use axum::Json;
/// use serde_json::json;
///
/// #[nest(prefix = "/api/v1")]
/// mod api {
///     use super::*;
///
///     #[route(path = "/hello", method = "GET")]
///     async fn hello_world() -> Json<serde_json::Value> {
///         Json(json!({ "message": "Hello, world!" }))
///     }
/// }
/// ```
#[proc_macro_error]
#[proc_macro_attribute]
pub fn nest(args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse attribute args using darling directly
    let attr_args = match darling::ast::NestedMeta::parse_meta_list(args.into()) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(darling::Error::from(e).write_errors());
        }
    };

    let args = match NestArgs::from_list(&attr_args) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(e.write_errors());
        }
    };

    let module = parse_macro_input!(input as ItemMod);
    let mod_name = &module.ident;
    let mod_vis = &module.vis;
    let mod_attrs = &module.attrs;

    // Get the module content
    let (_, content) = match module.content {
        Some((_, content)) => ((), content),
        None => abort!(
            module.ident.span(),
            "This macro only supports inline modules with a body"
        ),
    };

    // Extract the route prefix
    let _prefix = &args.prefix;
    let prefix_lit = LitStr::new(&args.prefix, proc_macro2::Span::call_site());

    // Generate the output
    let output = quote! {
        #(#mod_attrs)*
        #mod_vis mod #mod_name {
            #(#content)*
        }

        // Register the module's routes with the prefix
        #[doc(hidden)]
        #[inventory::submit]
        static MODULE_ROUTES: navius_http::RouteRegistration = navius_http::RouteRegistration {
            path: #prefix_lit,
            methods: vec![],
            handler_name: stringify!(#mod_name),
            handler: |router| {
                // Find all route registration functions in this module
                let mut module_router = axum::Router::new();

                // Add logic to collect routes from the module
                // and nest them under the prefix

                router.nest(#prefix_lit, module_router)
            },
        };
    };

    output.into()
}

// Definition for NaviusAppArgs using darling
#[derive(Debug, Default, FromMeta)]
struct NaviusAppArgs {
    name: Option<String>,
    #[darling(default)]
    routes: Option<String>,
    #[darling(default)]
    plugins: Option<String>,
}

/// Main application macro for Navius applications
///
/// This macro creates a zero-boilerplate Navius application by adding the necessary
/// bootstrapping code.
///
/// # Example
///
/// ```rust
/// use navius_macros::navius_app;
///
/// #[navius_app(
///     name = "my-app",
///     routes = [self::api, crate::admin]
/// )]
/// async fn main() {
///     // Any custom code here
///     println!("App started!");
/// }
/// ```
#[proc_macro_error]
#[proc_macro_attribute]
pub fn navius_app(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse attribute args using darling directly
    let attr_args = match darling::ast::NestedMeta::parse_meta_list(attr.into()) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(darling::Error::from(e).write_errors());
        }
    };

    let args = match NaviusAppArgs::from_list(&attr_args) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(e.write_errors());
        }
    };

    let input_fn = parse_macro_input!(item as ItemFn);

    // Extract function information
    let fn_name = &input_fn.sig.ident;
    let fn_vis = &input_fn.vis;
    let fn_body = &input_fn.block;
    let fn_args = &input_fn.sig.inputs;
    let return_type = &input_fn.sig.output;

    // Get app name or use default
    let app_name = match &args.name {
        Some(name) => quote! { #name },
        None => quote! { env!("CARGO_PKG_NAME") },
    };

    // Build the new function with all the boilerplate
    let new_fn = quote! {
        #[tokio::main]
        #fn_vis async fn #fn_name(#fn_args) #return_type {
            // Initialize tracing
            tracing_subscriber::fmt::init();

            // Create application
            let app_name = #app_name;
            tracing::info!("Starting application: {}", app_name);

            // Create the WebPlugin with a router
            let web_plugin = navius_http::WebPlugin::new()
                .with_host("127.0.0.1") // Can be overridden by config
                .with_port(3000);        // Can be overridden by config

            // Create app builder
            let mut app_builder = navius_core::di::Application::builder();

            // Add the WebPlugin
            // app_builder = app_builder.with_plugin(web_plugin);

            // Add any additional plugins from arguments
            // TODO: Parse plugin list from args.plugins

            // Run user code inside the function
            #fn_body

            // Run the application
            let app = app_builder.build();
            tracing::info!("Application built successfully");
        }
    };

    new_fn.into()
}
