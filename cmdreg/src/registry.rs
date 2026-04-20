use anyhow::Result;

/// Metadata for a single command parameter.
#[cfg(feature = "metadata")]
#[derive(Debug, Clone, serde::Serialize)]
pub struct CommandParamMeta {
    /// Parameter name (as it appears in Rust source).
    pub name: &'static str,
    /// Parameter type (as a string, e.g. `"String"`, `"bool"`).
    pub r#type: &'static str,
}

/// Metadata describing a registered command.
#[cfg(feature = "metadata")]
#[derive(Debug, Clone, serde::Serialize)]
pub struct CommandMeta {
    /// Full command name (e.g. `"fs.read"`, `"ping"`).
    pub name: &'static str,
    /// Whether the handler is async.
    pub is_async: bool,
    /// Handler style: `"plain"` or `"classic"`.
    pub style: &'static str,
    /// Parameters (empty for classic style or no-param handlers).
    pub params: &'static [CommandParamMeta],
    /// Return type as a string (e.g. `"Vec<String>"`, `"CommandResult"`, `"()"`).
    pub return_type: &'static str,
}

/// A registration entry collected by the `#[command]` macro via `inventory`.
pub struct CommandRegistration {
    pub register: fn() -> Result<()>,
    #[cfg(feature = "metadata")]
    pub meta: CommandMeta,
}

inventory::collect!(CommandRegistration);

/// Execute all auto-registered command handlers collected by `#[command]` macros.
pub fn reg_all_commands() -> Result<()> {
    for reg in inventory::iter::<CommandRegistration> {
        (reg.register)()?;
    }
    Ok(())
}

/// Collect metadata for all commands registered via `#[command]` macros.
#[cfg(feature = "metadata")]
pub fn get_all_command_metas() -> Vec<&'static CommandMeta> {
    inventory::iter::<CommandRegistration>
        .into_iter()
        .map(|reg| &reg.meta)
        .collect()
}

/// Export all command metadata to a JSON file at the given path.
#[cfg(feature = "metadata")]
pub fn export_commands_json<P: AsRef<std::path::Path>>(path: P) -> Result<()> {
    let metas = get_all_command_metas();
    let json = serde_json::to_string_pretty(&metas)?;
    std::fs::write(path, json)?;
    Ok(())
}
