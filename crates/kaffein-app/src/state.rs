use kaffein_domain::{Document, DocumentId, ProjectTree};
use kaffein_i18n::{keys, Text};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ThemePreference {
    #[default]
    Dark,
    Light,
}

impl ThemePreference {
    pub fn label_key(self) -> &'static str {
        match self {
            Self::Dark => keys::STATUS_THEME_DARK,
            Self::Light => keys::STATUS_THEME_LIGHT,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MessageSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatusMessage {
    pub severity: MessageSeverity,
    /// Deferred text, so a locale change re-renders the existing message.
    pub text: Text,
}

impl StatusMessage {
    pub fn info(text: impl Into<Text>) -> Self {
        Self {
            severity: MessageSeverity::Info,
            text: text.into(),
        }
    }

    pub fn warning(text: impl Into<Text>) -> Self {
        Self {
            severity: MessageSeverity::Warning,
            text: text.into(),
        }
    }

    pub fn error(text: impl Into<Text>) -> Self {
        Self {
            severity: MessageSeverity::Error,
            text: text.into(),
        }
    }
}

/// Everything the UI reads. Mutation happens through commands.
#[derive(Debug)]
pub struct AppState {
    documents: Vec<Document>,
    active: Option<DocumentId>,
    project: Option<ProjectTree>,
    theme: ThemePreference,
    status: Option<StatusMessage>,
    show_project_panel: bool,
    show_bottom_panel: bool,
    quit_requested: bool,
    next_document_id: u64,
}

impl AppState {
    pub fn new(theme: ThemePreference) -> Self {
        Self {
            documents: Vec::new(),
            active: None,
            project: None,
            theme,
            status: None,
            show_project_panel: true,
            show_bottom_panel: true,
            quit_requested: false,
            next_document_id: 1,
        }
    }

    pub fn documents(&self) -> &[Document] {
        &self.documents
    }

    pub fn active_document(&self) -> Option<&Document> {
        let active = self.active?;
        self.documents
            .iter()
            .find(|document| document.id() == active)
    }

    pub fn active_document_mut(&mut self) -> Option<&mut Document> {
        let active = self.active?;
        self.documents
            .iter_mut()
            .find(|document| document.id() == active)
    }

    pub fn document(&self, id: DocumentId) -> Option<&Document> {
        self.documents.iter().find(|document| document.id() == id)
    }

    pub fn document_mut(&mut self, id: DocumentId) -> Option<&mut Document> {
        self.documents
            .iter_mut()
            .find(|document| document.id() == id)
    }

    pub fn find_document_by_path(&self, path: &std::path::Path) -> Option<&Document> {
        self.documents
            .iter()
            .find(|document| document.path() == path)
    }

    pub fn active_id(&self) -> Option<DocumentId> {
        self.active
    }

    /// Allocates the next document identity. Identity is independent of the
    /// path so a document can later be renamed without losing its history.
    pub fn allocate_document_id(&mut self) -> DocumentId {
        let id = DocumentId(self.next_document_id);
        self.next_document_id += 1;
        id
    }

    pub fn add_document(&mut self, document: Document) {
        self.active = Some(document.id());
        self.documents.push(document);
    }

    pub fn activate(&mut self, id: DocumentId) -> bool {
        if self.document(id).is_some() {
            self.active = Some(id);
            true
        } else {
            false
        }
    }

    pub fn remove_document(&mut self, id: DocumentId) -> Option<Document> {
        let index = self
            .documents
            .iter()
            .position(|document| document.id() == id)?;
        let removed = self.documents.remove(index);
        if self.active == Some(id) {
            self.active = self
                .documents
                .get(index.min(self.documents.len().saturating_sub(1)))
                .map(Document::id);
        }
        Some(removed)
    }

    pub fn project(&self) -> Option<&ProjectTree> {
        self.project.as_ref()
    }

    pub fn set_project(&mut self, project: ProjectTree) {
        self.project = Some(project);
    }

    pub fn theme(&self) -> ThemePreference {
        self.theme
    }

    pub fn set_theme(&mut self, theme: ThemePreference) {
        self.theme = theme;
    }

    pub fn toggle_theme(&mut self) -> ThemePreference {
        self.theme = match self.theme {
            ThemePreference::Dark => ThemePreference::Light,
            ThemePreference::Light => ThemePreference::Dark,
        };
        self.theme
    }

    pub fn status(&self) -> Option<&StatusMessage> {
        self.status.as_ref()
    }

    pub fn set_status(&mut self, status: StatusMessage) {
        self.status = Some(status);
    }

    pub fn clear_status(&mut self) {
        self.status = None;
    }

    pub fn show_project_panel(&self) -> bool {
        self.show_project_panel
    }

    pub fn toggle_project_panel(&mut self) {
        self.show_project_panel = !self.show_project_panel;
    }

    pub fn show_bottom_panel(&self) -> bool {
        self.show_bottom_panel
    }

    pub fn toggle_bottom_panel(&mut self) {
        self.show_bottom_panel = !self.show_bottom_panel;
    }

    pub fn quit_requested(&self) -> bool {
        self.quit_requested
    }

    pub fn request_quit(&mut self) {
        self.quit_requested = true;
    }

    pub fn dirty_document_count(&self) -> usize {
        self.documents
            .iter()
            .filter(|document| document.is_dirty())
            .count()
    }

    pub fn has_dirty_documents(&self) -> bool {
        self.dirty_document_count() > 0
    }
}
