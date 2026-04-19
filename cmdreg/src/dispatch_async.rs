use anyhow::{anyhow, Result};
use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, LazyLock},
};
use tokio::sync::RwLock;

use crate::command_map::CommandMap;
use crate::context::CommandContext;
use crate::handler_async::CommandHandlerAsync;
use crate::response::CommandResult;

type HandlerResultAsync<'a> = Pin<Box<dyn Future<Output = CommandResult> + Send + 'a>>;

type HandlerValueAsync =
    Box<dyn for<'b> Fn(&'b str, CommandContext<'b>) -> HandlerResultAsync<'b> + Send + Sync>;

type AsyncMap = CommandMap<&'static str, HandlerValueAsync>;

static COMMANDS_ASYNC: LazyLock<Arc<RwLock<AsyncMap>>> =
    LazyLock::new(|| Arc::new(RwLock::new(AsyncMap::new())));

/// Register an async command handler.
pub fn reg_command_async<T, H>(command: &'static str, handler: H) -> Result<()>
where
    H: CommandHandlerAsync<T> + Send + Sync + 'static,
{
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let mut commands = COMMANDS_ASYNC.write().await;
        let f = Arc::new(handler);
        let wrapped_fn =
            for<'b> move |key: &'b str, ctx: CommandContext<'b>| -> HandlerResultAsync<'b> {
                let f = Arc::clone(&f);
                Box::pin(async move { f.call(key, ctx).await })
            };
        commands.reg(command, Box::new(wrapped_fn));
    });
    Ok(())
}

/// Unregister an async command.
pub fn unreg_command_async(command: &'static str) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        COMMANDS_ASYNC.write().await.unreg(&command);
    });
    Ok(())
}

/// Clear all registered async commands.
pub fn clear_commands_async() -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        COMMANDS_ASYNC.write().await.clear();
    });
    Ok(())
}

/// Get all registered async command keys.
pub fn get_command_async_keys() -> Result<Vec<String>> {
    let rt = tokio::runtime::Runtime::new()?;
    Ok(rt.block_on(async { COMMANDS_ASYNC.read().await.keys() }))
}

/// Get the number of registered async commands.
pub fn get_command_async_len() -> Result<usize> {
    let rt = tokio::runtime::Runtime::new()?;
    Ok(rt.block_on(async { COMMANDS_ASYNC.read().await.len() }))
}

/// Invoke an async command by key.
pub async fn invoke_command_async<'a>(key: &'a str, ctx: CommandContext<'a>) -> CommandResult {
    let commands = COMMANDS_ASYNC.read().await;
    if let Some(cmd) = commands.get(&key) {
        cmd(key, ctx).await
    } else {
        Err(anyhow!("async command not found: {}", key))
    }
}
