//! Heading path, anchors, notes and merging (B05, B08, B12 to B18, B22, B27 to B30).

mod common;

use std::sync::Arc;

use common::*;
use kb_core::mock::*;
use kb_core::*;

#[test]
fn b05_headings_and_model_prefixes_never_appear_in_chunk_texts() {
    let document = md(
        "kopf.md",
        &[
            (heading(1), "Alphakopfwort"),
            (ElementKind::Text, "Erster Absatz unter dem Titel."),
            (heading(2), "Betakopfwort Zusatz"),
            (ElementKind::Text, "Zweiter Absatz. Er hat zwei Sätze."),
            (ElementKind::ListItem, "Ein Punkt der Liste."),
            (heading(3), "Gammakopfwort"),
            (ElementKind::Text, "Dritter Absatz ganz unten."),
            (heading(2), "Deltakopfwort"),
            (ElementKind::Caption, "Bildunterschrift am Ende."),
        ],
    );
    for (_, counter) in counters() {
        for limit in LIMITS {
            let chunks = chunk(&chunker(&counter, limit), &document);
            assert!(!chunks.is_empty());
            for chunk in &chunks {
                for word in [
                    "Alphakopfwort",
                    "Betakopfwort",
                    "Gammakopfwort",
                    "Deltakopfwort",
                ] {
                    assert!(!chunk.text.contains(word), "{word} in {:?}", chunk.text);
                }
                for prefix in ["title:", "task:", "query:", "text:"] {
                    assert!(!chunk.text.contains(prefix), "{prefix} in {:?}", chunk.text);
                }
            }
        }
    }
}

#[test]
fn b12_heading_path_follows_levels_1_2_3_2_1() {
    let document = md(
        "ebenen.md",
        &[
            (heading(1), "A"),
            (ElementKind::Text, "unter a"),
            (heading(2), "B"),
            (ElementKind::Text, "unter b"),
            (heading(3), "C"),
            (ElementKind::Text, "unter c"),
            (heading(2), "D"),
            (ElementKind::Text, "unter d"),
            (heading(1), "E"),
            (ElementKind::Text, "unter e"),
        ],
    );
    for limit in LIMITS {
        let chunks = chunk(&mock_chunker(limit), &document);
        let expected: Vec<(&str, Vec<&str>)> = vec![
            ("a", vec!["A"]),
            ("b", vec!["A", "B"]),
            ("c", vec!["A", "B", "C"]),
            ("d", vec!["A", "D"]),
            ("e", vec!["E"]),
        ];
        for (letter, path) in expected {
            for chunk in chunks.iter().filter(|c| c.text.ends_with(letter)) {
                assert_eq!(chunk.provenance.heading_path, path, "{letter} at {limit}");
            }
        }
        assert_eq!(chunks.iter().filter(|c| c.text.ends_with('d')).count(), 1);
    }
}

#[test]
fn b13_content_before_the_first_heading_has_an_empty_path() {
    let document = paragraphs(
        "seite.html",
        DocumentFormat::Html,
        &[
            (
                ElementKind::Text,
                "Einleitung der Seite vor jeder Überschrift.",
            ),
            (ElementKind::Text, "Zweiter Absatz der Einleitung."),
            (heading(1), "Erste Überschrift"),
            (ElementKind::Text, "Text danach."),
        ],
    );
    for limit in LIMITS {
        let chunks = chunk(&mock_chunker(limit), &document);
        for chunk in &chunks {
            let intro = chunk
                .provenance
                .anchors
                .iter()
                .all(|a| matches!(a, Anchor::Paragraph { element } if *element < 2));
            assert_eq!(
                chunk.provenance.heading_path.is_empty(),
                intro,
                "{:?}",
                chunk.text
            );
        }
        assert!(chunks.iter().any(|c| c.provenance.heading_path.is_empty()));
    }
}

