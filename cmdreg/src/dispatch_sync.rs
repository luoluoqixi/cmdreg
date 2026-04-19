use anyhow::{anyhow, Result};
use std::sync::{Arc, LazyLock, RwLock};

use crate::command_map::CommandMap;
use crate::context::CommandContext;
use crate::handler::CommandHandler;
use crate::response::CommandResult;

type HandlerValue = Box<dyn Fn(&str, CommandContext) -> CommandResult + Send + Sync>;

type SyncMap = CommandMap<&'static str, HandlerValue>;

static COMMANDS: LazyLock<Arc<RwLock<SyncMap>>> =
    LazyLock::new(|| Arc::new(RwLock::new(SyncMap::new())));

/// Register a sync command handler.
pub fn reg_command<T, H>(command: &'static str, handler: H) -> Result<()>
where
    H: CommandHandler<T>,
{
    match COMMANDS.write() {
        Ok(mut commands) => {
            let wrapped_fn =
                move |key: &str, ctx: CommandContext| -> CommandResult { handler.call(key, ctx) };
            commands.reg(command, Box::new(wrapped_fn));
            Ok(())
        }
        Err(err) => Err(anyhow!(err.to_string())),
    }
}

/// Unregister a sync command.
pub fn unreg_command(command: &'static str) -> Result<()> {
    match COMMANDS.write() {
        Ok(mut commands) => {
            commands.unreg(&command);
            Ok(())
        }
        Err(err) => Err(anyhow!(err.to_string())),
    }
}

/// Clear all registered sync commands.
pub fn clear_commands() -> Result<()> {
    match COMMANDS.write() {
        Ok(mut commands) => {
            commands.clear();
            Ok(())
        }
        Err(err) => Err(anyhow!(err.to_string())),
    }
}

/// Get all registered sync command keys.
pub fn get_command_keys() -> Result<Vec<String>> {
    match COMMANDS.read() {
        Ok(commands) => Ok(commands.keys()),
        Err(err) => Err(anyhow!(err.to_string())),
    }
}

/// Get the number of registered sync commands.
pub fn get_command_len() -> Result<usize> {
    match COMMANDS.read() {
        Ok(commands) => Ok(commands.len()),
        Err(err) => Err(anyhow!(err.to_string())),
    }
}

/// Invoke a sync command by key.
pub fn invoke_command(key: &str, ctx: CommandContext) -> CommandResult {
    match COMMANDS.read() {
        Ok(commands) => {
            if let Some(cmd) = commands.get(&key) {
                cmd(key, ctx)
            } else {
                Err(anyhow!("command not found: {}", key))
            }
        }
        Err(err) => Err(anyhow!(err.to_string())),
    }
}
