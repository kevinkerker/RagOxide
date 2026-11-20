//! German compound splitting with the word list of the hyphenation project
//! (`wortliste`, MIT licence, ADR 0130).
//!
//! The package carries a derived, compressed list (`data/wortliste.txt.zst`): one
//! line per entry, lower case, compound boundaries kept as runs of `=` whose length is
//! the boundary level (`kranken=haus==betten`). [`derive()`] builds it from the
//! original list; the `build_wordlist` example writes the file.
//!
//! Known words are split at every boundary level, so `mehr=wert==steuer===satz`
//! yields `mehr`, `wert`, `steuer`, `satz`, `mehrwert` and `mehrwertsteuer`. Prefix
//! and suffix boundaries of the original list are not split. Unknown words with at
//! least [`MIN_FALLBACK_CHARS`] characters are split into as few words of the list
//! as possible, allowing the linking elements in [`LINKS`]. Parts shorter than
//! [`MIN_PART_CHARS`] characters are dropped.

use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, OnceLock};

/// The derived word list, compressed with zstd.
const DERIVED: &[u8] = include_bytes!("../data/wortliste.txt.zst");

/// Parts shorter than this many characters are neither emitted nor used.
pub const MIN_PART_CHARS: usize = 3;
/// Unknown words shorter than this many characters are not split.
pub const MIN_FALLBACK_CHARS: usize = 8;
/// Linking elements tried between two parts of an unknown word.
pub const LINKS: [&str; 8] = ["", "s", "es", "n", "en", "e", "er", "ens"];

/// Builds the derived list from the text of the original `wortliste`.
///
/// Each entry becomes its lower-case word; if the last spelling field marks
/// compound boundaries with `=` and its letters equal the word, the boundaries are
/// kept. Comments, words shorter than [`MIN_PART_CHARS`] characters and duplicate
/// lines are dropped; the lines are sorted and end with `\n`.
pub fn derive(source: &str) -> String {
    let mut lines = std::collections::BTreeSet::new();
    for line in source.lines() {
        let line = line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        let mut fields = line.split(';');
        let word = fields.next().unwrap_or("").trim().to_lowercase();
        if word.chars().count() < MIN_PART_CHARS || word.contains('=') {
            continue;
        }
        let spelling = fields.rfind(|f| !is_placeholder(f)).unwrap_or("");
        let marked: String = spelling
            .chars()
            .filter(|c| c.is_alphabetic() || *c == '=')
            .flat_map(char::to_lowercase)
            .collect();
        let letters: String = marked.chars().filter(|c| *c != '=').collect();
        if marked.contains('=') && letters == word {
            lines.insert(marked.trim_matches('=').to_string());
        } else {
            lines.insert(word);
        }
    }
    let mut out = String::new();
    for l in lines {
        out.push_str(&l);
        out.push('\n');
    }
    out
}

fn is_placeholder(field: &str) -> bool {
    let f = field.trim();
    f.len() >= 3
        && f.starts_with('-')
        && f.ends_with('-')
        && f[1..f.len() - 1].chars().all(|c| c.is_ascii_digit())
}

/// Parts of a marked entry at every boundary level, without the whole word.
fn parts_of(marked: &str) -> (String, Vec<String>) {
    let mut segments: Vec<(String, usize)> = Vec::new();
    let mut current = String::new();
    let mut run = 0;
    for c in marked.chars() {
        if c == '=' {
            run += 1;
            continue;
        }
        if run > 0 {
            segments.push((std::mem::take(&mut current), run));
            run = 0;
        }
        current.push(c);
    }
    segments.push((current, 0));
    let word: String = segments.iter().map(|(s, _)| s.as_str()).collect();
    let max_level = segments.iter().map(|(_, l)| *l).max().unwrap_or(0);
    let mut parts = Vec::new();
    for level in 1..=max_level {
        let mut part = String::new();
        for (segment, boundary) in &segments {
            part.push_str(segment);
            if *boundary >= level || *boundary == 0 {
                if part != word && part.chars().count() >= MIN_PART_CHARS && !parts.contains(&part)
                {
                    parts.push(part.clone());
                }
                part.clear();
            }
        }
    }
    (word, parts)
}

/// Splits lower-case German words into their compound parts.
pub struct Decompounder {
    known: HashMap<String, Vec<String>>,
    lexicon: HashSet<String>,
    cache: Mutex<HashMap<String, Vec<String>>>,
    cache_limit: usize,
}

impl Decompounder {
    /// Parses a derived list as produced by [`derive()`].
    pub fn from_derived(text: &str) -> Self {
        let mut known: HashMap<String, Vec<String>> = HashMap::new();
        let mut lexicon = HashSet::new();
        for line in text.lines().filter(|l| !l.is_empty()) {
            let (word, parts) = parts_of(line);
            if !parts.is_empty() {
                let entry = known.entry(word.clone()).or_default();
                for p in parts {
                    if !entry.contains(&p) {
                        entry.push(p);
                    }
                }
            }
            lexicon.insert(word);
        }
        Self {
            known,
            lexicon,
            cache: Mutex::new(HashMap::new()),
            cache_limit: crate::analysis::CACHE_LIMIT,
        }
    }