#[test]
fn b14_headings_are_taken_as_the_parser_gives_them() {
    let in_brackets = md(
        "klammern.md",
        &[
            (heading(1), "Rede"),
            (heading(2), "(Beifall bei der Fraktion)"),
            (ElementKind::Text, "Weiter im Text."),
        ],
    );
    let chunks = chunk(&mock_chunker(400), &in_brackets);
    assert_eq!(
        chunks[0].provenance.heading_path,
        ["Rede", "(Beifall bei der Fraktion)"]
    );

    let only_level_1 = md(
        "flach.md",
        &[
            (heading(1), "Art 1"),
            (ElementKind::Text, "eins"),
            (heading(1), "Art 2"),
            (ElementKind::Text, "zwei"),
            (heading(1), "Art 3"),
            (ElementKind::Text, "drei"),
        ],
    );
    assert_eq!(
        paths(&chunk(&mock_chunker(400), &only_level_1)),
        [vec!["Art 1"], vec!["Art 2"], vec!["Art 3"]]
    );

    let jump = md(
        "sprung.md",
        &[
            (heading(1), "Eins"),
            (heading(3), "Drei"),
            (ElementKind::Text, "unter drei"),
            (heading(3), "Drei b"),
            (ElementKind::Text, "unter drei b"),
            (heading(2), "Zwei"),
            (ElementKind::Text, "unter zwei"),
        ],
    );
    assert_eq!(
        paths(&chunk(&mock_chunker(400), &jump)),
        [
            vec!["Eins", "Drei"],
            vec!["Eins", "Drei b"],
            vec!["Eins", "Zwei"]
        ]
    );

    let level_0 = md(
        "null.md",
        &[
            (heading(0), "Null"),
            (ElementKind::Text, "unter null"),
            (heading(2), "Zwei"),
            (ElementKind::Text, "unter zwei"),
            (heading(1), "Eins"),
            (ElementKind::Text, "unter eins"),
            (heading(0), "Wieder null"),
            (ElementKind::Text, "zuletzt"),
        ],
    );
    assert_eq!(
        paths(&chunk(&mock_chunker(400), &level_0)),
        [
            vec!["Null"],
            vec!["Null", "Zwei"],
            vec!["Eins"],
            vec!["Wieder null"]
        ]
    );

    let mut odd: Vec<(ElementKind, String)> = Vec::new();
    for (i, level) in [255_u8, 0, 7, 3, 1, 200, 2, 0, 9, 9, 1]
        .into_iter()
        .enumerate()
    {
        odd.push((heading(level), format!("H{i}")));
        odd.push((ElementKind::Text, format!("Text {i}.")));
    }
    let odd: Vec<(ElementKind, &str)> = odd.iter().map(|(k, t)| (k.clone(), t.as_str())).collect();
    for limit in LIMITS {
        let chunks = chunk(&mock_chunker(limit), &md("schief.md", &odd));
        assert!(chunks.iter().all(|c| !c.provenance.heading_path.is_empty()));
    }
}

#[test]
fn b16_pieces_of_a_split_element_keep_all_its_anchors() {
    let documents = fixtures::documents();
    let pdf = &documents["pruefordnung.pdf"];
    let scan = &documents["wartung-scan.png"];
    // The paragraph across the page break and the paragraph across two columns.
    for (document, split) in [(pdf, &pdf.elements[5]), (scan, &scan.elements[1])] {
        for (name, counter) in counters() {
            for limit in [1, 5, 20] {
                let pieces: Vec<Chunk> = chunk(&chunker(&counter, limit), document)
                    .into_iter()
                    .filter(|c| c.provenance.anchors.contains(&split.anchors[1]))
                    .collect();
                let dense = |text: &str| -> String {
                    text.chars().filter(|c| !c.is_whitespace()).collect()
                };
                let joined: String = pieces.iter().map(|c| dense(&c.text)).collect();
                assert_eq!(joined, dense(&split.text), "{name} at {limit}");
                if count(counter.as_ref(), &split.text) > limit {
                    assert!(pieces.len() > 1, "{name} at {limit}");
                }
                for piece in &pieces {
                    assert_eq!(piece.provenance.anchors, split.anchors, "{name} at {limit}");
                }
            }
        }
    }
}

