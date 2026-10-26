use kaffein_domain::{FileSystem, ProjectLoader};

/// Infrastructure implementations injected by the binary. Tests substitute
/// in-memory fakes at this seam.
pub struct Services {
    pub fs: Box<dyn FileSystem>,
    pub projects: Box<dyn ProjectLoader>,
}

impl Services {
    pub fn new(fs: Box<dyn FileSystem>, projects: Box<dyn ProjectLoader>) -> Self {
        Self { fs, projects }
    }
}
