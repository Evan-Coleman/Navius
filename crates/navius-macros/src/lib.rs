use proc_macro::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Attribute, Expr, Ident, Item, ItemFn, ItemMod, Lit, LitStr, Meta, Path as SynPath, Token,
    Visibility,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    token,
};

// Helper structure to parse #[route(...)] arguments
#[derive(Debug, Default)]
struct RouteArgs {
    path: Option<LitStr>,
    methods: Vec<Ident>, // Store methods as Ident (e.g., GET, POST)
                         // Add fields for other args like auth_policy, middleware etc.
}

impl Parse for RouteArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut args = RouteArgs::default();
        let vars = Punctuated::<Meta, Token![,]>::parse_terminated(input)?;
        let mut path_found = false;

        for meta in vars.iter() {
            // Use iter() to get spans if needed
            match meta {
                Meta::NameValue(nv) => {
                    if nv.path.is_ident("path") {
                        path_found = true;
                        // Ensure the expression is a literal first
                        if let Expr::Lit(expr_lit) = &nv.value {
                            // Now check if the literal is a string
                            if let Lit::Str(lit) = &expr_lit.lit {
                                args.path = Some(lit.clone());
                            } else {
                                return Err(syn::Error::new_spanned(
                                    &expr_lit.lit,
                                    "Expected string literal for path",
                                ));
                            }
                        } else {
                            return Err(syn::Error::new_spanned(
                                &nv.value,
                                "Expected literal expression for path",
                            ));
                        }
                    } else if nv.path.is_ident("method") {
                        // Handle method = "GET" or method = ["GET", "POST"]
                        match &nv.value {
                            // Single method: method = "GET"
                            Expr::Lit(expr_lit) => {
                                if let Lit::Str(lit) = &expr_lit.lit {
                                    args.methods
                                        .push(Ident::new(&lit.value().to_uppercase(), lit.span()));
                                } else {
                                    return Err(syn::Error::new_spanned(
                                        &expr_lit.lit,
                                        "Expected string literal for method",
                                    ));
                                }
                            }
                            // Multiple methods: method = ["GET", "POST"]
                            Expr::Array(array) => {
                                for expr in &array.elems {
                                    if let Expr::Lit(expr_lit) = expr {
                                        if let Lit::Str(lit) = &expr_lit.lit {
                                            args.methods.push(Ident::new(
                                                &lit.value().to_uppercase(),
                                                lit.span(),
                                            ));
                                        } else {
                                            return Err(syn::Error::new_spanned(
                                                expr,
                                                "Expected string literal for method",
                                            ));
                                        }
                                    } else {
                                        return Err(syn::Error::new_spanned(
                                            expr,
                                            "Expected string literal for method",
                                        ));
                                    }
                                }
                            }
                            _ => {
                                return Err(syn::Error::new_spanned(
                                    &nv.value,
                                    "Expected string literal or array of string literals for method",
                                ));
                            }
                        }
                    }
                    // TODO: Add parsing for other arguments (auth_policy, middleware, etc.)
                }
                _ => {
                    return Err(syn::Error::new_spanned(
                        meta,
                        "Unsupported attribute format",
                    ));
                }
            }
        }

        // Check if path was found after parsing all arguments
        if !path_found {
            // Error needs a span. We can use the span of the input stream.
            return Err(syn::Error::new(
                input.span(),
                "The 'path' argument is required for #[route]",
            ));
        }

        Ok(args)
    }
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
#[proc_macro_attribute]
pub fn route(args: TokenStream, input: TokenStream) -> TokenStream {
    let route_args = parse_macro_input!(args as RouteArgs);
    let input_fn = parse_macro_input!(input as ItemFn);
    
    // Extract path and method
    let path = &route_args.path;
    let method = &route_args.methods;
    
    // Extract function information
    let fn_name = &input_fn.sig.ident;
    let fn_vis = &input_fn.vis;
    let fn_body = &input_fn.block;
    let fn_args = &input_fn.sig.inputs;
    let fn_output = &input_fn.sig.output;
    let fn_generics = &input_fn.sig.generics;
    let fn_asyncness = &input_fn.sig.asyncness;
    
    // Register the route in the module's route registry
    let register_fn = Ident::new(
        &format!("__register_route_{}", fn_name),
        proc_macro2::Span::call_site(),
    );
    
    // Generate method code based on the input
    let method_code = match method {
        MethodValue::Single(method_str) => {
            quote! {
                vec![#method_str.to_string()]
            }
        }
        MethodValue::Multiple(methods) => {
            quote! {
                vec![#(#methods.to_string()),*]
            }
        }
    };
    
    // Generate the output
    let output = quote! {
        // Original function
        #fn_vis #fn_asyncness fn #fn_name #fn_generics(#fn_args) #fn_output #fn_body
        
        // Registration function called by the module's registry
        #[doc(hidden)]
        pub fn #register_fn() -> navius_http::RouteRegistration {
            navius_http::RouteRegistration {
                path: #path,
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
                    router.route(#path, route_fn)
                },
            }
        }
        
        // Add to module's route registry when initialized
        inventory::submit! {
            navius_http::ROUTE_REGISTRY.register(#register_fn())
        }
    };
    
    output.into()
}

/// Argument type for the route macro.
#[derive(Parse)]
struct RouteArgs {
    #[named(required)]
    path: LitStr,
    
    #[named(required)]
    method: MethodValue,
}

/// Method value that can be either a single method or an array of methods.
enum MethodValue {
    Single(LitStr),
    Multiple(Punctuated<LitStr, Token![,]>),
}

impl Parse for MethodValue {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(syn::token::Bracket) {
            let content;
            let _ = bracketed!(content in input);
            let methods = Punctuated::<LitStr, Token![,]>::parse_terminated(&content)?;
            Ok(MethodValue::Multiple(methods))
        } else {
            let method = input.parse::<LitStr>()?;
            Ok(MethodValue::Single(method))
        }
    }
}

// Helper structure to parse #[nest(...)] arguments
#[derive(Debug, Default)]
struct NestArgs {
    prefix: Option<LitStr>,
    // TODO: Add field for middleware
}

impl Parse for NestArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut args = NestArgs::default();
        let vars = Punctuated::<Meta, Token![,]>::parse_terminated(input)?;

        for meta in vars.iter() {
            match meta {
                Meta::NameValue(nv) => {
                    if nv.path.is_ident("prefix") {
                        if let Expr::Lit(expr_lit) = &nv.value {
                            if let Lit::Str(lit) = &expr_lit.lit {
                                args.prefix = Some(lit.clone());
                            } else { /* error */
                            }
                        } else { /* error */
                        }
                    }
                    // TODO: Add parsing for middleware
                }
                _ => {
                    return Err(syn::Error::new_spanned(
                        meta,
                        "Unsupported attribute format",
                    ));
                }
            }
        }
        Ok(args)
    }
}

