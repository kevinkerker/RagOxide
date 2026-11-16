//! Every mock implements its trait, works as a trait object and keeps the documented
//! contract.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use futures::executor::block_on;
use futures::{FutureExt, StreamExt};
use kb_core::mock::*;
use kb_core::*;

fn manifest() -> ModelManifest {
    ModelManifest::from_toml_str(include_str!("../../models.toml")).expect("models.toml")
}

#[test]
fn every_mock_is_usable_as_trait_object() {
    let store: Arc<dyn Store> = Arc::new(MockStore::new());
    let embedder: Arc<dyn Embedder> = Arc::new(MockEmbedder::default());
    let index: Arc<dyn Index> = Arc::new(MockIndex::new(embedder.space(), 0.5));
    let reranker: Arc<dyn Reranker> = Arc::new(MockReranker);
    let generator: Arc<dyn Generator> = Arc::new(MockGenerator::new("ok"));
    let tokens: Arc<dyn TokenCounter> = Arc::new(MockTokenCounter);
    let parser: Arc<dyn Parser> = Arc::new(MockParser::new());
    let chunker: Arc<dyn Chunker> = Arc::new(MockChunker::new(Arc::clone(&tokens), 400));
    let _hub: Arc<dyn ModelHub> = Arc::new(MockModelHub::new(
        manifest(),
        HardwareInfo {
            total_ram_bytes: 16 << 30,
            logical_cpus: 8,
        },
        PathBuf::from(r"C:\Modelle"),
    ));
    let _ingestor: Arc<dyn Ingestor> = Arc::new(MockIngestor::new(
        parser,
        chunker,
        Arc::clone(&embedder),
        Arc::clone(&index),
        store,
        Arc::new(MockFiles::fixtures()),
    ));
    let _answerer: Arc<dyn Answerer> = Arc::new(MockAnswerer::new(
        embedder,
        index,
        Some(reranker),
        generator,
        PipelineConfig::default(),
    ));
}

fn source(name: &str) -> SourceFile {
    let path = Path::new(r"D:\Eingang").join(name);
    let hash = placeholder_hash(&path).unwrap();
    SourceFile::new(DocumentId::new("doc-42"), &path, hash).unwrap()
}

#[test]
fn parser_returns_fixture_with_requested_source() {
    block_on(async {
        let parser = MockParser::new();
        let source = source("pruefordnung.pdf");
        let document = parser
            .parse(&source, CancellationToken::new())
            .await
            .unwrap();
        assert_eq!(document.source, source);
        assert_eq!(document.format, DocumentFormat::Pdf);

        let error = parser
            .parse(&self::source("notes.txt"), CancellationToken::new())
            .await
            .unwrap_err();
        assert!(matches!(error, Error::UnsupportedFormat(_)));
        let error = parser
            .parse(&self::source("unknown.pdf"), CancellationToken::new())
            .await
            .unwrap_err();
        assert!(matches!(error, Error::Parse(_)));

        let cancel = CancellationToken::new();
        cancel.cancel();
        let error = parser.parse(&source, cancel).await.unwrap_err();
        assert!(matches!(error, Error::Cancelled));
    });
}

#[test]
fn chunker_skips_headings_furniture_and_empty_pictures() {
    block_on(async {
        let documents = fixtures::documents();
        let chunker = MockChunker::default();
        let chunks = chunker.chunk(&documents["schulung.pptx"]).await.unwrap();
        let texts: Vec<&str> = chunks.iter().map(|c| c.text.as_str()).collect();
        assert_eq!(
            texts,
            [
                "Grundregeln für alle Beschäftigten",
                "Fluchtwege müssen jederzeit frei bleiben.",
                "Feuerlöscher hängen neben jeder Tür.",
                "Notiz für die Vortragenden: Wie oft die Fluchtwege geprüft werden, regelt die Prüfordnung. Der Hausmeister geht sie monatlich ab.",
                "Abbildung 1: Lageplan des Erdgeschosses",
            ]
        );
        assert_eq!(
            chunks[1].provenance.heading_path,
            ["Sicherheitsunterweisung 2026", "Fluchtwege"]
        );
        let pdf = chunker.chunk(&documents["pruefordnung.pdf"]).await.unwrap();
        assert!(pdf.iter().all(|c| !c.text.contains("Seite 2 von 2")));
    });
}

#[test]
fn no_chunk_exceeds_the_limit() {
    block_on(async {
        for limit in [3, 4, 5, 10, 22, 512] {
            let chunker = MockChunker::new(Arc::new(MockTokenCounter), limit);
            for (file, document) in fixtures::documents() {
                let chunks = chunker.chunk(&document).await.unwrap();
                for (ordinal, chunk) in chunks.iter().enumerate() {
                    let tokens = MockTokenCounter.count_tokens(&chunk.text).await.unwrap();
                    assert!(tokens <= limit, "{file} limit {limit}: {:?}", chunk.text);
                    assert!(!chunk.provenance.anchors.is_empty());
                    assert_eq!(
                        chunk.id,
                        ChunkId::new(&document.source.document_id, ordinal as u32)
                    );
                }
            }
        }
    });
}

fn sheet_range(start_row: u32, end_row: u32) -> Vec<Anchor> {
    vec![Anchor::Sheet {
        sheet: "Budget".into(),
        range: CellRange {
            start_row,
            start_col: 1,
            end_row,
            end_col: 3,
        },
    }]
}

#[test]
fn table_over_the_limit_is_split_at_rows_with_repeated_header() {
    block_on(async {
        let documents = fixtures::documents();
        let budget = &documents["budget.xlsx"];
        // The whole table has 28 tokens, header plus one row 21.
        let chunker = MockChunker::new(Arc::new(MockTokenCounter), 22);
        let chunks = chunker.chunk(budget).await.unwrap();
        let table: Vec<&Chunk> = chunks.iter().filter(|c| c.text.starts_with('|')).collect();
        assert_eq!(table.len(), 2);
        assert_eq!(
            table[0].text,
            "| Category | Q1 | Q2 |\n| --- | --- | --- |\n| Travel | 12,000 | 9,500 |"
        );
        assert_eq!(
            table[1].text,
            "| Category | Q1 | Q2 |\n| --- | --- | --- |\n| Training | 8,000 | 11,000 |"
        );
        // Each part is anchored to its own rows (B3:D4 with the header, then B5:D5).
        assert_eq!(table[0].provenance.anchors, sheet_range(2, 3));
        assert_eq!(table[1].provenance.anchors, sheet_range(4, 4));

        // PDF: the table has 30 tokens; each part gets one box around its cells.
        let chunker = MockChunker::new(Arc::new(MockTokenCounter), 29);
        let chunks = chunker.chunk(&documents["pruefordnung.pdf"]).await.unwrap();
        let parts: Vec<&Chunk> = chunks
            .iter()
            .filter(|c| c.text.starts_with("| Gerät"))
            .collect();
        assert_eq!(parts.len(), 2);
        let Anchor::Page { page, bbox } = &parts[1].provenance.anchors[0] else {
            panic!("page anchor expected");
        };
        assert_eq!(*page, 2);
        assert_eq!((bbox.top, bbox.bottom), (213.0, 240.0));
    });
}

#[test]
fn row_over_the_limit_is_split_like_text_and_keeps_its_anchor() {
    block_on(async {
        let documents = fixtures::documents();
        let chunker = MockChunker::new(Arc::new(MockTokenCounter), 4);
        let chunks = chunker.chunk(&documents["budget.xlsx"]).await.unwrap();
        let travel: Vec<&Chunk> = chunks
            .iter()
            .filter(|c| c.provenance.anchors == sheet_range(2, 3))
            .collect();
        assert_eq!(travel.len(), 2);
        assert_eq!(travel[0].text, "Travel | 12,000 |");
        assert_eq!(travel[1].text, "9,500");
    });
}

