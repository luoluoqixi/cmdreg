#![feature(closure_lifetime_binder)]
#![doc = include_str!("../../README.md")]

mod command_map;
mod context;
mod dispatch_async;
mod dispatch_sync;
mod extractor;
mod handler;
mod handler_async;
mod response;

mod callback;

#[cfg(feature = "macros")]
mod registry;

// Core types
pub use command_map::CommandMap;
pub use context::CommandContext;
pub use extractor::{FromCommandArgs, Json, NoArgsBody};
pub use handler::CommandHandler;
pub use handler_async::CommandHandlerAsync;
pub use response::{CommandResponse, CommandResult};

// Sync dispatch
pub use dispatch_sync::{
    clear_commands, get_command_keys, get_command_len, invoke_command, reg_command, unreg_command,
};

// Async dispatch
pub use dispatch_async::{
    clear_commands_async, get_command_async_keys, get_command_async_len, invoke_command_async,
    reg_command_async, unreg_command_async,
};

// Callback dispatch
pub use callback::{
    clear_command_callback, get_command_callback_keys, get_command_callback_len,
    invoke_command_callback, invoke_command_callback_lazy, reg_command_callback,
    unreg_command_callback, CommandHandlerValueCallback, CommandHandlerValueCallbackResult,
};

// Macros & auto-registration
#[cfg(feature = "macros")]
pub use cmdreg_macros::command;
#[cfg(feature = "macros")]
pub use inventory;
#[cfg(feature = "macros")]
pub use registry::{reg_all_commands, CommandRegistration};
