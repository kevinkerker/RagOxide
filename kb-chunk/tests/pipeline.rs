//! Test data and the mock pipeline (B39, B40) and the measurements of the report (B47).

mod common;

use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use common::*;
use kb_chunk::DocumentChunker;
use kb_core::mock::*;
use kb_core::*;

#[test]
fn b39_every_mock_chunk_is_contained_in_exactly_one_chunk() {
    let documents = fixtures::documents();
    let chunker = mock_chunker(400);
    let mut checked = 0;
    for expected in fixtures::chunks() {
        let document = documents
            .values()
            .find(|d| d.source.document_id == *expected.document_id())
            .expect("fixture document");
        let chunks = chunk(&chunker, document);
        let holding: Vec<&Chunk> = chunks
            .iter()
            .filter(|c| c.text.contains(&expected.text))
            .collect();
        assert_eq!(holding.len(), 1, "{:?}", expected.text);
        for anchor in &expected.provenance.anchors {
            assert!(
                holding[0].provenance.anchors.contains(anchor),
                "{anchor:?} of {:?}",
                expected.text
            );
        }
        checked += 1;
    }
    assert_eq!(checked, 20);
}

#[test]
fn b39_fixtures_keep_every_rule_at_400() {
    futures::executor::block_on(async {
        let counter = MockTokenCounter;
        let chunker = mock_chunker(400);
        for document in fixtures::documents().values() {
            let chunks = chunker.chunk(document).await.unwrap();
            check_guarantees(document, &chunks, &counter, 400).await;
            check_no_loss(document, &chunks);
        }
    });
}

#[test]
fn b40_the_mock_ingestor_indexes_all_fixtures_with_this_chunker() {
    futures::executor::block_on(async {
        let store = Arc::new(MockStore::new());
        store
            .update_settings(Box::new(|s| {
                s.add_folder(Path::new(MockFiles::FIXTURE_FOLDER))
            }))
            .await
            .unwrap();
        let embedder = Arc::new(MockEmbedder::default());
        let index = Arc::new(MockIndex::new(embedder.space(), 0.5));
        let chunker = Arc::new(DocumentChunker::new(Arc::new(MockTokenCounter), 400));
        let ingestor = MockIngestor::new(
            Arc::new(MockParser::new()),
            Arc::clone(&chunker) as Arc<dyn Chunker>,
            Arc::clone(&embedder) as Arc<dyn Embedder>,
            Arc::clone(&index) as Arc<dyn Index>,
            Arc::clone(&store) as Arc<dyn Store>,
            Arc::new(MockFiles::fixtures()),
        );
        let results = ingestor
            .sync(
                Arc::new(|_| {}),
                Arc::new(|_| {}),
                SyncMode::Normal,
                CancellationToken::new(),
            )
            .await
            .unwrap();
        assert_eq!(results.len(), 7);
        assert!(
            results
                .iter()
                .all(|r| matches!(r.status, FileStatus::Indexed { .. })),
            "{results:?}"
        );
        let documents = fixtures::documents();
        let records = store.documents().await.unwrap();
        assert_eq!(records.len(), 7);
        let mut total = 0;
        for record in records {
            let name = record.path.file_name().unwrap().to_str().unwrap();
            let expected = chunker.chunk(&documents[name]).await.unwrap().len();
            assert_eq!(
                record.status,
                DocumentStatus::Indexed {
                    chunks: expected as u32
                },
                "{name}"
            );
            total += expected;
        }
        assert_eq!(index.len(), total);

        let query = embedder.embed_query(fixtures::HITS_QUERY).await.unwrap();
        let hits = index
            .search(&SearchRequest::new(fixtures::HITS_QUERY, 20), &query)
            .await
            .unwrap();
        let notes = Anchor::Slide {
            slide: 2,
            in_notes: true,
        };
        assert!(
            hits.iter()
                .any(|h| h.chunk.provenance.anchors.contains(&notes)),
            "{hits:?}"
        );
    });
}

/// Duration and counter calls for a document, printed for the report.
fn measure(label: &str, document: &Document) -> (usize, usize) {
    let counter = Arc::new(CountingCounter::new(Arc::new(MockTokenCounter)));
    let chunker = DocumentChunker::new(Arc::clone(&counter) as Arc<dyn TokenCounter>, 400);
    let text_bytes: usize = document.elements.iter().map(|e| e.text.len()).sum();
    let started = Instant::now();
    let chunks = chunk(&chunker, document);
    let elapsed = started.elapsed();
    println!(
        "{label} ({text_bytes} bytes of element text): {} chunks, {} counter calls over {} bytes, {:.1} ms",
        chunks.len(),
        counter.calls(),
        counter.bytes(),
        elapsed.as_secs_f64() * 1000.0
    );
    // For comparison: the mock chunker on the same document.
    let mock_counter = Arc::new(CountingCounter::new(Arc::new(MockTokenCounter)));
    let mock = MockChunker::new(Arc::clone(&mock_counter) as Arc<dyn TokenCounter>, 400);
    let started = Instant::now();
    let mock_chunks = futures::executor::block_on(mock.chunk(document)).unwrap();
    println!(
        "{label}, MockChunker: {} chunks, {} counter calls over {} bytes, {:.1} ms",
        mock_chunks.len(),
        mock_counter.calls(),
        mock_counter.bytes(),
        started.elapsed().as_secs_f64() * 1000.0
    );
    (chunks.len(), counter.calls())
}

#[test]
fn b47_measurements_table_and_paragraphs_with_10000_items() {
    let table = document(
        "gross.xlsx",
        DocumentFormat::Xlsx,
        vec![excel_table(10_000)],
    );
    let (parts, calls) = measure("table with 10,000 rows", &table);
    assert!(parts > 100);
    assert!(
        calls < 4 * parts,
        "counting grows with the parts, not the rows"
    );

    let items: Vec<(ElementKind, String)> = (0..10_000)
        .map(|i| {
            (
                ElementKind::Text,
                format!("Absatz {i} mit einem kurzen Satz aus zehn Wörtern hier."),
            )
        })
        .collect();
    let mut items: Vec<(ElementKind, &str)> =
        items.iter().map(|(k, t)| (k.clone(), t.as_str())).collect();
    items.insert(0, (heading(1), "Viele Absätze"));
    let paragraphs = md("absaetze.md", &items);
    let (chunks, calls) = measure("document with 10,000 paragraphs", &paragraphs);
    assert!(chunks > 100);
    assert!(
        calls < 10_000 + 4 * chunks,
        "one count per paragraph plus a few per chunk"
    );
}
