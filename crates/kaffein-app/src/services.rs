use kaffein_domain::{FileSystem, ProjectLoader};
use kaffein_i18n::Localizer;

/// Infrastructure implementations injected by the binary. Tests substitute
/// in-memory fakes at this seam.
pub struct Services {
    pub fs: Box<dyn FileSystem>,
    pub projects: Box<dyn ProjectLoader>,
    pub localizer: Localizer,
}

impl Services {
    pub fn new(
        fs: Box<dyn FileSystem>,
        projects: Box<dyn ProjectLoader>,
        localizer: Localizer,
    ) -> Self {
        Self {
            fs,
            projects,
            localizer,
        }
    }
}
