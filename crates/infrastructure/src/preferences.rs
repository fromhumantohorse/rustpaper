use rustpaper_application::AppError;
use rustpaper_i18n::Language;
use serde::{Deserialize, Serialize};

use crate::paths::AppPaths;
use crate::persistence::JsonFile;

const PREFERENCES_FILE: &str = "preferences.json";

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Preferences {
    pub language: Option<Language>,
}

impl Preferences {
    pub fn effective_language(&self) -> Language {
        self.language
            .or_else(|| sys_locale::get_locale().and_then(|locale| Language::from_locale(&locale)))
            .unwrap_or_default()
    }
}

pub struct PreferencesStore {
    file: JsonFile,
}

impl PreferencesStore {
    pub fn new(paths: &AppPaths) -> Self {
        Self {
            file: JsonFile::new(paths.runtime_file(PREFERENCES_FILE)),
        }
    }

    pub fn load(&self) -> Preferences {
        self.file.read().unwrap_or_else(|error| {
            log::warn!("préférences illisibles, valeurs par défaut utilisées : {error}");
            Preferences::default()
        })
    }

    pub fn save(&self, preferences: &Preferences) -> Result<(), AppError> {
        self.file.write(preferences)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_language_wins_over_system_locale() {
        let preferences = Preferences {
            language: Some(Language::Hi),
        };
        assert_eq!(preferences.effective_language(), Language::Hi);
    }

    #[test]
    fn round_trips_through_disk() {
        let root = std::env::temp_dir().join(format!("rustpaper-prefs-{}", std::process::id()));
        let store = PreferencesStore::new(&AppPaths::rooted(&root));
        let preferences = Preferences {
            language: Some(Language::Ar),
        };
        store.save(&preferences).unwrap();
        let loaded = store.load();
        std::fs::remove_dir_all(root).unwrap();
        assert_eq!(loaded, preferences);
    }
}
