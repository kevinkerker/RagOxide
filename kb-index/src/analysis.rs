//! The own text analysis in front of the full-text index (ADR 0129, ADR 0130).
//!
//! Text and query are normalised to NFC and split into words at every character
//! that is not alphanumeric; words are lower-cased, words of 40 bytes or more are
//! dropped. Then, by the chunk's language:
//!
//! - **German** (`de`): German stop words (Snowball list) are dropped; every word
//!   yields its Snowball stem, the stems of its compound parts ([`crate::wordlist`])
//!   and, for past participles of the form `ge…t`, the stem without `ge-` and `-t`
//!   (and without `-et`).
//! - **English** (`en`): English stop words (Snowball list) are dropped; every word
//!   yields its Snowball stem.
//! - **Default** (no language or any other language): the German and English stop
//!   words (both Snowball lists) are dropped; every other word yields itself, lower
//!   case, without stem or compound parts.
//!
//! Every token carries the prefix of its analysis (`de:`, `en:`, `xx:`), so the
//! analyses of different languages never meet in the token column. The query
//! carries no language; it is analysed with all three analyses and matches a chunk
//! through the analysis of the chunk's language.

use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

use lancedb::index::scalar::FtsIndexBuilder;
use unicode_normalization::UnicodeNormalization;

use crate::wordlist::Decompounder;

/// Byte length from which words are dropped (as LanceDB's own tokenizer does).
pub const TOKEN_LENGTH_LIMIT: usize = 40;

/// Entries a cache of the analysis holds at most; a full cache is emptied.
pub const CACHE_LIMIT: usize = 200_000;

/// The analysis chosen for a chunk.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Analysis {
    /// German: stop words, stems, compound parts, participles.
    German,
    /// English: stop words, stems.
    English,
    /// Default analyser: lower-case words without German and English stop words.
    Default,
}

impl Analysis {
    /// The analysis for a chunk's `language` (ISO 639-1, case and region ignored).
    pub fn for_language(language: Option<&str>) -> Self {
        let primary = language
            .map(|l| {
                l.split(['-', '_'])
                    .next()
                    .unwrap_or("")
                    .trim()
                    .to_ascii_lowercase()
            })
            .unwrap_or_default();
        match primary.as_str() {
            "de" => Analysis::German,
            "en" => Analysis::English,
            _ => Analysis::Default,
        }
    }

    fn prefix(self) -> &'static str {
        match self {
            Analysis::German => "de:",
            Analysis::English => "en:",
            Analysis::Default => "xx:",
        }
    }

    fn lance_language(self) -> Option<&'static str> {
        match self {
            Analysis::German => Some("German"),
            Analysis::English => Some("English"),
            Analysis::Default => None,
        }
    }
}

/// Lower-cased NFC words of `text`, in order, with repetitions.
pub fn words(text: &str) -> Vec<String> {
    let nfc: String = text.nfc().collect();
    nfc.split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(|w| w.to_lowercase())
        .filter(|w| w.len() < TOKEN_LENGTH_LIMIT)
        .collect()
}

/// Stemmer, stop-word lists and compound splitting, with caches.
pub struct Analyzer {
    decompounder: &'static Decompounder,
    stem_params: HashMap<Analysis, FtsIndexBuilder>,
    stop_params: HashMap<Analysis, FtsIndexBuilder>,
    stems: Mutex<HashMap<(Analysis, String), String>>,
    stops: Mutex<HashMap<(Analysis, String), bool>>,
    cache_limit: usize,
}

impl Analyzer {
    /// An analyzer with the embedded word list (parsed on first use).
    pub fn new() -> Self {
        Self::with_decompounder(Decompounder::embedded())
    }

    /// An analyzer with a given word list.
    pub fn with_decompounder(decompounder: &'static Decompounder) -> Self {
        let mut stem_params = HashMap::new();
        let mut stop_params = HashMap::new();
        for a in [Analysis::German, Analysis::English] {
            let language = a.lance_language().unwrap_or("English");
            let base = || {
                FtsIndexBuilder::default()
                    .base_tokenizer("raw".to_string())
                    .lower_case(true)
                    .ascii_folding(false)
                    .max_token_length(None)
                    .language(language)
                    .expect("built-in Snowball language")
            };
            stem_params.insert(a, base().stem(true).remove_stop_words(false));
            stop_params.insert(a, base().stem(false).remove_stop_words(true));
        }
        Self {
            decompounder,
            stem_params,
            stop_params,
            stems: Mutex::new(HashMap::new()),
            stops: Mutex::new(HashMap::new()),
            cache_limit: CACHE_LIMIT,
        }
    }

    /// Snowball stem of a lower-case word; the word itself for the default analysis.
    fn stem(&self, analysis: Analysis, word: &str) -> String {
        let Some(params) = self.stem_params.get(&analysis) else {
            return word.to_string();
        };
        let key = (analysis, word.to_string());
        if let Some(s) = self.stems.lock().ok().and_then(|c| c.get(&key).cloned()) {
            return s;
        }
        let stem = lancedb::tokenize(word, params)
            .ok()
            .and_then(|t| t.into_iter().next())
            .map(|t| t.text)
            .unwrap_or_else(|| word.to_string());
        if let Ok(mut c) = self.stems.lock() {
            if c.len() >= self.cache_limit {
                c.clear();
            }
            c.insert(key, stem.clone());
        }
        stem
    }

