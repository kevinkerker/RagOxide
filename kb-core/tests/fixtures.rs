//! The JSON fixtures load and cover what Phase 1 needs.

use std::collections::HashSet;

use futures::executor::block_on;
use kb_core::mock::{MockChunker, fixtures, placeholder_hash};
use kb_core::*;

#[test]
fn fixtures_load() {
    assert_eq!(fixtures::documents().len(), 7);
    assert!(!fixtures::chunks().is_empty());
    assert!(!fixtures::hits().is_empty());
}

#[test]
fn documents_cover_every_format_and_anchor_variant() {
    let documents = fixtures::documents();
    let formats: HashSet<DocumentFormat> = documents.values().map(|d| d.format).collect();
    assert_eq!(formats.len(), 7, "one document per format");
    let mut variants = HashSet::new();
    for document in documents.values() {
        for anchor in document.elements.iter().flat_map(|e| &e.anchors) {
            variants.insert(match anchor {
                Anchor::Page { .. } => "page",
                Anchor::Slide { .. } => "slide",
                Anchor::Sheet { .. } => "sheet",
                Anchor::Paragraph { .. } => "paragraph",
            });
        }
    }
    assert_eq!(
        variants,
        HashSet::from(["page", "slide", "sheet", "paragraph"])
    );
}

#[test]
fn a_slide_with_notes_covers_both_slide_anchors() {
    let notes = Anchor::Slide {
        slide: 2,
        in_notes: true,
    };
    let on_slide = Anchor::Slide {
        slide: 2,
        in_notes: false,
    };
    assert_ne!(notes, on_slide, "a place in the notes is another anchor");

    let slides = &fixtures::documents()["schulung.pptx"];
    let anchors: Vec<&Anchor> = slides.elements.iter().flat_map(|e| &e.anchors).collect();
    assert!(anchors.contains(&&on_slide), "an element on slide 2");
    assert!(anchors.contains(&&notes), "an element in its notes");

    let chunk = fixtures::chunks()
        .into_iter()
        .find(|c| c.provenance.anchors == [notes.clone()])
        .expect("a chunk from the notes");
    assert!(chunk.text.starts_with("Notiz"));
    assert!(
        fixtures::hits()
            .iter()
            .any(|h| h.chunk.provenance.anchors == [notes.clone()]),
        "a hit from the notes"
    );
}

#[test]
fn documents_cover_languages_tables_and_page_breaks() {
    let documents = fixtures::documents();
    let all_text: String = documents
        .values()
        .flat_map(|d| d.elements.iter().map(|e| e.text.as_str()))
        .collect::<Vec<_>>()
        .join(" ");
    for umlaut in ['ä', 'ö', 'ü', 'Ü', 'ß'] {
        assert!(all_text.contains(umlaut), "no {umlaut} in fixtures");
    }
    let languages: HashSet<Option<&str>> =
        documents.values().map(|d| d.language.as_deref()).collect();
    assert_eq!(languages, HashSet::from([Some("de"), Some("en")]));

    let tables: Vec<&Table> = documents
        .values()
        .flat_map(|d| &d.elements)
        .filter_map(|e| match &e.kind {
            ElementKind::Table(t) => Some(t),
            _ => None,
        })
        .collect();
    assert!(tables.len() >= 2, "tables in PDF and XLSX expected");
    for table in tables {
        assert_eq!(table.cells.len() as u32, table.num_rows * table.num_cols);
        assert!(table.cells.iter().any(|c| c.column_header));
        assert!(table.cells.iter().all(|c| c.anchor.is_some()));
    }

    let pdf = &documents["pruefordnung.pdf"];
    let across_pages = pdf.elements.iter().find(|e| {
        let pages: HashSet<u32> = e
            .anchors
            .iter()
            .filter_map(|a| match a {
                Anchor::Page { page, .. } => Some(*page),
                _ => None,
            })
            .collect();
        e.kind == ElementKind::Text && pages.len() == 2
    });
    assert!(across_pages.is_some(), "no paragraph across two pages");
}

#[test]
fn table_cells_are_relative_and_anchors_absolute() {
    let documents = fixtures::documents();
    let xlsx = &documents["budget.xlsx"];
    let ElementKind::Table(table) = &xlsx.elements[0].kind else {
        panic!("first element of budget.xlsx is a table");
    };
    let first = &table.cells[0];
    assert_eq!((first.row, first.col), (0, 0));
    // The table starts at B3.
    assert_eq!(
        first.anchor,
        Some(Anchor::Sheet {
            sheet: "Budget".into(),
            range: CellRange {
                start_row: 2,
                start_col: 1,
                end_row: 2,
                end_col: 1,
            },
        })
    );
}