/// Groups related routes under a common path prefix.
///
/// This macro allows you to group related routes under a common prefix path.
/// All route handlers defined in the module will be automatically prefixed with
/// the specified path.
///
/// # Example
///
/// ```rust
/// use navius_macros::{nest, route};
/// use axum::Json;
///
/// #[nest(prefix = "/api/v1")]
/// mod api {
///     use super::*;
///
///     #[route(path = "/hello", method = "GET")]
///     async fn hello() -> &'static str {
///         "Hello from API v1!"
///     }
///
///     #[route(path = "/status", method = "GET")]
///     async fn status() -> &'static str {
///         "API is operational"
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn nest(args: TokenStream, input: TokenStream) -> TokenStream {
    let nest_args = parse_macro_input!(args as NestArgs);
    let input_mod = parse_macro_input!(input as ItemMod);
    
    // Extract prefix
    let prefix = &nest_args.prefix;
    
    // Extract module information
    let mod_name = &input_mod.ident;
    let mod_vis = &input_mod.vis;
    let mod_items = if let Some((_, items)) = &input_mod.content {
        items
    } else {
        return syn::Error::new_spanned(
            &input_mod,
            "Module must have a body",
        )
        .to_compile_error()
        .into();
    };
    
    // Generate code to store the prefix
    let prefix_const = quote! {
        #[doc(hidden)]
        pub const __NAVIUS_NEST_PREFIX: Option<&'static str> = Some(#prefix);
    };
    
    // Generate the get_router function that collects all routes
    let get_router_fn = quote! {
        #[doc(hidden)]
        pub fn get_router() -> Option<axum::Router> {
            use navius_http::RouteRegistry;
            
            let mut registry = RouteRegistry::new();
            
            // Collect all registered routes
            for route in navius_http::ROUTE_REGISTRY.iter() {
                registry.register(route);
            }
            
            Some(registry.build_router())
        }
        
        #[doc(hidden)]
        pub fn get_prefix() -> Option<String> {
            Some(#prefix.to_string())
        }
    };
    
    // Generate the output
    let output = quote! {
        #mod_vis mod #mod_name {
            #(#mod_items)*
            
            #prefix_const
            #get_router_fn
        }
    };
    
    output.into()
}

