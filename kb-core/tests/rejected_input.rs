//! Input the store and the index refuse: a second record for a path (K42) and boxes or
//! page sizes that are not finite (K43, K47). The fixed test data hold none of them.

use std::path::{Path, PathBuf};

use futures::executor::block_on;
use kb_core::mock::*;
use kb_core::*;

fn record(id: &str, path: &str) -> DocumentRecord {
    DocumentRecord {
        id: DocumentId::new(id),
        path: PathBuf::from(path),
        format: DocumentFormat::Pdf,
        title: None,
        content_hash: None,
        size_bytes: None,
        modified_at: None,
        status: DocumentStatus::Queued,
    }
}

fn is_invalid_input<T: std::fmt::Debug>(result: &Result<T>) -> bool {
    matches!(result, Err(Error::InvalidInput(_)))
}

// ----------------------------------------------------------------- one record per path

#[test]
fn a_new_id_for_a_taken_path_is_refused_in_any_spelling() {
    block_on(async {
        let store = MockStore::new();
        store
            .upsert_document(&record("a", "C:\\Ablage\\Pr\u{fc}fung.pdf"))
            .await
            .unwrap();
        let before = store.documents().await.unwrap();
        for path in [
            "C:\\Ablage\\Pr\u{fc}fung.pdf",
            "c:\\ABLAGE\\PR\u{dc}FUNG.PDF",
            "C:\\Ablage\\Pru\u{308}fung.pdf",
        ] {
            let result = store.upsert_document(&record("b", path)).await;
            assert!(is_invalid_input(&result), "{path}: {result:?}");
            assert_eq!(store.documents().await.unwrap(), before, "{path}");
        }
        assert!(
            store
                .document(&DocumentId::new("b"))
                .await
                .unwrap()
                .is_none()
        );
    });
}

#[test]
fn a_replacement_with_the_path_of_another_record_is_refused() {
    block_on(async {
        let store = MockStore::new();
        let a = record("a", r"C:\Ablage\a.pdf");
        let b = record("b", r"C:\Ablage\b.pdf");
        store.upsert_document(&a).await.unwrap();
        store.upsert_document(&b).await.unwrap();
        let before = store.documents().await.unwrap();
        let result = store
            .upsert_document(&record("b", r"C:\ablage\A.PDF"))
            .await;
        assert!(is_invalid_input(&result), "{result:?}");
        assert_eq!(store.documents().await.unwrap(), before);
        assert_eq!(store.document(&a.id).await.unwrap(), Some(a));
        assert_eq!(store.document(&b.id).await.unwrap(), Some(b));
    });
}

#[test]
fn a_replacement_with_the_same_or_a_free_path_succeeds() {
    block_on(async {
        let store = MockStore::new();
        store
            .upsert_document(&record("a", r"C:\Ablage\a.pdf"))
            .await
            .unwrap();
        // Same id, same path: replaced.
        let indexed = DocumentRecord {
            title: Some("A".into()),
            ..record("a", r"C:\Ablage\a.pdf")
        };
        store.upsert_document(&indexed).await.unwrap();
        assert_eq!(
            store.document(&DocumentId::new("a")).await.unwrap(),
            Some(indexed)
        );
        // Same id, free path: moved, found under the new path only.
        let moved = record("a", r"C:\Ablage\neu.pdf");
        store.upsert_document(&moved).await.unwrap();
        assert_eq!(store.documents().await.unwrap(), vec![moved.clone()]);
        assert_eq!(
            store
                .document_by_path(Path::new(r"C:\Ablage\neu.pdf"))
                .await
                .unwrap(),
            Some(moved)
        );
        assert!(
            store
                .document_by_path(Path::new(r"C:\Ablage\a.pdf"))
                .await
                .unwrap()
                .is_none()
        );
    });
}

// ------------------------------------------------------------------ finite numbers

