use std::path::PathBuf;

use kaffein_domain::DocumentId;

use crate::services::Services as ServiceSet;
use crate::state::AppState;

/// Commands that are listed in the registry but are not user-facing menu
/// entries (for example, the editor's text replacement command).
pub mod command_id {
    pub const PROJECT_OPEN: &str = "project.open";
    pub const FILE_NEW: &str = "file.new";
    pub const FILE_OPEN: &str = "file.open";
    pub const FILE_SAVE: &str = "file.save";
    pub const FILE_CLOSE: &str = "file.close";
    pub const EDIT_UNDO: &str = "edit.undo";
    pub const EDIT_REDO: &str = "edit.redo";
    pub const EDITOR_REPLACE_TEXT: &str = "editor.replace_text";
    pub const EDITOR_ACTIVATE: &str = "editor.activate";
    pub const VIEW_TOGGLE_PROJECT: &str = "view.toggle_project";
    pub const VIEW_TOGGLE_BOTTOM: &str = "view.toggle_bottom";
    pub const VIEW_TOGGLE_THEME: &str = "view.toggle_theme";
    pub const APP_QUIT: &str = "app.quit";
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CommandArgs {
    None,
    Path(PathBuf),
    Document(DocumentId),
    ReplaceText {
        document: DocumentId,
        text: String,
        at_ms: u64,
    },
}

impl CommandArgs {
    pub fn none() -> Self {
        Self::None
    }

    pub fn path(path: impl Into<PathBuf>) -> Self {
        Self::Path(path.into())
    }
}

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    N,
    O,
    S,
    Z,
    Y,
    Q,
}

/// A key binding expressed in application terms. The UI maps it to egui.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyBinding {
    pub key: Key,
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
}

impl KeyBinding {
    pub const fn ctrl(key: Key) -> Self {
        Self {
            key,
            ctrl: true,
            shift: false,
            alt: false,
        }
    }

    pub const fn ctrl_shift(key: Key) -> Self {
        Self {
            key,
            ctrl: true,
            shift: true,
            alt: false,
        }
    }

    pub fn display(self) -> String {
        let mut text = String::new();
        if self.ctrl {
            text.push_str("Ctrl+");
        }
        if self.shift {
            text.push_str("Shift+");
        }
        if self.alt {
            text.push_str("Alt+");
        }
        text.push(match self.key {
            Key::N => 'N',
            Key::O => 'O',
            Key::S => 'S',
            Key::Z => 'Z',
            Key::Y => 'Y',
            Key::Q => 'Q',
        });
        text
    }
}

/// The argument a command needs from a user-facing dialog, if any.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArgKind {
    None,
    NewFile,
    File,
    Directory,
    Text,
}

impl ArgKind {
    pub fn needs_path_prompt(self) -> bool {
        matches!(self, Self::NewFile | Self::File | Self::Directory)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Menu {
    File,
    Edit,
    View,
}

impl Menu {
    pub fn title(self) -> &'static str {
        match self {
            Self::File => "File",
            Self::Edit => "Edit",
            Self::View => "View",
        }
    }

    pub const ALL: [Menu; 3] = [Self::File, Self::Edit, Self::View];
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MenuPlacement {
    pub menu: Menu,
    pub order: u16,
    pub separator_before: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ToolbarPlacement {
    pub order: u16,
    pub label: &'static str,
}

pub type CommandFn = fn(&mut AppState, &mut ServiceSet, CommandArgs) -> CommandOutcome;
pub type EnablementFn = fn(&AppState) -> Enablement;
pub type CheckedFn = fn(&AppState) -> bool;

#[derive(Clone, Copy)]
pub struct Command {
    pub id: &'static str,
    pub title: &'static str,
    pub category: &'static str,
    pub menu: Option<MenuPlacement>,
    pub toolbar: Option<ToolbarPlacement>,
    pub shortcuts: &'static [KeyBinding],
    pub args: ArgKind,
    pub can_execute: EnablementFn,
    pub checked: Option<CheckedFn>,
    pub run: CommandFn,
}

impl Command {
    pub fn primary_shortcut(&self) -> Option<KeyBinding> {
        self.shortcuts.first().copied()
    }

    pub fn enablement(&self, state: &AppState) -> Enablement {
        (self.can_execute)(state)
    }
}

impl std::fmt::Debug for Command {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Command")
            .field("id", &self.id)
            .field("title", &self.title)
            .field("category", &self.category)
            .finish_non_exhaustive()
    }
}
