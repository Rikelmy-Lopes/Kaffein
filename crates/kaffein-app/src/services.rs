use std::sync::Arc;

use kaffein_domain::{FileSystem, JavaRunner, ProjectLoader};
use kaffein_i18n::Localizer;

/// Infrastructure implementations injected by the binary. Tests substitute
/// in-memory fakes at this seam.
pub struct Services {
    pub fs: Box<dyn FileSystem>,
    pub projects: Box<dyn ProjectLoader>,
    pub java: Arc<dyn JavaRunner>,
    pub localizer: Localizer,
}

impl Services {
    pub fn new(
        fs: Box<dyn FileSystem>,
        projects: Box<dyn ProjectLoader>,
        java: Arc<dyn JavaRunner>,
        localizer: Localizer,
    ) -> Self {
        Self {
            fs,
            projects,
            java,
            localizer,
        }
    }
}
