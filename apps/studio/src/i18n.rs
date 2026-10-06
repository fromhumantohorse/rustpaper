use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::Display;

use rustpaper_application::AppError;
use rustpaper_domain::DomainError;
use rustpaper_i18n::{fill, tr, Key, Language};
use unicode_bidi::BidiInfo;

const ARABIC_RANGES: [(char, char); 5] = [
    ('\u{0600}', '\u{06FF}'),
    ('\u{0750}', '\u{077F}'),
    ('\u{08A0}', '\u{08FF}'),
    ('\u{FB50}', '\u{FDFF}'),
    ('\u{FE70}', '\u{FEFF}'),
];

pub struct Translator {
    language: Language,
    texts: HashMap<Key, String>,
}

impl Translator {
    pub fn new(language: Language) -> Self {
        let texts = Key::ALL
            .iter()
            .map(|key| (*key, display(tr(language, *key)).into_owned()))
            .collect();
        Self { language, texts }
    }

    pub fn language(&self) -> Language {
        self.language
    }

    pub fn get(&self, key: Key) -> &str {
        self.texts.get(&key).map_or("", String::as_str)
    }

    pub fn format(&self, key: Key, arguments: &[&dyn Display]) -> String {
        display(&fill(tr(self.language, key), arguments)).into_owned()
    }

    pub fn error(&self, error: &AppError) -> String {
        let (key, detail): (Key, &dyn Display) = match error {
            AppError::Domain(DomainError::UnsupportedMedia(media)) => (Key::ErrUnsupported, media),
            AppError::Domain(DomainError::MissingFile(path)) => (Key::ErrMissing, path),
            AppError::Domain(DomainError::OutOfRange { field, .. }) => (Key::ErrRange, field),
            AppError::Domain(DomainError::WallpaperNotFound(id)) => (Key::ErrNotFound, id),
            AppError::Domain(DomainError::AlreadyImported(path)) => (Key::ErrDuplicate, path),
            AppError::Storage(detail) => (Key::ErrStorage, detail),
            AppError::Platform(detail) => (Key::ErrSystem, detail),
            AppError::Renderer(detail) => (Key::ErrRenderer, detail),
        };
        self.format(key, &[detail])
    }
}

pub fn display(text: &str) -> Cow<'_, str> {
    if !text.chars().any(is_arabic) {
        return Cow::Borrowed(text);
    }
    Cow::Owned(
        text.split('\n')
            .map(right_to_left_line)
            .collect::<Vec<_>>()
            .join("\n"),
    )
}

fn is_arabic(character: char) -> bool {
    ARABIC_RANGES
        .iter()
        .any(|(first, last)| (*first..=*last).contains(&character))
}

fn mirrored(character: char) -> char {
    match character {
        '(' => ')',
        ')' => '(',
        '[' => ']',
        ']' => '[',
        '{' => '}',
        '}' => '{',
        '<' => '>',
        '>' => '<',
        '«' => '»',
        '»' => '«',
        other => other,
    }
}

fn mirror_right_to_left_brackets(text: &str) -> String {
    let bidi = BidiInfo::new(text, None);
    text.char_indices()
        .map(|(index, character)| {
            if bidi.levels[index].is_rtl() {
                mirrored(character)
            } else {
                character
            }
        })
        .collect()
}

fn right_to_left_line(line: &str) -> String {
    let shaped = mirror_right_to_left_brackets(&ar_reshaper::reshape_line(line));
    let bidi = BidiInfo::new(&shaped, None);
    bidi.paragraphs
        .iter()
        .map(|paragraph| {
            bidi.reorder_line(paragraph, paragraph.range.clone())
                .into_owned()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn latin_text_is_untouched() {
        assert!(matches!(display("Library"), Cow::Borrowed("Library")));
    }

    #[test]
    fn arabic_is_shaped_and_reversed_for_display() {
        let shown = display("المكتبة");
        assert_ne!(shown, "المكتبة");
        assert!(shown
            .chars()
            .all(|character| !('\u{0600}'..='\u{06FF}').contains(&character)));
    }

    #[test]
    fn brackets_keep_their_visual_orientation_in_arabic() {
        let shown = display("ابحث (Ctrl+F)");
        assert!(shown.contains("(Ctrl+F)"), "{shown}");
    }

    #[test]
    fn every_language_builds_a_complete_table() {
        for language in Language::ALL {
            let translator = Translator::new(language);
            assert!(Key::ALL.iter().all(|key| !translator.get(*key).is_empty()));
        }
    }
}
