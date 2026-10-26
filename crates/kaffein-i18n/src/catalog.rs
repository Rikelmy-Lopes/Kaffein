//! Parsing and flattening of a nested locale catalog.

use std::collections::HashMap;

use serde_json::{Map, Value};

use crate::locale::Locale;

/// One parsed locale file: catalog metadata plus flat dot-separated entries.
#[derive(Debug)]
pub(super) struct Catalog {
    meta: CatalogMeta,
    entries: HashMap<String, String>,
}

/// The formatting rules that differ between locales.
#[derive(Debug)]
pub(super) struct CatalogMeta {
    locale: Locale,
    decimal_separator: char,
    group_separator: char,
    date_pattern: String,
}

impl Catalog {
    pub(super) fn text(&self, key: &str) -> Option<&str> {
        self.entries.get(key).map(String::as_str)
    }

    pub(super) fn meta(&self) -> &CatalogMeta {
        &self.meta
    }

    /// Layers an external catalog over a bundled one, so a translation file can
    /// override individual entries without having to repeat the whole catalog.
    pub(super) fn merge(base: &Catalog, overlay: Catalog) -> Catalog {
        let mut entries = base.entries.clone();
        entries.extend(overlay.entries);
        Catalog {
            meta: overlay.meta,
            entries,
        }
    }
}

impl CatalogMeta {
    pub(super) fn locale(&self) -> &Locale {
        &self.locale
    }

    pub(super) fn decimal_separator(&self) -> char {
        self.decimal_separator
    }

    pub(super) fn group_separator(&self) -> char {
        self.group_separator
    }

    pub(super) fn date_pattern(&self) -> &str {
        &self.date_pattern
    }
}

/// Parses a nested JSON object into flat dot-separated keys.
pub(super) fn parse(code: &str, source: &str) -> Result<Catalog, String> {
    if !Locale::is_valid_code(code) {
        return Err(format!("invalid locale code '{code}'"));
    }

    let value: Value = serde_json::from_str(source).map_err(|error| error.to_string())?;
    let Value::Object(mut object) = value else {
        return Err("locale catalog must be a JSON object".to_owned());
    };

    let meta = object
        .remove("meta")
        .ok_or_else(|| "locale catalog needs a 'meta' object".to_owned())?;
    let meta = parse_meta(code, &meta)?;

    let mut entries = HashMap::new();
    for (key, value) in object {
        flatten(&key, &value, &mut entries)?;
    }
    if entries.is_empty() {
        return Err("locale catalog has no entries".to_owned());
    }

    Ok(Catalog { meta, entries })
}

fn parse_meta(code: &str, value: &Value) -> Result<CatalogMeta, String> {
    let Value::Object(object) = value else {
        return Err("'meta' must be a JSON object".to_owned());
    };
    let name = string_field(object, "name")?;
    let decimal_separator = char_field(object, "decimal")?;
    let group_separator = char_field(object, "group")?;
    let date_pattern = string_field(object, "date")?;
    for token in ["YYYY", "MM", "DD"] {
        if !date_pattern.contains(token) {
            return Err(format!("date pattern must contain '{token}'"));
        }
    }

    Ok(CatalogMeta {
        locale: Locale::new(code, name),
        decimal_separator,
        group_separator,
        date_pattern,
    })
}

fn string_field(object: &Map<String, Value>, name: &str) -> Result<String, String> {
    match object.get(name) {
        Some(Value::String(text)) if !text.is_empty() => Ok(text.clone()),
        _ => Err(format!("meta.{name} must be a non-empty string")),
    }
}

fn char_field(object: &Map<String, Value>, name: &str) -> Result<char, String> {
    let text = string_field(object, name)?;
    let mut chars = text.chars();
    match (chars.next(), chars.next()) {
        (Some(value), None) => Ok(value),
        _ => Err(format!("meta.{name} must be one character")),
    }
}

fn flatten(key: &str, value: &Value, catalog: &mut HashMap<String, String>) -> Result<(), String> {
    match value {
        Value::Object(object) => {
            for (child, value) in object {
                flatten(&format!("{key}.{child}"), value, catalog)?;
            }
            Ok(())
        }
        Value::String(text) => {
            if text.is_empty() {
                return Err(format!("locale entry '{key}' is empty"));
            }
            catalog.insert(key.to_owned(), text.clone());
            Ok(())
        }
        _ => Err(format!("locale entry '{key}' must be a string")),
    }
}
