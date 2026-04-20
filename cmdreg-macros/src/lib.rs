use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, FnArg, ItemFn, LitStr, Pat, PatType, ReturnType, Type, TypePath};

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
/// # Two styles
///
/// ## Classic style (extractor-based)
///
/// Use `Json<T>` extractors and return `CommandResult` explicitly:
///
/// ```rust,ignore
/// use cmdreg::{command, Json, CommandResult, CommandResponse};
///
/// #[command("fs")]
/// fn exists(Json(args): Json<ExistsArgs>) -> CommandResult {
///     CommandResponse::json(true)
/// }
/// ```
///
/// ## Plain style (auto-generated)
///
/// Use plain parameters and any `Serialize` return type. The macro
/// auto-generates a `#[derive(Deserialize)]` struct and wraps the return
/// value with `CommandResponse::json()`:
///
/// ```rust,ignore
/// use cmdreg::command;
///
/// #[command("fs")]
/// fn get_file_list(path: String, recursive: bool) -> Vec<String> {
///     vec![]
/// }
/// // Equivalent to defining a camelCase-renamed args struct + Json<T> wrapper
/// ```
///
/// Supported return types in plain style:
/// - `T: Serialize` → wrapped with `CommandResponse::json(value)`
/// - `Result<T: Serialize>` → unwrapped with `?`, then wrapped with json
/// - `CommandResult` → passed through directly
/// - `()` / no return → `Ok(CommandResponse::None)`
#[proc_macro_attribute]
pub fn command(attr: TokenStream, item: TokenStream) -> TokenStream {
    let prefix: Option<LitStr> = if attr.is_empty() {
        None
    } else {
        Some(parse_macro_input!(attr as LitStr))
    };
    let input = parse_macro_input!(item as ItemFn);

    let fn_name = &input.sig.ident;
    let fn_name_str = fn_name.to_string();
    let fn_name_raw = fn_name_str.strip_prefix("r#").unwrap_or(&fn_name_str);
    let is_async = input.sig.asyncness.is_some();

    let command_name = match &prefix {
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

    // New style if: plain params exist, OR no params but return is not CommandResult
    let needs_new_style = if has_params {
        all_plain
    } else {
        !is_cmd_result
    };

    if !needs_new_style {
        // ── Classic style: register the original function directly ──
        let reg_call = if is_async {
            quote! { cmdreg::reg_command_async(#command_name, #fn_name) }
        } else {
            quote! { cmdreg::reg_command(#command_name, #fn_name) }
        };

        let expanded = quote! {
            #input

            fn #reg_fn_name() -> ::anyhow::Result<()> {
                #reg_call
            }

            cmdreg::inventory::submit! {
                cmdreg::CommandRegistration {
                    register: #reg_fn_name,
                }
            }
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
    let struct_def = if has_params {
        quote! {
            #[derive(cmdreg::__serde::Deserialize)]
            #[serde(crate = "cmdreg::__serde", rename_all = "camelCase")]
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

    let expanded = quote! {
        #input

        #struct_def

        #wrapper_fn

        fn #reg_fn_name() -> ::anyhow::Result<()> {
            #reg_call
        }

        cmdreg::inventory::submit! {
            cmdreg::CommandRegistration {
                register: #reg_fn_name,
            }
        }
    };

    expanded.into()
}
