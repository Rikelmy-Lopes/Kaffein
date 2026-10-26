//! External locale catalogs and the persisted language preference.

use std::ffi::OsStr;
use std::path::PathBuf;

use kaffein_i18n::{Locale, LocaleFile, LocaleSource};

/// Largest external catalog accepted. Catalogs are small; the cap keeps a
/// stray or hostile file from consuming the UI thread's memory budget.
pub const MAX_LOCALE_BYTES: u64 = 256 * 1024;

/// Reads `*.json` catalogs from a directory and stores the chosen locale in a
/// settings file.
#[derive(Clone, Debug)]
pub struct FileLocaleSource {
    locales: PathBuf,
    settings: PathBuf,
}

impl FileLocaleSource {
    pub fn new(locales: impl Into<PathBuf>, settings: impl Into<PathBuf>) -> Self {
        Self {
            locales: locales.into(),
            settings: settings.into(),
        }
    }

    /// The user-configuration location used by the desktop binary.
    ///
    /// `KAFFEIN_CONFIG_DIR` overrides it, which keeps tests and portable
    /// installs out of the real profile.
    pub fn discover() -> Self {
        let root = config_root();
        Self::new(root.join("locales"), root.join("settings.json"))
    }
}

impl LocaleSource for FileLocaleSource {
    fn catalogs(&self) -> Result<Vec<LocaleFile>, String> {
        let entries = match std::fs::read_dir(&self.locales) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => return Err(format!("{}: {error}", self.locales.display())),
        };

        let mut files = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(OsStr::to_str) != Some("json") {
                continue;
            }
            let Some(code) = path.file_stem().and_then(OsStr::to_str) else {
                continue;
            };
            if !Locale::is_valid_code(code) {
                continue;
            }
            let Ok(metadata) = std::fs::metadata(&path) else {
                continue;
            };
            if metadata.len() > MAX_LOCALE_BYTES {
                continue;
            }
            let Ok(json) = std::fs::read_to_string(&path) else {
                continue;
            };
            files.push(LocaleFile {
                code: code.to_owned(),
                json,
            });
        }
        files.sort_by(|left, right| left.code.cmp(&right.code));
        Ok(files)
    }

    fn preferred_locale(&self) -> Option<String> {
        let source = std::fs::read_to_string(&self.settings).ok()?;
        let value: serde_json::Value = serde_json::from_str(&source).ok()?;
        value
            .get("locale")
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned)
    }

    fn store_preferred_locale(&self, code: &str) -> Result<(), String> {
        let mut settings = std::fs::read_to_string(&self.settings)
            .ok()
            .and_then(|source| serde_json::from_str::<serde_json::Value>(&source).ok())
            .filter(serde_json::Value::is_object)
            .unwrap_or_else(|| serde_json::json!({}));
        settings["locale"] = serde_json::Value::String(code.to_owned());

        if let Some(parent) = self.settings.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|error| format!("{}: {error}", parent.display()))?;
        }
        let text = serde_json::to_string_pretty(&settings).map_err(|error| error.to_string())?;
        std::fs::write(&self.settings, text)
            .map_err(|error| format!("{}: {error}", self.settings.display()))
    }
}

fn config_root() -> PathBuf {
    if let Some(path) = std::env::var_os("KAFFEIN_CONFIG_DIR") {
        return PathBuf::from(path);
    }
    #[cfg(windows)]
    if let Some(path) = std::env::var_os("APPDATA") {
        return PathBuf::from(path).join("Kaffein");
    }
    #[cfg(target_os = "macos")]
    if let Some(home) = std::env::var_os("HOME") {
        return PathBuf::from(home).join("Library/Application Support/Kaffein");
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        if let Some(path) = std::env::var_os("XDG_CONFIG_HOME") {
            return PathBuf::from(path).join("kaffein");
        }
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(home).join(".config/kaffein");
        }
    }
    PathBuf::from(".kaffein")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::TempDir;

    #[test]
    fn missing_locale_directory_is_not_an_error() {
        let dir = TempDir::new("locale-missing");
        let source =
            FileLocaleSource::new(dir.path().join("locales"), dir.path().join("settings.json"));

        assert_eq!(source.catalogs().expect("catalogs"), Vec::new());
        assert_eq!(source.preferred_locale(), None);
    }

    #[test]
    fn reads_valid_catalogs_and_ignores_invalid_names() {
        let dir = TempDir::new("locale-read");
        let locales = dir.path().join("locales");
        std::fs::create_dir_all(&locales).expect("locales");
        std::fs::write(locales.join("de.json"), "{}").expect("de");
        std::fs::write(locales.join("english.json"), "{}").expect("invalid code");
        std::fs::write(locales.join("notes.txt"), "ignored").expect("ignored");

        let source = FileLocaleSource::new(&locales, dir.path().join("settings.json"));
        let files = source.catalogs().expect("catalogs");

        assert_eq!(files.len(), 1);
        assert_eq!(files[0].code, "de");
    }

    #[test]
    fn ignores_catalogs_over_the_size_cap() {
        let dir = TempDir::new("locale-cap");
        let locales = dir.path().join("locales");
        std::fs::create_dir_all(&locales).expect("locales");
        let oversized = vec![b' '; MAX_LOCALE_BYTES as usize + 1];
        std::fs::write(locales.join("de.json"), &oversized).expect("oversized");
        std::fs::write(locales.join("en.json"), "{}").expect("en");

        let source = FileLocaleSource::new(&locales, dir.path().join("settings.json"));
        let files = source.catalogs().expect("catalogs");

        assert_eq!(files.len(), 1);
        assert_eq!(files[0].code, "en");
    }

    #[test]
    fn persists_the_preferred_locale_without_losing_other_settings() {
        let dir = TempDir::new("locale-persist");
        let settings = dir.path().join("settings.json");
        std::fs::write(&settings, r#"{"telemetry":false}"#).expect("settings");
        let source = FileLocaleSource::new(dir.path().join("locales"), &settings);

        source.store_preferred_locale("pt-BR").expect("store");

        let reloaded = FileLocaleSource::new(dir.path().join("locales"), &settings);
        assert_eq!(reloaded.preferred_locale().as_deref(), Some("pt-BR"));
        let saved: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&settings).expect("read")).expect("json");
        assert_eq!(saved["telemetry"], serde_json::json!(false));
    }
}
