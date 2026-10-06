use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    #[default]
    En,
    Fr,
    Es,
    Ru,
    Ar,
    Zh,
    Hi,
}

impl Language {
    pub const ALL: [Self; 7] = [
        Self::En,
        Self::Fr,
        Self::Es,
        Self::Ru,
        Self::Ar,
        Self::Zh,
        Self::Hi,
    ];

    pub fn native_name(self) -> &'static str {
        match self {
            Self::En => "English",
            Self::Fr => "Français",
            Self::Es => "Español",
            Self::Ru => "Русский",
            Self::Ar => "العربية",
            Self::Zh => "中文",
            Self::Hi => "हिन्दी",
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Fr => "fr",
            Self::Es => "es",
            Self::Ru => "ru",
            Self::Ar => "ar",
            Self::Zh => "zh",
            Self::Hi => "hi",
        }
    }

    pub fn is_right_to_left(self) -> bool {
        self == Self::Ar
    }

    pub fn from_locale(tag: &str) -> Option<Self> {
        let primary = tag.split(['-', '_', '.']).next()?.to_ascii_lowercase();
        Self::ALL
            .into_iter()
            .find(|language| language.code() == primary)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_system_locales() {
        assert_eq!(Language::from_locale("fr-FR"), Some(Language::Fr));
        assert_eq!(Language::from_locale("zh_Hans_CN"), Some(Language::Zh));
        assert_eq!(Language::from_locale("ar"), Some(Language::Ar));
        assert_eq!(Language::from_locale("de-DE"), None);
    }

    #[test]
    fn serializes_as_iso_code() {
        assert_eq!(serde_json::to_string(&Language::Hi).unwrap(), "\"hi\"");
    }
}
