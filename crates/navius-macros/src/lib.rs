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

/// Procedural macro to define a route on an async function.
/// Parses arguments like path, method, etc., but currently only serves as a marker
/// for the #[nest] macro. It returns the input function unmodified.
#[proc_macro_attribute]
pub fn route(args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse args to validate them, but don't use the result here.
    let _parsed_args = parse_macro_input!(args as RouteArgs);
    let input_fn = parse_macro_input!(input as ItemFn);

    // Return the function unmodified. #[nest] handles the rest.
    quote! { #input_fn }.into()
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

/// Procedural macro to define nested routes within a module.
#[proc_macro_attribute]
pub fn nest(args: TokenStream, input: TokenStream) -> TokenStream {
    let nest_args = parse_macro_input!(args as NestArgs);
    let mut input_mod = parse_macro_input!(input as ItemMod);
    let mod_ident = &input_mod.ident;
    let route_defs = {
        let mut route_defs = Vec::new();
        let mut errors = Vec::new();
        let items = if let Some((_, ref mut items)) = input_mod.content {
            items
        } else {
            errors.push(syn::Error::new_spanned(
                &input_mod.ident,
                "#[nest] must be applied to an inline module (mod name { ... })",
            ));
            let error_stream = errors.into_iter().map(|e| e.to_compile_error());
            return quote! { #input_mod #(#error_stream)* }.into();
        };
        for item in items.iter_mut() {
            if let Item::Fn(item_fn) = item {
                let mut route_attr_index = None;
                for (i, attr) in item_fn.attrs.iter().enumerate() {
                    if attr.path().is_ident("route") {
                        route_attr_index = Some(i);
                        break;
                    }
                }
                if let Some(index) = route_attr_index {
                    let attr = item_fn.attrs.remove(index);
                    match attr.parse_args::<RouteArgs>() {
                        Ok(route_args) => {
                            route_defs.push((route_args, item_fn.sig.ident.clone()));
                        }
                        Err(e) => errors.push(e),
                    }
                }
            }
        }
        if !errors.is_empty() {
            let error_stream = errors.into_iter().map(|e| e.to_compile_error());
            return quote! { #input_mod #(#error_stream)* }.into();
        }
        route_defs
    };

    let route_calls = {
        let mut stream = proc_macro2::TokenStream::new();
        for (route_args, handler_name) in route_defs {
            let path = route_args.path.expect("Path validated");
            let methods_to_gen = if route_args.methods.is_empty() {
                vec![Ident::new("GET", path.span())]
            } else {
                route_args.methods
            };
            let method_chain = {
                let mut chain = proc_macro2::TokenStream::new();
                let mut first = true;
                for method in methods_to_gen {
                    let method_lower =
                        Ident::new(&method.to_string().to_lowercase(), method.span());
                    if first {
                        chain.extend(quote! { ::axum::routing::#method_lower(#handler_name) });
                        first = false;
                    } else {
                        chain.extend(quote! { .#method_lower(#handler_name) });
                    }
                }
                chain
            };
            stream.extend(quote! {
                router = router.route(#path, #method_chain);
            });
        }
        stream
    };
    let middleware_layer = quote! {};
    let router_fn_name = Ident::new(&format!("__navius_router_{}", mod_ident), mod_ident.span());

    let generated_router_fn = quote! {
        pub fn #router_fn_name() -> ::axum::Router<()> {
            use ::axum::routing::{self, get, post, put, delete, patch, head, options, trace};
            use ::axum::Router;
            let mut router = Router::<()>::new();
            #route_calls
            router
        }
    };

    let prefix_str = nest_args
        .prefix
        .map(|p| p.value())
        .unwrap_or_else(|| "".to_string());
    let prefix_const_name = Ident::new("__NAVIUS_NEST_PREFIX", proc_macro2::Span::call_site());
    let generated_prefix_const = quote! {
        #[doc(hidden)]
        pub const #prefix_const_name: &str = #prefix_str;
    };

    let mut errors = Vec::new();
    match &mut input_mod.content {
        Some((_, items)) => {
            match syn::parse2::<Item>(generated_router_fn.clone()) {
                Ok(item) => items.push(item),
                Err(e) => errors.push(syn::Error::new_spanned(
                    &input_mod.ident,
                    format!(
                        "Failed to parse generated router function: {}\nGenerated Code:\n{}",
                        e,
                        generated_router_fn.to_string()
                    ),
                )),
            }
            match syn::parse2::<Item>(generated_prefix_const.clone()) {
                Ok(item) => items.push(item),
                Err(e) => errors.push(syn::Error::new_spanned(
                    &input_mod.ident,
                    format!("Failed to parse generated prefix const: {}", e),
                )),
            }
        }
        None => { /* Handled */ }
    }
    if !errors.is_empty() {
        let error_stream = errors.into_iter().map(|e| e.to_compile_error());
        return quote! { #input_mod #(#error_stream)* }.into();
    }

    quote! { #input_mod }.into()
}

// Helper structure to parse #[navius_router(...)] arguments
#[derive(Debug, Default)]
struct NaviusRouterArgs {
    modules: Vec<SynPath>, // Store module paths (e.g., self::api, crate::admin)
}

impl Parse for NaviusRouterArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut args = NaviusRouterArgs::default();
        let vars = Punctuated::<Meta, Token![,]>::parse_terminated(input)?;

        for meta in vars.iter() {
            match meta {
                Meta::NameValue(nv) => {
                    if nv.path.is_ident("modules") {
                        if let Expr::Array(array) = &nv.value {
                            for expr in &array.elems {
                                if let Expr::Path(expr_path) = expr {
                                    args.modules.push(expr_path.path.clone());
                                } else {
                                    return Err(syn::Error::new_spanned(
                                        expr,
                                        "Expected module path",
                                    ));
                                }
                            }
                        } else {
                            return Err(syn::Error::new_spanned(
                                &nv.value,
                                "Expected array of module paths for modules",
                            ));
                        }
                    }
                    // TODO: Parse other potential top-level args?
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

/// Top-level macro to collect nested routers and build the main application router.
#[proc_macro_attribute]
pub fn navius_router(args: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = &input_fn.sig.ident;
    let fn_vis = &input_fn.vis;
    let fn_return_type = &input_fn.sig.output;
    let fn_inputs = &input_fn.sig.inputs;
    if !fn_inputs.is_empty() {
        return syn::Error::new_spanned(
            fn_inputs,
            "Function annotated with #[navius_router] should take no arguments",
        )
        .to_compile_error()
        .into();
    }

    let parsed_args = parse_macro_input!(args as NaviusRouterArgs);

    // Generate the Router::nest() calls
    let mut nest_calls = proc_macro2::TokenStream::new();
    for mod_path in parsed_args.modules {
        let prefix_path = quote! { #mod_path::__NAVIUS_NEST_PREFIX };
        let router_fn_ident = Ident::new(
            &format!(
                "__navius_router_{}",
                mod_path.segments.last().unwrap().ident
            ),
            proc_macro2::Span::call_site(),
        );
        let router_fn_path = quote! { #mod_path::#router_fn_ident };

        nest_calls.extend(quote! {
            router = router.nest(#prefix_path, #router_fn_path());
        });
    }

    // TODO: Apply global middleware?
    let global_middleware = quote! {}; // Placeholder

    // Generate the final function body
    let output = quote! {
        #fn_vis fn #fn_name() #fn_return_type {
            println!("Building Navius Router via macro...");
            // Initialize router WITHOUT state
            let mut router = ::axum::Router::new();

            #nest_calls // Apply nesting calls

            // Apply global middleware
            router = router #global_middleware;

            println!("Router built!");
            router // Return the combined, state-less router
        }
    };

    output.into()
}
