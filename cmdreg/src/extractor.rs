use anyhow::{anyhow, Result};

use crate::context::CommandContext;

/// A wrapper for JSON-deserialized command arguments (axum-style extractor).
#[derive(Debug)]
pub struct Json<T>(pub T)
where
    T: for<'de> serde::Deserialize<'de>;

/// Marker type for handlers that take no arguments.
#[derive(Debug)]
pub struct NoArgsBody;

/// Trait for extracting typed arguments from a [`CommandContext`].
pub trait FromCommandArgs: Sized {
    fn from_args(key: &str, ctx: &CommandContext) -> Result<Self>;
}

impl<T> FromCommandArgs for Json<T>
where
    T: for<'de> serde::Deserialize<'de>,
{
    fn from_args(key: &str, ctx: &CommandContext) -> Result<Self> {
        match ctx {
            CommandContext::String(args) => {
                let input_t = serde_json::from_str::<T>(args)?;
                Ok(Json(input_t))
            }
            CommandContext::Value(args) => {
                let input_t = T::deserialize(*args)?;
                Ok(Json(input_t))
            }
            CommandContext::None => {
                Err(anyhow!("expected JSON body but no body provided: {}", key))
            }
        }
    }
}

impl FromCommandArgs for () {
    fn from_args(_: &str, _: &CommandContext) -> Result<Self> {
        Ok(())
    }
}