#[test]
fn embedder_returns_normalised_vectors_of_its_space() {
    block_on(async {
        let embedder = MockEmbedder::new(16);
        assert_eq!(embedder.space().dimension, 16);
        let chunks = fixtures::chunks();
        let vectors = embedder
            .embed_documents(&chunks, CancellationToken::new())
            .await
            .unwrap();
        assert_eq!(vectors.len(), chunks.len());
        let query = embedder.embed_query("Prüffristen").await.unwrap();
        for vector in vectors.iter().chain([&query]) {
            assert_eq!(vector.len(), 16);
            let norm: f32 = vector.iter().map(|v| v * v).sum::<f32>().sqrt();
            assert!((norm - 1.0).abs() < 1e-5);
        }
        let cancel = CancellationToken::new();
        cancel.cancel();
        let error = embedder.embed_documents(&chunks, cancel).await.unwrap_err();
        assert!(matches!(error, Error::Cancelled));
    });
}

#[test]
fn reranker_scores_in_unit_interval_in_input_order() {
    block_on(async {
        let chunks = fixtures::chunks();
        let scores = MockReranker
            .rerank("Fluchtwege frei", &chunks, CancellationToken::new())
            .await
            .unwrap();
        assert_eq!(scores.len(), chunks.len());
        assert!(scores.iter().all(|s| (0.0..=1.0).contains(s)));
        let best = scores.iter().cloned().fold(0.0, f32::max);
        let best_chunk = &chunks[scores.iter().position(|s| *s == best).unwrap()];
        assert!(best_chunk.text.starts_with("Fluchtwege müssen"));
        let cancel = CancellationToken::new();
        cancel.cancel();
        let error = MockReranker.rerank("x", &chunks, cancel).await.unwrap_err();
        assert!(matches!(error, Error::Cancelled));
    });
}

async fn filled_index() -> (MockEmbedder, MockIndex) {
    let embedder = MockEmbedder::default();
    let index = MockIndex::new(embedder.space(), 0.5);
    let chunks = fixtures::chunks();
    let embeddings = embedder
        .embed_documents(&chunks, CancellationToken::new())
        .await
        .unwrap();
    let entries: Vec<IndexEntry> = chunks
        .into_iter()
        .zip(embeddings)
        .map(|(chunk, embedding)| IndexEntry { chunk, embedding })
        .collect();
    index.add(&entries).await.unwrap();
    (embedder, index)
}

fn request(text: &str, limit: u32, mode: SearchMode) -> SearchRequest {
    SearchRequest {
        mode,
        ..SearchRequest::new(text, limit)
    }
}

#[test]
fn index_searches_in_all_modes() {
    block_on(async {
        let (embedder, index) = filled_index().await;
        let query = embedder
            .embed_query("Travel expenses receipts")
            .await
            .unwrap();
        for mode in [SearchMode::Hybrid, SearchMode::Vector, SearchMode::FullText] {
            let hits = index
                .search(&request("Travel expenses receipts", 3, mode), &query)
                .await
                .unwrap();
            assert!(!hits.is_empty() && hits.len() <= 3, "{mode:?}");
            assert_eq!(
                hits[0].chunk.document_id().as_str(),
                "doc-onboarding",
                "{mode:?}"
            );
            assert!(hits.iter().all(|h| h.rerank_score.is_none()));
            assert!(
                hits.iter()
                    .all(|h| h.vector_score.is_some() == (mode != SearchMode::FullText))
            );
            assert!(
                hits.windows(2)
                    .all(|w| w[0].retrieval_score >= w[1].retrieval_score)
            );
        }
    });
}

#[test]
fn index_returns_nearest_chunks_in_vector_mode_and_only_matches_in_full_text_mode() {
    block_on(async {
        let (embedder, index) = filled_index().await;
        let query = embedder.embed_query("Quantenchromodynamik").await.unwrap();
        let all = index
            .search(
                &request("Quantenchromodynamik", 1000, SearchMode::Vector),
                &query,
            )
            .await
            .unwrap();
        assert_eq!(all.len(), index.len());
        let none = index
            .search(
                &request("Quantenchromodynamik", 1000, SearchMode::FullText),
                &query,
            )
            .await
            .unwrap();
        assert!(none.is_empty());
    });
}

#[test]
fn index_document_filter_none_some_and_empty() {
    block_on(async {
        let (embedder, index) = filled_index().await;
        let query = embedder.embed_query("Travel expenses").await.unwrap();
        let only_budget = SearchRequest {
            documents: Some(vec![DocumentId::new("doc-budget")]),
            ..SearchRequest::new("Travel expenses", 100)
        };
        let hits = index.search(&only_budget, &query).await.unwrap();
        assert!(!hits.is_empty());
        assert!(
            hits.iter()
                .all(|h| h.chunk.document_id().as_str() == "doc-budget")
        );
        for mode in [SearchMode::Hybrid, SearchMode::Vector, SearchMode::FullText] {
            let empty_filter = SearchRequest {
                documents: Some(Vec::new()),
                mode,
                ..SearchRequest::new("Travel expenses", 100)
            };
            assert!(
                index
                    .search(&empty_filter, &query)
                    .await
                    .unwrap()
                    .is_empty()
            );
        }
    });
}

#[test]
fn index_replaces_deletes_and_rejects_other_spaces() {
    block_on(async {
        let (embedder, index) = filled_index().await;
        let total = index.len();
        let first = fixtures::chunks().remove(0);
        let embedding = embedder
            .embed_documents(std::slice::from_ref(&first), CancellationToken::new())
            .await
            .unwrap()
            .remove(0);
        index
            .add(&[IndexEntry {
                chunk: first.clone(),
                embedding,
            }])
            .await
            .unwrap();
        assert_eq!(index.len(), total);

        let id = DocumentId::new("doc-onboarding");
        index.delete_document(&id).await.unwrap();
        let query = embedder.embed_query("Travel").await.unwrap();
        let hits = index
            .search(&SearchRequest::new("Travel", 100), &query)
            .await
            .unwrap();
        assert!(hits.iter().all(|h| h.chunk.document_id() != &id));
        index.delete_document(&id).await.unwrap();

        let error = index
            .add(&[IndexEntry {
                chunk: first,
                embedding: vec![1.0],
            }])
            .await
            .unwrap_err();
        assert!(matches!(error, Error::EmbeddingMismatch(_)));
        let error = index
            .search(&SearchRequest::new("x", 1), &[1.0])
            .await
            .unwrap_err();
        assert!(matches!(error, Error::EmbeddingMismatch(_)));
        assert_eq!(index.space(), embedder.space());
    });
}

fn generation(text: &str) -> GenerationRequest {
    GenerationRequest {
        level: AnswerLevel::Fast,
        messages: vec![Message {
            role: Role::User,
            content: text.into(),
        }],
        max_tokens: None,
        temperature: None,
    }
}

