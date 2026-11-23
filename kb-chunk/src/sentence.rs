//! Sentence boundaries of running text, with the German cases that are not one.

use std::ops::Range;

/// Characters that end a sentence.
const TERMINALS: [char; 4] = ['.', '!', '?', '…'];

/// Characters that may follow the end of a sentence before the space: closing quotes
/// and brackets.
const CLOSERS: [char; 10] = ['"', '\'', '“', '”', '‘', '’', '»', '«', ')', ']'];

/// Characters that may precede a word: opening quotes and brackets.
const OPENERS: [char; 10] = ['"', '\'', '„', '“', '‚', '‘', '»', '«', '(', '['];

/// Abbreviations, lower-cased and without their period, after which a period never
/// ends a sentence.
const ABBREVIATIONS: &[&str] = &[
    "abs", "abschn", "abt", "allg", "anl", "anm", "apr", "aufl", "aug", "bd", "bspw", "bzgl",
    "bzw", "ca", "dez", "dgl", "dipl", "dr", "ebd", "einschl", "entspr", "etc", "evtl", "feb",
    "ff", "gem", "geb", "ggf", "hrsg", "hs", "inkl", "insb", "ing", "jan", "jh", "jul", "jun",
    "kap", "lfd", "lt", "max", "min", "mio", "mrd", "mär", "nov", "nr", "nrn", "okt", "prof", "rd",
    "rn", "sep", "sept", "sog", "st", "std", "str", "tel", "tsd", "usw", "vgl", "vs", "zzgl",
    "ziff", "zit", "buchst", "halbs", "verf", "vorm", "approx", "dept", "fig", "mr", "mrs", "ms",
];

/// Abbreviations that are also words: their period ends no sentence only before a
/// number ("Art. 5", but "eine andere Art. Sie").
const BEFORE_NUMBER: &[&str] = &["art", "no"];

/// Byte ranges of the sentences of `text`, in order, without surrounding whitespace.
/// Together they cover every non-whitespace character of `text`.
///
/// A sentence ends at `.`, `!`, `?` or `…` (several in a row count as one end), possibly
/// followed by closing quotes or brackets, when whitespace follows and the next
/// character is not a lower-case letter. A single `.` ends no sentence after an
/// abbreviation from the lists above, after a single letter ("z. B.", initials), after
/// a word with an inner period ("z.B.", "e.g."), and after a number of one to three
/// digits (ordinals and dates such as "21. Bundestag" or "7. September").
pub(crate) fn sentences(text: &str) -> Vec<Range<usize>> {
    let chars: Vec<(usize, char)> = text.char_indices().collect();
    let mut sentences = Vec::new();
    let mut start: Option<usize> = None;
    let mut i = 0;
    while i < chars.len() {
        let (position, c) = chars[i];
        if start.is_none() {
            if c.is_whitespace() {
                i += 1;
                continue;
            }
            start = Some(position);
        }
        if !TERMINALS.contains(&c) {
            i += 1;
            continue;
        }
        let mut j = i;
        while j < chars.len() && (TERMINALS.contains(&chars[j].1) || CLOSERS.contains(&chars[j].1))
        {
            j += 1;
        }
        let Some(&(end, after)) = chars.get(j) else {
            break;
        };
        if !after.is_whitespace() {
            i = j;
            continue;
        }
        let mut k = j;
        while k < chars.len() && chars[k].1.is_whitespace() {
            k += 1;
        }
        let Some(&(_, next)) = chars.get(k) else {
            break;
        };
        if let Some(s) = start
            && ends_sentence(text, position, &chars[i..j], next)
        {
            sentences.push(s..end);
            start = None;
        }
        i = k;
    }
    if let Some(s) = start {
        let end = text.trim_end().len();
        if end > s {
            sentences.push(s..end);
        }
    }
    sentences
}

/// Decides whether the terminal run `run`, starting at byte `position` and followed by
/// whitespace and `next`, ends a sentence.
fn ends_sentence(text: &str, position: usize, run: &[(usize, char)], next: char) -> bool {
    if next.is_lowercase() {
        return false;
    }
    let mut terminals = run
        .iter()
        .map(|(_, c)| *c)
        .filter(|c| TERMINALS.contains(c));
    if !(terminals.next() == Some('.') && terminals.next().is_none()) {
        return true;
    }
    let before = &text[..position];
    let word_start = before
        .char_indices()
        .rev()
        .find(|(_, c)| c.is_whitespace())
        .map_or(0, |(i, c)| i + c.len_utf8());
    let word = before[word_start..].trim_start_matches(OPENERS);
    if word.is_empty() {
        return true;
    }
    if word.contains('.') {
        return false;
    }
    let mut letters = word.chars();
    if letters.next().is_some_and(char::is_alphabetic) && letters.next().is_none() {
        return false;
    }
    if word.len() <= 3 && word.bytes().all(|b| b.is_ascii_digit()) {
        return false;
    }
    let lower = word.to_lowercase();
    if ABBREVIATIONS.contains(&lower.as_str()) {
        return false;
    }
    !(BEFORE_NUMBER.contains(&lower.as_str()) && next.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::sentences;

    fn split(text: &str) -> Vec<&str> {
        sentences(text).into_iter().map(|r| &text[r]).collect()
    }

    #[test]
    fn plain_sentences() {
        assert_eq!(
            split("Erster Satz. Zweiter Satz! Dritter Satz? Vierter"),
            ["Erster Satz.", "Zweiter Satz!", "Dritter Satz?", "Vierter"]
        );
        assert_eq!(split("  Nur ein Satz.  "), ["Nur ein Satz."]);
        assert!(split("   ").is_empty());
    }

    #[test]
    fn german_cases_that_are_no_sentence_end() {
        let cases = [
            "Kühlgeräte, z. B. Gefriertruhen, werden geprüft.",
            "Nach § 5 Abs. 2 Nr. 3 gilt das.",
            "Gemäß Art. 5 Satz 1 bzw. Satz 2 gilt das.",
            "Der 21. Deutsche Bundestag trat am 25. März 2025 zusammen.",
            "Am 7. September 1949 trat der 1. Bundestag zusammen.",
            "Das gilt vgl. Kap. 3 auch hier.",
            "Siehe z.B. den Anhang.",
            "Dr. Müller und Prof. Schmidt sind da.",
            "Die Frist endet am 05.11.2026 um 12 Uhr.",
            "Er sagte: „Wir prüfen. dann gehen wir.“",
            "Rund 3,5 Mio. Euro sind eingeplant.",
        ];
        for case in cases {
            assert_eq!(split(case), [case], "{case}");
        }
    }

    #[test]
    fn sentence_ends_after_words_numbers_and_quotes() {
        assert_eq!(
            split("Das ist eine besondere Art. Sie ist selten."),
            ["Das ist eine besondere Art.", "Sie ist selten."]
        );
        assert_eq!(
            split("Die Wahl war 2025. Danach kam der Bundestag zusammen."),
            ["Die Wahl war 2025.", "Danach kam der Bundestag zusammen."]
        );
        assert_eq!(
            split("Er sagte „Ja.“ Dann ging er. (2) Neuer Absatz."),
            ["Er sagte „Ja.“", "Dann ging er.", "(2) Neuer Absatz."]
        );
        assert_eq!(
            split("Warten... Dann weiter."),
            ["Warten...", "Dann weiter."]
        );
    }
}
