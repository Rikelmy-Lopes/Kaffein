//! Stable identifiers for every localised text entry.
//!
//! A key is not user-facing copy; it is the contract between code and the
//! locale catalogs. Renaming one is a breaking change. Register every leaf key
//! with [`define_keys!`] so [`ALL`] can prove the catalog is complete.

macro_rules! define_keys {
    ($($name:ident => $key:literal),* $(,)?) => {
        $(
            pub const $name: &str = $key;
        )*

        /// Every leaf key that must resolve in the active catalog.
        pub const ALL: &[&str] = &[$($key),*];
    };
}

/// A key that is a prefix for a plural family (`<key>.one`, `<key>.other`).
macro_rules! define_prefix {
    ($($name:ident => $key:literal),* $(,)?) => {
        $(
            pub const $name: &str = $key;
        )*
    };
}

define_prefix! {
    STATUS_LINE_COUNT => "status.line_count",
    MESSAGE_APP_DIRTY_DOCUMENTS => "message.app.dirty_documents",
}

define_keys! {
    APP_TITLE => "app.title",

    COMMON_CANCEL => "common.cancel",
    COMMON_CANCELLED => "common.cancelled",

    MENU_FILE => "menu.file",
    MENU_EDIT => "menu.edit",
    MENU_VIEW => "menu.view",
    MENU_RUN => "menu.run",
    MENU_LANGUAGE => "menu.language",

    CATEGORY_PROJECT => "category.project",
    CATEGORY_FILE => "category.file",
    CATEGORY_EDIT => "category.edit",
    CATEGORY_VIEW => "category.view",
    CATEGORY_RUN => "category.run",
    CATEGORY_APPLICATION => "category.application",
    CATEGORY_EDITOR => "category.editor",

    COMMAND_PROJECT_OPEN_TITLE => "command.project.open.title",
    COMMAND_PROJECT_OPEN_TOOLBAR => "command.project.open.toolbar",
    COMMAND_FILE_NEW_TITLE => "command.file.new.title",
    COMMAND_FILE_NEW_SCRATCH_TITLE => "command.file.new_scratch.title",
    COMMAND_FILE_OPEN_TITLE => "command.file.open.title",
    COMMAND_FILE_OPEN_TOOLBAR => "command.file.open.toolbar",
    COMMAND_FILE_SAVE_TITLE => "command.file.save.title",
    COMMAND_FILE_SAVE_TOOLBAR => "command.file.save.toolbar",
    COMMAND_FILE_CLOSE_TITLE => "command.file.close.title",
    COMMAND_RUN_EXECUTE_TITLE => "command.run.execute.title",
    COMMAND_RUN_EXECUTE_TOOLBAR => "command.run.execute.toolbar",
    COMMAND_EDIT_UNDO_TITLE => "command.edit.undo.title",
    COMMAND_EDIT_UNDO_TOOLBAR => "command.edit.undo.toolbar",
    COMMAND_EDIT_REDO_TITLE => "command.edit.redo.title",
    COMMAND_EDIT_REDO_TOOLBAR => "command.edit.redo.toolbar",
    COMMAND_VIEW_TOGGLE_PROJECT_TITLE => "command.view.toggle_project.title",
    COMMAND_VIEW_TOGGLE_BOTTOM_TITLE => "command.view.toggle_bottom.title",
    COMMAND_VIEW_TOGGLE_THEME_TITLE => "command.view.toggle_theme.title",
    COMMAND_APP_QUIT_TITLE => "command.app.quit.title",
    COMMAND_APP_SET_LOCALE_TITLE => "command.app.set_locale.title",
    COMMAND_EDITOR_REPLACE_TEXT_TITLE => "command.editor.replace_text.title",
    COMMAND_EDITOR_ACTIVATE_TITLE => "command.editor.activate.title",
    COMMAND_EDITOR_COMPLETE_TITLE => "command.editor.complete.title",
    COMMAND_EDITOR_COMPLETE_SELECT_TITLE => "command.editor.complete_select.title",
    COMMAND_EDITOR_COMPLETE_ACCEPT_TITLE => "command.editor.complete_accept.title",
    COMMAND_EDITOR_COMPLETE_DISMISS_TITLE => "command.editor.complete_dismiss.title",

    STATUS_NO_PROJECT => "status.no_project",
    STATUS_LINE_COUNT_ONE => "status.line_count.one",
    STATUS_LINE_COUNT_OTHER => "status.line_count.other",
    STATUS_ENCODING_UTF8 => "status.encoding.utf8",
    STATUS_LINE_ENDING_LF => "status.line_ending.lf",
    STATUS_LINE_ENDING_CRLF => "status.line_ending.crlf",
    STATUS_THEME_DARK => "status.theme.dark",
    STATUS_THEME_LIGHT => "status.theme.light",

    EDITOR_WELCOME => "editor.welcome",
    EDITOR_CLOSE_FILE => "editor.close_file",
    COMPLETION_HINT => "completion.hint",

    PROJECT_PANEL_HEADER => "project_panel.header",
    PROJECT_PANEL_OPEN => "project_panel.open",
    PROJECT_PANEL_OPEN_TOOLTIP => "project_panel.open_tooltip",
    PROJECT_PANEL_EMPTY => "project_panel.empty",
    PROJECT_PANEL_OPEN_PROJECT => "project_panel.open_project",

    BOTTOM_PANEL_TAB_PROBLEMS => "bottom_panel.tab.problems",
    BOTTOM_PANEL_TAB_BUILD => "bottom_panel.tab.build",
    BOTTOM_PANEL_TAB_CONSOLE => "bottom_panel.tab.console",
    BOTTOM_PANEL_EMPTY_PROBLEMS => "bottom_panel.empty.problems",
    BOTTOM_PANEL_EMPTY_BUILD => "bottom_panel.empty.build",
    BOTTOM_PANEL_EMPTY_CONSOLE => "bottom_panel.empty.console",
    BOTTOM_PANEL_CONSOLE_RUNNING => "bottom_panel.console.running",
    BOTTOM_PANEL_CONSOLE_COMPILE_FAILED => "bottom_panel.console.compile_failed",
    BOTTOM_PANEL_CONSOLE_COMPILE_TIMED_OUT => "bottom_panel.console.compile_timed_out",
    BOTTOM_PANEL_CONSOLE_RUN_FINISHED => "bottom_panel.console.run_finished",
    BOTTOM_PANEL_CONSOLE_RUN_EXIT => "bottom_panel.console.run_exit",
    BOTTOM_PANEL_CONSOLE_RUN_TIMED_OUT => "bottom_panel.console.run_timed_out",
    BOTTOM_PANEL_CONSOLE_RUN_FAILED => "bottom_panel.console.run_failed",
    BOTTOM_PANEL_CONSOLE_STDOUT => "bottom_panel.console.stdout",
    BOTTOM_PANEL_CONSOLE_STDERR => "bottom_panel.console.stderr",
    BOTTOM_PANEL_CONSOLE_NO_OUTPUT => "bottom_panel.console.no_output",
    BOTTOM_PANEL_CONSOLE_TRUNCATED => "bottom_panel.console.truncated",

    PROMPT_DIRECTORY_LABEL => "prompt.directory.label",
    PROMPT_DIRECTORY_HINT => "prompt.directory.hint",
    PROMPT_DIRECTORY_ACTION => "prompt.directory.action",
    PROMPT_NEW_FILE_LABEL => "prompt.new_file.label",
    PROMPT_NEW_FILE_HINT => "prompt.new_file.hint",
    PROMPT_NEW_FILE_ACTION => "prompt.new_file.action",
    PROMPT_FILE_LABEL => "prompt.file.label",
    PROMPT_FILE_HINT => "prompt.file.hint",
    PROMPT_FILE_ACTION => "prompt.file.action",
    PROMPT_ERROR_PATH_REQUIRED => "prompt.error.path_required",
    PROMPT_DEFAULT_NEW_FILE_NAME => "prompt.default_new_file_name",

    KEYBINDING_CTRL => "keybinding.ctrl",
    KEYBINDING_SHIFT => "keybinding.shift",
    KEYBINDING_ALT => "keybinding.alt",

    ERROR_KIND_USER_INPUT => "error.kind.user_input",
    ERROR_KIND_UNSUPPORTED_STATE => "error.kind.unsupported_state",
    ERROR_KIND_TOOL_FAILURE => "error.kind.tool_failure",
    ERROR_KIND_INTERNAL => "error.kind.internal",

    ENABLEMENT_NO_DOCUMENT => "enablement.no_document",
    ENABLEMENT_NOTHING_TO_UNDO => "enablement.nothing_to_undo",
    ENABLEMENT_NOTHING_TO_REDO => "enablement.nothing_to_redo",
    ENABLEMENT_UNKNOWN_COMMAND => "enablement.unknown_command",
    ENABLEMENT_NOT_JAVA => "enablement.not_java",
    ENABLEMENT_RUN_IN_PROGRESS => "enablement.run_in_progress",

    MESSAGE_WORKSPACE_ERROR => "message.workspace.error",
    MESSAGE_WORKSPACE_UNREGISTERED => "message.workspace.unregistered",
    MESSAGE_WORKSPACE_REQUIRES_PATH => "message.workspace.requires_path",
    MESSAGE_FILE_ENTER_PATH => "message.file.enter_path",
    MESSAGE_FILE_ENTER_NAME => "message.file.enter_name",
    MESSAGE_FILE_CANNOT_CREATE => "message.file.cannot_create",
    MESSAGE_FILE_EXISTS => "message.file.exists",
    MESSAGE_FILE_CREATED => "message.file.created",
    MESSAGE_FILE_CANNOT_OPEN => "message.file.cannot_open",
    MESSAGE_FILE_SWITCHED => "message.file.switched",
    MESSAGE_FILE_CANNOT_READ => "message.file.cannot_read",
    MESSAGE_FILE_OPENED => "message.file.opened",
    MESSAGE_FILE_NO_DOCUMENT => "message.file.no_document",
    MESSAGE_FILE_NO_CHANGES => "message.file.no_changes",
    MESSAGE_FILE_CANNOT_SAVE => "message.file.cannot_save",
    MESSAGE_FILE_SAVED => "message.file.saved",
    MESSAGE_FILE_DOCUMENT_GONE => "message.file.document_gone",
    MESSAGE_FILE_UNSAVED_CHANGES => "message.file.unsaved_changes",
    MESSAGE_FILE_CLOSED => "message.file.closed",
    MESSAGE_FILE_SCRATCH_SAVE => "message.file.scratch_save",
    MESSAGE_RUN_NO_DOCUMENT => "message.run.no_document",
    MESSAGE_RUN_NOT_JAVA => "message.run.not_java",
    MESSAGE_RUN_EMPTY => "message.run.empty",
    MESSAGE_RUN_ALREADY_RUNNING => "message.run.already_running",
    MESSAGE_PROJECT_ENTER_PATH => "message.project.enter_path",
    MESSAGE_PROJECT_CANNOT_OPEN => "message.project.cannot_open",
    MESSAGE_PROJECT_CANNOT_READ => "message.project.cannot_read",
    MESSAGE_PROJECT_OPENED => "message.project.opened",
    MESSAGE_EDIT_NO_DOCUMENT => "message.edit.no_document",
    MESSAGE_EDIT_NOTHING_TO_UNDO => "message.edit.nothing_to_undo",
    MESSAGE_EDIT_NOTHING_TO_REDO => "message.edit.nothing_to_redo",
    MESSAGE_EDIT_REPLACE_REQUIRES_TEXT => "message.edit.replace_requires_text",
    MESSAGE_EDIT_DOCUMENT_GONE => "message.edit.document_gone",
    MESSAGE_EDIT_ACTIVATE_REQUIRES_DOCUMENT => "message.edit.activate_requires_document",
    MESSAGE_COMPLETION_INVALID_ARGS => "message.completion.invalid_args",
    MESSAGE_COMPLETION_DOCUMENT_GONE => "message.completion.document_gone",
    MESSAGE_COMPLETION_STALE => "message.completion.stale",
    MESSAGE_VIEW_THEME => "message.view.theme",
    MESSAGE_APP_DIRTY_DOCUMENTS_ONE => "message.app.dirty_documents.one",
    MESSAGE_APP_DIRTY_DOCUMENTS_OTHER => "message.app.dirty_documents.other",
    MESSAGE_APP_LOCALE_CHANGED => "message.app.locale_changed",
    MESSAGE_APP_LOCALE_UNKNOWN => "message.app.locale_unknown",
    MESSAGE_APP_LOCALE_SAVE_FAILED => "message.app.locale_save_failed",
    MESSAGE_APP_LOCALE_REQUIRES_CODE => "message.app.locale_requires_code",
    MESSAGE_PATH_REQUIRED => "message.path_required",

    INFRA_IO => "infra.io",
    INFRA_NOT_A_FILE => "infra.not_a_file",
    INFRA_FILE_TOO_LARGE => "infra.file_too_large",
    INFRA_INVALID_UTF8 => "infra.invalid_utf8",
    INFRA_WRITE_TOO_LARGE => "infra.write_too_large",
    INFRA_NOT_A_DIRECTORY => "infra.not_a_directory",
}