#[test]
fn generator_streams_phases_then_text_records_and_stops_on_cancel() {
    block_on(async {
        let generator = MockGenerator::new("Jährlich, siehe [1].");
        let events: Vec<GenerationEvent> = generator
            .generate(
                generation("Wie oft wird geprüft?"),
                CancellationToken::new(),
            )
            .await
            .unwrap()
            .map(|p| p.unwrap())
            .collect()
            .await;
        let phase = |phase, progress| GenerationEvent::Phase { phase, progress };
        let text = |t: &str| GenerationEvent::Text(t.into());
        assert_eq!(
            events,
            [
                phase(AnswerPhase::StartingModel, None),
                phase(
                    AnswerPhase::ReadingSources,
                    Some(PhaseProgress { done: 2, total: 4 })
                ),
                phase(
                    AnswerPhase::ReadingSources,
                    Some(PhaseProgress { done: 4, total: 4 })
                ),
                phase(AnswerPhase::Writing, None),
                text("Jährlich, "),
                text("siehe "),
                text("[1]."),
            ]
        );
        assert_eq!(generator.requests(), [generation("Wie oft wird geprüft?")]);

        // The model of the level is loaded now: no StartingModel on the next call.
        let cancel = CancellationToken::new();
        let mut stream = generator
            .generate(generation("Wie oft?"), cancel.clone())
            .await
            .unwrap();
        assert!(matches!(
            stream.next().await,
            Some(Ok(GenerationEvent::Phase {
                phase: AnswerPhase::ReadingSources,
                ..
            }))
        ));
        cancel.cancel();
        assert!(matches!(stream.next().await, Some(Err(Error::Cancelled))));
        assert!(stream.next().await.is_none());

        // Another level replaces the loaded model, and counting loads too.
        let balanced = GenerationRequest {
            level: AnswerLevel::Balanced,
            ..generation("x")
        };
        let mut stream = generator
            .generate(balanced, CancellationToken::new())
            .await
            .unwrap();
        assert!(matches!(
            stream.next().await,
            Some(Ok(GenerationEvent::Phase {
                phase: AnswerPhase::StartingModel,
                ..
            }))
        ));
        assert_eq!(generator.loaded(), Some(AnswerLevel::Balanced));
        assert_eq!(
            generator
                .count_tokens(AnswerLevel::Fast, "zwei Wörter")
                .await
                .unwrap(),
            2
        );
        assert_eq!(generator.loaded(), Some(AnswerLevel::Fast));

        let crashing = MockGenerator::failing_after("a b c", 2);
        let items: Vec<Result<GenerationEvent>> = crashing
            .generate(generation("x"), CancellationToken::new())
            .await
            .unwrap()
            .collect()
            .await;
        assert!(matches!(items.last(), Some(Err(Error::Generator(_)))));
        let texts = items
            .iter()
            .filter(|i| matches!(i, Ok(GenerationEvent::Text(_))))
            .count();
        assert_eq!(texts, 2);
    });
}

#[test]
fn token_counter_counts_words() {
    block_on(async {
        let count = MockTokenCounter
            .count_tokens("| Gerät | Prüfdatum |")
            .await
            .unwrap();
        assert_eq!(count, 5);
    });
}

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

#[test]
fn store_keeps_documents_and_finds_them_by_path() {
    block_on(async {
        let store = MockStore::new();
        store
            .upsert_document(&record("b", r"C:\Ablage\B.pdf"))
            .await
            .unwrap();
        store
            .upsert_document(&record("a", r"C:\Ablage\a.pdf"))
            .await
            .unwrap();
        let order: Vec<String> = store
            .documents()
            .await
            .unwrap()
            .into_iter()
            .map(|r| r.id.0)
            .collect();
        assert_eq!(order, ["a", "b"], "sorted by path_key, not by byte order");
        let found = store
            .document_by_path(Path::new("c:/ablage/./b.PDF"))
            .await
            .unwrap();
        assert_eq!(found.unwrap().id.as_str(), "b");

        let document = fixtures::documents().remove("schulung.pptx").unwrap();
        let id = document.source.document_id.clone();
        store.save_content(&document).await.unwrap();
        assert_eq!(store.content(&id).await.unwrap(), Some(document));
        store.delete_document(&id).await.unwrap();
        assert!(store.content(&id).await.unwrap().is_none());
        store.delete_document(&id).await.unwrap();
    });
}

#[test]
fn store_keeps_chats_and_settings() {
    block_on(async {
        let store = MockStore::new();
        let chat = |id: &str, created: u64, updated: u64| Chat {
            id: ChatId::new(id),
            title: format!("Chat {id}"),
            created_at: Timestamp(created),
            updated_at: Timestamp(updated),
            turns: Vec::new(),
        };
        // `a` was created first but asked last; `c` and `b` tie and go by id.
        store.save_chat(&chat("a", 100, 900)).await.unwrap();
        store.save_chat(&chat("c", 300, 500)).await.unwrap();
        store.save_chat(&chat("b", 200, 500)).await.unwrap();
        let order: Vec<String> = store
            .chats()
            .await
            .unwrap()
            .into_iter()
            .map(|c| c.id.0)
            .collect();
        assert_eq!(order, ["a", "b", "c"], "by updated_at, newest first");
        let summary = &store.chats().await.unwrap()[0];
        assert_eq!(
            (summary.created_at, summary.updated_at),
            (Timestamp(100), Timestamp(900))
        );
        store.delete_chat(&ChatId::new("a")).await.unwrap();
        assert!(store.chat(&ChatId::new("a")).await.unwrap().is_none());

        assert_eq!(store.settings().await.unwrap(), Settings::default());
        let settings = Settings {
            answer_level: AnswerLevel::Balanced,
            ui_language: UiLanguage::English,
            ..Settings::default()
        };
        store.save_settings(&settings).await.unwrap();
        assert_eq!(store.settings().await.unwrap(), settings);
    });
}

#[test]
fn model_hub_provides_one_model_per_role_of_the_tier() {
    block_on(async {
        let hardware = HardwareInfo {
            total_ram_bytes: 8_000_000_000,
            logical_cpus: 4,
        };
        let hub = MockModelHub::new(manifest(), hardware, PathBuf::from(r"C:\Modelle"));
        let tier = HardwareTier::from_total_ram(hub.hardware().unwrap().total_ram_bytes);
        assert_eq!(tier, HardwareTier::BelowMinimum);
        let events = Arc::new(Mutex::new(Vec::new()));
        let sink = Arc::clone(&events);
        let models = hub
            .provide(
                tier,
                AnswerLevel::Fast,
                None,
                Arc::new(move |e| sink.lock().unwrap().push(e)),
                CancellationToken::new(),
            )
            .await
            .unwrap();
        let generator = models
            .iter()
            .find(|m| m.entry.role == ModelRole::Generator)
            .unwrap();
        assert_eq!(generator.entry.id, "gemma-4-e2b");
        assert_eq!(generator.dir, PathBuf::from(r"C:\Modelle\gemma-4-e2b"));
        let files: usize = models.iter().map(|m| m.entry.files.len()).sum();
        let ready = events
            .lock()
            .unwrap()
            .iter()
            .filter(|e| e.status == ModelFileStatus::Ready)
            .count();
        assert_eq!(ready, files);
        assert!(
            models.iter().all(|m| m.entry.role != ModelRole::Reranker),
            "no reranker below the minimum"
        );
        let error = hub
            .provide(
                tier,
                AnswerLevel::Balanced,
                None,
                Arc::new(|_| {}),
                CancellationToken::new(),
            )
            .await
            .unwrap_err();
        assert!(
            matches!(error, Error::InvalidInput(_)),
            "higher levels are not offered below the minimum"
        );

        let cancel = CancellationToken::new();
        cancel.cancel();
        let error = hub
            .provide(tier, AnswerLevel::Fast, None, Arc::new(|_| {}), cancel)
            .await
            .unwrap_err();
        assert!(matches!(error, Error::Cancelled));
    });
}

#[test]
fn hardware_tier_thresholds() {
    const GIB: u64 = 1 << 30;
    const MIB: u64 = 1 << 20;
    // A 16 GB laptop typically reports about 15.7 GiB, a 12 GB one about 11.7 GiB.
    let tier = HardwareTier::from_total_ram;
    assert_eq!(tier(16 * GIB - 300 * MIB), HardwareTier::Supported);
    assert_eq!(tier(15 * GIB), HardwareTier::Supported);
    assert_eq!(tier(15 * GIB - 1), HardwareTier::BelowMinimum);
    assert_eq!(tier(12 * GIB - 300 * MIB), HardwareTier::BelowMinimum);
    assert_eq!(tier(2 * GIB), HardwareTier::BelowMinimum, "no lower limit");
}

