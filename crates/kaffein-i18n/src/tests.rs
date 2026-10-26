//! Catalog completeness, fallback, formatting, and locale-switch tests.

use std::sync::{Arc, Mutex};

use crate::{keys, Date, Locale, LocaleError, LocaleFile, LocaleSource, Localizer, Text};

fn localizer_for(code: &str) -> Localizer {
    let mut localizer = Localizer::bundled();
    localizer.set_locale(code).expect("bundled locale");
    localizer
}

#[test]
fn every_declared_key_resolves_in_every_bundled_locale() {
    for locale in Localizer::bundled().available_locales() {
        let localizer = localizer_for(locale.code());
        for key in keys::ALL {
            let text = localizer.text(key);
            assert!(
                !text.is_empty(),
                "empty '{}' localization for '{key}'",
                locale.code()
            );
            assert_ne!(
                text,
                *key,
                "missing '{}' localization for '{key}'",
                locale.code()
            );
        }
    }
}

#[test]
fn english_is_the_default_and_portuguese_is_available() {
    let localizer = Localizer::default();

    assert_eq!(localizer.locale().code(), "en");
    assert_eq!(localizer.text(keys::MENU_FILE), "File");
    let locales = localizer.available_locales();
    assert!(locales.iter().any(|locale| locale.code() == "en"));
    assert!(locales.iter().any(|locale| locale.code() == "pt-BR"));
}

#[test]
fn placeholders_are_replaced() {
    let localizer = Localizer::english();

    let text = localizer.format(keys::MESSAGE_FILE_SAVED, &[("name", "App.java")]);

    assert_eq!(text, "Saved App.java");
}

#[test]
fn plural_families_select_the_counted_form_and_format_the_number() {
    let english = Localizer::english();
    let portuguese = localizer_for("pt-BR");

    assert_eq!(english.count(keys::STATUS_LINE_COUNT, 1), "1 line");
    assert_eq!(english.count(keys::STATUS_LINE_COUNT, 2), "2 lines");
    assert_eq!(english.count(keys::STATUS_LINE_COUNT, 1234), "1,234 lines");
    assert_eq!(
        portuguese.count(keys::STATUS_LINE_COUNT, 1234),
        "1.234 linhas"
    );
}

#[test]
fn numbers_and_dates_follow_the_catalog_metadata() {
    let english = Localizer::english();
    let portuguese = localizer_for("pt-BR");
    let date = Date::new(2026, 10, 26);

    assert_eq!(english.number(-1234567), "-1,234,567");
    assert_eq!(portuguese.number(-1234567), "-1.234.567");
    assert_eq!(english.decimal(1234.5, 2), "1,234.50");
    assert_eq!(portuguese.decimal(1234.5, 2), "1.234,50");
    assert_eq!(english.date(date), "10/26/2026");
    assert_eq!(portuguese.date(date), "26/10/2026");
    assert_eq!(Date::from_unix_seconds(0), Date::new(1970, 1, 1));
}

#[test]
fn locale_codes_are_validated() {
    assert!(Locale::is_valid_code("en"));
    assert!(Locale::is_valid_code("pt-BR"));
    assert!(Locale::is_valid_code("zh-Hans"));
    assert!(!Locale::is_valid_code(""));
    assert!(!Locale::is_valid_code("../en"));
    assert!(!Locale::is_valid_code("english"));
    assert!(!Locale::is_valid_code("en-"));
}

#[test]
fn text_resolves_at_render_time_and_follows_a_locale_change() {
    let mut localizer = Localizer::english();
    let text = Text::message(keys::MESSAGE_WORKSPACE_ERROR)
        .arg("kind", Text::key(keys::ERROR_KIND_USER_INPUT))
        .arg("message", Text::literal("boom"));

    assert_eq!(text.resolve(&localizer), "invalid input: boom");
    localizer.set_locale("pt-BR").expect("switch");
    assert_eq!(text.resolve(&localizer), "entrada inválida: boom");
}

