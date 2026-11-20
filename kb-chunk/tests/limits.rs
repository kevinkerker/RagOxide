//! Token limit, no loss, sentence, word and character splits (B09, B21, B22, B24, B25).

mod common;

use std::sync::Arc;

use common::*;
use kb_core::mock::*;
use kb_core::*;

#[test]
fn b21_no_chunk_exceeds_the_limit_with_any_counter() {
    futures::executor::block_on(async {
        for (name, counter) in counters() {
            for limit in LIMITS {
                let chunker = chunker(&counter, limit);
                for document in all_documents() {
                    let chunks = chunker.chunk(&document).await.unwrap();
                    for chunk in &chunks {
                        let tokens = counter.count_tokens(&chunk.text).await.unwrap();
                        assert!(
                            tokens <= limit,
                            "{name}, {} at {limit}: {tokens} in {:?}",
                            document.source.name,
                            chunk.text
                        );
                    }
                }
            }
        }
    });
}

#[test]
fn b21_the_quadratic_counter_is_not_additive() {
    let counter = QuadraticCounter;
    let a = "eins zwei drei vier fünf";
    let b = "sechs sieben acht neun zehn";
    let joined = format!("{a}{SEPARATOR}{b}");
    assert!(count(&counter, &joined) > count(&counter, a) + count(&counter, b));
}

#[test]
fn b09_no_content_is_lost_or_doubled() {
    for limit in LIMITS {
        let chunker = mock_chunker(limit);
        for document in all_documents() {
            let chunks = chunk(&chunker, &document);
            check_no_loss(&document, &chunks);
        }
    }
}