#[test]
fn answerer_search_reranks_and_limits() {
    block_on(async {
        let (embedder, index) = filled_index().await;
        let answerer = MockAnswerer::new(
            Arc::new(embedder),
            Arc::new(index),
            Some(Arc::new(MockReranker)),
            Arc::new(MockGenerator::new("")),
            PipelineConfig::default(),
        );
        let hits = answerer
            .search(
                &SearchRequest::new(fixtures::HITS_QUERY, 4),
                CancellationToken::new(),
            )
            .await
            .unwrap();
        assert_eq!(hits, fixtures::hits());
    });
}

#[test]
fn document_format_from_path() {
    let cases = [
        ("a.PDF", Some(DocumentFormat::Pdf)),
        ("scan.jpeg", Some(DocumentFormat::Image)),
        ("scan.tiff", Some(DocumentFormat::Image)),
        ("b.Docx", Some(DocumentFormat::Docx)),
        ("c.pptx", Some(DocumentFormat::Pptx)),
        ("d.xlsx", Some(DocumentFormat::Xlsx)),
        ("e.htm", Some(DocumentFormat::Html)),
        ("f.md", Some(DocumentFormat::Markdown)),
        ("g.bmp", None),
        ("h.webp", None),
        ("no_extension", None),
    ];
    for (path, expected) in cases {
        assert_eq!(
            DocumentFormat::from_path(Path::new(path)),
            expected,
            "{path}"
        );
    }
}

#[cfg(windows)]
#[test]
fn canonical_path_and_key_on_windows() {
    let canonical = |p: &str| canonical_path(Path::new(p)).unwrap();
    assert_eq!(
        canonical(r"c:/Ablage/./Berichte/../Prüfplan.PDF"),
        PathBuf::from(r"C:\Ablage\Prüfplan.PDF")
    );
    assert_eq!(
        canonical(r"\\?\C:\Ablage\x.pdf"),
        PathBuf::from(r"C:\Ablage\x.pdf")
    );
    assert_eq!(
        canonical(r"\\server\share\Ordner\"),
        PathBuf::from(r"\\server\share\Ordner")
    );
    assert_eq!(canonical(r"C:\..\x.pdf"), PathBuf::from(r"C:\x.pdf"));
    assert!(matches!(
        canonical_path(Path::new(r"Ablage\x.pdf")),
        Err(Error::InvalidInput(_))
    ));
    assert_eq!(
        path_key(Path::new(r"C:\ABLAGE\Übersicht.pdf")).unwrap(),
        path_key(Path::new(r"c:/ablage/übersicht.PDF")).unwrap()
    );
    let source = SourceFile::new(DocumentId::new("d"), Path::new("c:/a/B.pdf"), "h").unwrap();
    assert_eq!(
        (source.path, source.name),
        (PathBuf::from(r"C:\a\B.pdf"), "B.pdf".to_string())
    );
}

#[test]
fn invalid_input_is_rejected_by_every_method() {
    block_on(async {
        let (embedder, index) = filled_index().await;
        let query = embedder.embed_query("Fluchtwege").await.unwrap();
        let invalid = [
            SearchRequest::new("  ", 3),
            SearchRequest::new("Fluchtwege", 0),
        ];
        for request in &invalid {
            let error = index.search(request, &query).await.unwrap_err();
            assert!(matches!(error, Error::InvalidInput(_)), "{request:?}");
        }
        let error = embedder.embed_query(" ").await.unwrap_err();
        assert!(matches!(error, Error::InvalidInput(_)));
        let error = MockReranker
            .rerank("", &fixtures::chunks(), CancellationToken::new())
            .await
            .unwrap_err();
        assert!(matches!(error, Error::InvalidInput(_)));

        let answerer = MockAnswerer::new(
            Arc::new(embedder),
            Arc::new(index),
            Some(Arc::new(MockReranker)),
            Arc::new(MockGenerator::new("")),
            PipelineConfig::default(),
        );
        for request in &invalid {
            let error = answerer
                .search(request, CancellationToken::new())
                .await
                .unwrap_err();
            assert!(matches!(error, Error::InvalidInput(_)), "{request:?}");
        }
        let error = answerer
            .ask(
                " ",
                &[],
                AnswerLevel::Fast,
                UiLanguage::German,
                CancellationToken::new(),
            )
            .await
            .err()
            .unwrap();
        assert!(matches!(error, Error::InvalidInput(_)));
        let cancel = CancellationToken::new();
        cancel.cancel();
        let error = answerer
            .search(&SearchRequest::new("Fluchtwege", 3), cancel)
            .await
            .unwrap_err();
        assert!(matches!(error, Error::Cancelled));
    });
}

#[test]
fn embedder_normalises_text_without_words() {
    block_on(async {
        let vector = MockEmbedder::default().embed_query("?!").await.unwrap();
        let norm: f32 = vector.iter().map(|v| v * v).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-5);
    });
}

/// A table whose second and third body rows are joined by a cell with `row_span: 2`.
fn spanned_table() -> Document {
    let mut document = fixtures::documents().remove("budget.xlsx").unwrap();
    let cell = |row: u32, col: u32, text: &str, row_span: u32| TableCell {
        row,
        col,
        row_span,
        col_span: 1,
        text: text.into(),
        column_header: row == 0,
        row_header: false,
        anchor: Some(Anchor::Sheet {
            sheet: "Plan".into(),
            range: CellRange {
                start_row: row,
                start_col: col,
                end_row: row + row_span - 1,
                end_col: col,
            },
        }),
    };
    let cells = vec![
        cell(0, 0, "Team", 1),
        cell(0, 1, "Task", 1),
        cell(1, 0, "Alpha", 1),
        cell(1, 1, "Audit", 1),
        cell(2, 0, "Beta", 2),
        cell(2, 1, "Review", 1),
        cell(3, 1, "Report", 1),
        cell(4, 0, "Gamma", 1),
        cell(4, 1, "Archive", 1),
    ];
    document.elements = vec![Element {
        kind: ElementKind::Table(Table {
            num_rows: 5,
            num_cols: 2,
            cells,
        }),
        text: ["cell"; 30].join(" "),
        anchors: vec![Anchor::Sheet {
            sheet: "Plan".into(),
            range: CellRange {
                start_row: 0,
                start_col: 0,
                end_row: 4,
                end_col: 1,
            },
        }],
    }];
    document
}

#[test]
fn split_never_cuts_through_a_row_span() {
    block_on(async {
        let document = spanned_table();
        // Header (2 lines) plus one plain row: 15 tokens. The joined rows 2 and 3: 20.
        let chunker = MockChunker::new(Arc::new(MockTokenCounter), 20);
        let chunks = chunker.chunk(&document).await.unwrap();
        let texts: Vec<&str> = chunks.iter().map(|c| c.text.as_str()).collect();
        assert_eq!(
            texts,
            [
                "| Team | Task |\n| --- | --- |\n| Alpha | Audit |",
                "| Team | Task |\n| --- | --- |\n| Beta | Review |\n|  | Report |",
                "| Team | Task |\n| --- | --- |\n| Gamma | Archive |",
            ]
        );
        assert!(
            chunks
                .iter()
                .all(|c| !c.text.contains("Review") || c.text.contains("Report"))
        );

        // A row group that alone exceeds the limit is split like text, as one unit.
        let chunker = MockChunker::new(Arc::new(MockTokenCounter), 4);
        let chunks = chunker.chunk(&document).await.unwrap();
        let group: Vec<&str> = chunks
            .iter()
            .filter(|c| {
                c.provenance.anchors
                    == vec![Anchor::Sheet {
                        sheet: "Plan".into(),
                        range: CellRange {
                            start_row: 2,
                            start_col: 0,
                            end_row: 3,
                            end_col: 1,
                        },
                    }]
            })
            .map(|c| c.text.as_str())
            .collect();
        assert_eq!(group, ["Beta | Review |", "Report"]);
    });
}

#[test]
fn pipeline_config_context_holds_prompt_and_answer() {
    let config = PipelineConfig::default();
    config.validate().unwrap();
    assert!(config.prompt_token_budget + config.max_answer_tokens <= config.context_tokens);
    let too_small = PipelineConfig {
        context_tokens: config.prompt_token_budget,
        ..config
    };
    assert!(matches!(too_small.validate(), Err(Error::Config(_))));
}