    /// The decompounder of the embedded list, parsed on first use.
    pub fn embedded() -> &'static Decompounder {
        static LIST: OnceLock<Decompounder> = OnceLock::new();
        LIST.get_or_init(|| {
            let bytes = zstd::decode_all(DERIVED).expect("embedded word list is valid zstd");
            let text = String::from_utf8(bytes).expect("embedded word list is UTF-8");
            Decompounder::from_derived(&text)
        })
    }

    /// Number of words in the list.
    pub fn len(&self) -> usize {
        self.lexicon.len()
    }

    /// Whether the list is empty.
    pub fn is_empty(&self) -> bool {
        self.lexicon.is_empty()
    }

    /// The parts of a lower-case word, without the word itself; empty if it is not
    /// a compound.
    pub fn parts(&self, word: &str) -> Vec<String> {
        if let Some(p) = self.known.get(word) {
            return p.clone();
        }
        if self.lexicon.contains(word) || word.chars().count() < MIN_FALLBACK_CHARS {
            return Vec::new();
        }
        if let Some(p) = self
            .cache
            .lock()
            .map(|c| c.get(word).cloned())
            .ok()
            .flatten()
        {
            return p;
        }
        let chars: Vec<char> = word.chars().collect();
        let mut memo = HashMap::new();
        let mut out = Vec::new();
        if let Some(split) = self.split(&chars, 0, &mut memo)
            && split.len() > 1
        {
            for part in split {
                // A part that is itself a known compound adds its own parts.
                let sub = self.known.get(&part).cloned().unwrap_or_default();
                for p in std::iter::once(part).chain(sub) {
                    if !out.contains(&p) {
                        out.push(p);
                    }
                }
            }
        }
        if let Ok(mut cache) = self.cache.lock() {
            if cache.len() >= self.cache_limit {
                cache.clear();
            }
            cache.insert(word.to_string(), out.clone());
        }
        out
    }

    /// Fewest-parts split of `chars[start..]` into words of the list; longer first
    /// parts win ties.
    fn split(
        &self,
        chars: &[char],
        start: usize,
        memo: &mut HashMap<usize, Option<Vec<String>>>,
    ) -> Option<Vec<String>> {
        if let Some(r) = memo.get(&start) {
            return r.clone();
        }
        let rest: String = chars[start..].iter().collect();
        let mut best: Option<Vec<String>> = None;
        if self.lexicon.contains(&rest) {
            best = Some(vec![rest]);
        } else {
            let len = chars.len();
            for end in (start + MIN_PART_CHARS..=len.saturating_sub(MIN_PART_CHARS)).rev() {
                let head: String = chars[start..end].iter().collect();
                if !self.lexicon.contains(&head) {
                    continue;
                }
                for link in LINKS {
                    let next = end + link.chars().count();
                    if next + MIN_PART_CHARS > len {
                        continue;
                    }
                    let linked: String = chars[end..next].iter().collect();
                    if linked != link {
                        continue;
                    }
                    if let Some(tail) = self.split(chars, next, memo) {
                        let mut candidate = vec![head.clone()];
                        candidate.extend(tail);
                        if best.as_ref().is_none_or(|b| candidate.len() < b.len()) {
                            best = Some(candidate);
                        }
                    }
                }
            }
        }
        memo.insert(start, best.clone());
        best
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derive_keeps_boundaries_and_drops_prefix_marks() {
        let source = "# comment\nAal;Aal\nAuswahl;Aus<wahl\nMehrwertsteuersatz;Mehr=wert==steu-er===satz\nab;ab\nAalstrasse;-2-;-3-;-4-;-5-;Aal=stra-sse;Aal=stras-se;Aal=stras-se\n";
        assert_eq!(
            derive(source),
            "aal\naal=strasse\nauswahl\nmehr=wert==steuer===satz\n"
        );
    }

    #[test]
    fn known_words_split_on_every_level() {
        let d = Decompounder::from_derived("mehr=wert==steuer===satz\nauswahl\n");
        let p = d.parts("mehrwertsteuersatz");
        for want in [
            "mehr",
            "wert",
            "steuer",
            "satz",
            "mehrwert",
            "mehrwertsteuer",
        ] {
            assert!(p.contains(&want.to_string()), "{want} in {p:?}");
        }
        assert!(d.parts("auswahl").is_empty());
    }

    #[test]
    fn the_split_cache_is_emptied_at_its_limit() {
        let mut d = Decompounder::from_derived("nutzung\nvertrag\nhaus\n");
        d.cache_limit = 5;
        for i in 0..50 {
            let _ = d.parts(&format!("unbekanntwort{i}"));
        }
        assert!(d.cache.lock().unwrap().len() <= 5);
        assert_eq!(d.parts("nutzungsvertrag"), vec!["nutzung", "vertrag"]);
    }

    #[test]
    fn unknown_words_split_into_list_words_with_links() {
        let d = Decompounder::from_derived("nutzung\nvertrag\nhaus\n");
        assert_eq!(d.parts("nutzungsvertrag"), vec!["nutzung", "vertrag"]);
        assert!(d.parts("haus").is_empty());
        assert!(d.parts("xyzhausq").is_empty());
    }
}
