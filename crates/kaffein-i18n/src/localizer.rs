//! The localization manager: catalogs, locale selection, and formatting.

use std::collections::BTreeMap;
use std::fmt;
use std::sync::Arc;

use crate::catalog::{self, Catalog};
use crate::format::{self, Date};
use crate::locale::Locale;
use crate::source::LocaleSource;

const FALLBACK_CODE: &str = "en";

const BUNDLED_CATALOGS: &[(&str, &str)] = &[
    ("en", include_str!("../locales/en.json")),
    ("pt-BR", include_str!("../locales/pt-BR.json")),
];

/// A failure while changing the active locale.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LocaleError {
    /// No catalog is loaded for this code.
    Unknown(String),
    /// The catalog exists but the preference could not be written.
    Persistence(String),
}

impl fmt::Display for LocaleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unknown(code) => write!(formatter, "unknown locale '{code}'"),
            Self::Persistence(detail) => formatter.write_str(detail),
        }
    }
}

impl std::error::Error for LocaleError {}

/// Resolves localization keys against the active locale and its fallback.
#[derive(Clone)]
pub struct Localizer {
    catalogs: Arc<BTreeMap<String, Arc<Catalog>>>,
    active: String,
    fallback: String,
    source: Option<Arc<dyn LocaleSource>>,
}

impl Localizer {
    /// Loads the catalogs compiled into this build.
    pub fn bundled() -> Self {
        Self::from_catalogs(bundled_catalogs(), None, None)
    }

    /// The historical name for the default build, kept for callers and tests.
    pub fn english() -> Self {
        Self::bundled()
    }

    /// Loads embedded catalogs, then external overrides and additions from
    /// `source`. A preferred locale is used only when its catalog exists.
    pub fn with_source(source: Arc<dyn LocaleSource>) -> Self {
        let mut catalogs = bundled_catalogs();
        if let Ok(files) = source.catalogs() {
            for file in files {
                match catalog::parse(&file.code, &file.json) {
                    Ok(catalog) => {
                        let catalog = match catalogs.get(&file.code) {
                            Some(base) => Catalog::merge(base, catalog),
                            None => catalog,
                        };
                        catalogs.insert(file.code, Arc::new(catalog));
                    }
                    Err(_) => {
                        // A broken external catalog must not take the IDE down;
                        // the embedded catalog remains in effect.
                    }
                }
            }
        }
        let preferred = source.preferred_locale();
        Self::from_catalogs(catalogs, Some(source), preferred)
    }

    fn from_catalogs(
        catalogs: BTreeMap<String, Arc<Catalog>>,
        source: Option<Arc<dyn LocaleSource>>,
        preferred: Option<String>,
    ) -> Self {
        let fallback = if catalogs.contains_key(FALLBACK_CODE) {
            FALLBACK_CODE.to_owned()
        } else {
            catalogs
                .keys()
                .next()
                .cloned()
                .expect("at least one locale catalog must be bundled")
        };
        let active = preferred
            .filter(|code| catalogs.contains_key(code))
            .unwrap_or_else(|| fallback.clone());
        Self {
            catalogs: Arc::new(catalogs),
            active,
            fallback,
            source,
        }
    }

    /// The locale currently used for rendering.
    pub fn active_locale(&self) -> Locale {
        self.catalog(&self.active).meta().locale().clone()
    }

    /// Alias kept for code that reads the active language.
    pub fn locale(&self) -> Locale {
        self.active_locale()
    }

    /// Every loaded locale, ordered by language code.
    pub fn available_locales(&self) -> Vec<Locale> {
        self.catalogs
            .values()
            .map(|catalog| catalog.meta().locale().clone())
            .collect()
    }

