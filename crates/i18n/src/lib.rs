mod key;
mod language;
mod locales;

use std::fmt::Display;

pub use key::Key;
pub use language::Language;

pub fn tr(language: Language, key: Key) -> &'static str {
    match language {
        Language::En => locales::en::text(key),
        Language::Fr => locales::fr::text(key),
        Language::Es => locales::es::text(key),
        Language::Ru => locales::ru::text(key),
        Language::Ar => locales::ar::text(key),
        Language::Zh => locales::zh::text(key),
        Language::Hi => locales::hi::text(key),
    }
}

pub fn fill(template: &str, arguments: &[&dyn Display]) -> String {
    let mut output = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(start) = rest.find('{') {
        output.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        let argument = after
            .find('}')
            .and_then(|end| Some((after[..end].parse::<usize>().ok()?, end)))
            .and_then(|(index, end)| Some((arguments.get(index)?, end)));
        match argument {
            Some((argument, end)) => {
                output.push_str(&argument.to_string());
                rest = &after[end + 1..];
            }
            None => {
                output.push('{');
                rest = after;
            }
        }
    }
    output.push_str(rest);
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_language_translates_every_key() {
        for language in Language::ALL {
            for key in Key::ALL {
                assert!(!tr(language, *key).is_empty(), "{language:?} {key:?}");
            }
        }
    }

    #[test]
    fn placeholders_are_preserved_in_every_language() {
        for key in Key::ALL {
            let expected = tr(Language::En, *key).matches('{').count();
            for language in Language::ALL {
                let found = tr(language, *key).matches('{').count();
                assert_eq!(found, expected, "{language:?} {key:?}");
            }
        }
    }

    #[test]
    fn fills_positional_arguments() {
        assert_eq!(fill("{0} / {1}", &[&3, &"x"]), "3 / x");
    }

    #[test]
    fn arguments_are_never_reinterpreted_as_placeholders() {
        assert_eq!(fill("« {0} » {1}", &[&"{1}.mp4", &"ok"]), "« {1}.mp4 » ok");
    }

    #[test]
    fn unknown_or_malformed_placeholders_are_kept() {
        assert_eq!(fill("{x} {9} {", &[&1]), "{x} {9} {");
    }
}
