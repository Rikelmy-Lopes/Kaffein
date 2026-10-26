//! What a command reports back to its caller.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    UserInput,
    UnsupportedState,
    ToolFailure,
    Internal,
}

impl ErrorKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::UserInput => "invalid input",
            Self::UnsupportedState => "unsupported state",
            Self::ToolFailure => "tool failure",
            Self::Internal => "internal error",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandError {
    pub kind: ErrorKind,
    pub message: String,
}

impl CommandError {
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CommandOutcome {
    Success {
        message: Option<String>,
    },
    /// Precondition unmet. The reason is shown to the user; it is not a silent
    /// success.
    NoOp {
        reason: String,
    },
    Cancelled,
    Failed(CommandError),
}

impl CommandOutcome {
    pub fn success(message: impl Into<String>) -> Self {
        Self::Success {
            message: Some(message.into()),
        }
    }

    pub fn success_silent() -> Self {
        Self::Success { message: None }
    }

    pub fn no_op(reason: impl Into<String>) -> Self {
        Self::NoOp {
            reason: reason.into(),
        }
    }

    pub fn failed(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self::Failed(CommandError::new(kind, message))
    }

    pub fn message(&self) -> Option<&str> {
        match self {
            Self::Success { message } => message.as_deref(),
            Self::NoOp { reason } => Some(reason),
            Self::Cancelled => Some("Cancelled"),
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