#[test]
fn path_key_normalises_unicode() {
    let dir = std::env::temp_dir();
    let composed = dir.join("\u{dc}bersicht.pdf");
    let decomposed = dir.join("U\u{308}bersicht.pdf");
    assert_ne!(composed, decomposed);
    assert_eq!(path_key(&composed).unwrap(), path_key(&decomposed).unwrap());
}

#[test]
fn store_updates_settings_atomically_and_not_on_error() {
    block_on(async {
        let store = MockStore::new();
        let saved = store
            .update_settings(Box::new(|s| {
                s.color_scheme = ColorScheme::Dark;
                Ok(())
            }))
            .await
            .unwrap();
        assert_eq!(saved.color_scheme, ColorScheme::Dark);
        let error = store
            .update_settings(Box::new(|s| {
                s.color_scheme = ColorScheme::Light;
                Err(Error::InvalidInput("rejected".into()))
            }))
            .await
            .unwrap_err();
        assert!(matches!(error, Error::InvalidInput(_)));
        assert_eq!(
            store.settings().await.unwrap().color_scheme,
            ColorScheme::Dark
        );
    });
}

#[test]
fn model_hub_reports_missing_files_per_level_without_providing() {
    block_on(async {
        let hardware = HardwareInfo {
            total_ram_bytes: 16 << 30,
            logical_cpus: 8,
        };
        let hub = MockModelHub::new(manifest(), hardware, PathBuf::from(r"C:\Modelle"));
        let tier = HardwareTier::Supported;
        let missing = hub.missing(tier, AnswerLevel::Fast).await.unwrap();
        let files: usize = manifest()
            .models_for(tier, AnswerLevel::Fast)
            .unwrap()
            .iter()
            .map(|m| m.files.len())
            .sum();
        assert_eq!(missing.len(), files, "nothing provided yet");
        assert_eq!(
            hub.missing(tier, AnswerLevel::Fast).await.unwrap(),
            missing,
            "missing() loads nothing"
        );

        // The setup provides the fast level; a higher level then misses only its model.
        hub.provide(
            tier,
            AnswerLevel::Fast,
            None,
            Arc::new(|_| {}),
            CancellationToken::new(),
        )
        .await
        .unwrap();
        assert!(
            hub.missing(tier, AnswerLevel::Fast)
                .await
                .unwrap()
                .is_empty()
        );
        let models = |level| {
            let hub = &hub;
            async move {
                let mut ids: Vec<String> = hub
                    .missing(tier, level)
                    .await
                    .unwrap()
                    .into_iter()
                    .map(|m| m.model_id)
                    .collect();
                ids.dedup();
                ids
            }
        };
        assert_eq!(models(AnswerLevel::Balanced).await, ["gemma-4-e4b"]);
        assert_eq!(models(AnswerLevel::Thorough).await, ["gemma-4-12b"]);

        let resolved = hub
            .provide(
                tier,
                AnswerLevel::Thorough,
                None,
                Arc::new(|_| {}),
                CancellationToken::new(),
            )
            .await
            .unwrap();
        assert_eq!(resolved.last().unwrap().entry.id, "gemma-4-12b");
        assert!(
            hub.missing(tier, AnswerLevel::Thorough)
                .await
                .unwrap()
                .is_empty()
        );
        assert_eq!(models(AnswerLevel::Balanced).await, ["gemma-4-e4b"]);
    });
}

#[test]
fn error_kind_names_the_variant() {
    let cases = [
        (Error::Cancelled, ErrorKind::Cancelled),
        (Error::NotFound("x".into()), ErrorKind::NotFound),
        (
            Error::UnsupportedFormat("x".into()),
            ErrorKind::UnsupportedFormat,
        ),
        (Error::InvalidInput("x".into()), ErrorKind::InvalidInput),
        (Error::Parse("x".into()), ErrorKind::Parse),
        (Error::Chunk("x".into()), ErrorKind::Chunk),
        (Error::Model("x".into()), ErrorKind::Model),
        (
            Error::EmbeddingMismatch("x".into()),
            ErrorKind::EmbeddingMismatch,
        ),
        (Error::Index("x".into()), ErrorKind::Index),
        (Error::Network("x".into()), ErrorKind::Network),
        (Error::StorageFull("x".into()), ErrorKind::StorageFull),
        (
            Error::ChecksumMismatch("x".into()),
            ErrorKind::ChecksumMismatch,
        ),
        (Error::ModelMissing("x".into()), ErrorKind::ModelMissing),
        (Error::Generator("x".into()), ErrorKind::Generator),
        (Error::Store("x".into()), ErrorKind::Store),
        (Error::Config("x".into()), ErrorKind::Config),
        (Error::Io(std::io::Error::other("x")), ErrorKind::Io),
    ];
    for (error, kind) in cases {
        assert_eq!(error.kind(), kind, "{error}");
    }
}