fn notes_document() -> Document {
    document(
        "notizen.pptx",
        DocumentFormat::Pptx,
        vec![
            element(heading(1), "Vortrag", vec![slide(1, false)]),
            element(
                ElementKind::Text,
                "Inhalt der Folie eins.",
                vec![slide(1, false)],
            ),
            element(
                ElementKind::Text,
                "Notiz zu Folie eins.",
                vec![slide(1, true)],
            ),
            // Slide 2 is hidden: its notes are there, its content is not.
            element(
                ElementKind::Text,
                "Notiz zur ausgeblendeten Folie zwei.",
                vec![slide(2, true)],
            ),
            element(
                ElementKind::Text,
                "Inhalt der Folie drei.",
                vec![slide(3, false)],
            ),
        ],
    )
}

#[test]
fn b17_notes_keep_their_mark_and_stay_apart() {
    let notes = |chunks: &[Chunk], slide: u32| {
        chunks
            .iter()
            .filter(|c| {
                c.provenance.anchors.contains(&Anchor::Slide {
                    slide,
                    in_notes: true,
                })
            })
            .count()
    };
    for (_, counter) in counters() {
        for limit in LIMITS {
            let chunker = chunker(&counter, limit);
            let training = chunk(&chunker, &fixtures::documents()["schulung.pptx"]);
            assert!(notes(&training, 2) >= 1);
            let own = chunk(&chunker, &notes_document());
            assert!(notes(&own, 1) >= 1 && notes(&own, 2) >= 1, "at {limit}");
            for chunk in training.iter().chain(&own) {
                let flags: Vec<bool> = chunk
                    .provenance
                    .anchors
                    .iter()
                    .filter_map(|a| match a {
                        Anchor::Slide { in_notes, .. } => Some(*in_notes),
                        _ => None,
                    })
                    .collect();
                assert!(
                    flags.iter().all(|f| *f) || flags.iter().all(|f| !f),
                    "{flags:?}"
                );
            }
        }
    }
    let own = chunk(&mock_chunker(400), &notes_document());
    assert_eq!(
        texts(&own),
        [
            "Inhalt der Folie eins.",
            "Notiz zu Folie eins.\n\nNotiz zur ausgeblendeten Folie zwei.",
            "Inhalt der Folie drei.",
        ]
    );
    assert_eq!(own[1].provenance.anchors, [slide(1, true), slide(2, true)]);
}

#[test]
fn b18_footnotes_and_note_like_text_are_plain_running_text() {
    let word = paragraphs(
        "fussnote.docx",
        DocumentFormat::Docx,
        &[
            (heading(1), "Vertrag"),
            (ElementKind::Text, "Der Vertrag gilt ab sofort."),
            (
                ElementKind::Text,
                "1 Fußnote: Gemeint ist das Datum der Unterschrift.",
            ),
        ],
    );
    let chunks = chunk(&mock_chunker(400), &word);
    assert_eq!(chunks.len(), 1, "the footnote merges like a paragraph");
    let slides = document(
        "folie.pptx",
        DocumentFormat::Pptx,
        vec![
            element(
                ElementKind::Text,
                "Inhalt der Folie.",
                vec![slide(4, false)],
            ),
            element(
                ElementKind::Text,
                "Notiz: Das steht auf der Folie.",
                vec![slide(4, false)],
            ),
        ],
    );
    let chunks = chunk(&mock_chunker(400), &slides);
    assert_eq!(chunks.len(), 1, "no note is recognised by its text");
    assert_eq!(chunks[0].provenance.anchors, [slide(4, false)]);
}

