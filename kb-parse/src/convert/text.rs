//! Text helpers shared by all formats: NFC, whitespace, language.

use unicode_normalization::UnicodeNormalization;

/// Unicode NFC form of `text` (contract of `Element::text` and `TableCell::text`).
pub(crate) fn nfc(text: &str) -> String {
    text.nfc().collect()
}

/// NFC with every run of whitespace (line breaks included) collapsed to one space and
/// trimmed. Used for table cells, so that a cell is one line of the Markdown table, and
/// for paragraphs assembled from inline pieces.
pub(crate) fn collapse(text: &str) -> String {
    nfc(&text.split_whitespace().collect::<Vec<_>>().join(" "))
}

/// Function words that are frequent in German text and rare in English text.
const GERMAN: &[&str] = &[
    "der", "die", "das", "und", "ist", "nicht", "mit", "von", "den", "dem", "des", "ein", "eine",
    "einer", "zu", "im", "auf", "für", "sich", "auch", "als", "bei", "nach", "wird", "werden",
    "oder", "über", "sind", "wurde", "durch",
];

/// Function words that are frequent in English text and rare in German text.
const ENGLISH: &[&str] = &[
    "the", "and", "of", "to", "is", "that", "for", "with", "are", "this", "on", "be", "by", "as",
    "from", "or", "which", "an", "was", "were", "it", "not", "at", "have", "has",
];

/// Predominant language of `texts`, `"de"` or `"en"`, by counting frequent function
/// words of both languages; `None` if neither occurs or both occur equally often.
pub(crate) fn detect_language<'a>(texts: impl IntoIterator<Item = &'a str>) -> Option<String> {
    let (mut german, mut english) = (0_usize, 0_usize);
    for text in texts {
        for word in text
            .split(|c: char| !c.is_alphabetic())
            .filter(|w| !w.is_empty())
        {
            let word = word.to_lowercase();
            if GERMAN.contains(&word.as_str()) {
                german += 1;
            } else if ENGLISH.contains(&word.as_str()) {
                english += 1;
            }
        }
    }
    match german.cmp(&english) {
        std::cmp::Ordering::Greater => Some("de".to_string()),
        std::cmp::Ordering::Less => Some("en".to_string()),
        std::cmp::Ordering::Equal => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nfc_composes_decomposed_umlauts() {
        assert_eq!(nfc("Gro\u{0308}\u{00DF}e"), "Gr\u{00F6}\u{00DF}e");
    }

    #[test]
    fn collapse_joins_lines_and_trims() {
        assert_eq!(
            collapse("  Kranken-\nhäuser\n insgesamt "),
            "Kranken- häuser insgesamt"
        );
    }

    #[test]
    fn language_follows_the_function_words() {
        assert_eq!(
            detect_language(["Der Bundestag ist das Parlament und wird gewählt."]),
            Some("de".to_string())
        );
        assert_eq!(
            detect_language(["The table is split into rows and columns of the sheet."]),
            Some("en".to_string())
        );
        assert_eq!(detect_language(["12 345", ""]), None);
    }
}