#[test]
fn a_missing_key_falls_back_to_english() {
    let source = MemorySource::new(
        vec![LocaleFile {
            code: "xx".to_owned(),
            json: r#"{
                "meta": {"name": "Test", "decimal": ".", "group": ",", "date": "YYYY-MM-DD"},
                "app": {"title": "Custom"}
            }"#
            .to_owned(),
        }],
        Some("xx"),
    );
    let localizer = Localizer::with_source(Arc::new(source));

    assert_eq!(localizer.locale().code(), "xx");
    assert_eq!(localizer.text(keys::APP_TITLE), "Custom");
    assert_eq!(localizer.text(keys::MENU_FILE), "File");
}

#[test]
fn external_catalogs_override_embedded_ones_and_broken_files_are_ignored() {
    let source = MemorySource::new(
        vec![
            LocaleFile {
                code: "en".to_owned(),
                json: r#"{
                    "meta": {"name": "English", "decimal": ".", "group": ",", "date": "MM/DD/YYYY"},
                    "app": {"title": "Overridden"}
                }"#
                .to_owned(),
            },
            LocaleFile {
                code: "de".to_owned(),
                json: "not json".to_owned(),
            },
        ],
        None,
    );
    let localizer = Localizer::with_source(Arc::new(source));

    assert_eq!(localizer.text(keys::APP_TITLE), "Overridden");
    assert_eq!(localizer.text(keys::MENU_FILE), "File");
    assert!(!localizer
        .available_locales()
        .iter()
        .any(|locale| locale.code() == "de"));
}

#[test]
fn switching_a_locale_persists_and_is_restored() {
    let source = Arc::new(MemorySource::new(Vec::new(), None));
    let mut localizer = Localizer::with_source(source.clone());

    localizer.set_locale("pt-BR").expect("switch");

    assert_eq!(source.saved_locale(), Some("pt-BR".to_owned()));
    let reloaded = Localizer::with_source(source);
    assert_eq!(reloaded.locale().code(), "pt-BR");
    assert_eq!(reloaded.text(keys::MENU_FILE), "Arquivo");
}

#[test]
fn an_unknown_locale_is_rejected_without_changing_the_active_one() {
    let source = Arc::new(MemorySource::new(Vec::new(), None));
    let mut localizer = Localizer::with_source(source.clone());

    let error = localizer.set_locale("zz").expect_err("unknown locale");

    assert_eq!(error, LocaleError::Unknown("zz".to_owned()));
    assert_eq!(localizer.locale().code(), "en");
    assert_eq!(source.saved_locale(), None);
}

#[test]
fn a_persistence_failure_keeps_the_previous_locale() {
    let mut localizer = Localizer::with_source(Arc::new(FailingSource));

    let error = localizer
        .set_locale("pt-BR")
        .expect_err("persistence failure");

    assert_eq!(error, LocaleError::Persistence("denied".to_owned()));
    assert_eq!(localizer.locale().code(), "en");
}

#[derive(Default)]
struct MemorySource {
    files: Vec<LocaleFile>,
    preferred: Mutex<Option<String>>,
    saved: Mutex<Option<String>>,
}

impl MemorySource {
    fn new(files: Vec<LocaleFile>, preferred: Option<&str>) -> Self {
        Self {
            files,
            preferred: Mutex::new(preferred.map(str::to_owned)),
            saved: Mutex::new(None),
        }
    }

    fn saved_locale(&self) -> Option<String> {
        self.saved.lock().expect("saved locale").clone()
    }
}

impl LocaleSource for MemorySource {
    fn catalogs(&self) -> Result<Vec<LocaleFile>, String> {
        Ok(self.files.clone())
    }

    fn preferred_locale(&self) -> Option<String> {
        self.preferred.lock().expect("preferred locale").clone()
    }

    fn store_preferred_locale(&self, code: &str) -> Result<(), String> {
        *self.preferred.lock().expect("preferred locale") = Some(code.to_owned());
        *self.saved.lock().expect("saved locale") = Some(code.to_owned());
        Ok(())
    }
}

struct FailingSource;

impl LocaleSource for FailingSource {
    fn catalogs(&self) -> Result<Vec<LocaleFile>, String> {
        Ok(Vec::new())
    }

    fn preferred_locale(&self) -> Option<String> {
        None
    }

    fn store_preferred_locale(&self, _code: &str) -> Result<(), String> {
        Err("denied".to_owned())
    }
}