    /// Whether a lower-case word is a stop word of the analysis.
    fn is_stop_word(&self, analysis: Analysis, word: &str) -> bool {
        let Some(params) = self.stop_params.get(&analysis) else {
            return false;
        };
        let key = (analysis, word.to_string());
        if let Some(s) = self.stops.lock().ok().and_then(|c| c.get(&key).copied()) {
            return s;
        }
        let stop = lancedb::tokenize(word, params)
            .map(|t| t.is_empty())
            .unwrap_or(false);
        if let Ok(mut c) = self.stops.lock() {
            if c.len() >= self.cache_limit {
                c.clear();
            }
            c.insert(key, stop);
        }
        stop
    }

    /// Tokens of a word under one analysis, without prefix.
    fn word_tokens(&self, analysis: Analysis, word: &str, out: &mut Vec<String>) {
        let stop = match analysis {
            Analysis::Default => {
                self.is_stop_word(Analysis::German, word)
                    || self.is_stop_word(Analysis::English, word)
            }
            _ => self.is_stop_word(analysis, word),
        };
        if stop {
            return;
        }
        out.push(self.stem(analysis, word));
        if analysis != Analysis::German {
            return;
        }
        for part in self.decompounder.parts(word) {
            if !self.is_stop_word(analysis, &part) {
                out.push(self.stem(analysis, &part));
            }
        }
        out.extend(self.participle_stems(word));
    }

    /// `gewählt` -> stem of `wähl`; `geleitet` -> stems of `leite` and `leit`.
    fn participle_stems(&self, word: &str) -> Vec<String> {
        let chars: Vec<char> = word.chars().collect();
        if chars.len() < 6 || !word.starts_with("ge") || !word.ends_with('t') {
            return Vec::new();
        }
        let core: String = chars[2..chars.len() - 1].iter().collect();
        let mut out = vec![self.stem(Analysis::German, &core)];
        if let Some(shorter) = core.strip_suffix('e') {
            out.push(self.stem(Analysis::German, shorter));
        }
        out
    }

    /// Prefixed tokens of a chunk text under its analysis, in order, with repetitions.
    pub fn text_tokens(&self, text: &str, analysis: Analysis) -> Vec<String> {
        let mut raw = Vec::new();
        for w in words(text) {
            self.word_tokens(analysis, &w, &mut raw);
        }
        raw.into_iter()
            .map(|t| format!("{}{t}", analysis.prefix()))
            .collect()
    }

    /// Prefixed tokens of a query under all analyses, each token once, in order.
    pub fn query_tokens(&self, query: &str) -> Vec<String> {
        let mut seen = HashSet::new();
        let mut out = Vec::new();
        let words = words(query);
        for analysis in [Analysis::German, Analysis::English, Analysis::Default] {
            let mut raw = Vec::new();
            for w in &words {
                self.word_tokens(analysis, w, &mut raw);
            }
            for t in raw {
                let token = format!("{}{t}", analysis.prefix());
                if seen.insert(token.clone()) {
                    out.push(token);
                }
            }
        }
        out
    }
}

impl Default for Analyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn analyzer() -> Analyzer {
        static LIST: std::sync::OnceLock<Decompounder> = std::sync::OnceLock::new();
        let list = LIST.get_or_init(|| {
            Decompounder::from_derived("bundes=tag\nkanzler\nbundes=kanzler\nwahl\n")
        });
        Analyzer::with_decompounder(list)
    }

    #[test]
    fn caches_are_emptied_at_their_limit() {
        let mut a = analyzer();
        a.cache_limit = 10;
        let text: Vec<String> = (0..100).map(|i| format!("wort{i}")).collect();
        let tokens = a.text_tokens(&text.join(" "), Analysis::German);
        assert_eq!(tokens.len(), 100, "analysis still complete");
        assert!(a.stems.lock().unwrap().len() <= 10);
        assert!(a.stops.lock().unwrap().len() <= 10);
        assert_eq!(
            a.text_tokens("Bundeskanzler", Analysis::German),
            a.text_tokens("Bundeskanzler", Analysis::German)
        );
    }

    #[test]
    fn language_codes_choose_the_analysis() {
        assert_eq!(Analysis::for_language(Some("de")), Analysis::German);
        assert_eq!(Analysis::for_language(Some("DE-at")), Analysis::German);
        assert_eq!(Analysis::for_language(Some("en_GB")), Analysis::English);
        assert_eq!(Analysis::for_language(Some("fr")), Analysis::Default);
        assert_eq!(Analysis::for_language(None), Analysis::Default);
    }

    #[test]
    fn words_split_at_non_alphanumerics_and_lower_case() {
        assert_eq!(
            words("ÄRZTE, \"Bundestag\"-Wahl (2025)"),
            vec!["ärzte", "bundestag", "wahl", "2025"]
        );
        assert!(words("?!").is_empty());
    }

    #[test]
    fn german_tokens_drop_stop_words_and_add_parts_and_participles() {
        let a = analyzer();
        let t = a.text_tokens("der Bundeskanzler wurde gewählt", Analysis::German);
        assert!(!t.contains(&"de:der".to_string()), "{t:?}");
        assert!(
            t.contains(&"de:kanzl".to_string()) || t.iter().any(|x| x.starts_with("de:kanzl")),
            "{t:?}"
        );
        assert!(t.contains(&"de:wahl".to_string()), "{t:?}");
    }

    #[test]
    fn default_analysis_keeps_words() {
        let a = analyzer();
        assert_eq!(
            a.text_tokens("Le Parlement und the vote", Analysis::Default),
            vec!["xx:le", "xx:parlement", "xx:vote"]
        );
    }
}