    /// Switches the active locale and persists the choice when a source is
    /// configured. Persistence happens before the switch, so a failed write
    /// leaves the interface in its previous language.
    pub fn set_locale(&mut self, code: &str) -> Result<(), LocaleError> {
        if !self.catalogs.contains_key(code) {
            return Err(LocaleError::Unknown(code.to_owned()));
        }
        if let Some(source) = &self.source {
            source
                .store_preferred_locale(code)
                .map_err(LocaleError::Persistence)?;
        }
        self.active = code.to_owned();
        Ok(())
    }

    /// Returns the text for `key`, or the key itself when no entry exists.
    ///
    /// A missing key in the active locale falls back to the fallback locale.
    /// Missing in both is a programming error; debug builds assert so tests
    /// catch it, while release builds degrade to a visible key.
    pub fn text<'a>(&'a self, key: &'a str) -> &'a str {
        if let Some(text) = self.lookup(key) {
            return text;
        }
        debug_assert!(false, "missing localization key: {key}");
        key
    }

    /// Looks a key up in the active locale, then the fallback locale.
    fn lookup<'a>(&'a self, key: &str) -> Option<&'a str> {
        if let Some(text) = self.catalog(&self.active).text(key) {
            return Some(text);
        }
        self.catalog(&self.fallback).text(key)
    }

    /// Substitutes `{name}` placeholders in the catalog entry for `key`.
    pub fn format(&self, key: &str, arguments: &[(&str, &str)]) -> String {
        let mut text = self.text(key).to_owned();
        for (name, value) in arguments {
            text = text.replace(&format!("{{{name}}}"), value);
        }
        text
    }

    /// Renders a key with a plural count and resolved arguments. The count is
    /// formatted with the active locale's separators.
    pub fn render(&self, key: &str, plural: Option<usize>, arguments: &[(&str, String)]) -> String {
        let mut text = match plural {
            Some(count) => {
                let suffix = if count == 1 { "one" } else { "other" };
                let plural_key = format!("{key}.{suffix}");
                match self.lookup(&plural_key) {
                    Some(template) => template.to_owned(),
                    None => {
                        debug_assert!(false, "missing plural localization: {plural_key}");
                        plural_key
                    }
                }
            }
            None => self.text(key).to_owned(),
        };
        for (name, value) in arguments {
            text = text.replace(&format!("{{{name}}}"), value);
        }
        text
    }

    /// Selects the `one` or `other` form of a plural family.
    pub fn count(&self, key: &str, count: usize) -> String {
        self.render(key, Some(count), &[("count", self.number(count as i64))])
    }

    /// Formats an integer with the active locale's group separator.
    pub fn number(&self, value: i64) -> String {
        format::number(self.active_catalog().meta(), value)
    }

    /// Formats a decimal with the active locale's separators.
    pub fn decimal(&self, value: f64, fraction_digits: usize) -> String {
        format::decimal(self.active_catalog().meta(), value, fraction_digits)
    }

    /// Formats a date with the active locale's date pattern.
    pub fn date(&self, date: Date) -> String {
        format::date(self.active_catalog().meta(), date)
    }

    fn catalog(&self, code: &str) -> &Catalog {
        self.catalogs
            .get(code)
            .map(Arc::as_ref)
            .expect("active and fallback locales must be loaded")
    }

    fn active_catalog(&self) -> &Catalog {
        self.catalog(&self.active)
    }
}

impl fmt::Debug for Localizer {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Localizer")
            .field("active", &self.active)
            .field("fallback", &self.fallback)
            .field("catalogs", &self.catalogs.keys().collect::<Vec<_>>())
            .finish_non_exhaustive()
    }
}

impl Default for Localizer {
    fn default() -> Self {
        Self::bundled()
    }
}

fn bundled_catalogs() -> BTreeMap<String, Arc<Catalog>> {
    let mut catalogs = BTreeMap::new();
    for (code, source) in BUNDLED_CATALOGS {
        let catalog = catalog::parse(code, source)
            .unwrap_or_else(|error| panic!("invalid bundled '{code}' locale catalog: {error}"));
        catalogs.insert((*code).to_owned(), Arc::new(catalog));
    }
    catalogs
}