#[test]
fn b22_the_heading_path_does_not_count() {
    let long_heading = "Sehr lange Überschrift ".repeat(200);
    let body = [
        (
            ElementKind::Text,
            "Ein Absatz mit genau acht Wörtern im Text.",
        ),
        (ElementKind::Text, "Noch ein Absatz mit sechs Wörtern."),
    ];
    for (_, counter) in counters() {
        for limit in LIMITS {
            let short = md(
                "kurz.md",
                &[(heading(1), "Kurz"), body[0].clone(), body[1].clone()],
            );
            let long = md(
                "lang.md",
                &[
                    (heading(1), &long_heading),
                    body[0].clone(),
                    body[1].clone(),
                ],
            );
            let chunker = chunker(&counter, limit);
            assert_eq!(
                texts(&chunk(&chunker, &short)),
                texts(&chunk(&chunker, &long))
            );
        }
    }
}

#[test]
fn b27_consecutive_whole_element_chunks_would_exceed_the_limit_joined() {
    // Every element fits at every limit used here, so every chunk consists of whole
    // elements; heading and notes boundaries are known from the documents.
    let mut items: Vec<(ElementKind, String)> = Vec::new();
    for section in 0..3 {
        items.push((heading(1), format!("Abschnitt {section}")));
        for i in 0..25 {
            let words = [
                "Kurz.",
                "Zwei Wörter.",
                "Hier sind drei.",
                "Und hier sind vier.",
            ];
            items.push((ElementKind::Text, words[(i + section) % 4].to_owned()));
        }
    }
    let items: Vec<(ElementKind, &str)> =
        items.iter().map(|(k, t)| (k.clone(), t.as_str())).collect();
    let document = md("viele.md", &items);
    for (name, counter) in counters() {
        let limits: &[usize] = match name {
            "chars" => &[20, 45, 100, 400],
            _ => &[5, 9, 20, 400],
        };
        for &limit in limits {
            let chunks = chunk(&chunker(&counter, limit), &document);
            for pair in chunks.windows(2) {
                if pair[0].provenance.heading_path != pair[1].provenance.heading_path {
                    continue;
                }
                let joined = format!("{}{SEPARATOR}{}", pair[0].text, pair[1].text);
                assert!(
                    count(counter.as_ref(), &joined) > limit,
                    "{name} at {limit}: {joined:?}"
                );
            }
            let merged = chunks.iter().filter(|c| c.text.contains(SEPARATOR)).count();
            assert!(limit < 9 || merged > 0, "{name} at {limit} merges");
        }
    }
}

#[test]
fn b28_never_merged_across_headings_tables_or_notes() {
    let table = Table {
        num_rows: 2,
        num_cols: 2,
        cells: vec![
            cell(0, 0, "Spalte", true, None),
            cell(0, 1, "Wert", true, None),
            cell(1, 0, "Tabelleninhalt", false, None),
            cell(1, 1, "42", false, None),
        ],
    };
    let document = document(
        "grenzen.pptx",
        DocumentFormat::Pptx,
        vec![
            element(heading(1), "Titel", vec![slide(1, false)]),
            element(ElementKind::Text, "Absatz a.", vec![slide(1, false)]),
            element(heading(2), "Unterkapitel", vec![slide(1, false)]),
            element(ElementKind::Text, "Absatz b.", vec![slide(1, false)]),
            element(
                ElementKind::Table(table),
                "| Spalte | Wert |\n| --- | --- |\n| Tabelleninhalt | 42 |",
                vec![slide(1, false)],
            ),
            element(ElementKind::Text, "Absatz c.", vec![slide(1, false)]),
            element(ElementKind::Text, "Notiz n.", vec![slide(1, true)]),
            element(ElementKind::Text, "Absatz d.", vec![slide(2, false)]),
        ],
    );
    for (_, counter) in counters() {
        let chunks = chunk(&chunker(&counter, 400), &document);
        assert_eq!(
            texts(&chunks),
            [
                "Absatz a.",
                "Absatz b.",
                "| Spalte | Wert |\n| --- | --- |\n| Tabelleninhalt | 42 |",
                "Absatz c.",
                "Notiz n.",
                "Absatz d.",
            ]
        );
    }
}

