use serde_json::Value;

/// The context passed to a command handler, carrying the input arguments.
#[derive(Clone, Debug)]
pub enum CommandContext<'a> {
    /// JSON string input.
    String(&'a String),
    /// Pre-parsed `serde_json::Value` input.
    Value(&'a Value),
    /// No arguments provided.
    None,
}

unsafe impl Send for CommandContext<'_> {}

impl<'a> CommandContext<'a> {
    pub fn from_string(args: Option<&'a String>) -> Self {
        match args {
            Some(s) => CommandContext::String(s),
            None => CommandContext::None,
        }
    }

    pub fn from_value(args: Option<&'a Value>) -> Self {
        match args {
            Some(v) => CommandContext::Value(v),
            None => CommandContext::None,
        }
    }

    pub fn is_some(&self) -> bool {
        !matches!(self, CommandContext::None)
    }

    pub fn is_none(&self) -> bool {
        matches!(self, CommandContext::None)
    }
}
