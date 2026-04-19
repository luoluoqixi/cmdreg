use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, ItemFn, LitStr};

/// Mark a function as a command handler and auto-register it.
///
/// Automatically selects `reg_command` or `reg_command_async` based on whether
/// the function is `async`.
///
/// - With a prefix: `#[command("prefix")]` → command name is `"{prefix}.{fn_name}"`.
/// - Without a prefix: `#[command]` → command name is `"{fn_name}"`.
///
/// # Examples
///
/// ```rust,ignore
/// use cmdreg::{command, Json, CommandResult, CommandResponse};
///
/// #[command("workspace")]
/// async fn get_workspace(Json(args): Json<Args>) -> CommandResult {
///     CommandResponse::json(&result)
/// }
/// // Registers as "workspace.get_workspace"
///
/// #[command("fs")]
/// fn exists(Json(args): Json<ExistsArgs>) -> CommandResult {
///     CommandResponse::json(true)
/// }
/// // Registers as "fs.exists"
///
/// #[command]
/// fn ping() -> CommandResult {
///     CommandResponse::json("pong")
/// }
/// // Registers as "ping"
/// ```
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

    expanded.into()
}