/// The fixture PDF: two pages, element boxes and a table with cell boxes.
fn pdf() -> Document {
    fixtures::documents().remove("pruefordnung.pdf").unwrap()
}

/// The first box among `anchors`.
fn first_box(anchors: &mut [Anchor]) -> &mut BoundingBox {
    anchors
        .iter_mut()
        .find_map(|a| match a {
            Anchor::Page { bbox, .. } => Some(bbox),
            _ => None,
        })
        .expect("an anchor with a box")
}

fn with_page_width(value: f32) -> Document {
    let mut document = pdf();
    document.pages[0].width = value;
    document
}

fn with_element_box(value: f32) -> Document {
    let mut document = pdf();
    let element = document
        .elements
        .iter_mut()
        .find(|e| e.anchors.iter().any(|a| matches!(a, Anchor::Page { .. })))
        .unwrap();
    first_box(&mut element.anchors).right = value;
    document
}

fn with_cell_box(value: f32) -> Document {
    let mut document = pdf();
    let cell = document
        .elements
        .iter_mut()
        .find_map(|e| match &mut e.kind {
            ElementKind::Table(table) => table.cells.iter_mut().find(|c| c.anchor.is_some()),
            _ => None,
        })
        .expect("a table cell with a box");
    first_box(std::slice::from_mut(cell.anchor.as_mut().unwrap())).top = value;
    document
}

#[test]
fn a_document_with_a_number_that_is_not_finite_is_not_stored() {
    block_on(async {
        let id = pdf().source.document_id;
        let bad = [
            ("NaN page width", with_page_width(f32::NAN)),
            ("+inf in an element box", with_element_box(f32::INFINITY)),
            ("-inf in a cell box", with_cell_box(f32::NEG_INFINITY)),
            ("+inf page width", with_page_width(f32::INFINITY)),
        ];
        // Without earlier content: nothing stored afterwards.
        let store = MockStore::new();
        for (what, document) in &bad {
            let result = store.save_content(document).await;
            assert!(is_invalid_input(&result), "{what}: {result:?}");
            assert!(store.content(&id).await.unwrap().is_none(), "{what}");
        }
        // With earlier content: it stays as it was.
        store.save_content(&pdf()).await.unwrap();
        for (what, document) in &bad {
            let result = store.save_content(document).await;
            assert!(is_invalid_input(&result), "{what}: {result:?}");
            assert_eq!(store.content(&id).await.unwrap(), Some(pdf()), "{what}");
        }
    });
}

fn chat_citing(chunk: Chunk, title: &str) -> Chat {
    Chat {
        id: ChatId::new("c1"),
        title: title.into(),
        created_at: Timestamp(1),
        updated_at: Timestamp(2),
        turns: vec![ChatTurn {
            question: "Wie oft wird ein Gerät geprüft?".into(),
            answer: Answer {
                text: "Jährlich [1].".into(),
                citations: vec![Citation { number: 1, chunk }],
                no_sources: false,
                level: AnswerLevel::Fast,
                outcome: AnswerOutcome::Complete,
            },
        }],
    }
}

/// A fixture chunk with a box.
fn paged_chunk() -> Chunk {
    fixtures::chunks()
        .into_iter()
        .find(|c| {
            c.provenance
                .anchors
                .iter()
                .any(|a| matches!(a, Anchor::Page { .. }))
        })
        .expect("a chunk with a box")
}

fn with_chunk_box(value: f32) -> Chunk {
    let mut chunk = paged_chunk();
    first_box(&mut chunk.provenance.anchors).left = value;
    chunk
}

#[test]
fn a_chat_with_a_box_that_is_not_finite_in_a_citation_is_not_stored() {
    block_on(async {
        let store = MockStore::new();
        let id = ChatId::new("c1");
        let result = store
            .save_chat(&chat_citing(with_chunk_box(f32::NAN), "neu"))
            .await;
        assert!(is_invalid_input(&result), "{result:?}");
        assert!(store.chat(&id).await.unwrap().is_none());
        assert!(store.chats().await.unwrap().is_empty());

        let saved = chat_citing(paged_chunk(), "alt");
        store.save_chat(&saved).await.unwrap();
        for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            let result = store
                .save_chat(&chat_citing(with_chunk_box(value), "neu"))
                .await;
            assert!(is_invalid_input(&result), "{value}: {result:?}");
            assert_eq!(store.chat(&id).await.unwrap(), Some(saved.clone()));
        }
    });
}

