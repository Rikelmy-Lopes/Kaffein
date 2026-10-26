//! What a command reports back to its caller.

use kaffein_i18n::{keys, Text};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    UserInput,
    UnsupportedState,
    ToolFailure,
    Internal,
}

impl ErrorKind {
    pub fn key(self) -> &'static str {
        match self {
            Self::UserInput => keys::ERROR_KIND_USER_INPUT,
            Self::UnsupportedState => keys::ERROR_KIND_UNSUPPORTED_STATE,
            Self::ToolFailure => keys::ERROR_KIND_TOOL_FAILURE,
            Self::Internal => keys::ERROR_KIND_INTERNAL,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandError {
    pub kind: ErrorKind,
    pub message: Text,
}

impl CommandError {
    pub fn new(kind: ErrorKind, message: impl Into<Text>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CommandOutcome {
    Success {
        message: Option<Text>,
    },
    /// Precondition unmet. The reason is shown to the user; it is not a silent
    /// success.
    NoOp {
        reason: Text,
    },
    Cancelled,
    Failed(CommandError),
}

impl CommandOutcome {
    pub fn success(message: impl Into<Text>) -> Self {
        Self::Success {
            message: Some(message.into()),
        }
    }

    pub fn success_silent() -> Self {
        Self::Success { message: None }
    }

    pub fn no_op(reason: impl Into<Text>) -> Self {
        Self::NoOp {
            reason: reason.into(),
        }
    }

    pub fn failed(kind: ErrorKind, message: impl Into<Text>) -> Self {
        Self::Failed(CommandError::new(kind, message))
    }

    /// The deferred text, resolved by the UI against the active locale.
    pub fn message(&self) -> Option<&Text> {
        match self {
            Self::Success { message } => message.as_ref(),
            Self::NoOp { reason } => Some(reason),
            Self::Cancelled => None,
            Self::Failed(error) => Some(&error.message),
        }
    }

    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success { .. })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Enablement {
    Enabled,
    /// The reason is a localization key so the UI can render it in the active
    /// language.
    Disabled(&'static str),
}

impl Enablement {
    pub fn is_enabled(self) -> bool {
        matches!(self, Self::Enabled)
    }

    pub fn reason(self) -> Option<&'static str> {
        match self {
            Self::Enabled => None,
            Self::Disabled(reason) => Some(reason),
        }
    }
}
