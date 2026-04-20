use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, FnArg, ItemFn, LitStr, Pat, PatType, ReturnType, Type, TypePath};

/// Compact a token stream string by removing unnecessary spaces around punctuation.
fn compact_type_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            ' ' => {
                // Drop space if previous char is punctuation-like, or peek next
                // We'll add space and trim later
                out.push(' ');
            }
            _ => out.push(ch),
        }
    }
    // Remove spaces around < > , ( ) [ ] & :
    let mut result = out;
    for &punct in &[
        "< ", " <", "> ", " >", "( ", " (", ") ", " )", "[ ", " [", "] ", " ]", " ,", "& ",
    ] {
        let compact = punct.trim();
        result = result.replace(punct, compact);
    }
    // Restore space after comma
    result = result.replace(',', ", ");
    // Collapse multiple spaces
    while result.contains("  ") {
        result = result.replace("  ", " ");
    }
    result.trim().to_string()
}

/// Get the return type as a string for metadata.
fn return_type_string(ret: &ReturnType) -> String {
    match ret {
        ReturnType::Default => "()".to_string(),
        ReturnType::Type(_, ty) => compact_type_string(&quote!(#ty).to_string()),
    }
}

/// Read the global `rename_all` default from `[package.metadata.cmdreg]` in the
/// calling crate's `Cargo.toml`.
fn read_global_rename_all() -> Option<String> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").ok()?;
    let cargo_toml_path = std::path::Path::new(&manifest_dir).join("Cargo.toml");
    let content = std::fs::read_to_string(cargo_toml_path).ok()?;
    let table: toml::Table = content.parse().ok()?;
    table
        .get("package")?
        .get("metadata")?
        .get("cmdreg")?
        .get("rename_all")?
        .as_str()
        .map(String::from)
}

/// Parsed `#[command(...)]` attribute arguments.
struct CommandAttr {
    prefix: Option<LitStr>,
    rename_all: Option<LitStr>,
}

impl syn::parse::Parse for CommandAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut prefix = None;
        let mut rename_all = None;

        if input.is_empty() {
            return Ok(Self { prefix, rename_all });
        }

        // Try to parse a string literal first (the prefix)
        if input.peek(LitStr) {
            prefix = Some(input.parse()?);
            if input.is_empty() {
                return Ok(Self { prefix, rename_all });
            }
            input.parse::<syn::Token![,]>()?;
        }

        // Parse key = value pairs
        while !input.is_empty() {
            let key: syn::Ident = input.parse()?;
            input.parse::<syn::Token![=]>()?;
            let value: LitStr = input.parse()?;

            if key == "rename_all" {
                rename_all = Some(value);
            } else {
                return Err(syn::Error::new_spanned(
                    key,
                    "unknown attribute, expected `rename_all`",
                ));
            }

            if !input.is_empty() {
                input.parse::<syn::Token![,]>()?;
            }
        }

        Ok(Self { prefix, rename_all })
    }
}

/// Check if the return type is `CommandResult`.
fn is_command_result(ret: &ReturnType) -> bool {
    if let ReturnType::Type(_, ty) = ret {
        if let Type::Path(TypePath { path, .. }) = ty.as_ref() {
            if let Some(last) = path.segments.last() {
                return last.ident == "CommandResult";
            }
        }
    }
    false
}

/// Check if the return type looks like `Result<...>`.
fn is_result_type(ret: &ReturnType) -> bool {
    if let ReturnType::Type(_, ty) = ret {
        if let Type::Path(TypePath { path, .. }) = ty.as_ref() {
            if let Some(last) = path.segments.last() {
                return last.ident == "Result";
            }
        }
    }
    false
}

/// Check if the return type is `()` or omitted.
fn is_unit_return(ret: &ReturnType) -> bool {
    match ret {
        ReturnType::Default => true,
        ReturnType::Type(_, ty) => {
            if let Type::Tuple(tuple) = ty.as_ref() {
                tuple.elems.is_empty()
            } else {
                false
            }
        }
    }
}

