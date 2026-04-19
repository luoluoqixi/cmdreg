use anyhow::Result;
use serde::Serialize;

/// The result type returned by all command handlers.
pub type CommandResult = Result<CommandResponse>;

/// The response from a command handler.
pub enum CommandResponse {
    /// A JSON-serialized string response.
    Json(String),
    /// No response body.
    None,
}

impl CommandResponse {
    /// Serialize `data` into a JSON `CommandResponse`.
    pub fn json<T: Serialize>(data: T) -> Result<Self> {
        Ok(Self::Json(serde_json::to_string(&data)?))
    }

    pub fn from_option(option: Option<String>) -> Self {
        match option {
            Some(s) => CommandResponse::Json(s),
            None => CommandResponse::None,
        }
    }

    pub fn into_option(self) -> Option<String> {
        match self {
            CommandResponse::Json(s) => Some(s),
            CommandResponse::None => None,
        }
    }

    pub fn is_none(&self) -> bool {
        !self.is_some()
    }

    pub const fn is_some(&self) -> bool {
        matches!(*self, CommandResponse::Json(_))
    }
}