fn finite(anchors: &[Anchor]) -> bool {
    anchors.iter().all(|a| match a {
        Anchor::Page { bbox, .. } => [bbox.left, bbox.top, bbox.right, bbox.bottom]
            .iter()
            .all(|v| v.is_finite()),
        _ => true,
    })
}

#[test]
fn every_box_and_page_size_of_the_fixtures_is_finite() {
    let mut boxes = 0;
    let mut count = |anchors: &[Anchor]| {
        boxes += anchors
            .iter()
            .filter(|a| matches!(a, Anchor::Page { .. }))
            .count();
        finite(anchors)
    };
    for (name, document) in fixtures::documents() {
        for page in &document.pages {
            assert!(page.width.is_finite() && page.height.is_finite(), "{name}");
        }
        for element in &document.elements {
            assert!(count(&element.anchors), "{name}");
            if let ElementKind::Table(table) = &element.kind {
                for cell in &table.cells {
                    assert!(count(cell.anchor.as_slice()), "{name}");
                }
            }
        }
    }
    for chunk in fixtures::chunks() {
        assert!(count(&chunk.provenance.anchors), "{}", chunk.id);
    }
    for hit in fixtures::hits() {
        assert!(count(&hit.chunk.provenance.anchors), "{}", hit.chunk.id);
    }
    assert!(boxes > 0, "the fixtures carry boxes");
}

async fn entries(embedder: &MockEmbedder, chunks: Vec<Chunk>) -> Vec<IndexEntry> {
    let embeddings = embedder
        .embed_documents(&chunks, CancellationToken::new())
        .await
        .unwrap();
    chunks
        .into_iter()
        .zip(embeddings)
        .map(|(chunk, embedding)| IndexEntry { chunk, embedding })
        .collect()
}

/// The hits of one query in all three modes.
async fn search_all(index: &MockIndex, query: &[f32]) -> Vec<Vec<SearchHit>> {
    let mut all = Vec::new();
    for mode in [SearchMode::Vector, SearchMode::FullText, SearchMode::Hybrid] {
        let request = SearchRequest {
            mode,
            ..SearchRequest::new("Gerät geprüft jährlich ersetzt", 100)
        };
        all.push(index.search(&request, query).await.unwrap());
    }
    all
}

#[test]
fn the_index_adds_nothing_from_a_call_with_a_box_that_is_not_finite() {
    block_on(async {
        let embedder = MockEmbedder::default();
        let index = MockIndex::new(embedder.space(), 0.5);
        let chunks = fixtures::chunks();
        let (first, rest) = chunks.split_at(1);
        index
            .add(&entries(&embedder, first.to_vec()).await)
            .await
            .unwrap();
        let query = embedder.embed_query("Gerät geprüft").await.unwrap();
        let before = search_all(&index, &query).await;
        for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            // Valid entries before and after the bad one, and a replacement of the
            // stored chunk: none of them is added.
            let mut bad = with_chunk_box(value);
            bad.id = ChunkId::new(&DocumentId::new("neu"), 0);
            let mut replacement = first[0].clone();
            replacement.text = "ersetzt".into();
            let mut call = rest.to_vec();
            call.insert(1, bad);
            call.push(replacement);
            let result = index.add(&entries(&embedder, call).await).await;
            assert!(is_invalid_input(&result), "{value}: {result:?}");
            assert_eq!(index.len(), 1, "{value}");
            assert_eq!(search_all(&index, &query).await, before, "{value}");
        }
    });
}
