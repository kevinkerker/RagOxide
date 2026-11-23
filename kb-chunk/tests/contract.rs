//! Basic promises of the contract: construction, ids, source, text, language, what is
//! left out, invalid input, counter errors, the limit as parameter (B01 to B11, B19,
//! B20, B23, B26).

mod common;

use std::sync::Arc;
use std::task::{Context, Poll, Waker};

use common::*;
use kb_chunk::DocumentChunker;
use kb_core::mock::*;
use kb_core::*;

#[test]
fn b01_built_from_counter_and_limit_and_used_as_trait_object() {
    fn send_sync<T: Send + Sync>() {}
    send_sync::<DocumentChunker>();

    let tokens: Arc<dyn TokenCounter> = Arc::new(MockTokenCounter);
    let limit = PipelineConfig::default().chunk_max_tokens as usize;
    let chunker: Arc<dyn Chunker> = Arc::new(DocumentChunker::new(Arc::clone(&tokens), limit));
    let embedder = Arc::new(MockEmbedder::default());
    let index = Arc::new(MockIndex::new(embedder.space(), 0.5));
    let _ingestor: Arc<dyn Ingestor> = Arc::new(MockIngestor::new(
        Arc::new(MockParser::new()),
        Arc::clone(&chunker),
        embedder,
        index,
        Arc::new(MockStore::new()),
        Arc::new(MockFiles::fixtures()),
    ));
    let document = &fixtures::documents()["faq.html"];
    let chunks = futures::executor::block_on(chunker.chunk(document)).unwrap();
    assert!(!chunks.is_empty());
}

#[test]
fn b02_ids_are_gapless_ordinals_of_the_document() {
    for limit in LIMITS {
        let chunker = mock_chunker(limit);
        for document in all_documents() {
            let chunks = chunk(&chunker, &document);
            for (n, chunk) in chunks.iter().enumerate() {
                assert_eq!(
                    chunk.id,
                    ChunkId::new(&document.source.document_id, n as u32),
                    "{} at {limit}",
                    document.source.name
                );
                assert_eq!(
                    chunk.id.as_str(),
                    format!("{}#{n}", document.source.document_id)
                );
            }
        }
    }
}

#[test]
fn b03_b04_source_is_the_document_source_and_text_is_never_blank() {
    for limit in LIMITS {
        let chunker = mock_chunker(limit);
        for document in all_documents() {
            for chunk in chunk(&chunker, &document) {
                assert_eq!(chunk.provenance.source, document.source);
                assert_eq!(chunk.document_id(), &document.source.document_id);
                assert!(!chunk.text.trim().is_empty(), "{}", document.source.name);
            }
        }
    }
}

#[test]
fn general_guarantees_hold_for_every_document_counter_and_limit() {
    futures::executor::block_on(async {
        for (_, counter) in counters() {
            for limit in LIMITS {
                let chunker = chunker(&counter, limit);
                for document in all_documents() {
                    let chunks = chunker.chunk(&document).await.unwrap();
                    check_guarantees(&document, &chunks, counter.as_ref(), limit).await;
                }
            }
        }
    });
}

#[test]
fn b06_language_is_the_documents_without_exception() {
    let mut document = md(
        "sprache.md",
        &[
            (heading(1), "Sprache"),
            (ElementKind::Text, "This paragraph is written in English."),
            (ElementKind::Text, "Dieser Absatz ist deutsch."),
        ],
    );
    for language in [Some("de".to_owned()), Some("en".to_owned()), None] {
        document.language = language.clone();
        for limit in LIMITS {
            let chunks = chunk(&mock_chunker(limit), &document);
            assert!(!chunks.is_empty());
            assert!(chunks.iter().all(|c| c.language == language));
        }
    }
}

#[test]
fn b07_equal_input_gives_equal_output() {
    let tokens: Arc<dyn TokenCounter> = Arc::new(MockTokenCounter);
    for limit in LIMITS {
        let first = Arc::new(DocumentChunker::new(Arc::clone(&tokens), limit));
        let second = DocumentChunker::new(Arc::clone(&tokens), limit);
        for document in all_documents() {
            let a = chunk(&first, &document);
            assert_eq!(a, chunk(&first, &document));
            assert_eq!(a, chunk(&second, &document));
            // Two calls at the same time on the same object.
            let threads: Vec<_> = (0..2)
                .map(|_| {
                    let chunker = Arc::clone(&first);
                    let document = document.clone();
                    std::thread::spawn(move || chunk(&chunker, &document))
                })
                .collect();
            for thread in threads {
                assert_eq!(thread.join().unwrap(), a);
            }
        }
    }
}