/// Recursively check whether a type contains a reference (`&T`).
fn contains_reference(ty: &Type) -> bool {
    match ty {
        Type::Reference(_) => true,
        Type::Path(TypePath { path, .. }) => path.segments.iter().any(|seg| {
            if let syn::PathArguments::AngleBracketed(args) = &seg.arguments {
                args.args.iter().any(|arg| {
                    if let syn::GenericArgument::Type(inner) = arg {
                        contains_reference(inner)
                    } else {
                        false
                    }
                })
            } else {
                false
            }
        }),
        Type::Tuple(tuple) => tuple.elems.iter().any(contains_reference),
        Type::Array(arr) => contains_reference(&arr.elem),
        Type::Slice(sl) => contains_reference(&sl.elem),
        Type::Paren(p) => contains_reference(&p.elem),
        _ => false,
    }
}

/// Mark a function as a command handler and auto-register it.
///
/// Automatically selects `reg_command` or `reg_command_async` based on whether
/// the function is `async`.
///
/// - With a prefix: `#[command("prefix")]` → command name is `"{prefix}.{fn_name}"`.
/// - Without a prefix: `#[command]` → command name is `"{fn_name}"`.
///
/// # Options
///
/// - `rename_all = "camelCase"` — apply serde rename to the auto-generated args
///   struct. Accepts any value supported by serde (e.g. `"camelCase"`,
///   `"snake_case"`, `"PascalCase"`, `"SCREAMING_SNAKE_CASE"`, `"kebab-case"`).
///   When omitted, no rename is applied (field names match Rust parameter names).
///
/// ```rust,ignore
/// #[command("fs", rename_all = "camelCase")]
/// fn get_file_list(file_path: String, is_recursive: bool) -> Vec<String> {
///     vec![]
/// }
/// // JSON: {"filePath": "...", "isRecursive": true}
/// ```
///
/// # Two styles
///
/// ## Classic style (extractor-based)
///
/// Use `Json<T>` extractors with any return type:
///
/// ```rust,ignore
/// #[command("fs")]
/// fn exists(Json(args): Json<ExistsArgs>) -> bool {
///     true
/// }
/// ```
///
/// ## Plain style (auto-generated)
///
/// Use plain parameters with any return type:
///
/// ```rust,ignore
/// #[command("fs")]
/// fn get_file_list(path: String, recursive: bool) -> Vec<String> {
///     vec![]
/// }
/// ```
///
/// # Return type handling (both styles)
///
/// - `T: Serialize` → wrapped with `CommandResponse::json(value)`
/// - `Result<T: Serialize>` → unwrapped with `?`, then wrapped with json
/// - `CommandResult` → passed through directly
/// - `()` / no return → `Ok(CommandResponse::None)`
#[proc_macro_attribute]
pub fn command(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = parse_macro_input!(attr as CommandAttr);
    let input = parse_macro_input!(item as ItemFn);

    let fn_name = &input.sig.ident;
    let fn_name_str = fn_name.to_string();
    let fn_name_raw = fn_name_str.strip_prefix("r#").unwrap_or(&fn_name_str);
    let is_async = input.sig.asyncness.is_some();

    let command_name = match &attr.prefix {
        Some(p) if !p.value().is_empty() => format!("{}.{}", p.value(), fn_name_raw),
        _ => fn_name_raw.to_string(),
    };

    let reg_fn_name = format_ident!("__cmdreg_auto_reg_{}", fn_name_raw);

    // Collect typed parameters (excluding `self`)
    let params: Vec<&PatType> = input
        .sig
        .inputs
        .iter()
        .filter_map(|arg| {
            if let FnArg::Typed(pat_type) = arg {
                Some(pat_type)
            } else {
                None
            }
        })
        .collect();

    let has_params = !params.is_empty();
    let all_plain = params.iter().all(|pt| matches!(*pt.pat, Pat::Ident(_)));
    let is_cmd_result = is_command_result(&input.sig.output);

    // Plain style only when all params are plain identifiers (not extractor patterns)
    let needs_plain_style = has_params && all_plain;

    // ── Build metadata for CommandRegistration ──
    let return_type_str = return_type_string(&input.sig.output);
    let style_str = if needs_plain_style {
        "plain"
    } else {
        "classic"
    };

    // For plain style, we can emit param metadata; for classic style, params = []
    let meta_params = if needs_plain_style {
        let param_metas: Vec<_> = params
            .iter()
            .map(|pt| {
                let name_str = if let Pat::Ident(pat_ident) = &*pt.pat {
                    let s = pat_ident.ident.to_string();
                    s.strip_prefix("r#").unwrap_or(&s).to_string()
                } else {
                    "?".to_string()
                };
                let ty = &pt.ty;
                let type_str = compact_type_string(&quote!(#ty).to_string());
                quote! {
                    cmdreg::CommandParamMeta {
                        name: #name_str,
                        r#type: #type_str,
                    }
                }
            })
            .collect();
        quote! { &[#(#param_metas),*] }
    } else {
        quote! { &[] }
    };

    let meta_expr = quote! {
        cmdreg::CommandMeta {
            name: #command_name,
            is_async: #is_async,
            style: #style_str,
            params: #meta_params,
            return_type: #return_type_str,
        }
    };

    // Helper: generate inventory::submit! via cmdreg's helper macro,
    // which handles the metadata cfg internally (no cfg in expanded code).
    let submit_block = |reg_fn: &syn::Ident| {
        quote! {
            cmdreg::__submit_registration!(#reg_fn, #meta_expr);
        }
    };

    if !needs_plain_style {
        // ── Classic style (extractor params or no params) ──

        if is_cmd_result {
            // Already returns CommandResult → register directly
            let reg_call = if is_async {
                quote! { cmdreg::reg_command_async(#command_name, #fn_name) }
            } else {
                quote! { cmdreg::reg_command(#command_name, #fn_name) }
            };

            let submit = submit_block(&reg_fn_name);

            let expanded = quote! {
                #input

                fn #reg_fn_name() -> ::anyhow::Result<()> {
                    #reg_call
                }

                #submit
            };
            return expanded.into();
        }

        // Non-CommandResult return → generate forwarding wrapper
        let wrapper_name = format_ident!("__cmdreg_wrapper_{}", fn_name_raw);

        let wrapper_params: Vec<_> = params
            .iter()
            .enumerate()
            .map(|(i, pt)| {
                let name = format_ident!("__cmdreg_p{}", i);
                let ty = &pt.ty;
                quote! { #name: #ty }
            })
            .collect();

        let call_args: Vec<_> = (0..params.len())
            .map(|i| {
                let name = format_ident!("__cmdreg_p{}", i);
                quote! { #name }
            })
            .collect();

        let await_suffix = if is_async {
            quote! { .await }
        } else {
            quote! {}
        };
        let fn_call = quote! { #fn_name(#(#call_args),*) #await_suffix };

        let body = if is_unit_return(&input.sig.output) {
            quote! { #fn_call; Ok(cmdreg::CommandResponse::None) }
        } else if is_result_type(&input.sig.output) {
            quote! { cmdreg::CommandResponse::json(#fn_call?) }
        } else {
            quote! { cmdreg::CommandResponse::json(#fn_call) }
        };

        let async_kw = if is_async {
            quote! { async }
        } else {
            quote! {}
        };

        let wrapper_fn = quote! {
            #[doc(hidden)]
            #[allow(non_snake_case)]
            #async_kw fn #wrapper_name(#(#wrapper_params),*) -> cmdreg::CommandResult {
                #body
            }
        };

        let reg_call = if is_async {
            quote! { cmdreg::reg_command_async(#command_name, #wrapper_name) }
        } else {
            quote! { cmdreg::reg_command(#command_name, #wrapper_name) }
        };

        let submit = submit_block(&reg_fn_name);

        let expanded = quote! {
            #input

            #wrapper_fn

            fn #reg_fn_name() -> ::anyhow::Result<()> {
                #reg_call
            }

            #submit
        };
        return expanded.into();
    }

    // ── Plain style: generate args struct + wrapper function ──

    // Check for reference types in parameters — they can't be deserialized
    for pt in &params {
        if contains_reference(&pt.ty) {
            let name = if let Pat::Ident(pat_ident) = &*pt.pat {
                pat_ident.ident.to_string()
            } else {
                "?".to_string()
            };
            return syn::Error::new_spanned(
                &pt.ty,
                format!(
                    "#[command] plain-style parameter `{}` contains a reference type, \
                     which cannot be deserialized from JSON. Use an owned type instead \
                     (e.g. `String` instead of `&str`).",
                    name
                ),
            )
            .to_compile_error()
            .into();
        }
    }

    let struct_name = format_ident!("__CmdregArgs_{}", fn_name_raw);
    let wrapper_name = format_ident!("__cmdreg_wrapper_{}", fn_name_raw);

    let param_names: Vec<&syn::Ident> = params
        .iter()
        .map(|pt| {
            if let Pat::Ident(pat_ident) = &*pt.pat {
                &pat_ident.ident
            } else {
                unreachable!()
            }
        })
        .collect();
    let param_types: Vec<&Type> = params.iter().map(|pt| pt.ty.as_ref()).collect();

    // Args struct (when there are params)
    let effective_rename_all = attr
        .rename_all
        .as_ref()
        .map(|lit| lit.value())
        .or_else(read_global_rename_all);

    let serde_attrs = if let Some(ref rename_str) = effective_rename_all {
        quote! { #[serde(crate = "cmdreg::__serde", rename_all = #rename_str)] }
    } else {
        quote! { #[serde(crate = "cmdreg::__serde")] }
    };

    let struct_def = if has_params {
        quote! {
            #[derive(cmdreg::__serde::Deserialize)]
            #serde_attrs
            #[doc(hidden)]
            #[allow(non_camel_case_types)]
            struct #struct_name {
                #( #param_names: #param_types, )*
            }
        }
    } else {
        quote! {}
    };

    // Build function call expression
    let await_suffix = if is_async {
        quote! { .await }
    } else {
        quote! {}
    };

    let fn_call = if has_params {
        let call_args: Vec<_> = param_names
            .iter()
            .map(|name| quote! { args.#name })
            .collect();
        quote! { #fn_name(#( #call_args ),*) #await_suffix }
    } else {
        quote! { #fn_name() #await_suffix }
    };

    // Return-value conversion
    let body = if is_cmd_result {
        fn_call
    } else if is_unit_return(&input.sig.output) {
        quote! { #fn_call; Ok(cmdreg::CommandResponse::None) }
    } else if is_result_type(&input.sig.output) {
        quote! { cmdreg::CommandResponse::json(#fn_call?) }
    } else {
        quote! { cmdreg::CommandResponse::json(#fn_call) }
    };

    // Wrapper function signature
    let wrapper_params = if has_params {
        quote! { cmdreg::Json(args): cmdreg::Json<#struct_name> }
    } else {
        quote! {}
    };
    let async_kw = if is_async {
        quote! { async }
    } else {
        quote! {}
    };

    let wrapper_fn = quote! {
        #[doc(hidden)]
        #[allow(non_snake_case)]
        #async_kw fn #wrapper_name(#wrapper_params) -> cmdreg::CommandResult {
            #body
        }
    };

    // Registration
    let reg_call = if is_async {
        quote! { cmdreg::reg_command_async(#command_name, #wrapper_name) }
    } else {
        quote! { cmdreg::reg_command(#command_name, #wrapper_name) }
    };

    let submit = submit_block(&reg_fn_name);

    let expanded = quote! {
        #input

        #struct_def

        #wrapper_fn

        fn #reg_fn_name() -> ::anyhow::Result<()> {
            #reg_call
        }

        #submit
    };

    expanded.into()
}
