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

    // Convert method string to axum::routing::MethodFilter
    let method_str = &args.method;
    let method_filter = match method_str.as_str() {
        "GET" => quote! { axum::routing::MethodFilter::GET },
        "POST" => quote! { axum::routing::MethodFilter::POST },
        "PUT" => quote! { axum::routing::MethodFilter::PUT },
        "DELETE" => quote! { axum::routing::MethodFilter::DELETE },
        "PATCH" => quote! { axum::routing::MethodFilter::PATCH },
        "OPTIONS" => quote! { axum::routing::MethodFilter::OPTIONS },
        "HEAD" => quote! { axum::routing::MethodFilter::HEAD },
        _ => abort!(
            input_fn.sig.ident,
            "Unsupported HTTP method: {}",
            method_str
        ),
    };

    let path_lit = LitStr::new(&args.path, proc_macro2::Span::call_site());

    let register_fn_name = Ident::new(
        &format!("__register_route_{}", fn_name),
        proc_macro2::Span::call_site(),
    );
    let output = quote! {
        #fn_vis #fn_asyncness fn #fn_name #fn_generics(#fn_args) #fn_output #fn_body
        #[doc(hidden)]
        pub fn #register_fn_name(router: axum::Router) -> axum::Router {
            use axum::routing::on;
            let route_fn = on(#method_filter, #fn_name);
            router.route(#path_lit, route_fn)
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
    let content = match module.content {
        Some((_, content)) => content,
        None => abort!(
            module.ident.span(),
            "This macro only supports inline modules with a body"
        ),
    };

    // Just expand the module body, do not generate registration statics
    let output = quote! {
        #(#mod_attrs)*
        #mod_vis mod #mod_name {
            #(#content)*
        }
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
    let fn_name = &input_fn.sig.ident;
    let fn_vis = &input_fn.vis;
    let fn_body = &input_fn.block;
    let fn_args = &input_fn.sig.inputs;
    let return_type = &input_fn.sig.output;

    let app_name = match &args.name {
        Some(name) => quote! { #name },
        None => quote! { env!("CARGO_PKG_NAME") },
    };

    let new_fn = quote! {
        #[tokio::main]
        #fn_vis async fn #fn_name(#fn_args) #return_type {
            tracing_subscriber::fmt::init();
            let app_name = #app_name;
            tracing::info!("Starting application: {}", app_name);
            #fn_body
        }
    };

    new_fn.into()
}