#[test]
fn b10_furniture_and_pictures_without_text_are_left_out() {
    let document = paragraphs(
        "arten.docx",
        DocumentFormat::Docx,
        &[
            (ElementKind::Furniture, "Kopfzeile Furnituretext"),
            (ElementKind::Text, "Fliesstext"),
            (ElementKind::Picture, ""),
            (ElementKind::Picture, "  \n "),
            (ElementKind::ListItem, "Listenpunkt"),
            (ElementKind::Caption, "Bildunterschrift"),
            (ElementKind::Picture, "Bildtext"),
            (ElementKind::Furniture, "Seite 1"),
        ],
    );
    for limit in LIMITS {
        let chunks = chunk(&mock_chunker(limit), &document);
        let all: String = texts(&chunks).join(" ");
        assert!(!all.contains("Furnituretext") && !all.contains("Seite"));
        for word in ["Fliesstext", "Listenpunkt", "Bildunterschrift", "Bildtext"] {
            assert!(all.contains(word), "{word} at {limit}");
        }
        let anchored: Vec<u32> = chunks
            .iter()
            .flat_map(|c| &c.provenance.anchors)
            .map(|a| match a {
                Anchor::Paragraph { element } => *element,
                other => panic!("{other:?}"),
            })
            .collect();
        assert_eq!(
            anchored,
            [1, 4, 5, 6],
            "pictures without text add no anchor"
        );
    }
    let training = chunk(&mock_chunker(400), &fixtures::documents()["schulung.pptx"]);
    let picture_slide = Anchor::Slide {
        slide: 3,
        in_notes: false,
    };
    let with_picture: Vec<&Chunk> = training
        .iter()
        .filter(|c| c.provenance.anchors.contains(&picture_slide))
        .collect();
    assert_eq!(texts(&training).len(), 4);
    assert_eq!(with_picture.len(), 1, "only the caption on slide 3");
    assert_eq!(
        with_picture[0].text,
        "Abbildung 1: Lageplan des Erdgeschosses"
    );
}

#[test]
fn b11_documents_without_text_give_an_empty_list() {
    let empty = document("leer.pdf", DocumentFormat::Pdf, Vec::new());
    let furniture = document(
        "kopf.pdf",
        DocumentFormat::Pdf,
        vec![
            element(
                ElementKind::Furniture,
                "Kopfzeile",
                vec![page(1, 10.0, 20.0)],
            ),
            element(
                ElementKind::Furniture,
                "Seite 1",
                vec![page(1, 800.0, 810.0)],
            ),
        ],
    );
    let blank = md(
        "leer.md",
        &[
            (ElementKind::Text, ""),
            (ElementKind::Text, " \n\t "),
            (ElementKind::Picture, ""),
            (ElementKind::Caption, "   "),
        ],
    );
    let headings = md(
        "titel.md",
        &[
            (heading(1), "Titel"),
            (heading(2), "Kapitel"),
            (heading(1), "Noch ein Titel"),
        ],
    );
    for document in [empty, furniture, blank, headings] {
        for limit in LIMITS {
            assert_eq!(chunk(&mock_chunker(limit), &document), Vec::new());
        }
    }
    // A heading without content before the next heading is no chunk of its own.
    let document = md(
        "kapitel.md",
        &[
            (heading(1), "Leeres Kapitel"),
            (heading(1), "Volles Kapitel"),
            (ElementKind::Text, "Inhalt."),
        ],
    );
    let chunks = chunk(&mock_chunker(400), &document);
    assert_eq!(texts(&chunks), ["Inhalt."]);
    assert_eq!(chunks[0].provenance.heading_path, ["Volles Kapitel"]);
}

#[test]
fn b19_an_element_without_anchor_is_invalid_input() {
    for kind in [
        ElementKind::Text,
        ElementKind::Heading { level: 1 },
        ElementKind::Furniture,
        ElementKind::Picture,
    ] {
        let mut document = md(
            "anker.md",
            &[(ElementKind::Text, "Vorher."), (kind, "Ohne Anker.")],
        );
        document.elements[1].anchors.clear();
        for limit in LIMITS {
            let error = try_chunk(&mock_chunker(limit), &document).unwrap_err();
            assert!(matches!(error, Error::InvalidInput(_)), "{error}");
        }
    }
    // An anchor of another format is a broken contract as well.
    let mut document = md("anker.md", &[(ElementKind::Text, "Seite statt Absatz.")]);
    document.elements[0].anchors = vec![page(1, 0.0, 10.0)];
    let error = try_chunk(&mock_chunker(400), &document).unwrap_err();
    assert!(matches!(error, Error::InvalidInput(_)), "{error}");
}

