//! Localised user-facing text for Kaffein.
//!
//! All interface copy lives in the JSON catalogs under `locales/`. Code names
//! a stable key from [`keys`]; it never embeds English text. Catalogs are also
//! loaded from an external [`LocaleSource`] at runtime, so a new language is a
//! JSON file that appears in the generated language menu, not a code change.
//!
//! Module layout:
//!
//! - `keys` — stable identifiers for every catalog entry,
//! - `locale` — language tags and their validation,
//! - `catalog` — parsing flattened entries and formatting metadata,
//! - `localizer` — active/fallback lookup, switching, and formatting,
//! - `source` — the seam infrastructure implements for external files,
//! - `text` — deferred text that re-renders when the locale changes,
//! - `format` — culture-specific number and date formatting.

pub mod keys;

mod catalog;
mod format;
mod locale;
mod localizer;
mod source;
mod text;

#[cfg(test)]
mod tests;

pub use format::Date;
pub use locale::Locale;
pub use localizer::{LocaleError, Localizer};
pub use source::{LocaleFile, LocaleSource};
pub use text::{Text, TextArgument, TextValue};
