use anyhow::Result;

/// A registration entry collected by the `#[command]` macro via `inventory`.
pub struct CommandRegistration {
    pub register: fn() -> Result<()>,
}

inventory::collect!(CommandRegistration);

/// Execute all auto-registered command handlers collected by `#[command]` macros.
pub fn reg_all_commands() -> Result<()> {
    for reg in inventory::iter::<CommandRegistration> {
        (reg.register)()?;
    }
    Ok(())
}