#[test]
fn b24_long_text_is_split_at_sentence_ends_grundgesetz() {
    let articles = grundgesetz_articles();
    assert_eq!(
        articles.iter().map(|(t, _)| t.as_str()).collect::<Vec<_>>(),
        ["Art 17a", "Art 18", "Art 19"]
    );
    let text: String = articles
        .iter()
        .map(|(_, t)| t.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    let document = md(
        "grundgesetz.md",
        &[(heading(1), "Grundrechte"), (ElementKind::Text, &text)],
    );
    // The longest sentence (Art 17a (1)) has 73 words, so at 80 every sentence fits;
    // the whole text has 271 words.
    for limit in [80, 100, 120] {
        let chunks = chunk(&mock_chunker(limit), &document);
        assert!(chunks.len() >= 3, "{limit}");
        let joined: Vec<&str> = texts(&chunks);
        assert_eq!(
            joined.join(" "),
            text,
            "the pieces are the text, cut at spaces"
        );
        for piece in &joined[..joined.len() - 1] {
            assert!(piece.ends_with('.'), "{piece:?}");
            for not_an_end in ["Abs.", "Art.", "z. B.", "Nr."] {
                assert!(!piece.ends_with(not_an_end), "{piece:?}");
            }
            let last_word = piece.split_whitespace().last().unwrap();
            let digits = last_word.trim_end_matches('.');
            assert!(
                digits.is_empty() || !digits.chars().all(|c| c.is_ascii_digit()),
                "{piece:?}"
            );
        }
        for piece in &joined[1..] {
            let first = piece.chars().next().unwrap();
            assert!(first.is_uppercase() || first == '(', "{piece:?}");
        }
    }
}

#[test]
fn b24_sentence_ends_with_german_abbreviations() {
    let text = "Nach § 5 Abs. 2 Nr. 3 gilt z. B. dies. Der 21. Bundestag tagt am 7. September. \
                Gemäß Art. 5 bzw. Art. 8 gilt das. Dr. Müller prüft am 05.11.2026 alles. Ende.";
    let document = md("abk.md", &[(ElementKind::Text, text)]);
    let chunks = chunk(&mock_chunker(11), &document);
    assert_eq!(
        texts(&chunks),
        [
            "Nach § 5 Abs. 2 Nr. 3 gilt z. B. dies.",
            "Der 21. Bundestag tagt am 7. September.",
            "Gemäß Art. 5 bzw. Art. 8 gilt das.",
            "Dr. Müller prüft am 05.11.2026 alles. Ende.",
        ]
    );
}

#[test]
fn b25_too_long_sentences_are_split_at_words() {
    let sentence = "eins zwei drei vier fünf sechs sieben acht neun zehn elf zwölf.";
    let document = md("satz.md", &[(ElementKind::Text, sentence)]);
    let chunks = chunk(&mock_chunker(5), &document);
    assert_eq!(
        texts(&chunks),
        [
            "eins zwei drei vier fünf",
            "sechs sieben acht neun zehn",
            "elf zwölf."
        ]
    );
}

#[test]
fn b25_too_long_words_are_split_at_characters() {
    let word = "Grundstücksverkehrsgenehmigungszuständigkeitsübertragungsverordnung€😀ß";
    let document = md("wort.md", &[(ElementKind::Text, word)]);
    let counter: Arc<dyn TokenCounter> = Arc::new(CharCounter);
    for limit in [1, 2, 5, 7, 20] {
        let chunks = chunk(&chunker(&counter, limit), &document);
        assert!(chunks.len() > 1);
        assert_eq!(texts(&chunks).concat(), word);
        for chunk in &chunks {
            assert!(chunk.text.chars().count() <= limit);
        }
        // As many characters as fit: all pieces but the last are full.
        for chunk in &chunks[..chunks.len() - 1] {
            assert_eq!(chunk.text.chars().count(), limit);
        }
    }
    // A word among others: the words around it stay whole.
    let document = md(
        "worte.md",
        &[(
            ElementKind::Text,
            "kurz Grundstücksverkehrsgenehmigung kurz",
        )],
    );
    let chunks = chunk(&chunker(&counter, 10), &document);
    assert_eq!(
        texts(&chunks),
        ["kurz", "Grundstück", "sverkehrsg", "enehmigung", "kurz"]
    );
}

/// Counts every character as two tokens, so no character fits a limit of 1.
struct DoubleCharCounter;

#[async_trait]
impl TokenCounter for DoubleCharCounter {
    async fn count_tokens(&self, text: &str) -> Result<usize> {
        Ok(2 * text.chars().count())
    }
}

#[test]
fn b25_a_character_over_the_limit_is_an_error_not_a_loop() {
    let document = md("zeichen.md", &[(ElementKind::Text, "ab")]);
    let chunker = kb_chunk::DocumentChunker::new(Arc::new(DoubleCharCounter), 1);
    let error = try_chunk(&chunker, &document).unwrap_err();
    assert!(matches!(error, Error::Chunk(_)), "{error}");
}

#[test]
fn b22_separators_line_breaks_and_markdown_count() {
    // Separator: see b30 in structure.rs. Table Markdown with the mock counter: the
    // header line, the separator line and a row are 7 tokens each, so a table of two
    // rows (28 tokens) does not fit into 27.
    let document = &fixtures::documents()["budget.xlsx"];
    let table = &document.elements[0];
    assert_eq!(count(&MockTokenCounter, &table.text), 28);
    let whole = |limit: usize| {
        chunk(&mock_chunker(limit), document)
            .iter()
            .any(|c| c.text == table.text)
    };
    assert!(whole(28));
    assert!(!whole(27));
}

#[test]
fn b51_an_element_exactly_at_the_limit_stays_one_piece() {
    // Split pieces are trimmed; an element that fits keeps its text unchanged, with the
    // whitespace at its edges.
    let text = "  eins zwei drei.  ";
    let document = md("genau.md", &[(ElementKind::Text, text)]);
    assert_eq!(texts(&chunk(&mock_chunker(3), &document)), [text]);
    let counter: Arc<dyn TokenCounter> = Arc::new(CharCounter);
    let exact = text.chars().count();
    assert_eq!(texts(&chunk(&chunker(&counter, exact), &document)), [text]);
    assert_eq!(
        texts(&chunk(&chunker(&counter, exact - 1), &document)),
        ["eins zwei drei."]
    );
}