#[test]
fn documents_satisfy_the_parser_contract() {
    for (file, document) in fixtures::documents() {
        let format = DocumentFormat::from_path(file.as_ref()).expect("known extension");
        assert_eq!(document.format, format, "{file}");
        assert_eq!(document.source.name, file);
        assert_eq!(
            document.source.path,
            canonical_path(&document.source.path).unwrap()
        );
        assert_eq!(
            document.source.content_hash,
            placeholder_hash(&document.source.path).unwrap()
        );
        let paged = matches!(format, DocumentFormat::Pdf | DocumentFormat::Image);
        assert_eq!(
            !document.pages.is_empty(),
            paged,
            "{file}: pages only for paged formats"
        );
        for (index, element) in document.elements.iter().enumerate() {
            assert!(
                !element.anchors.is_empty(),
                "{file}: element {index} without anchor"
            );
            for anchor in &element.anchors {
                match (format, anchor) {
                    (DocumentFormat::Pdf | DocumentFormat::Image, Anchor::Page { page, bbox }) => {
                        let size = document
                            .pages
                            .iter()
                            .find(|p| p.page == *page)
                            .expect("page exists");
                        assert!(bbox.left <= bbox.right && bbox.top <= bbox.bottom);
                        assert!(bbox.right <= size.width && bbox.bottom <= size.height);
                    }
                    (DocumentFormat::Pptx, Anchor::Slide { slide, .. }) => assert!(*slide >= 1),
                    (DocumentFormat::Xlsx, Anchor::Sheet { range, .. }) => {
                        assert!(range.start_row <= range.end_row);
                        assert!(range.start_col <= range.end_col);
                    }
                    (
                        DocumentFormat::Docx | DocumentFormat::Html | DocumentFormat::Markdown,
                        Anchor::Paragraph { element },
                    ) => assert_eq!(*element as usize, index, "{file}: own index"),
                    (format, anchor) => panic!("{file}: {anchor:?} does not belong to {format:?}"),
                }
            }
        }
    }
}

#[test]
fn chunks_fixture_is_the_mock_chunker_output() {
    block_on(async {
        let mut produced = Vec::new();
        for document in fixtures::documents().values() {
            produced.extend(MockChunker::default().chunk(document).await.unwrap());
        }
        assert_eq!(produced, fixtures::chunks());
    });
}

#[test]
fn chunks_carry_complete_provenance() {
    let documents = fixtures::documents();
    let sources: Vec<&SourceFile> = documents.values().map(|d| &d.source).collect();
    let mut seen = HashSet::new();
    for chunk in fixtures::chunks() {
        assert!(sources.contains(&&chunk.provenance.source));
        assert!(
            seen.insert(chunk.id.clone()),
            "duplicate chunk id {}",
            chunk.id
        );
        assert!(!chunk.text.trim().is_empty());
        assert!(!chunk.provenance.anchors.is_empty());
        assert!(chunk.language.is_some());
    }
    let paragraph = fixtures::chunks()
        .into_iter()
        .find(|c| c.text.starts_with("Jedes Gerät"))
        .expect("paragraph chunk");
    assert_eq!(
        paragraph.id,
        ChunkId::new(&DocumentId::new("doc-pruefordnung"), 1)
    );
    let provenance = &paragraph.provenance;
    assert_eq!(provenance.source.name, "pruefordnung.pdf");
    assert_eq!(
        provenance.heading_path,
        ["Prüfordnung für Kühl- und Gefriergeräte", "§ 2 Prüffristen"]
    );
    assert_eq!(provenance.anchors.len(), 2);
}

#[test]
fn hits_refer_to_fixture_chunks_and_are_ordered() {
    let chunks = fixtures::chunks();
    let hits = fixtures::hits();
    for hit in &hits {
        assert!(
            chunks.contains(&hit.chunk),
            "hit {} not in chunks.json",
            hit.chunk.id
        );
        let score = hit.rerank_score.expect("hits are reranked");
        assert!((0.0..=1.0).contains(&score));
        assert!(hit.vector_score.is_some());
    }
    assert!(
        hits.windows(2)
            .all(|w| w[0].rerank_score.unwrap() >= w[1].rerank_score.unwrap())
    );
    assert!(hits[0].chunk.text.starts_with("Jedes Gerät"));
}

#[test]
fn pages_say_whether_their_text_comes_from_ocr() {
    for document in fixtures::documents().values() {
        let ocr = document.format == DocumentFormat::Image;
        assert!(
            document.pages.iter().all(|p| p.from_ocr == ocr),
            "{:?}",
            document.format
        );
    }
}

#[test]
fn spreadsheet_cells_carry_the_displayed_value_into_chunks() {
    let budget: Vec<Chunk> = fixtures::chunks()
        .into_iter()
        .filter(|c| c.document_id().as_str() == "doc-budget")
        .collect();
    let table = budget.iter().find(|c| c.text.starts_with('|')).unwrap();
    assert!(table.text.contains("| 12,000 |"), "{}", table.text);
    assert!(!table.text.contains("12000"));
}

#[test]
fn a_paragraph_continues_in_the_next_column_of_the_same_page() {
    let documents = fixtures::documents();
    let column_break = documents.values().flat_map(|d| &d.elements).any(|e| {
        let pages: Vec<u32> = e
            .anchors
            .iter()
            .filter_map(|a| match a {
                Anchor::Page { page, .. } => Some(*page),
                _ => None,
            })
            .collect();
        pages.len() >= 2 && pages.windows(2).any(|w| w[0] == w[1])
    });
    assert!(column_break, "no paragraph with two boxes on one page");
}