#[test]
fn content_hash_is_sha256_of_all_bytes() {
    let abc = content_hash(&b"abc"[..]).unwrap();
    assert_eq!(
        abc,
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    // Longer than the read buffer: one million times "a".
    let million = content_hash(std::io::Read::take(std::io::repeat(b'a'), 1_000_000)).unwrap();
    assert_eq!(
        million,
        "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0"
    );
}

#[test]
fn model_hub_reports_storage_copies_from_a_package_and_names_a_full_disk() {
    block_on(async {
        let hardware = HardwareInfo {
            total_ram_bytes: 16 << 30,
            logical_cpus: 8,
        };
        let hub = MockModelHub::new(manifest(), hardware, PathBuf::from(r"C:\Modelle"));
        let storage = hub.storage().unwrap();
        assert_eq!(storage.dir, PathBuf::from(r"C:\Modelle"));
        assert!(storage.free_bytes > 0);

        let events = Arc::new(Mutex::new(Vec::new()));
        let sink = Arc::clone(&events);
        hub.provide(
            HardwareTier::Supported,
            AnswerLevel::Fast,
            Some(PathBuf::from(r"E:\RagOxide-Modelle")),
            Arc::new(move |e| sink.lock().unwrap().push(e)),
            CancellationToken::new(),
        )
        .await
        .unwrap();
        assert!(
            events
                .lock()
                .unwrap()
                .iter()
                .any(|e| matches!(e.status, ModelFileStatus::Copying { .. }))
        );

        let full = MockModelHub::new(manifest(), hardware, PathBuf::from(r"C:\Modelle"))
            .with_free_bytes(1_000_000_000);
        let error = full
            .provide(
                HardwareTier::Supported,
                AnswerLevel::Fast,
                None,
                Arc::new(|_| {}),
                CancellationToken::new(),
            )
            .await
            .unwrap_err();
        assert_eq!(error.kind(), ErrorKind::StorageFull);
    });
}

#[test]
fn models_are_provided_in_role_order_so_ingestion_can_start_early() {
    block_on(async {
        let hardware = HardwareInfo {
            total_ram_bytes: 16 << 30,
            logical_cpus: 8,
        };
        let hub = MockModelHub::new(manifest(), hardware, PathBuf::from(r"C:\Modelle"));
        let missing = hub
            .missing(HardwareTier::Supported, AnswerLevel::Balanced)
            .await
            .unwrap();
        assert!(missing.iter().any(|m| m.role == ModelRole::Generator));
        assert!(missing.iter().any(|m| m.role.is_needed_for_ingestion()));

        let events = Arc::new(Mutex::new(Vec::new()));
        let sink = Arc::clone(&events);
        hub.provide(
            HardwareTier::Supported,
            AnswerLevel::Balanced,
            None,
            Arc::new(move |e| sink.lock().unwrap().push(e)),
            CancellationToken::new(),
        )
        .await
        .unwrap();
        let rank = |role: ModelRole| {
            ModelRole::PROVIDE_ORDER
                .iter()
                .position(|r| *r == role)
                .unwrap()
        };
        let order: Vec<usize> = events
            .lock()
            .unwrap()
            .iter()
            .map(|e| rank(e.role))
            .collect();
        assert!(
            order.windows(2).all(|w| w[0] <= w[1]),
            "roles in PROVIDE_ORDER"
        );
        let last_ingestion = events
            .lock()
            .unwrap()
            .iter()
            .rposition(|e| e.role.is_needed_for_ingestion())
            .unwrap();
        let first_generator = events
            .lock()
            .unwrap()
            .iter()
            .position(|e| e.role == ModelRole::Generator)
            .unwrap();
        assert!(
            last_ingestion < first_generator,
            "ingestion is ready before the answer model"
        );
        let answer_models: Vec<String> = events
            .lock()
            .unwrap()
            .iter()
            .filter(|e| e.role == ModelRole::Generator)
            .map(|e| e.model_id.clone())
            .collect();
        assert_eq!(answer_models.first().unwrap(), "gemma-4-e2b");
        assert_eq!(
            answer_models.last().unwrap(),
            "gemma-4-e4b",
            "the fast model before that of the level"
        );
    });
}

#[test]
fn generation_sequences_are_checked_against_the_contract() {
    block_on(async {
        let events: Vec<GenerationEvent> = MockGenerator::new("a b")
            .generate(generation("x y"), CancellationToken::new())
            .await
            .unwrap()
            .map(|e| e.unwrap())
            .collect()
            .await;
        assert_eq!(generation_order_violation(&events), None);
    });
    let phase = |phase, progress| GenerationEvent::Phase { phase, progress };
    let text = GenerationEvent::Text("a".into());
    // A backend that reports no prompt phase still has to send ReadingSources.
    let without_reading = [
        phase(AnswerPhase::StartingModel, None),
        phase(AnswerPhase::Writing, None),
        text.clone(),
    ];
    assert!(generation_order_violation(&without_reading).is_some());
    let minimal = [
        phase(AnswerPhase::ReadingSources, None),
        phase(AnswerPhase::Writing, None),
        text.clone(),
    ];
    assert_eq!(generation_order_violation(&minimal), None);
    let text_first = [text.clone(), phase(AnswerPhase::Writing, None)];
    assert!(generation_order_violation(&text_first).is_some());
    let waiting = [
        phase(AnswerPhase::Waiting, None),
        phase(AnswerPhase::ReadingSources, None),
        phase(AnswerPhase::Writing, None),
    ];
    assert!(generation_order_violation(&waiting).is_some());
    let searching = [
        phase(AnswerPhase::Searching, None),
        phase(AnswerPhase::ReadingSources, None),
        phase(AnswerPhase::Writing, None),
    ];
    assert!(generation_order_violation(&searching).is_some());
    let shrinking = [
        phase(
            AnswerPhase::ReadingSources,
            Some(PhaseProgress { done: 5, total: 9 }),
        ),
        phase(
            AnswerPhase::ReadingSources,
            Some(PhaseProgress { done: 3, total: 9 }),
        ),
        phase(AnswerPhase::Writing, None),
    ];
    assert!(generation_order_violation(&shrinking).is_some());
}

fn turn(question: &str, answer: &str) -> ChatTurn {
    ChatTurn {
        question: question.into(),
        answer: Answer {
            text: answer.into(),
            citations: Vec::new(),
            no_sources: false,
            level: AnswerLevel::Fast,
            outcome: AnswerOutcome::Complete,
        },
    }
}

#[test]
fn the_rewriting_history_keeps_the_newest_whole_turns_within_its_budget() {
    block_on(async {
        let (embedder, index) = filled_index().await;
        let generator = Arc::new(MockGenerator::new("Jährlich [1]."));
        let config = PipelineConfig {
            rewrite_history_tokens: 5,
            ..PipelineConfig::default()
        };
        let answerer = MockAnswerer::new(
            Arc::new(embedder),
            Arc::new(index),
            Some(Arc::new(MockReranker)),
            Arc::clone(&generator) as Arc<dyn Generator>,
            config,
        );
        // 2, 2 and 3 tokens: the newest two fit into 5, the oldest does not.
        let history = [
            turn("eins", "alt"),
            turn("zwei", "mittel"),
            turn("drei", "neu und"),
        ];
        answerer
            .ask(
                fixtures::HITS_QUERY,
                &history,
                AnswerLevel::Fast,
                UiLanguage::German,
                CancellationToken::new(),
            )
            .await
            .unwrap()
            .collect::<Vec<_>>()
            .await;
        let rewrite = &generator.requests()[0];
        assert_eq!(rewrite.messages[0].content, MockAnswerer::REWRITE_PROMPT);
        let questions: Vec<&str> = rewrite
            .messages
            .iter()
            .filter(|m| m.role == Role::User)
            .map(|m| m.content.as_str())
            .collect();
        assert_eq!(questions, ["zwei", "drei", fixtures::HITS_QUERY]);

        // Not even the newest turn fits: no rewriting call, the answer call comes first.
        let generator = Arc::new(MockGenerator::new("Jährlich [1]."));
        let (embedder, index) = filled_index().await;
        let answerer = MockAnswerer::new(
            Arc::new(embedder),
            Arc::new(index),
            Some(Arc::new(MockReranker)),
            Arc::clone(&generator) as Arc<dyn Generator>,
            PipelineConfig {
                rewrite_history_tokens: 1,
                ..PipelineConfig::default()
            },
        );
        answerer
            .ask(
                fixtures::HITS_QUERY,
                &history,
                AnswerLevel::Fast,
                UiLanguage::German,
                CancellationToken::new(),
            )
            .await
            .unwrap()
            .collect::<Vec<_>>()
            .await;
        assert_eq!(generator.requests().len(), 1);
        assert_eq!(
            generator.requests()[0].messages[0].content,
            MockAnswerer::SYSTEM_PROMPT
        );
    });
}

#[test]
fn the_rewriting_budget_must_fit_the_prompt_budget() {
    let config = PipelineConfig {
        rewrite_history_tokens: 9000,
        ..PipelineConfig::default()
    };
    assert!(matches!(config.validate(), Err(Error::Config(_))));

    // The review case: 2200 of history pass the first rule, but the rewriting call with
    // its answer does not fit into 3328 tokens.
    let config = PipelineConfig {
        prompt_token_budget: 2200,
        rewrite_history_tokens: 2200,
        context_tokens: 3328,
        ..PipelineConfig::default()
    };
    assert!(matches!(config.validate(), Err(Error::Config(_))));
    let fits = PipelineConfig {
        rewrite_history_tokens: 3328
            - PipelineConfig::REWRITE_FIXED_TOKENS
            - config.max_answer_tokens,
        ..config
    };
    assert!(fits.validate().is_ok(), "exactly full is allowed");
}

#[test]
fn the_default_pipeline_follows_the_measurement() {
    let config = PipelineConfig::default();
    assert_eq!(config.max_sources, 8, "ADR 0082");
    assert_eq!(config.prompt_token_budget, 4000, "ADR 0081");
    assert_eq!(config.chunk_max_tokens, 400);
    assert_eq!(config.context_tokens, 5120);
    assert_eq!(config.rerank_candidates, 20, "ADR 0078");
    // Eight full sources with headers of 50 to 80 tokens, system instruction (about
    // 100) and question with template (about 50) need 3750 to 3990 tokens and use up
    // the budget; a ninth full source would not fit (ADR 0082).
    let instruction_and_question = 150;
    let prompt = |sources: u32, header: u32| {
        sources * (config.chunk_max_tokens + header) + instruction_and_question
    };
    assert!(prompt(config.max_sources, 80) <= config.prompt_token_budget);
    assert!(
        prompt(config.max_sources, 50) >= 3750,
        "the budget is used up"
    );
    assert!(
        prompt(config.max_sources + 1, 80) > config.prompt_token_budget,
        "nine full sources do not fit"
    );
    assert!(config.validate().is_ok());
    assert!(
        config.context_tokens >= config.prompt_token_budget + config.max_answer_tokens,
        "prompt and answer fit into the context"
    );
    assert!(
        config.context_tokens < 9216,
        "below the 9216 tokens the memory was measured with (ADR 0081)"
    );
}

#[test]
fn a_saved_level_the_tier_does_not_offer_falls_back_to_fast() {
    let m = ModelManifest::from_toml_str(include_str!("../../models.toml")).unwrap();
    let below = HardwareTier::BelowMinimum;
    let supported = HardwareTier::Supported;
    let mut settings = Settings {
        answer_level: AnswerLevel::Thorough,
        ..Settings::default()
    };
    assert_eq!(
        m.effective_level(settings.answer_level, below),
        AnswerLevel::Fast
    );
    assert_eq!(
        m.effective_level(settings.answer_level, supported),
        AnswerLevel::Thorough
    );
    for level in AnswerLevel::ALL {
        settings.answer_level = level;
        let effective = m.effective_level(settings.answer_level, below);
        assert!(m.levels(below).contains(&effective));
        assert_eq!(settings.answer_level, level, "the saved level stays");
    }
}

#[test]
fn overlapping_provide_calls_fetch_each_file_once() {
    block_on(async {
        let hardware = HardwareInfo {
            total_ram_bytes: 16 << 30,
            logical_cpus: 8,
        };
        let gate = Gate::new();
        let hub = MockModelHub::new(manifest(), hardware, PathBuf::from(r"C:\Modelle"))
            .with_fetch_gate(gate.clone());
        let tier = HardwareTier::Supported;
        let recorder = || {
            let events = Arc::new(Mutex::new(Vec::<ModelEvent>::new()));
            let sink = Arc::clone(&events);
            let callback: ModelProgressCallback = Arc::new(move |e| sink.lock().unwrap().push(e));
            (events, callback)
        };
        let (fast_events, fast_progress) = recorder();
        let (high_events, high_progress) = recorder();

        // The setup provides the fast level, the user chooses the thorough one: both
        // need the base equipment and wait for its first file.
        let cancel_fast = CancellationToken::new();
        let mut fast = hub.provide(
            tier,
            AnswerLevel::Fast,
            None,
            fast_progress,
            cancel_fast.clone(),
        );
        let mut high = hub.provide(
            tier,
            AnswerLevel::Thorough,
            None,
            high_progress,
            CancellationToken::new(),
        );
        assert!((&mut fast).now_or_never().is_none());
        assert!((&mut high).now_or_never().is_none());
        let first = manifest().models_for(tier, AnswerLevel::Fast).unwrap()[0].clone();
        let downloading = |events: &Mutex<Vec<ModelEvent>>| {
            events.lock().unwrap().iter().any(|e| {
                e.model_id == first.id && matches!(e.status, ModelFileStatus::Downloading { .. })
            })
        };
        assert!(
            downloading(&fast_events) && downloading(&high_events),
            "both report it"
        );

        // Cancelling one call ends only that call; the file goes on for the other.
        cancel_fast.cancel();
        assert!(matches!(
            (&mut fast).now_or_never(),
            Some(Err(Error::Cancelled))
        ));
        gate.open();
        let models = high.await.unwrap();
        assert_eq!(models.last().unwrap().entry.id, "gemma-4-12b");
        for entry in manifest().models_for(tier, AnswerLevel::Thorough).unwrap() {
            for file in &entry.files {
                assert_eq!(hub.fetches(&entry.id, &file.path), 1, "{}", file.path);
            }
        }

        // The fast level is ready now without fetching anything again.
        hub.provide(
            tier,
            AnswerLevel::Fast,
            None,
            Arc::new(|_| {}),
            CancellationToken::new(),
        )
        .await
        .unwrap();
        assert!(
            hub.missing(tier, AnswerLevel::Fast)
                .await
                .unwrap()
                .is_empty()
        );
        assert_eq!(hub.fetches(&first.id, &first.files[0].path), 1);

        // Both calls run to their end: every shared file is still fetched once.
        let gate = Gate::new();
        let hub = MockModelHub::new(manifest(), hardware, PathBuf::from(r"C:\Modelle"))
            .with_fetch_gate(gate.clone());
        let mut fast = hub.provide(
            tier,
            AnswerLevel::Fast,
            None,
            Arc::new(|_| {}),
            CancellationToken::new(),
        );
        let mut high = hub.provide(
            tier,
            AnswerLevel::Balanced,
            None,
            Arc::new(|_| {}),
            CancellationToken::new(),
        );
        assert!((&mut fast).now_or_never().is_none());
        assert!((&mut high).now_or_never().is_none());
        gate.open();
        let (fast, high) = futures::join!(fast, high);
        fast.unwrap();
        high.unwrap();
        for entry in manifest().models_for(tier, AnswerLevel::Balanced).unwrap() {
            for file in &entry.files {
                assert_eq!(hub.fetches(&entry.id, &file.path), 1, "{}", file.path);
            }
        }
    });
}

#[test]
fn the_reranker_scores_heading_and_text_independent_of_the_call() {
    block_on(async {
        let chunks = fixtures::chunks();
        let query = "Prüfung Fluchtwege";
        let together = MockReranker
            .rerank(query, &chunks, CancellationToken::new())
            .await
            .unwrap();
        for (chunk, score) in chunks.iter().zip(&together) {
            let alone = MockReranker
                .rerank(query, std::slice::from_ref(chunk), CancellationToken::new())
                .await
                .unwrap();
            assert_eq!(alone, [*score], "{}", chunk.id);
        }
        let mut reversed = chunks.clone();
        reversed.reverse();
        let mut scores = MockReranker
            .rerank(query, &reversed, CancellationToken::new())
            .await
            .unwrap();
        scores.reverse();
        assert_eq!(scores, together, "the order of the call does not matter");

        // The heading counts: the same text under a matching heading scores higher.
        let mut plain = chunks[0].clone();
        plain.text = "Alle zwei Jahre.".into();
        plain.provenance.heading_path = vec!["Allgemeines".into()];
        let mut headed = plain.clone();
        headed.provenance.heading_path = vec!["Prüfung der Fluchtwege".into()];
        let scores = MockReranker
            .rerank(query, &[plain, headed], CancellationToken::new())
            .await
            .unwrap();
        assert!(scores[1] > scores[0], "{scores:?}");
    });
}

#[test]
fn a_partial_file_is_continued_by_the_next_call() {
    block_on(async {
        let hardware = HardwareInfo {
            total_ram_bytes: 16 << 30,
            logical_cpus: 8,
        };
        let gate = Gate::new();
        let hub = MockModelHub::new(manifest(), hardware, PathBuf::from(r"C:\Modelle"))
            .with_fetch_gate(gate.clone());
        let tier = HardwareTier::Supported;
        let first = manifest().models_for(tier, AnswerLevel::Fast).unwrap()[0].clone();
        let file = first.files[0].clone();
        let size = file.size_bytes.unwrap();
        let first_event = |events: &Mutex<Vec<ModelEvent>>| {
            events
                .lock()
                .unwrap()
                .iter()
                .find(|e| {
                    e.model_id == first.id
                        && matches!(
                            e.status,
                            ModelFileStatus::Copying { .. } | ModelFileStatus::Downloading { .. }
                        )
                })
                .map(|e| e.status.clone())
        };

        // A download is cancelled while the file is half there.
        let events = Arc::new(Mutex::new(Vec::new()));
        let sink = Arc::clone(&events);
        let cancel = CancellationToken::new();
        let mut download = hub.provide(
            tier,
            AnswerLevel::Fast,
            None,
            Arc::new(move |e| sink.lock().unwrap().push(e)),
            cancel.clone(),
        );
        assert!((&mut download).now_or_never().is_none());
        assert_eq!(
            first_event(&events),
            Some(ModelFileStatus::Downloading {
                done_bytes: 0,
                total_bytes: Some(size)
            })
        );
        cancel.cancel();
        assert!(matches!(
            (&mut download).now_or_never(),
            Some(Err(Error::Cancelled))
        ));
        assert!(
            hub.missing(tier, AnswerLevel::Fast)
                .await
                .unwrap()
                .iter()
                .any(|m| m.model_id == first.id),
            "a partial file is not ready"
        );

        // A later call copying from an offline package continues it.
        gate.open();
        let events = Arc::new(Mutex::new(Vec::new()));
        let sink = Arc::clone(&events);
        hub.provide(
            tier,
            AnswerLevel::Fast,
            Some(PathBuf::from(r"E:\RagOxide-Modelle")),
            Arc::new(move |e| sink.lock().unwrap().push(e)),
            CancellationToken::new(),
        )
        .await
        .unwrap();
        assert_eq!(
            first_event(&events),
            Some(ModelFileStatus::Copying {
                done_bytes: size / 2,
                total_bytes: Some(size)
            }),
            "the first progress names the part already there"
        );
        assert_eq!(hub.fetches(&first.id, &file.path), 1);
    });
}

#[test]
fn below_the_minimum_the_hub_neither_lists_nor_provides_the_drafter() {
    block_on(async {
        let hardware = HardwareInfo {
            total_ram_bytes: 8 << 30,
            logical_cpus: 4,
        };
        let hub = MockModelHub::new(manifest(), hardware, PathBuf::from(r"C:\Modelle"));
        let drafter = "mtp-gemma-4-E2B-it.gguf";
        let files = |tier| {
            let hub = &hub;
            async move {
                hub.missing(tier, AnswerLevel::Fast)
                    .await
                    .unwrap()
                    .into_iter()
                    .filter(|m| m.model_id == "gemma-4-e2b")
                    .map(|m| m.file)
                    .collect::<Vec<_>>()
            }
        };
        assert_eq!(
            files(HardwareTier::BelowMinimum).await,
            ["gemma-4-E2B-it-Q4_K_M.gguf"]
        );
        assert!(
            files(HardwareTier::Supported)
                .await
                .contains(&drafter.to_string())
        );
        let resolved = hub
            .provide(
                HardwareTier::BelowMinimum,
                AnswerLevel::Fast,
                None,
                Arc::new(|_| {}),
                CancellationToken::new(),
            )
            .await
            .unwrap();
        let e2b = resolved
            .iter()
            .find(|m| m.entry.id == "gemma-4-e2b")
            .unwrap();
        assert_eq!(e2b.entry.mtp_draft, None);
        assert_eq!(hub.fetches("gemma-4-e2b", drafter), 0, "not downloaded");
    });
}

/// [`Index`] that returns eight fixed hits, `Text der Quelle 1.` to `Text der Quelle 8.`.
struct EightSources(EmbeddingSpace);

#[async_trait]
impl Index for EightSources {
    fn space(&self) -> EmbeddingSpace {
        self.0.clone()
    }

    async fn add(&self, _entries: &[IndexEntry]) -> Result<()> {
        Ok(())
    }

    async fn delete_document(&self, _id: &DocumentId) -> Result<()> {
        Ok(())
    }

    async fn search(&self, request: &SearchRequest, _embedding: &[f32]) -> Result<Vec<SearchHit>> {
        Ok(fixtures::chunks()
            .into_iter()
            .take(8)
            .enumerate()
            .map(|(i, mut chunk)| {
                chunk.text = format!("Text der Quelle {}.", i + 1);
                SearchHit {
                    chunk,
                    retrieval_score: 1.0 - i as f32 / 10.0,
                    vector_score: Some(0.9),
                    rerank_score: None,
                }
            })
            .take(request.limit as usize)
            .collect())
    }
}

/// [`Generator`] over a [`MockGenerator`] whose count is the sum of the costs of the
/// sources in the text (`costs[i]` for `Text der Quelle {i + 1}.`).
struct SourceCosts {
    inner: Arc<MockGenerator>,
    costs: [usize; 8],
}

#[async_trait]
impl Generator for SourceCosts {
    fn check_level(&self, level: AnswerLevel) -> Result<()> {
        self.inner.check_level(level)
    }

    async fn count_tokens(&self, _level: AnswerLevel, text: &str) -> Result<usize> {
        Ok((0..8)
            .filter(|i| text.contains(&format!("Text der Quelle {}.", i + 1)))
            .map(|i| self.costs[i])
            .sum())
    }

    async fn generate(
        &self,
        request: GenerationRequest,
        cancel: CancellationToken,
    ) -> Result<GenerationStream> {
        self.inner.generate(request, cancel).await
    }
}

/// Asks with eight relevant sources and the given costs and budget; returns the final
/// answer, the number of sources sent with `Sources` and the numbers of the sources in
/// the answer prompt.
async fn answer_with_budget(
    reply: &str,
    costs: [usize; 8],
    budget: u32,
) -> (Answer, usize, Vec<u32>) {
    let embedder = MockEmbedder::default();
    let index = EightSources(embedder.space());
    let inner = Arc::new(MockGenerator::new(reply));
    let answerer = MockAnswerer::new(
        Arc::new(embedder),
        Arc::new(index),
        None,
        Arc::new(SourceCosts {
            inner: Arc::clone(&inner),
            costs,
        }),
        PipelineConfig {
            prompt_token_budget: budget,
            rewrite_history_tokens: 0,
            ..PipelineConfig::default()
        },
    );
    let events: Vec<AnswerEvent> = answerer
        .ask(
            "Was steht in den Quellen?",
            &[],
            AnswerLevel::Fast,
            UiLanguage::German,
            CancellationToken::new(),
        )
        .await
        .unwrap()
        .map(|e| e.unwrap())
        .collect()
        .await;
    assert_eq!(answer_order_violation(&events, false), None);
    let sent = events
        .iter()
        .find_map(|e| match e {
            AnswerEvent::Sources { hits } => Some(hits.len()),
            _ => None,
        })
        .unwrap();
    let Some(AnswerEvent::Done { answer }) = events.last() else {
        unreachable!()
    };
    let prompt = &inner.requests()[0].messages[1].content;
    let in_prompt = (1..=8)
        .filter(|n| prompt.contains(&format!("Text der Quelle {n}.")))
        .collect();
    (answer.clone(), sent, in_prompt)
}

#[test]
fn sources_beyond_the_prompt_budget_are_dropped_from_the_end() {
    block_on(async {
        // The eighth source brings the prompt over the budget: all eight are sent with
        // `Sources`, the prompt holds seven, and the marker [8] is removed.
        let (answer, sent, in_prompt) =
            answer_with_budget("Antwort [1] und [8].", [100; 8], 750).await;
        assert_eq!(sent, 8);
        assert_eq!(in_prompt, [1, 2, 3, 4, 5, 6, 7]);
        assert_eq!(answer.text, "Antwort [1] und .");
        let cited: Vec<u32> = answer.citations.iter().map(|c| c.number).collect();
        assert_eq!(cited, [1]);

        // A middle source does not fit: it is not skipped for later ones, all from it on
        // are left out.
        let mut costs = [100; 8];
        costs[3] = 1000;
        let (answer, sent, in_prompt) =
            answer_with_budget("Antwort [1] [3] [5].", costs, 750).await;
        assert_eq!(sent, 8);
        assert_eq!(in_prompt, [1, 2, 3]);
        let cited: Vec<u32> = answer.citations.iter().map(|c| c.number).collect();
        assert_eq!(cited, [1, 3]);
        assert!(!answer.text.contains("[5]"));

        // Everything fits: all eight go into the prompt.
        let (_, _, in_prompt) = answer_with_budget("Antwort [8].", [100; 8], 4000).await;
        assert_eq!(in_prompt, [1, 2, 3, 4, 5, 6, 7, 8]);
    });
}