#[test]
fn b29_merged_chunks_carry_every_anchor_once_in_document_order() {
    let slides = document(
        "folie.pptx",
        DocumentFormat::Pptx,
        vec![
            element(ElementKind::Text, "Erster Absatz.", vec![slide(2, false)]),
            element(ElementKind::Text, "Zweiter Absatz.", vec![slide(2, false)]),
        ],
    );
    let chunks = chunk(&mock_chunker(400), &slides);
    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0].provenance.anchors, [slide(2, false)]);

    let pdf = document(
        "seite.pdf",
        DocumentFormat::Pdf,
        vec![
            element(
                ElementKind::Text,
                "Oben auf der Seite.",
                vec![page(1, 100.0, 120.0)],
            ),
            element(
                ElementKind::Text,
                "Weiter unten.",
                vec![page(1, 200.0, 220.0)],
            ),
            element(
                ElementKind::Text,
                "Über den Umbruch.",
                vec![page(1, 700.0, 790.0), page(2, 40.0, 60.0)],
            ),
        ],
    );
    let chunks = chunk(&mock_chunker(400), &pdf);
    assert_eq!(chunks.len(), 1);
    assert_eq!(
        chunks[0].provenance.anchors,
        [
            page(1, 100.0, 120.0),
            page(1, 200.0, 220.0),
            page(1, 700.0, 790.0),
            page(2, 40.0, 60.0)
        ]
    );

    let word = &fixtures::documents()["onboarding.docx"];
    let chunks = chunk(&mock_chunker(400), word);
    assert_eq!(
        chunks[0].provenance.anchors,
        [
            Anchor::Paragraph { element: 2 },
            Anchor::Paragraph { element: 3 },
            Anchor::Paragraph { element: 4 }
        ]
    );
}

#[test]
fn b30_merged_elements_are_joined_by_a_blank_line() {
    let chunks = chunk(
        &mock_chunker(400),
        &fixtures::documents()["onboarding.docx"],
    );
    assert_eq!(
        chunks[0].text,
        "Pick up your badge at the front desk before 9 am. Your manager will walk you through the safety briefing.\n\nBring a valid photo ID.\n\nSet up your laptop together with the IT team."
    );
    // The separator counts: with the character counter two elements of 10 characters
    // fit into 22 tokens, not into 21.
    let document = md(
        "trenner.md",
        &[
            (ElementKind::Text, "zehn zeich"),
            (ElementKind::Text, "zehn zeich"),
        ],
    );
    let counter: Arc<dyn TokenCounter> = Arc::new(CharCounter);
    assert_eq!(chunk(&chunker(&counter, 22), &document).len(), 1);
    assert_eq!(chunk(&chunker(&counter, 21), &document).len(), 2);
}

#[test]
fn b08_chunks_and_their_anchors_are_in_reading_order() {
    let document = md(
        "reihenfolge.md",
        &[
            (ElementKind::Text, "eins zwei drei vier fünf sechs"),
            (ElementKind::Text, "sieben acht"),
            (heading(1), "Mitte"),
            (ElementKind::Text, "neun zehn elf zwölf"),
        ],
    );
    for limit in LIMITS {
        let chunks = chunk(&mock_chunker(limit), &document);
        let all: Vec<String> = chunks.iter().flat_map(|c| words(&c.text)).collect();
        assert_eq!(
            all.join(" "),
            "eins zwei drei vier fünf sechs sieben acht neun zehn elf zwölf"
        );
        let firsts: Vec<u32> = chunks
            .iter()
            .map(|c| match c.provenance.anchors[0] {
                Anchor::Paragraph { element } => element,
                _ => unreachable!(),
            })
            .collect();
        assert!(firsts.windows(2).all(|w| w[0] <= w[1]), "{firsts:?}");
    }
}