fn table_document(table: Table) -> Document {
    document(
        "fehler.xlsx",
        DocumentFormat::Xlsx,
        vec![element(
            ElementKind::Table(table),
            "| A | B |\n| --- | --- |\n| 1 | 2 |",
            vec![sheet("Fehler", 0, 0, 1, 1)],
        )],
    )
}

fn good_cells() -> Vec<TableCell> {
    vec![
        cell(0, 0, "A", true, None),
        cell(0, 1, "B", true, None),
        cell(1, 0, "1", false, None),
        cell(1, 1, "2", false, None),
    ]
}

#[test]
fn b20_broken_table_data_is_invalid_input_without_panic() {
    let mut cases = Vec::new();
    let mut zero_row_span = good_cells();
    zero_row_span[2].row_span = 0;
    cases.push(("row_span 0", 2, 2, zero_row_span));
    let mut zero_col_span = good_cells();
    zero_col_span[3].col_span = 0;
    cases.push(("col_span 0", 2, 2, zero_col_span));
    let mut below = good_cells();
    below[3].row = 2;
    cases.push(("row outside", 2, 2, below));
    let mut right = good_cells();
    right[1].col = 5;
    cases.push(("column outside", 2, 2, right));
    cases.push(("num_rows 0", 0, 2, good_cells()));
    cases.push(("num_cols 0", 2, 0, good_cells()));
    let mut twice = good_cells();
    twice[3].col = 0;
    cases.push(("two cells in one place", 2, 2, twice));
    let mut huge_span = good_cells();
    huge_span[2].row_span = u32::MAX;
    huge_span[2].col_span = u32::MAX;
    for (name, num_rows, num_cols, cells) in cases {
        let document = table_document(Table {
            num_rows,
            num_cols,
            cells,
        });
        for limit in LIMITS {
            let error = try_chunk(&mock_chunker(limit), &document).unwrap_err();
            assert!(matches!(error, Error::InvalidInput(_)), "{name}: {error}");
        }
    }
    // Spans reaching past the table are clamped, not an error.
    let document = table_document(Table {
        num_rows: 2,
        num_cols: 2,
        cells: huge_span,
    });
    for limit in LIMITS {
        assert!(try_chunk(&mock_chunker(limit), &document).is_ok());
    }
    // A table with `num_rows` 0 and no cells is a table without cells.
    let document = table_document(Table {
        num_rows: 0,
        num_cols: 0,
        cells: Vec::new(),
    });
    let chunks = chunk(&mock_chunker(400), &document);
    assert_eq!(chunks.len(), 1);
}

#[test]
fn b23_a_counter_error_ends_the_call_with_exactly_that_error() {
    let document = &edge_documents()[1];
    let reference = Arc::new(CountingCounter::new(Arc::new(MockTokenCounter)));
    let tokens: Arc<dyn TokenCounter> = reference.clone();
    let _ = chunk(&DocumentChunker::new(tokens, 20), document);
    let calls = reference.calls();
    assert!(calls > 10, "{calls}");
    for fail_at in 1..=calls {
        let chunker = DocumentChunker::new(Arc::new(FailingCounter::new(fail_at)), 20);
        let error = try_chunk(&chunker, document).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Model, "call {fail_at}");
        assert_eq!(
            error.to_string(),
            format!("model error: tokenizer failed at call {fail_at}")
        );
    }
}

#[test]
fn b26_limit_is_a_parameter_and_zero_acts_as_one() {
    let document = &fixtures::documents()["pruefordnung.pdf"];
    let at_400 = chunk(&mock_chunker(400), document);
    let at_10 = chunk(&mock_chunker(10), document);
    assert!(at_10.len() > at_400.len());
    let tokens: Arc<dyn TokenCounter> = Arc::new(MockTokenCounter);
    for document in all_documents() {
        let zero = chunk(&DocumentChunker::new(Arc::clone(&tokens), 0), &document);
        assert_eq!(zero, chunk(&mock_chunker(1), &document));
        assert!(zero.iter().all(|c| c.text.split_whitespace().count() <= 1));
    }
}

#[test]
fn chunking_yields_to_the_executor_between_counts() {
    let chunker = mock_chunker(400);
    let document = &fixtures::documents()["onboarding.docx"];
    let mut future = chunker.chunk(document);
    let mut context = Context::from_waker(Waker::noop());
    assert!(future.as_mut().poll(&mut context).is_pending());
    let chunks = loop {
        if let Poll::Ready(result) = future.as_mut().poll(&mut context) {
            break result.unwrap();
        }
    };
    assert_eq!(chunks, chunk(&chunker, document));
}