/// Arguments for the nest macro.
#[derive(Parse)]
struct NestArgs {
    #[named(required)]
    prefix: LitStr,
}

// Helper structure to parse #[auto_config(...)] arguments
#[derive(Debug)]
struct AutoConfigArgs {
    configurator_type: SynPath,
}

impl Parse for AutoConfigArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let _paren = syn::parenthesized!(content in input);
        let configurator_type = content.parse()?;

        Ok(AutoConfigArgs { configurator_type })
    }
}

/// Procedural macro to add automatic configuration to a main function.
///
/// This macro transforms a simple main function with App::new() into one
/// that automatically loads configuration based on the provided configurator type.
///
/// Example:
/// ```rust
/// #[auto_config(WebConfigurator)]
/// #[tokio::main]
/// async fn main() {
///     App::new()
///         .add_plugin(WebPlugin)
///         .run()
///         .await
/// }
/// ```
#[proc_macro_attribute]
pub fn auto_config(args: TokenStream, input: TokenStream) -> TokenStream {
    let config_args = parse_macro_input!(args as AutoConfigArgs);
    let input_fn = parse_macro_input!(input as ItemFn);

    // Extract the function body to wrap it with configuration loading
    let fn_body = input_fn.block.clone();
    let configurator_type = &config_args.configurator_type;

    // Create a new function body that wraps the original with configuration loading
    let new_fn_body = quote! {
        {
            // Initialize and load configuration
            let configurator = #configurator_type::new();
            let config = configurator.load_config().unwrap_or_else(|e| {
                eprintln!("Error loading configuration: {}", e);
                std::process::exit(1);
            });

            // Set up the environment based on configuration
            let environment = config.get_string("app.environment")
                .ok()
                .and_then(|env| navius_core::Environment::from_name(&env))
                .unwrap_or_default();

            tracing::info!("Environment: {}", environment.name());

            // Execute the original function body
            #fn_body
        }
    };

    // Create the new function with the wrapped body
    let mut new_fn = input_fn.clone();
    new_fn.block = syn::parse2(new_fn_body).unwrap();

    // Return the modified function
    quote! {
        #new_fn
    }
    .into()
}

// Helper structure to parse #[navius_router(...)] arguments
struct NaviusRouterArgs {
    modules: Vec<SynPath>, // Store module paths (e.g., self::api, crate::admin)
}

impl Parse for NaviusRouterArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        syn::parenthesized!(content in input);
        
        let mut modules = Vec::new();
        
        let meta: Meta = content.parse()?;
        if let Meta::NameValue(meta_name_value) = meta {
            if meta_name_value.path.is_ident("modules") {
                if let Expr::Array(array) = &meta_name_value.value {
                    for expr in &array.elems {
                        if let Expr::Path(expr_path) = expr {
                            modules.push(expr_path.path.clone());
                        } else {
                            return Err(syn::Error::new_spanned(
                                expr,
                                "Expected a module path",
                            ));
                        }
                    }
                } else {
                    return Err(syn::Error::new_spanned(
                        &meta_name_value.value,
                        "Expected an array of module paths",
                    ));
                }
            } else {
                return Err(syn::Error::new_spanned(
                    meta_name_value.path,
                    "Expected 'modules' attribute",
                ));
            }
        } else {
            return Err(syn::Error::new_spanned(
                meta,
                "Expected 'modules = [...]' format",
            ));
        }
        
        Ok(NaviusRouterArgs { modules })
    }
}

/// Procedural macro to automatically discover and register routes in a module.
///
/// This macro scans the specified modules for functions marked with #[route]
/// and modules marked with #[nest], then generates a function that registers
/// all discovered routes with a RouteRegistry.
///
/// Example:
/// ```rust
/// #[navius_router(modules = [self::api, crate::admin])]
/// fn register_routes(registry: &mut RouteRegistry) {
///     // The macro will replace this function body with code that registers all discovered routes
/// }
/// ```
#[proc_macro_attribute]
pub fn navius_router(args: TokenStream, input: TokenStream) -> TokenStream {
    let router_args = parse_macro_input!(args as NaviusRouterArgs);
    let input_fn = parse_macro_input!(input as ItemFn);
    
    // Check that the function has the right signature
    if input_fn.sig.inputs.len() != 1 {
        return syn::Error::new_spanned(
            &input_fn.sig,
            "Function must take exactly one argument: &mut RouteRegistry",
        )
        .to_compile_error()
        .into();
    }
    
    // Extract the modules to scan
    let modules = &router_args.modules;
    
    // Generate registration code for each module
    let module_registrations = modules.iter().map(|module| {
        let module_path = module.to_token_stream().to_string();
        let router_fn_name = format!("__navius_router_{}", module_path.replace("::", "_"));
        let router_fn_ident = Ident::new(&router_fn_name, proc_macro2::Span::call_site());
        
        quote! {
            // Try to find and call the generated router function for this module
            if let Ok(router) = #module::#router_fn_ident() {
                let path = if let Some(prefix) = #module::__NAVIUS_NEST_PREFIX {
                    prefix.to_string()
                } else {
                    "".to_string()
                };
                navius_http::server::route_discovery::_register_discovered_router(registry, &path, router);
            }
        }
    });
    
    // Generate the new function body
    let fn_name = &input_fn.sig.ident;
    let fn_args = &input_fn.sig.inputs;
    let vis = &input_fn.vis;
    
    let new_fn = quote! {
        #vis fn #fn_name(#fn_args) {
            use navius_http::server::route_discovery;
            
            #(#module_registrations)*
        }
    };
    
    new_fn.into()
}

