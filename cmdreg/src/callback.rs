use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, LazyLock},
};
use tokio::sync::RwLock;

use crate::command_map::CommandMap;

pub type CommandHandlerValueCallbackResult =
    Pin<Box<dyn Future<Output = Result<Option<String>>> + Send>>;

pub type CommandHandlerValueCallback =
    Box<dyn Fn(Option<String>) -> CommandHandlerValueCallbackResult + Send + Sync>;

type CallbackMap = CommandMap<String, CommandHandlerValueCallback>;

static COMMANDS_CALLBACK: LazyLock<Arc<RwLock<CallbackMap>>> =
    LazyLock::new(|| Arc::new(RwLock::new(CallbackMap::new())));

/// Register a callback command handler.
pub fn reg_command_callback(command: String, handler: CommandHandlerValueCallback) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        COMMANDS_CALLBACK.write().await.reg(command, handler);
    });
    Ok(())
}

/// Unregister a callback command.
pub fn unreg_command_callback(command: &String) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        COMMANDS_CALLBACK.write().await.unreg(command);
    });
    Ok(())
}

/// Clear all registered callback commands.
pub fn clear_command_callback() -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        COMMANDS_CALLBACK.write().await.clear();
    });
    Ok(())
}

/// Get all registered callback command keys.
pub fn get_command_callback_keys() -> Result<Vec<String>> {
    let rt = tokio::runtime::Runtime::new()?;
    Ok(rt.block_on(async { COMMANDS_CALLBACK.read().await.keys() }))
}

/// Get the number of registered callback commands.
pub fn get_command_callback_len() -> Result<usize> {
    let rt = tokio::runtime::Runtime::new()?;
    Ok(rt.block_on(async { COMMANDS_CALLBACK.read().await.len() }))
}

/// Invoke a callback command by key.
pub async fn invoke_command_callback(
    key: impl AsRef<str>,
    args: Option<String>,
) -> Result<Option<String>> {
    let commands = COMMANDS_CALLBACK.read().await;
    let k = key.as_ref().to_string();
    if let Some(cmd) = commands.get(&k) {
        cmd(args).await
    } else {
        Err(anyhow!("callback command not found: {}", k))
    }
}

/// Invoke a callback command with automatic serialization/deserialization.
pub async fn invoke_command_callback_lazy<T, TRet>(
    key: impl AsRef<str>,
    args: Option<T>,
) -> Result<Option<TRet>>
where
    T: Serialize,
    TRet: for<'de> Deserialize<'de>,
{
    let args = match args {
        Some(args) => Some(serde_json::to_string(&args)?),
        None => None,
    };
    let res = invoke_command_callback(key, args).await?;
    match res {
        Some(res) => Ok(Some(serde_json::from_str::<TRet>(&res)?)),
        None => Ok(None),
    }
}
