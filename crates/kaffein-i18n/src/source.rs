//! The seam infrastructure uses to load and persist locale catalogs.

/// One catalog read from an external source.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocaleFile {
    pub code: String,
    pub json: String,
}

/// External catalog storage and the preferred-locale setting.
///
/// The localization crate ships embedded catalogs and treats these files as
/// overrides or additions; a missing directory is not an error.
pub trait LocaleSource: Send + Sync {
    fn catalogs(&self) -> Result<Vec<LocaleFile>, String>;
    fn preferred_locale(&self) -> Option<String>;
    fn store_preferred_locale(&self, code: &str) -> Result<(), String>;
}