/// Creates a zero-boilerplate Navius application.
///
/// This macro simplifies the creation of a Navius application by automatically handling:
/// - Tracing initialization
/// - Configuration loading
/// - Router setup
/// - Web plugin configuration
/// - Application building and running
///
/// # Example
///
/// ```rust
/// use navius_macros::navius_app;
/// use axum::Json;
/// 
/// #[navius_app(
///     name = "my-app",
///     routes = [api_routes],
///     plugins = [WebPlugin]
/// )]
/// async fn main() {
///     // Add any custom initialization here
///     tracing::info!("Application started!");
/// }
/// ```
#[proc_macro_attribute]
pub fn navius_app(args: TokenStream, input: TokenStream) -> TokenStream {
    let app_args = parse_macro_input!(args as NaviusAppArgs);
    let input_fn = parse_macro_input!(input as ItemFn);
    
    // Extract app name, default to "navius-app" if not provided
    let app_name = app_args.name.unwrap_or_else(|| {
        LitStr::new("navius-app", proc_macro2::Span::call_site())
    });
    
    // Extract route modules
    let route_modules = app_args.routes;
    
    // Extract plugins
    let plugins = app_args.plugins;
    
    // Extract the original function body
    let original_body = input_fn.block;
    
    // Generate the new function body with automatic setup
    let fn_name = &input_fn.sig.ident;
    let fn_args = &input_fn.sig.inputs;
    let vis = &input_fn.vis;
    let return_type = &input_fn.sig.output;
    
    let new_fn = quote! {
        #[tokio::main]
        #vis async fn #fn_name(#fn_args) #return_type {
            // Initialize tracing
            tracing_subscriber::fmt()
                .with_max_level(tracing::Level::DEBUG)
                .init();
            
            // Load configuration
            let configurator = navius_core::config::ConfiguratorBuilder::new()
                .with_prefix("NAVIUS")
                .build();
            
            let config = match configurator.load_config() {
                Ok(config) => config,
                Err(e) => {
                    tracing::error!("Failed to load configuration: {}", e);
                    return Err(navius_core::error::Error::internal(
                        format!("Failed to load configuration: {}", e)
                    ));
                }
            };
            
            // Create router
            let mut app_router = axum::Router::new();
            
            // Add route modules
            #(
                if let Some(router) = #route_modules::get_router() {
                    let path = #route_modules::get_prefix().unwrap_or_else(|| "".to_string());
                    app_router = app_router.nest(&path, router);
                }
            )*
            
            // Get server host and port from config
            let host = config
                .get_string("server.host")
                .unwrap_or_else(|_| "127.0.0.1".to_string());
            
            let port = config
                .get_int("server.port")
                .unwrap_or(3000) as u16;
            
            // Create web plugin
            let web_plugin = navius_http::WebPlugin::new()
                .with_host(host)
                .with_port(port)
                .with_router(app_router);
            
            // Build application
            let mut app_builder = navius_core::app::App::new(#app_name);
            
            // Add plugins
            #(
                app_builder = app_builder.add_plugin(#plugins::new());
            )*
            
            // Add web plugin
            app_builder = app_builder.add_web_plugin(web_plugin);
            
            // Execute the original function body
            #original_body
            
            // Run the application
            app_builder.run().await
        }
    };
    
    new_fn.into()
}

/// Arguments for the navius_app macro
#[derive(Parse)]
struct NaviusAppArgs {
    #[bracket(commaSeparated, optional)]
    routes: Punctuated<Expr, Token![,]>,
    
    #[bracket(commaSeparated, optional)]
    plugins: Punctuated<Expr, Token![,]>,
    
    #[named(optional)]
    name: Option<LitStr>,
}
