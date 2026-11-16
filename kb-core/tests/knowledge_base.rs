//! Question to answer with citation through `KnowledgeBase`, using mocks only, plus
//! chat storage, the library scope (watched folders and exclusions, ADR 0025),
//! progress, failures, resuming and cancellation.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use futures::executor::block_on;
use futures::{FutureExt, StreamExt};
use kb_core::mock::*;
use kb_core::*;

const QUESTION: &str = "Wie oft wird ein Gerät geprüft?";

struct Setup {
    kb: KnowledgeBase,
    store: Arc<MockStore>,
    files: Arc<MockFiles>,
    generator: Arc<MockGenerator>,
    clock: Arc<MockClock>,
    events: Arc<Mutex<Vec<IngestEvent>>>,
    scope: Arc<Mutex<Vec<ScopeEvent>>>,
}

impl Setup {
    fn progress(&self) -> ProgressCallback {
        let events = Arc::clone(&self.events);
        Arc::new(move |event| events.lock().unwrap().push(event))
    }

    fn scope_callback(&self) -> ScopeCallback {
        let scope = Arc::clone(&self.scope);
        Arc::new(move |event| scope.lock().unwrap().push(event))
    }

    fn events(&self) -> Vec<IngestEvent> {
        self.events.lock().unwrap().clone()
    }

    fn scope_events(&self) -> Vec<ScopeEvent> {
        self.scope.lock().unwrap().clone()
    }

    async fn sync(&self) -> Vec<FileResult> {
        self.kb
            .sync(
                self.progress(),
                self.scope_callback(),
                SyncMode::Normal,
                CancellationToken::new(),
            )
            .await
            .unwrap()
    }

    /// Registers the fixture folder and synchronises, as the command line does.
    async fn ingest_all(&self) -> Vec<FileResult> {
        self.kb
            .add_folder(Path::new(MockFiles::FIXTURE_FOLDER))
            .await
            .unwrap();
        self.sync().await
    }

    async fn ask(&self, question: &str, chat: Option<&ChatId>) -> Vec<AnswerEvent> {
        let stream = self
            .kb
            .ask(question, chat, AnswerLevel::Fast, CancellationToken::new())
            .await
            .unwrap();
        stream
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .map(|e| e.unwrap())
            .collect()
    }
}

fn setup_with(
    parser: Arc<dyn Parser>,
    reranker: Option<Arc<dyn Reranker>>,
    reply: &str,
    dimension: usize,
    config: PipelineConfig,
) -> Setup {
    let embedder: Arc<dyn Embedder> = Arc::new(MockEmbedder::new(dimension));
    let index: Arc<dyn Index> = Arc::new(MockIndex::new(embedder.space(), 0.5));
    let store = Arc::new(MockStore::new());
    let files = Arc::new(MockFiles::fixtures());
    let generator = Arc::new(MockGenerator::new(reply));
    let tokens: Arc<dyn TokenCounter> = Arc::new(MockTokenCounter);
    let ingestor = Arc::new(MockIngestor::new(
        parser,
        Arc::new(MockChunker::new(Arc::clone(&tokens), 400)),
        Arc::clone(&embedder),
        Arc::clone(&index),
        Arc::clone(&store) as Arc<dyn Store>,
        Arc::clone(&files),
    ));
    let answerer = Arc::new(MockAnswerer::new(
        embedder,
        index,
        reranker,
        Arc::clone(&generator) as Arc<dyn Generator>,
        config,
    ));
    let clock = Arc::new(MockClock::new(Timestamp(1_794_700_800_000)));
    Setup {
        kb: KnowledgeBase::new(ingestor, answerer, Arc::clone(&store) as Arc<dyn Store>)
            .with_clock(Arc::clone(&clock) as Arc<dyn Clock>),
        store,
        files,
        generator,
        clock,
        events: Arc::new(Mutex::new(Vec::new())),
        scope: Arc::new(Mutex::new(Vec::new())),
    }
}

fn setup(reply: &str) -> Setup {
    setup_with(
        Arc::new(MockParser::new()),
        Some(Arc::new(MockReranker)),
        reply,
        64,
        PipelineConfig::default(),
    )
}

/// The sequence of an answer in short: one label per event, repeated `reading` and
/// `delta` collapsed, `sources` marked `sources0` when empty.
fn shape(events: &[AnswerEvent]) -> Vec<&'static str> {
    let mut labels: Vec<&'static str> = Vec::new();
    for event in events {
        let label = match event {
            AnswerEvent::Status { phase, .. } => match phase {
                AnswerPhase::Waiting => "waiting",
                AnswerPhase::Searching => "searching",
                AnswerPhase::StartingModel => "starting",
                AnswerPhase::ReadingSources => "reading",
                AnswerPhase::Writing => "writing",
            },
            AnswerEvent::Sources { hits } if hits.is_empty() => "sources0",
            AnswerEvent::Sources { .. } => "sources",
            AnswerEvent::Delta { .. } => "delta",
            AnswerEvent::Done { .. } => "done",
        };
        if !(matches!(label, "reading" | "delta") && labels.last() == Some(&label)) {
            labels.push(label);
        }
    }
    labels
}

const CASE_A: [&str; 7] = [
    "searching",
    "sources",
    "starting",
    "reading",
    "writing",
    "delta",
    "done",
];
const CASE_B: [&str; 7] = [
    "starting",
    "searching",
    "sources",
    "reading",
    "writing",
    "delta",
    "done",
];

fn sources_of(events: &[AnswerEvent]) -> &Vec<SearchHit> {
    events
        .iter()
        .find_map(|e| match e {
            AnswerEvent::Sources { hits } => Some(hits),
            _ => None,
        })
        .expect("a Sources event")
}

fn done(events: &[AnswerEvent]) -> &Answer {
    match events.last() {
        Some(AnswerEvent::Done { answer }) => answer,
        other => panic!("last event must be Done, got {other:?}"),
    }
}

#[test]
fn question_to_answer_with_citations_through_mocks() {
    block_on(async {
        let reply = "Jedes Gerät wird mindestens einmal jährlich geprüft [1]. \
                     Die Ergebnisse stehen in der Übersicht [2].";
        let s = setup(reply);
        let results = s.ingest_all().await;
        assert_eq!(results.len(), 7);
        assert!(
            results
                .iter()
                .all(|r| matches!(r.status, FileStatus::Indexed { .. }))
        );

        let events = s.ask(QUESTION, None).await;
        assert_eq!(shape(&events), CASE_A);
        let hits = sources_of(&events);
        assert!(hits.len() >= 2);
        let streamed: String = events
            .iter()
            .filter_map(|e| match e {
                AnswerEvent::Delta { text } => Some(text.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(streamed, reply);
        let answer = done(&events);
        assert_eq!(answer.text, reply);
        assert!(!answer.no_sources);

        // Every number refers to the source at that position, not just the first one.
        let numbers: Vec<u32> = answer.citations.iter().map(|c| c.number).collect();
        assert_eq!(numbers, [1, 2]);
        for citation in &answer.citations {
            assert_eq!(citation.chunk, hits[citation.number as usize - 1].chunk);
        }
        assert_ne!(answer.citations[0].chunk, answer.citations[1].chunk);

        // The citation carries file, heading path and anchors: the paragraph continues
        // on page 2, so the jump target has both boxes.
        let provenance = &answer.citations[0].chunk.provenance;
        assert_eq!(provenance.source.name, "pruefordnung.pdf");
        assert_eq!(
            provenance.source.path,
            PathBuf::from(r"C:\Ablage\pruefordnung.pdf")
        );
        assert_eq!(provenance.heading_path.last().unwrap(), "§ 2 Prüffristen");
        let pages: Vec<u32> = provenance
            .anchors
            .iter()
            .map(|a| match a {
                Anchor::Page { page, .. } => *page,
                other => panic!("unexpected {other:?}"),
            })
            .collect();
        assert_eq!(pages, [1, 2]);

        // The citation is current: its hash matches the library record.
        let record =
            s.kb.document(&provenance.source.document_id)
                .await
                .unwrap()
                .unwrap();
        assert_eq!(
            record.content_hash.as_deref(),
            Some(provenance.source.content_hash.as_str())
        );

        let prompt = &s.generator.requests()[0].messages[1].content;
        assert!(prompt.contains(
            "[1] pruefordnung.pdf | Prüfordnung für Kühl- und Gefriergeräte > § 2 Prüffristen\nJedes Gerät"
        ));
        let content =
            s.kb.document_content(&provenance.source.document_id)
                .await
                .unwrap()
                .unwrap();
        assert_eq!(content.pages.len(), 2);
    });
}

#[test]
fn markers_without_source_are_removed() {
    block_on(async {
        let s = setup("Jährlich [1][9], in English [x].");
        s.ingest_all().await;
        let events = s.ask(QUESTION, None).await;
        let answer = done(&events);
        assert_eq!(answer.text, "Jährlich [1], in English [x].");
        let numbers: Vec<u32> = answer.citations.iter().map(|c| c.number).collect();
        assert_eq!(numbers, [1]);
    });
}

#[test]
fn no_relevant_source_gives_no_invented_answer_in_ui_language() {
    block_on(async {
        let s = setup("Invented answer [1].");
        s.ingest_all().await;
        let events = s.ask("Quantenchromodynamik?", None).await;
        let expected = |text: &str| {
            vec![
                AnswerEvent::Status {
                    phase: AnswerPhase::Searching,
                    progress: None,
                },
                AnswerEvent::Sources { hits: Vec::new() },
                AnswerEvent::Done {
                    answer: Answer {
                        text: text.into(),
                        citations: Vec::new(),
                        no_sources: true,
                        level: AnswerLevel::Fast,
                        outcome: AnswerOutcome::Complete,
                    },
                },
            ]
        };
        assert_eq!(events, expected(MockAnswerer::NO_SOURCES_DE));

        s.store
            .save_settings(&Settings {
                ui_language: UiLanguage::English,
                ..Settings::default()
            })
            .await
            .unwrap();
        let events = s.ask("Quantenchromodynamik?", None).await;
        assert_eq!(events, expected(MockAnswerer::NO_SOURCES_EN));
        assert!(
            s.generator.requests().is_empty(),
            "answer model must not be called"
        );
    });
}

#[test]
fn without_reranker_hits_keep_index_order_and_similarity_decides() {
    block_on(async {
        // Hashed bag-of-words vectors give low similarities for long chunks, and few
        // dimensions cause collisions: more dimensions and a lower threshold.
        let config = PipelineConfig {
            min_similarity: 0.2,
            ..PipelineConfig::default()
        };
        let s = setup_with(
            Arc::new(MockParser::new()),
            None,
            "Jährlich [1].",
            1024,
            config,
        );
        s.ingest_all().await;
        let hits =
            s.kb.search(&SearchRequest::new(QUESTION, 5), CancellationToken::new())
                .await
                .unwrap();
        assert!(hits.iter().all(|h| h.rerank_score.is_none()));
        assert!(
            hits.windows(2)
                .all(|w| w[0].retrieval_score >= w[1].retrieval_score)
        );

        let events = s.ask(QUESTION, None).await;
        let hits = sources_of(&events);
        let min = 0.2;
        assert!(!hits.is_empty());
        assert!(hits.iter().all(|h| h.vector_score.unwrap() >= min));
        assert!(!done(&events).no_sources);

        let events = s.ask("Zebrafink?", None).await;
        assert!(done(&events).no_sources);
    });
}

#[test]
fn chat_turns_are_saved_only_with_a_chat_id() {
    block_on(async {
        let s = setup("Jährlich [1].");
        s.ingest_all().await;
        s.ask(QUESTION, None).await;
        assert!(s.store.chats().await.unwrap().is_empty());

        let id = ChatId::new("chat-1");
        s.ask(QUESTION, Some(&id)).await;
        s.ask("Und Wärmepumpen?", Some(&id)).await;
        let chat = s.store.chat(&id).await.unwrap().unwrap();
        assert_eq!(chat.title, QUESTION);
        let questions: Vec<&str> = chat.turns.iter().map(|t| t.question.as_str()).collect();
        assert_eq!(questions, [QUESTION, "Und Wärmepumpen?"]);
        assert_eq!(chat.turns[0].answer.citations[0].number, 1);
    });
}

#[test]
fn a_cancelled_answer_ends_in_done_keeps_verified_text_and_is_saved() {
    block_on(async {
        let s = setup("Jedes Gerät wird jährlich geprüft [1]. Siehe [2 und mehr.");
        s.ingest_all().await;
        let cancel = CancellationToken::new();
        let id = ChatId::new("chat-1");
        let mut stream =
            s.kb.ask(QUESTION, Some(&id), AnswerLevel::Fast, cancel.clone())
                .await
                .unwrap();
        let mut events = Vec::new();
        // Read up to and including the delta that starts the second marker, then cancel.
        while let Some(event) = stream.next().await {
            let event = event.unwrap();
            let cut = matches!(&event, AnswerEvent::Delta { text } if text.starts_with("[2"));
            events.push(event);
            if cut {
                cancel.cancel();
                break;
            }
        }
        let rest: Vec<Result<AnswerEvent>> = stream.collect().await;
        assert_eq!(rest.len(), 1, "exactly one Done follows, no error item");
        let Ok(AnswerEvent::Done { answer }) = &rest[0] else {
            panic!("Done expected, got {:?}", rest[0]);
        };
        assert_eq!(answer.outcome, AnswerOutcome::Cancelled);
        assert_eq!(answer.text, "Jedes Gerät wird jährlich geprüft [1]. Siehe ");
        let numbers: Vec<u32> = answer.citations.iter().map(|c| c.number).collect();
        assert_eq!(
            numbers,
            [1],
            "only verified citations, the cut marker is gone"
        );

        let chat = s.store.chat(&id).await.unwrap().unwrap();
        assert_eq!(chat.turns.len(), 1);
        assert_eq!(chat.turns[0].question, QUESTION);
        assert_eq!(&chat.turns[0].answer, answer);
    });
}

#[test]
fn a_failed_generation_ends_in_done_with_its_category_and_is_saved() {
    block_on(async {
        let embedder: Arc<dyn Embedder> = Arc::new(MockEmbedder::default());
        let index: Arc<dyn Index> = Arc::new(MockIndex::new(embedder.space(), 0.5));
        let store = Arc::new(MockStore::new());
        let tokens: Arc<dyn TokenCounter> = Arc::new(MockTokenCounter);
        let ingestor = Arc::new(MockIngestor::new(
            Arc::new(MockParser::new()),
            Arc::new(MockChunker::new(Arc::clone(&tokens), 400)),
            Arc::clone(&embedder),
            Arc::clone(&index),
            Arc::clone(&store) as Arc<dyn Store>,
            Arc::new(MockFiles::fixtures()),
        ));
        let answerer = Arc::new(MockAnswerer::new(
            embedder,
            index,
            Some(Arc::new(MockReranker)),
            Arc::new(MockGenerator::failing_after(
                "Jedes Gerät wird jährlich geprüft [1]. Und weiter.",
                6,
            )),
            PipelineConfig::default(),
        ));
        let kb = KnowledgeBase::new(ingestor, answerer, Arc::clone(&store) as Arc<dyn Store>);
        kb.add_folder(Path::new(MockFiles::FIXTURE_FOLDER))
            .await
            .unwrap();
        kb.sync(
            Arc::new(|_| {}),
            Arc::new(|_| {}),
            SyncMode::Normal,
            CancellationToken::new(),
        )
        .await
        .unwrap();
        let id = ChatId::new("chat-1");
        let events: Vec<Result<AnswerEvent>> = kb
            .ask(
                QUESTION,
                Some(&id),
                AnswerLevel::Fast,
                CancellationToken::new(),
            )
            .await
            .unwrap()
            .collect()
            .await;
        assert!(events.iter().all(|e| e.is_ok()), "no error items");
        let Some(Ok(AnswerEvent::Done { answer })) = events.last() else {
            panic!("Done expected");
        };
        assert!(matches!(
            &answer.outcome,
            AnswerOutcome::Failed {
                kind: ErrorKind::Generator,
                ..
            }
        ));
        assert_eq!(answer.text, "Jedes Gerät wird jährlich geprüft [1]. ");
        assert_eq!(answer.citations.len(), 1);
        let saved = store.chat(&id).await.unwrap().unwrap();
        assert_eq!(&saved.turns[0].answer, answer);
    });
}

#[test]
fn each_case_runs_through_its_fixed_sequence() {
    block_on(async {
        let s = setup("Jedes Gerät wird jährlich geprüft [1].");
        s.ingest_all().await;
        // Case A: first question, also when the model is already loaded.
        for _ in 0..2 {
            let events = s.ask(QUESTION, None).await;
            assert_eq!(shape(&events), CASE_A);
            assert_eq!(answer_order_violation(&events, false), None);
        }
        let none = s.ask("Quantenchromodynamik?", None).await;
        assert_eq!(shape(&none), ["searching", "sources0", "done"]);
        assert_eq!(answer_order_violation(&none, false), None);

        // Case B: a follow-up in a chat with a complete turn.
        let id = ChatId::new("chat-1");
        s.ask(QUESTION, Some(&id)).await;
        let follow_up = s.ask("Und wie oft?", Some(&id)).await;
        assert_eq!(shape(&follow_up), CASE_B);
        assert_eq!(answer_order_violation(&follow_up, true), None);
        let none = s.ask("Quantenchromodynamik?", Some(&id)).await;
        assert_eq!(shape(&none), ["starting", "searching", "sources0", "done"]);
        assert_eq!(answer_order_violation(&none, true), None);
    });
}

#[test]
fn reading_progress_is_passed_on_unchanged() {
    block_on(async {
        let s = setup("Jedes Gerät wird jährlich geprüft [1].");
        s.ingest_all().await;
        let events = s.ask(QUESTION, None).await;
        let request = &s.generator.requests()[0];
        let total: u32 = request
            .messages
            .iter()
            .map(|m| m.content.split_whitespace().count() as u32)
            .sum();
        let progress: Vec<Option<PhaseProgress>> = events
            .iter()
            .filter_map(|e| match e {
                AnswerEvent::Status {
                    phase: AnswerPhase::ReadingSources,
                    progress,
                } => Some(*progress),
                _ => None,
            })
            .collect();
        assert_eq!(
            progress,
            [
                Some(PhaseProgress {
                    done: total / 2,
                    total
                }),
                Some(PhaseProgress { done: total, total }),
            ]
        );
    });
}

/// Records the history each question gets, then answers like the mock.
struct RecordingAnswerer {
    inner: MockAnswerer,
    histories: Mutex<Vec<Vec<String>>>,
}

#[async_trait]
impl Answerer for RecordingAnswerer {
    async fn search(
        &self,
        request: &SearchRequest,
        cancel: CancellationToken,
    ) -> Result<Vec<SearchHit>> {
        self.inner.search(request, cancel).await
    }

    async fn ask(
        &self,
        question: &str,
        history: &[ChatTurn],
        level: AnswerLevel,
        language: UiLanguage,
        cancel: CancellationToken,
    ) -> Result<AnswerStream> {
        let questions = history.iter().map(|t| t.question.clone()).collect();
        self.histories.lock().unwrap().push(questions);
        self.inner
            .ask(question, history, level, language, cancel)
            .await
    }
}

#[test]
fn follow_up_questions_see_only_complete_turns() {
    block_on(async {
        let s = setup("");
        let embedder: Arc<dyn Embedder> = Arc::new(MockEmbedder::default());
        let index: Arc<dyn Index> = Arc::new(MockIndex::new(embedder.space(), 0.5));
        let answerer = Arc::new(RecordingAnswerer {
            inner: MockAnswerer::new(
                embedder,
                index,
                None,
                Arc::new(MockGenerator::new("")),
                PipelineConfig::default(),
            ),
            histories: Mutex::new(Vec::new()),
        });
        let kb = KnowledgeBase::new(
            Arc::new(MockIngestor::new(
                Arc::new(MockParser::new()),
                Arc::new(MockChunker::default()),
                Arc::new(MockEmbedder::default()),
                Arc::new(MockIndex::new(MockEmbedder::default().space(), 0.5)),
                Arc::clone(&s.store) as Arc<dyn Store>,
                Arc::new(MockFiles::new()),
            )),
            Arc::clone(&answerer) as Arc<dyn Answerer>,
            Arc::clone(&s.store) as Arc<dyn Store>,
        );
        let turn = |question: &str, outcome: AnswerOutcome| ChatTurn {
            question: question.into(),
            answer: Answer {
                text: String::new(),
                citations: Vec::new(),
                no_sources: false,
                level: AnswerLevel::Fast,
                outcome,
            },
        };
        let id = ChatId::new("chat-1");
        s.store
            .save_chat(&Chat {
                id: id.clone(),
                title: "a".into(),
                created_at: Timestamp(1),
                updated_at: Timestamp(1),
                turns: vec![
                    turn("a", AnswerOutcome::Complete),
                    turn("b", AnswerOutcome::Cancelled),
                    turn(
                        "c",
                        AnswerOutcome::Failed {
                            kind: ErrorKind::Generator,
                            message: "crash".into(),
                        },
                    ),
                ],
            })
            .await
            .unwrap();
        kb.ask("d", Some(&id), AnswerLevel::Fast, CancellationToken::new())
            .await
            .unwrap()
            .collect::<Vec<_>>()
            .await;
        assert_eq!(*answerer.histories.lock().unwrap(), [vec!["a".to_string()]]);
        assert_eq!(s.store.chat(&id).await.unwrap().unwrap().turns.len(), 4);
    });
}

#[test]
fn a_chat_can_be_renamed_without_moving_in_the_list() {
    block_on(async {
        let s = setup("Jährlich [1].");
        s.ingest_all().await;
        let old = ChatId::new("chat-old");
        s.ask(QUESTION, Some(&old)).await;
        s.clock.advance(60_000);
        let new = ChatId::new("chat-new");
        s.ask(QUESTION, Some(&new)).await;
        let before = s.store.chat(&old).await.unwrap().unwrap();

        s.clock.advance(60_000);
        s.kb.rename_chat(&old, "  Prüffristen  ").await.unwrap();
        let after = s.store.chat(&old).await.unwrap().unwrap();
        assert_eq!(after.title, "Prüffristen", "trimmed");
        assert_eq!(
            after.updated_at, before.updated_at,
            "renaming is no question"
        );
        assert_eq!(s.store.chats().await.unwrap()[0].id, new);

        let empty = s.kb.rename_chat(&old, " ").await;
        assert!(matches!(empty, Err(Error::InvalidInput(_))));
        let unknown = s.kb.rename_chat(&ChatId::new("nope"), "x").await;
        assert!(matches!(unknown, Err(Error::NotFound(_))));
        assert_eq!(
            s.store.chat(&old).await.unwrap().unwrap().title,
            "Prüffristen"
        );
    });
}

#[test]
fn a_full_run_compares_every_file_by_content() {
    block_on(async {
        let s = setup("");
        s.ingest_all().await;
        let path = file("faq.html");
        // Content changed, size and time restored: the quick run cannot see it.
        s.files.modify_keeping_metadata(&path).unwrap();
        let quick = s.sync().await;
        assert!(matches!(
            status_of(&quick, &path),
            FileStatus::Unchanged { .. }
        ));

        let hashes = s.files.hash_count();
        let full =
            s.kb.sync(
                s.progress(),
                s.scope_callback(),
                SyncMode::Full,
                CancellationToken::new(),
            )
            .await
            .unwrap();
        assert_eq!(s.files.hash_count(), hashes + 7, "every file hashed");
        assert!(matches!(
            status_of(&full, &path),
            FileStatus::Indexed { .. }
        ));
        let others = full
            .iter()
            .filter(|r| r.path != path)
            .all(|r| matches!(r.status, FileStatus::Unchanged { .. }));
        assert!(others, "only the changed file is ingested again");
    });
}

#[test]
fn lists_and_ranges_of_numbers_become_single_markers() {
    block_on(async {
        let s = setup("A [1, 2]. B [1-2]. C [2,9]. D [2–1]. E [x].");
        s.ingest_all().await;
        let events = s.ask(QUESTION, None).await;
        let answer = done(&events);
        assert_eq!(answer.text, "A [1][2]. B [1][2]. C [2]. D [2–1]. E [x].");
        let numbers: Vec<u32> = answer.citations.iter().map(|c| c.number).collect();
        assert_eq!(numbers, [1, 2]);
    });
}

#[test]
fn every_generation_request_sets_the_answer_length() {
    block_on(async {
        let s = setup("Jährlich [1].");
        s.ingest_all().await;
        s.ask(QUESTION, None).await;
        let request = &s.generator.requests()[0];
        assert_eq!(
            request.max_tokens,
            Some(PipelineConfig::default().max_answer_tokens)
        );
    });
}

// ---------------------------------------------------------------- library scope

const FOLDER: &str = r"C:\Ablage";

fn file(name: &str) -> PathBuf {
    Path::new(FOLDER).join(name)
}

fn status_of<'a>(results: &'a [FileResult], path: &Path) -> &'a FileStatus {
    &results
        .iter()
        .find(|r| r.path == path)
        .unwrap_or_else(|| panic!("no result for {}", path.display()))
        .status
}

async fn paths_in_library(s: &Setup) -> Vec<PathBuf> {
    s.kb.documents()
        .await
        .unwrap()
        .into_iter()
        .map(|r| r.path)
        .collect()
}

async fn hits_of(s: &Setup, id: &DocumentId) -> usize {
    let request = SearchRequest {
        documents: Some(vec![id.clone()]),
        mode: SearchMode::Vector,
        ..SearchRequest::new("Prüfung", 100)
    };
    s.kb.search(&request, CancellationToken::new())
        .await
        .unwrap()
        .len()
}

#[test]
fn a_registered_folder_is_read_and_unsupported_files_are_skipped() {
    block_on(async {
        let s = setup("");
        s.files.add_file(&file("Foto.bmp")).unwrap();
        let results = s.ingest_all().await;
        assert_eq!(results.len(), 7);
        assert!(results.iter().all(|r| !r.path.ends_with("Foto.bmp")));
        assert!(s.events().iter().all(|e| !e.path.ends_with("Foto.bmp")));
        assert!(!paths_in_library(&s).await.contains(&file("Foto.bmp")));
        assert_eq!(
            s.scope_events(),
            [ScopeEvent::FolderAvailable {
                folder: PathBuf::from(FOLDER)
            }]
        );
    });
}

#[test]
fn a_vanished_file_is_removed_with_chunks_content_and_record_even_if_failed() {
    block_on(async {
        let parser = Arc::new(BrokenFileParser {
            inner: MockParser::new(),
        });
        let s = setup_with(
            parser,
            Some(Arc::new(MockReranker)),
            "",
            64,
            PipelineConfig::default(),
        );
        s.ingest_all().await;
        let failed =
            s.kb.document(&DocumentId::new("doc-onboarding"))
                .await
                .unwrap()
                .unwrap();
        assert!(matches!(
            failed.status,
            DocumentStatus::Failed {
                kind: ErrorKind::Parse,
                ..
            }
        ));

        s.files.remove(&file("onboarding.docx")).unwrap();
        s.files.remove(&file("faq.html")).unwrap();
        let results = s.sync().await;
        assert!(matches!(
            status_of(&results, &file("onboarding.docx")),
            FileStatus::Removed { .. }
        ));
        assert!(matches!(
            status_of(&results, &file("faq.html")),
            FileStatus::Removed { .. }
        ));
        let faq = DocumentId::new("doc-faq");
        assert!(s.kb.document(&faq).await.unwrap().is_none());
        assert!(s.kb.document_content(&faq).await.unwrap().is_none());
        assert_eq!(hits_of(&s, &faq).await, 0);
        assert!(s.kb.document(&failed.id).await.unwrap().is_none());
        assert_eq!(paths_in_library(&s).await.len(), 5);
    });
}

#[test]
fn a_deregistered_folder_takes_its_documents_but_a_nested_folder_keeps_its_own() {
    block_on(async {
        let s = setup("");
        let archive = file("Archiv");
        s.files.add_file(&archive.join("notizen.md")).unwrap();
        s.kb.add_folder(Path::new(FOLDER)).await.unwrap();
        s.kb.add_folder(&archive).await.unwrap();
        let results = s.sync().await;
        assert_eq!(results.len(), 8, "the nested file is processed once");

        s.kb.remove_folder(Path::new(FOLDER)).await.unwrap();
        let results = s.sync().await;
        assert_eq!(
            results
                .iter()
                .filter(|r| matches!(r.status, FileStatus::Removed { .. }))
                .count(),
            7
        );
        assert_eq!(paths_in_library(&s).await, [archive.join("notizen.md")]);
    });
}

#[test]
fn an_unreachable_folder_keeps_its_documents_and_is_named() {
    block_on(async {
        let s = setup("");
        s.ingest_all().await;
        s.events.lock().unwrap().clear();
        s.scope.lock().unwrap().clear();

        s.files.set_reachable(Path::new(FOLDER), false).unwrap();
        let results = s.sync().await;
        assert!(results.is_empty(), "nothing processed, nothing removed");
        assert_eq!(paths_in_library(&s).await.len(), 7);
        let events = s.scope_events();
        assert!(matches!(
            events.as_slice(),
            [ScopeEvent::FolderUnavailable { folder, kind: ErrorKind::Io, .. }]
                if folder == Path::new(FOLDER)
        ));

        // Deleted while unreachable: removed once the folder can be read again.
        s.files.remove(&file("faq.html")).unwrap();
        assert_eq!(s.sync().await.len(), 0);
        s.files.set_reachable(Path::new(FOLDER), true).unwrap();
        let results = s.sync().await;
        assert!(matches!(
            status_of(&results, &file("faq.html")),
            FileStatus::Removed { .. }
        ));
        assert_eq!(paths_in_library(&s).await.len(), 6);
    });
}

#[test]
fn documents_below_an_unreadable_subfolder_stay() {
    block_on(async {
        let s = setup("");
        let archive = file("Archiv");
        s.files.add_file(&archive.join("notizen.md")).unwrap();
        s.ingest_all().await;
        s.files.set_reachable(&archive, false).unwrap();
        s.sync().await;
        assert!(
            paths_in_library(&s)
                .await
                .contains(&archive.join("notizen.md"))
        );
    });
}

#[test]
fn excluded_files_and_subfolders_leave_the_library_and_come_back() {
    block_on(async {
        let s = setup("");
        let archive = file("Archiv");
        s.files.add_file(&archive.join("notizen.md")).unwrap();
        s.files
            .add_file(&archive.join("Alt").join("faq.html"))
            .unwrap();
        s.ingest_all().await;
        assert_eq!(paths_in_library(&s).await.len(), 9);

        s.kb.exclude(&file("budget.xlsx")).await.unwrap();
        s.kb.exclude(&archive).await.unwrap();
        let results = s.sync().await;
        let removed: Vec<&Path> = results
            .iter()
            .filter(|r| matches!(r.status, FileStatus::Removed { .. }))
            .map(|r| r.path.as_path())
            .collect();
        assert_eq!(
            removed.len(),
            3,
            "the file and everything below the subfolder"
        );
        assert!(!paths_in_library(&s).await.contains(&file("budget.xlsx")));
        assert_eq!(hits_of(&s, &DocumentId::new("doc-budget")).await, 0);
        assert!(
            s.files.exists(&file("budget.xlsx")),
            "the file stays on disk"
        );

        let settings = s.kb.include(&archive).await.unwrap();
        assert_eq!(settings.excluded, [file("budget.xlsx")]);
        let results = s.sync().await;
        assert!(matches!(
            status_of(&results, &archive.join("notizen.md")),
            FileStatus::Indexed { .. }
        ));
        assert_eq!(paths_in_library(&s).await.len(), 8);
    });
}

#[test]
fn exclusions_of_vanished_paths_are_dropped_unless_the_parent_is_unreadable() {
    block_on(async {
        let s = setup("");
        let archive = file("Archiv");
        s.files.add_file(&archive.join("notizen.md")).unwrap();
        s.ingest_all().await;
        s.kb.exclude(&archive.join("notizen.md")).await.unwrap();
        s.kb.exclude(&file("faq.html")).await.unwrap();

        // Parent unreadable: absence is not established, the exclusion stays.
        s.files.remove(&archive.join("notizen.md")).unwrap();
        s.files.set_reachable(&archive, false).unwrap();
        s.sync().await;
        assert_eq!(s.store.settings().await.unwrap().excluded.len(), 2);

        s.files.set_reachable(&archive, true).unwrap();
        s.files.remove(&file("faq.html")).unwrap();
        s.scope.lock().unwrap().clear();
        s.sync().await;
        assert!(s.store.settings().await.unwrap().excluded.is_empty());
        let dropped: Vec<PathBuf> = s
            .scope_events()
            .into_iter()
            .filter_map(|e| match e {
                ScopeEvent::ExclusionRemoved { path } => Some(path),
                _ => None,
            })
            .collect();
        assert_eq!(dropped.len(), 2);
    });
}

#[test]
fn exclusions_outside_every_watched_folder_are_dropped_by_the_next_run() {
    block_on(async {
        let s = setup("");
        // Written around the facade, as `save_settings` from an older client could.
        s.store
            .save_settings(&Settings {
                watched_folders: vec![PathBuf::from(FOLDER)],
                excluded: vec![PathBuf::from(r"D:\Anderswo\x.pdf")],
                ..Settings::default()
            })
            .await
            .unwrap();
        s.sync().await;
        assert!(s.store.settings().await.unwrap().excluded.is_empty());
    });
}

#[test]
fn a_changed_file_is_indexed_again_and_an_unchanged_one_skipped() {
    block_on(async {
        let s = setup("");
        s.ingest_all().await;
        s.files.modify(&file("faq.html")).unwrap();
        let results = s.sync().await;
        assert!(matches!(
            status_of(&results, &file("faq.html")),
            FileStatus::Indexed { .. }
        ));
        assert!(matches!(
            status_of(&results, &file("notizen.md")),
            FileStatus::Unchanged { .. }
        ));
    });
}

/// A non-mock implementation mixed with mocks: fails on one file name.
struct BrokenFileParser {
    inner: MockParser,
}

#[async_trait]
impl Parser for BrokenFileParser {
    async fn parse(&self, source: &SourceFile, cancel: CancellationToken) -> Result<Document> {
        if source.name == "onboarding.docx" {
            return Err(Error::Parse("file is encrypted".into()));
        }
        self.inner.parse(source, cancel).await
    }
}

#[test]
fn a_broken_file_does_not_stop_the_run_and_progress_is_reported() {
    block_on(async {
        let parser = Arc::new(BrokenFileParser {
            inner: MockParser::new(),
        });
        let s = setup_with(
            parser,
            Some(Arc::new(MockReranker)),
            "",
            64,
            PipelineConfig::default(),
        );
        let results = s.ingest_all().await;
        assert!(matches!(
            status_of(&results, &file("onboarding.docx")),
            FileStatus::Failed {
                kind: ErrorKind::Parse,
                ..
            }
        ));
        let events = s.events();
        let last = events.last().unwrap();
        assert_eq!((last.completed, last.total), (7, 7));
        let pdf: Vec<&FileStatus> = events
            .iter()
            .filter(|e| e.path.ends_with("pruefordnung.pdf"))
            .map(|e| &e.status)
            .collect();
        assert!(matches!(
            pdf.as_slice(),
            [
                FileStatus::Queued,
                FileStatus::Parsing,
                FileStatus::Chunking,
                FileStatus::Embedding,
                FileStatus::Indexing,
                FileStatus::Indexed { .. }
            ]
        ));
    });
}

#[test]
fn unchanged_requires_an_indexed_record() {
    block_on(async {
        let s = setup("");
        s.kb.add_folder(Path::new(FOLDER)).await.unwrap();
        let path = file("faq.html");
        // A failed earlier attempt (I/O error, so it is tried again) with matching size,
        // time and even a hash: there are no chunks, so the file must be processed,
        // not reported as unchanged by the shortcut.
        s.store
            .upsert_document(&DocumentRecord {
                id: DocumentId::new("doc-faq"),
                path: path.clone(),
                format: DocumentFormat::Html,
                title: None,
                content_hash: Some(placeholder_hash(&path).unwrap()),
                // Size and modification time match too: the shortcut must not apply.
                size_bytes: Some(1000),
                modified_at: Some(Timestamp(1_794_700_800_000)),
                status: DocumentStatus::Failed {
                    kind: ErrorKind::Io,
                    message: "file was open in another program".into(),
                },
            })
            .await
            .unwrap();
        let results = s.sync().await;
        assert!(matches!(
            status_of(&results, &path),
            FileStatus::Indexed { .. }
        ));
        let again = s.sync().await;
        assert!(matches!(
            status_of(&again, &path),
            FileStatus::Unchanged { .. }
        ));
    });
}

/// Cancels the run while parsing one file, once.
struct CancellingParser {
    inner: MockParser,
    name: &'static str,
    done: AtomicBool,
}

#[async_trait]
impl Parser for CancellingParser {
    async fn parse(&self, source: &SourceFile, cancel: CancellationToken) -> Result<Document> {
        if source.name == self.name && !self.done.swap(true, Ordering::SeqCst) {
            cancel.cancel();
            return Err(Error::Cancelled);
        }
        self.inner.parse(source, cancel).await
    }
}

#[test]
fn cancel_leaves_the_current_file_queued_without_chunks_and_content() {
    block_on(async {
        let parser = Arc::new(CancellingParser {
            inner: MockParser::new(),
            name: "faq.html",
            done: AtomicBool::new(true),
        });
        let s = setup_with(
            Arc::clone(&parser) as Arc<dyn Parser>,
            Some(Arc::new(MockReranker)),
            "",
            64,
            PipelineConfig::default(),
        );
        s.ingest_all().await;
        let id = DocumentId::new("doc-faq");
        let path = file("faq.html");

        // The file changed, and the run is cancelled on it.
        s.files.modify(&path).unwrap();
        parser.done.store(false, Ordering::SeqCst);
        let error =
            s.kb.sync(
                s.progress(),
                s.scope_callback(),
                SyncMode::Normal,
                CancellationToken::new(),
            )
            .await
            .unwrap_err();
        assert!(matches!(error, Error::Cancelled));

        let record = s.kb.document(&id).await.unwrap().unwrap();
        assert_eq!(record.status, DocumentStatus::Queued);
        assert!(s.kb.document_content(&id).await.unwrap().is_none());
        assert_eq!(hits_of(&s, &id).await, 0, "no chunks of the cancelled file");

        // The next run picks the file up again.
        let results = s.sync().await;
        assert!(matches!(
            status_of(&results, &path),
            FileStatus::Indexed { .. }
        ));
        assert!(s.kb.document_content(&id).await.unwrap().is_some());
    });
}

#[test]
fn queued_records_are_resumed_by_the_next_run() {
    block_on(async {
        let s = setup("");
        s.kb.add_folder(Path::new(FOLDER)).await.unwrap();
        s.store
            .upsert_document(&DocumentRecord {
                id: DocumentId::new("doc-faq"),
                path: file("faq.html"),
                format: DocumentFormat::Html,
                title: None,
                content_hash: None,
                size_bytes: None,
                modified_at: None,
                status: DocumentStatus::Queued,
            })
            .await
            .unwrap();
        let results = s.sync().await;
        assert!(matches!(
            status_of(&results, &file("faq.html")),
            FileStatus::Indexed { .. }
        ));
        assert!(
            s.kb.document_content(&DocumentId::new("doc-faq"))
                .await
                .unwrap()
                .is_some()
        );
    });
}

#[test]
fn cancelled_run_returns_cancelled() {
    block_on(async {
        let s = setup("");
        s.kb.add_folder(Path::new(FOLDER)).await.unwrap();
        let cancel = CancellationToken::new();
        cancel.cancel();
        let error =
            s.kb.sync(s.progress(), s.scope_callback(), SyncMode::Normal, cancel)
                .await
                .unwrap_err();
        assert!(matches!(error, Error::Cancelled));
    });
}

#[test]
fn watch_synchronises_and_runs_until_cancelled() {
    block_on(async {
        let s = setup("");
        s.kb.add_folder(Path::new(FOLDER)).await.unwrap();
        let cancel = CancellationToken::new();
        let watch = s.kb.watch(s.progress(), s.scope_callback(), cancel.clone());
        let stop = async { cancel.cancel() };
        let (result, ()) = futures::join!(watch, stop);
        result.unwrap();
        assert_eq!(paths_in_library(&s).await.len(), 7);
    });
}

#[test]
fn a_chat_can_be_deleted_through_the_facade() {
    block_on(async {
        let s = setup("Jährlich [1].");
        s.ingest_all().await;
        let id = ChatId::new("chat-1");
        s.ask(QUESTION, Some(&id)).await;
        assert!(s.store.chat(&id).await.unwrap().is_some());
        s.kb.delete_chat(&id).await.unwrap();
        assert!(s.store.chat(&id).await.unwrap().is_none());
        assert!(s.store.chats().await.unwrap().is_empty());
        s.kb.delete_chat(&id).await.unwrap();
    });
}

#[test]
fn chats_carry_their_creation_and_last_question_time() {
    block_on(async {
        let s = setup("Jährlich [1].");
        s.ingest_all().await;
        let start = s.clock.now();
        let old = ChatId::new("chat-old");
        s.ask(QUESTION, Some(&old)).await;
        s.clock.advance(60_000);
        s.ask("Und Wärmepumpen?", Some(&old)).await;
        let chat = s.store.chat(&old).await.unwrap().unwrap();
        assert_eq!(chat.created_at, start);
        assert_eq!(chat.updated_at, Timestamp(start.0 + 60_000));

        // A newer chat goes first ...
        s.clock.advance(60_000);
        let new = ChatId::new("chat-new");
        s.ask(QUESTION, Some(&new)).await;
        let order: Vec<ChatId> = s
            .store
            .chats()
            .await
            .unwrap()
            .into_iter()
            .map(|c| c.id)
            .collect();
        assert_eq!(order, [new.clone(), old.clone()]);

        // ... until a question in the older chat moves it up again.
        s.clock.advance(60_000);
        s.ask(QUESTION, Some(&old)).await;
        let list = s.store.chats().await.unwrap();
        assert_eq!(list[0].id, old);
        assert_eq!(list[0].created_at, start, "creation time stays");
        assert_eq!(list[0].updated_at, s.clock.now());
    });
}

#[test]
fn unchanged_size_and_time_skip_hashing_otherwise_the_hash_decides() {
    block_on(async {
        let s = setup("");
        s.ingest_all().await;
        let hashes = s.files.hash_count();
        let record =
            s.kb.document(&DocumentId::new("doc-faq"))
                .await
                .unwrap()
                .unwrap();
        assert_eq!(record.size_bytes, Some(1000));
        assert!(record.modified_at.is_some());

        // Nothing changed: all unchanged without a single hash.
        let results = s.sync().await;
        assert!(
            results
                .iter()
                .all(|r| matches!(r.status, FileStatus::Unchanged { .. }))
        );
        assert_eq!(s.files.hash_count(), hashes, "shortcut used");

        // Only the time changed: hashed once, unchanged, record takes the new time.
        let path = file("faq.html");
        s.files.touch(&path).unwrap();
        let results = s.sync().await;
        assert!(matches!(
            status_of(&results, &path),
            FileStatus::Unchanged { .. }
        ));
        assert_eq!(s.files.hash_count(), hashes + 1);
        s.sync().await;
        assert_eq!(
            s.files.hash_count(),
            hashes + 1,
            "new time stored, no hash again"
        );

        // Content and metadata changed: indexed again.
        s.files.modify(&path).unwrap();
        let results = s.sync().await;
        assert!(matches!(
            status_of(&results, &path),
            FileStatus::Indexed { .. }
        ));

        // The documented residual risk: content changed, size and time restored.
        s.files.modify_keeping_metadata(&path).unwrap();
        let results = s.sync().await;
        assert!(matches!(
            status_of(&results, &path),
            FileStatus::Unchanged { .. }
        ));
    });
}

// ---------------------------------------------------------------- order with a slow model start

/// A knowledge base over the fixtures with the given answer components.
async fn kb_with(
    embedder: Arc<dyn Embedder>,
    generator: Arc<dyn Generator>,
    config: PipelineConfig,
) -> (KnowledgeBase, Arc<MockStore>) {
    let index: Arc<dyn Index> = Arc::new(MockIndex::new(embedder.space(), 0.5));
    let store = Arc::new(MockStore::new());
    let ingestor = Arc::new(MockIngestor::new(
        Arc::new(MockParser::new()),
        Arc::new(MockChunker::default()),
        Arc::clone(&embedder),
        Arc::clone(&index),
        Arc::clone(&store) as Arc<dyn Store>,
        Arc::new(MockFiles::fixtures()),
    ));
    let answerer = Arc::new(MockAnswerer::new(
        embedder,
        index,
        Some(Arc::new(MockReranker)),
        generator,
        config,
    ));
    let kb = KnowledgeBase::new(ingestor, answerer, Arc::clone(&store) as Arc<dyn Store>);
    kb.add_folder(Path::new(MockFiles::FIXTURE_FOLDER))
        .await
        .unwrap();
    kb.sync(
        Arc::new(|_| {}),
        Arc::new(|_| {}),
        SyncMode::Normal,
        CancellationToken::new(),
    )
    .await
    .unwrap();
    (kb, store)
}

/// A knowledge base over the fixtures whose answer model is still loading: counting
/// waits for the returned gate.
async fn loading_model() -> (KnowledgeBase, Arc<MockStore>, Gate) {
    let loaded = Gate::new();
    let (kb, store) = kb_with(
        Arc::new(MockEmbedder::default()),
        Arc::new(GatedGenerator::loading(
            Arc::new(MockGenerator::new("Jährlich [1].")),
            loaded.clone(),
        )),
        PipelineConfig::default(),
    )
    .await;
    (kb, store, loaded)
}

fn is_starting(event: &Option<Option<Result<AnswerEvent>>>) -> bool {
    matches!(
        event,
        Some(Some(Ok(AnswerEvent::Status {
            phase: AnswerPhase::StartingModel,
            ..
        })))
    )
}

#[test]
fn a_first_question_shows_its_sources_before_waiting_for_the_model() {
    block_on(async {
        let (kb, _, loaded) = loading_model().await;
        let mut stream = kb
            .ask(QUESTION, None, AnswerLevel::Fast, CancellationToken::new())
            .await
            .unwrap();
        // While the model is still loading: search, sources, then the start phase ...
        assert!(matches!(
            stream.next().now_or_never(),
            Some(Some(Ok(AnswerEvent::Status {
                phase: AnswerPhase::Searching,
                ..
            })))
        ));
        let sources = stream.next().now_or_never();
        assert!(
            matches!(&sources, Some(Some(Ok(AnswerEvent::Sources { hits }))) if !hits.is_empty()),
            "sources must not wait for the model, got {sources:?}"
        );
        assert!(is_starting(&stream.next().now_or_never()));
        // ... and then the answer waits for the model.
        assert!(stream.next().now_or_never().is_none());

        loaded.open();
        let rest: Vec<AnswerEvent> = stream.map(|e| e.unwrap()).collect().await;
        assert!(matches!(
            rest.first(),
            Some(AnswerEvent::Status {
                phase: AnswerPhase::ReadingSources,
                ..
            })
        ));
        assert!(matches!(rest.last(), Some(AnswerEvent::Done { .. })));
    });
}

#[test]
fn a_follow_up_announces_the_model_start_before_rewriting_and_searching() {
    block_on(async {
        let (kb, store, loaded) = loading_model().await;
        let id = ChatId::new("chat-1");
        store
            .save_chat(&Chat {
                id: id.clone(),
                title: QUESTION.into(),
                created_at: Timestamp(1),
                updated_at: Timestamp(1),
                turns: vec![ChatTurn {
                    question: QUESTION.into(),
                    answer: Answer {
                        text: "Jährlich [1].".into(),
                        citations: Vec::new(),
                        no_sources: false,
                        level: AnswerLevel::Fast,
                        outcome: AnswerOutcome::Complete,
                    },
                }],
            })
            .await
            .unwrap();
        let mut stream = kb
            .ask(
                "Und Wärmepumpen?",
                Some(&id),
                AnswerLevel::Fast,
                CancellationToken::new(),
            )
            .await
            .unwrap();
        assert!(is_starting(&stream.next().now_or_never()));
        // Rewriting needs the model, so the sources wait for it.
        assert!(stream.next().now_or_never().is_none());

        loaded.open();
        let rest: Vec<AnswerEvent> = stream.map(|e| e.unwrap()).collect().await;
        assert_eq!(&shape(&rest)[..2], ["searching", "sources"]);
        let starts = rest
            .iter()
            .filter(|e| {
                matches!(
                    e,
                    AnswerEvent::Status {
                        phase: AnswerPhase::StartingModel,
                        ..
                    }
                )
            })
            .count();
        assert_eq!(starts, 0, "the start phase comes only once");
        assert!(matches!(rest.last(), Some(AnswerEvent::Done { .. })));
    });
}

// ---------------------------------------------------------------- folders, failures, cancel

#[test]
fn a_file_registered_as_folder_is_unavailable_and_nothing_is_ingested() {
    block_on(async {
        let s = setup("");
        let path = file("faq.html");
        s.kb.add_folder(&path).await.unwrap();
        let results = s.sync().await;
        assert!(results.is_empty());
        assert!(paths_in_library(&s).await.is_empty());
        assert!(matches!(
            s.scope_events().as_slice(),
            [ScopeEvent::FolderUnavailable { folder, .. }] if *folder == path
        ));
    });
}

#[test]
fn a_missing_watched_folder_is_unavailable_and_keeps_its_documents() {
    block_on(async {
        let s = setup("");
        s.ingest_all().await;
        s.files.remove(Path::new(FOLDER)).unwrap();
        s.scope.lock().unwrap().clear();
        let results = s.sync().await;
        assert!(results.is_empty());
        assert_eq!(paths_in_library(&s).await.len(), 7);
        assert!(matches!(
            s.scope_events().as_slice(),
            [ScopeEvent::FolderUnavailable { .. }]
        ));
    });
}

/// Fails on one file name while `failing` is set, with the given error.
struct SwitchableParser {
    inner: MockParser,
    name: &'static str,
    failing: AtomicBool,
    io: bool,
}

#[async_trait]
impl Parser for SwitchableParser {
    async fn parse(&self, source: &SourceFile, cancel: CancellationToken) -> Result<Document> {
        if source.name == self.name && self.failing.load(Ordering::SeqCst) {
            return Err(if self.io {
                Error::Io(std::io::Error::other("file is open in another program"))
            } else {
                Error::Parse("file is encrypted".into())
            });
        }
        self.inner.parse(source, cancel).await
    }
}

fn switchable(io: bool) -> (Setup, Arc<SwitchableParser>) {
    let parser = Arc::new(SwitchableParser {
        inner: MockParser::new(),
        name: "faq.html",
        failing: AtomicBool::new(false),
        io,
    });
    let s = setup_with(
        Arc::clone(&parser) as Arc<dyn Parser>,
        Some(Arc::new(MockReranker)),
        "",
        64,
        PipelineConfig::default(),
    );
    (s, parser)
}

#[test]
fn a_failure_removes_chunks_content_and_hash_of_the_old_version() {
    block_on(async {
        let (s, parser) = switchable(false);
        s.ingest_all().await;
        let id = DocumentId::new("doc-faq");
        let path = file("faq.html");
        s.files.modify(&path).unwrap();
        parser.failing.store(true, Ordering::SeqCst);
        let results = s.sync().await;
        assert!(matches!(
            status_of(&results, &path),
            FileStatus::Failed { .. }
        ));
        let record = s.kb.document(&id).await.unwrap().unwrap();
        assert_eq!(
            record.content_hash, None,
            "an old citation counts as outdated"
        );
        assert_eq!(
            record.size_bytes,
            Some(1001),
            "metadata of the failed attempt"
        );
        assert!(s.kb.document_content(&id).await.unwrap().is_none());
        assert_eq!(hits_of(&s, &id).await, 0);
    });
}

#[test]
fn a_failed_file_is_retried_when_changed_asked_for_full_or_after_an_io_error() {
    block_on(async {
        let (s, parser) = switchable(false);
        parser.failing.store(true, Ordering::SeqCst);
        s.ingest_all().await;
        let path = file("faq.html");

        // Same size and time, a parser error: not tried again, no hashing for it.
        let hashes = s.files.hash_count();
        let results = s.sync().await;
        assert!(
            results.iter().all(|r| r.path != path),
            "takes no part in the run"
        );
        assert_eq!(s.files.hash_count(), hashes);

        // Changed time: tried again.
        s.files.touch(&path).unwrap();
        let results = s.sync().await;
        assert!(matches!(
            status_of(&results, &path),
            FileStatus::Failed { .. }
        ));

        // "Try again" tries it again even unchanged, hashing no other file.
        let hashes = s.files.hash_count();
        let results =
            s.kb.sync(
                s.progress(),
                s.scope_callback(),
                SyncMode::RetryFailed,
                CancellationToken::new(),
            )
            .await
            .unwrap();
        assert!(matches!(
            status_of(&results, &path),
            FileStatus::Failed { .. }
        ));
        assert_eq!(s.files.hash_count(), hashes + 1, "only the failed file");

        // A full run tries it again even unchanged; now it succeeds.
        parser.failing.store(false, Ordering::SeqCst);
        let results =
            s.kb.sync(
                s.progress(),
                s.scope_callback(),
                SyncMode::Full,
                CancellationToken::new(),
            )
            .await
            .unwrap();
        assert!(matches!(
            status_of(&results, &path),
            FileStatus::Indexed { .. }
        ));

        // An I/O error (file open elsewhere) is tried again at the next run.
        let (s, parser) = switchable(true);
        parser.failing.store(true, Ordering::SeqCst);
        s.ingest_all().await;
        parser.failing.store(false, Ordering::SeqCst);
        let results = s.sync().await;
        assert!(matches!(
            status_of(&results, &path),
            FileStatus::Indexed { .. }
        ));
    });
}

#[test]
fn a_run_cancelled_before_it_starts_changes_nothing() {
    block_on(async {
        let s = setup("");
        s.ingest_all().await;
        s.kb.exclude(&file("budget.xlsx")).await.unwrap();
        s.kb.exclude(&file("gibt-es-nicht.pdf")).await.unwrap();
        let settings = s.store.settings().await.unwrap();
        let cancel = CancellationToken::new();
        cancel.cancel();
        let error =
            s.kb.sync(s.progress(), s.scope_callback(), SyncMode::Normal, cancel)
                .await
                .unwrap_err();
        assert!(matches!(error, Error::Cancelled));
        assert_eq!(
            s.store.settings().await.unwrap(),
            settings,
            "no exclusion dropped"
        );
        assert_eq!(paths_in_library(&s).await.len(), 7, "nothing removed");
        assert!(
            s.scope_events()
                .iter()
                .all(|e| !matches!(e, ScopeEvent::ExclusionRemoved { .. }))
        );
    });
}

#[test]
fn the_shortcut_needs_equal_size_too() {
    block_on(async {
        let s = setup("");
        s.ingest_all().await;
        let path = file("faq.html");
        s.files.modify_keeping_time(&path).unwrap();
        let results = s.sync().await;
        assert!(matches!(
            status_of(&results, &path),
            FileStatus::Indexed { .. }
        ));
    });
}

#[test]
fn an_excluded_document_below_an_unreadable_folder_is_removed() {
    block_on(async {
        let s = setup("");
        let archive = file("Archiv");
        s.files.add_file(&archive.join("notizen.md")).unwrap();
        s.ingest_all().await;
        s.files.set_reachable(&archive, false).unwrap();
        s.kb.exclude(&archive.join("notizen.md")).await.unwrap();
        s.sync().await;
        assert!(
            !paths_in_library(&s)
                .await
                .contains(&archive.join("notizen.md"))
        );
    });
}

#[test]
fn the_exclusion_of_an_empty_folder_stays() {
    block_on(async {
        let s = setup("");
        let empty = file("Leer");
        s.files.add_dir(&empty).unwrap();
        s.ingest_all().await;
        s.kb.exclude(&empty).await.unwrap();
        s.sync().await;
        assert_eq!(s.store.settings().await.unwrap().excluded, [empty]);
    });
}

#[test]
fn a_new_chat_takes_the_time_of_its_question_and_a_trimmed_title() {
    block_on(async {
        let s = setup("Jährlich [1].");
        s.ingest_all().await;
        let asked = s.clock.now();
        let id = ChatId::new("chat-1");
        let stream =
            s.kb.ask(
                "  Wie oft wird ein Gerät geprüft?  ",
                Some(&id),
                AnswerLevel::Fast,
                CancellationToken::new(),
            )
            .await
            .unwrap();
        // The answer takes a while; the chat keeps the time the question was asked.
        s.clock.advance(30_000);
        stream.collect::<Vec<_>>().await;
        let chat = s.store.chat(&id).await.unwrap().unwrap();
        assert_eq!((chat.created_at, chat.updated_at), (asked, asked));
        assert_eq!(chat.title, "Wie oft wird ein Gerät geprüft?");
    });
}

// ---------------------------------------------------------------- early ends

fn outcome_of_done(events: &[Result<AnswerEvent>]) -> AnswerOutcome {
    match events.last() {
        Some(Ok(AnswerEvent::Done { answer })) => answer.outcome.clone(),
        other => panic!("Done expected, got {other:?}"),
    }
}

async fn save_follow_up_chat(store: &MockStore, id: &ChatId) {
    store
        .save_chat(&Chat {
            id: id.clone(),
            title: QUESTION.into(),
            created_at: Timestamp(1),
            updated_at: Timestamp(1),
            turns: vec![ChatTurn {
                question: QUESTION.into(),
                answer: Answer {
                    text: "Jährlich [1].".into(),
                    citations: Vec::new(),
                    no_sources: false,
                    level: AnswerLevel::Fast,
                    outcome: AnswerOutcome::Complete,
                },
            }],
        })
        .await
        .unwrap();
}

#[test]
fn a_cancel_while_the_model_loads_ends_at_once() {
    block_on(async {
        // Case A: cancelled in StartingModel, after the sources.
        let (kb, _, _loaded) = loading_model().await;
        let cancel = CancellationToken::new();
        let mut stream = kb
            .ask(QUESTION, None, AnswerLevel::Fast, cancel.clone())
            .await
            .unwrap();
        for _ in 0..3 {
            stream
                .next()
                .now_or_never()
                .expect("searching, sources, starting");
        }
        assert!(
            stream.next().now_or_never().is_none(),
            "waits for the model"
        );
        cancel.cancel();
        let end = stream.next().now_or_never().expect("Done at once").unwrap();
        let Ok(AnswerEvent::Done { answer }) = end else {
            panic!("Done expected, got {end:?}");
        };
        assert_eq!(answer.outcome, AnswerOutcome::Cancelled);
        assert!(stream.next().now_or_never().expect("ended").is_none());

        // Case B: cancelled in StartingModel, before the search.
        let (kb, store, _loaded) = loading_model().await;
        let id = ChatId::new("chat-1");
        save_follow_up_chat(&store, &id).await;
        let cancel = CancellationToken::new();
        let mut stream = kb
            .ask("Und wie oft?", Some(&id), AnswerLevel::Fast, cancel.clone())
            .await
            .unwrap();
        assert!(is_starting(&stream.next().now_or_never()));
        assert!(
            stream.next().now_or_never().is_none(),
            "waits for the model"
        );
        cancel.cancel();
        let mut rest = Vec::new();
        while let Some(item) = stream.next().now_or_never().expect("no more waiting") {
            rest.push(item);
        }
        let labels: Vec<&str> = shape(
            &rest
                .iter()
                .map(|e| e.as_ref().unwrap().clone())
                .collect::<Vec<_>>(),
        );
        assert_eq!(labels, ["sources0", "done"]);
        assert_eq!(outcome_of_done(&rest), AnswerOutcome::Cancelled);
    });
}

/// An embedder whose query embedding fails, so the search fails.
struct BrokenQueries(MockEmbedder);

#[async_trait]
impl Embedder for BrokenQueries {
    fn space(&self) -> EmbeddingSpace {
        self.0.space()
    }

    async fn embed_query(&self, _query: &str) -> Result<Embedding> {
        Err(Error::Model("embedding model missing".into()))
    }

    async fn embed_documents(
        &self,
        chunks: &[Chunk],
        cancel: CancellationToken,
    ) -> Result<Vec<Embedding>> {
        self.0.embed_documents(chunks, cancel).await
    }
}

#[test]
fn an_end_before_the_search_is_done_sends_empty_sources_then_done() {
    block_on(async {
        let embedder = Arc::new(BrokenQueries(MockEmbedder::default()));
        let index: Arc<dyn Index> = Arc::new(MockIndex::new(embedder.space(), 0.5));
        let answerer = MockAnswerer::new(
            embedder,
            index,
            Some(Arc::new(MockReranker)),
            Arc::new(MockGenerator::new("x")),
            PipelineConfig::default(),
        );
        let collect = |history: Vec<ChatTurn>, cancel: CancellationToken| {
            let answerer = answerer.clone();
            async move {
                answerer
                    .ask(
                        QUESTION,
                        &history,
                        AnswerLevel::Fast,
                        UiLanguage::German,
                        cancel,
                    )
                    .await
                    .unwrap()
                    .collect::<Vec<_>>()
                    .await
            }
        };
        let plain = |items: &[Result<AnswerEvent>]| -> Vec<AnswerEvent> {
            items.iter().map(|e| e.as_ref().unwrap().clone()).collect()
        };

        // Case A, search fails.
        let a = collect(Vec::new(), CancellationToken::new()).await;
        assert_eq!(shape(&plain(&a)), ["searching", "sources0", "done"]);
        assert!(matches!(
            outcome_of_done(&a),
            AnswerOutcome::Failed {
                kind: ErrorKind::Model,
                ..
            }
        ));
        let Some(Ok(AnswerEvent::Done { answer })) = a.last() else {
            unreachable!()
        };
        assert!(!answer.no_sources, "a failure is no missing source");

        // Case B, search fails.
        let turn = ChatTurn {
            question: "a".into(),
            answer: Answer {
                text: "b".into(),
                citations: Vec::new(),
                no_sources: false,
                level: AnswerLevel::Fast,
                outcome: AnswerOutcome::Complete,
            },
        };
        let b = collect(vec![turn], CancellationToken::new()).await;
        assert_eq!(
            shape(&plain(&b)),
            ["starting", "searching", "sources0", "done"]
        );
        assert!(matches!(outcome_of_done(&b), AnswerOutcome::Failed { .. }));

        // Case A, cancelled before the search.
        let cancel = CancellationToken::new();
        cancel.cancel();
        let c = collect(Vec::new(), cancel).await;
        assert_eq!(shape(&plain(&c)), ["searching", "sources0", "done"]);
        assert_eq!(outcome_of_done(&c), AnswerOutcome::Cancelled);
    });
}

#[test]
fn a_file_registered_as_folder_keeps_no_old_document() {
    block_on(async {
        let s = setup("");
        s.ingest_all().await;
        let path = file("faq.html");
        s.kb.add_folder(&path).await.unwrap();
        s.kb.remove_folder(Path::new(FOLDER)).await.unwrap();
        s.scope.lock().unwrap().clear();
        s.sync().await;
        assert!(
            paths_in_library(&s).await.is_empty(),
            "no single file stays"
        );
        assert_eq!(hits_of(&s, &DocumentId::new("doc-faq")).await, 0);
        assert!(matches!(
            s.scope_events().as_slice(),
            [ScopeEvent::FolderUnavailable {
                kind: ErrorKind::InvalidInput,
                ..
            }]
        ));
    });
}

// ---------------------------------------------------------------- timing with gated components

/// Reads the events that are ready without waiting.
fn ready_events(stream: &mut AnswerStream, into: &mut Vec<AnswerEvent>) {
    while let Some(Some(item)) = stream.next().now_or_never() {
        into.push(item.unwrap());
    }
}

/// Reads the rest of a stream that must not wait any more.
fn rest_at_once(stream: &mut AnswerStream, into: &mut Vec<AnswerEvent>) {
    while let Some(item) = stream.next().now_or_never().expect("no more waiting") {
        into.push(item.unwrap());
    }
}

#[test]
fn searching_is_shown_and_a_cancel_ends_at_once_while_the_search_runs() {
    block_on(async {
        for follow_up in [false, true] {
            let gate = Gate::new();
            let (kb, store) = kb_with(
                Arc::new(GatedEmbedder::new(MockEmbedder::default(), gate.clone())),
                Arc::new(MockGenerator::new("Jährlich [1].")),
                PipelineConfig::default(),
            )
            .await;
            let id = ChatId::new("chat-1");
            if follow_up {
                save_follow_up_chat(&store, &id).await;
            }
            let cancel = CancellationToken::new();
            let mut stream = kb
                .ask(
                    QUESTION,
                    follow_up.then_some(&id),
                    AnswerLevel::Fast,
                    cancel.clone(),
                )
                .await
                .unwrap();
            let mut events = Vec::new();
            ready_events(&mut stream, &mut events);
            let waiting: &[&str] = if follow_up {
                &["starting", "searching"]
            } else {
                &["searching"]
            };
            assert_eq!(shape(&events), waiting, "while the embedder waits");

            cancel.cancel();
            rest_at_once(&mut stream, &mut events);
            assert_eq!(&shape(&events)[waiting.len()..], ["sources0", "done"]);
            assert_eq!(done(&events).outcome, AnswerOutcome::Cancelled);
            assert_eq!(answer_order_violation(&events, follow_up), None);
            assert!(!gate.is_open());
        }
    });
}

#[test]
fn a_cancel_ends_at_once_even_if_the_generator_does_not() {
    block_on(async {
        for follow_up in [false, true] {
            let gate = Gate::new();
            let generator = GatedGenerator::stalling(
                Arc::new(MockGenerator::new("Jährlich [1].")),
                gate.clone(),
            );
            let (kb, store) = kb_with(
                Arc::new(MockEmbedder::default()),
                Arc::new(generator),
                PipelineConfig::default(),
            )
            .await;
            let id = ChatId::new("chat-1");
            if follow_up {
                save_follow_up_chat(&store, &id).await;
            }
            let cancel = CancellationToken::new();
            let mut stream = kb
                .ask(
                    QUESTION,
                    follow_up.then_some(&id),
                    AnswerLevel::Fast,
                    cancel.clone(),
                )
                .await
                .unwrap();
            let mut events = Vec::new();
            ready_events(&mut stream, &mut events);
            // Case A waits in the answer call, case B already in the rewriting call.
            let (waiting, end): (&[&str], &[&str]) = if follow_up {
                (&["starting", "searching"], &["sources0", "done"])
            } else {
                (&["searching", "sources", "starting", "reading"], &["done"])
            };
            assert_eq!(shape(&events), waiting);

            cancel.cancel();
            rest_at_once(&mut stream, &mut events);
            assert_eq!(&shape(&events)[waiting.len()..], end);
            assert_eq!(done(&events).outcome, AnswerOutcome::Cancelled);
            assert_eq!(answer_order_violation(&events, follow_up), None);
        }
    });
}

#[test]
fn a_follow_up_that_is_not_rewritten_keeps_case_b() {
    block_on(async {
        let generator = Arc::new(MockGenerator::new("Jährlich [1]."));
        let config = PipelineConfig {
            rewrite_history_tokens: 1,
            ..PipelineConfig::default()
        };
        let (kb, store) = kb_with(
            Arc::new(MockEmbedder::default()),
            Arc::clone(&generator) as Arc<dyn Generator>,
            config,
        )
        .await;
        let id = ChatId::new("chat-1");
        save_follow_up_chat(&store, &id).await;
        let events: Vec<AnswerEvent> = kb
            .ask(
                "Und wie oft wird geprüft?",
                Some(&id),
                AnswerLevel::Fast,
                CancellationToken::new(),
            )
            .await
            .unwrap()
            .map(|e| e.unwrap())
            .collect()
            .await;
        assert_eq!(generator.requests().len(), 1, "no rewriting call");
        assert_eq!(shape(&events), CASE_B);
        assert_eq!(answer_order_violation(&events, true), None);
    });
}

#[test]
fn answers_run_one_after_another() {
    block_on(async {
        let (kb, _) = kb_with(
            Arc::new(MockEmbedder::default()),
            Arc::new(MockGenerator::new("Jährlich [1].")),
            PipelineConfig::default(),
        )
        .await;
        let ask = |cancel: CancellationToken| kb.ask(QUESTION, None, AnswerLevel::Fast, cancel);

        // The first answer takes its turn and stops after its sources.
        let mut first = ask(CancellationToken::new()).await.unwrap();
        let mut first_events = Vec::new();
        for _ in 0..2 {
            first_events.push(first.next().now_or_never().unwrap().unwrap().unwrap());
        }
        assert_eq!(shape(&first_events), ["searching", "sources"]);

        // Two more report `Waiting` and nothing else; a cancel ends the waiting one at
        // once.
        let cancel = CancellationToken::new();
        let mut second = ask(cancel.clone()).await.unwrap();
        let mut third = ask(CancellationToken::new()).await.unwrap();
        let mut second_events = Vec::new();
        let mut third_events = Vec::new();
        ready_events(&mut second, &mut second_events);
        ready_events(&mut third, &mut third_events);
        assert_eq!(shape(&second_events), ["waiting"], "waits for its turn");
        assert_eq!(shape(&third_events), ["waiting"], "waits for its turn");
        cancel.cancel();
        rest_at_once(&mut second, &mut second_events);
        assert_eq!(shape(&second_events), ["waiting", "sources0", "done"]);
        assert_eq!(answer_order_violation(&second_events, false), None);
        assert_eq!(done(&second_events).outcome, AnswerOutcome::Cancelled);

        // When the first is done, the third runs through case A after its wait.
        rest_at_once(&mut first, &mut first_events);
        assert_eq!(shape(&first_events), CASE_A, "no Waiting without a wait");
        rest_at_once(&mut third, &mut third_events);
        let mut expected = vec!["waiting"];
        expected.extend(CASE_A);
        assert_eq!(shape(&third_events), expected);
        assert_eq!(answer_order_violation(&third_events, false), None);
    });
}

#[test]
fn the_answer_order_checker_finds_violations() {
    block_on(async {
        let s = setup("Jedes Gerät wird jährlich geprüft [1].");
        s.ingest_all().await;
        let a = s.ask(QUESTION, None).await;
        assert_eq!(answer_order_violation(&a, false), None);
        assert!(
            answer_order_violation(&a, true).is_some(),
            "case A is no case B"
        );
        let id = ChatId::new("chat-1");
        s.ask(QUESTION, Some(&id)).await;
        let b = s.ask("Und wie oft?", Some(&id)).await;
        assert_eq!(answer_order_violation(&b, true), None);
        assert!(
            answer_order_violation(&b, false).is_some(),
            "case B is no case A"
        );

        let is = |event: &AnswerEvent, phase: AnswerPhase| matches!(event, AnswerEvent::Status { phase: p, .. } if *p == phase);
        let position =
            |events: &[AnswerEvent], phase| events.iter().position(|e| is(e, phase)).unwrap();
        type Mutation<'a> = Box<dyn Fn(&mut Vec<AnswerEvent>) + 'a>;
        let broken: Vec<(&str, Mutation<'_>)> = vec![
            ("sources before searching", Box::new(|e| e.swap(0, 1))),
            (
                "no reading",
                Box::new(move |e| e.retain(|x| !is(x, AnswerPhase::ReadingSources))),
            ),
            (
                "no writing",
                Box::new(move |e| e.retain(|x| !is(x, AnswerPhase::Writing))),
            ),
            ("done not last", Box::new(|e| e.truncate(e.len() - 1))),
            ("two sources", Box::new(|e| e.insert(2, e[1].clone()))),
            (
                "start twice",
                Box::new(move |e| {
                    let start = position(e, AnswerPhase::StartingModel);
                    e.insert(start, e[start].clone());
                }),
            ),
            (
                "progress goes back",
                Box::new(move |e| {
                    let first = position(e, AnswerPhase::ReadingSources);
                    e.swap(first, first + 1);
                }),
            ),
            (
                "text after an empty sources",
                Box::new(|e| e[1] = AnswerEvent::Sources { hits: Vec::new() }),
            ),
            (
                "complete without sources and without no_sources",
                Box::new(|e| {
                    let last = e.pop().unwrap();
                    e.truncate(2);
                    e[1] = AnswerEvent::Sources { hits: Vec::new() };
                    let AnswerEvent::Done { mut answer } = last else {
                        unreachable!()
                    };
                    answer.text.clear();
                    answer.citations.clear();
                    e.push(AnswerEvent::Done { answer });
                }),
            ),
        ];
        for (name, mutate) in broken {
            let mut events = a.clone();
            mutate(&mut events);
            assert!(
                answer_order_violation(&events, false).is_some(),
                "not found: {name}"
            );
        }

        // `Waiting` only at the start; an empty `Sources` may end a wait or a model start
        // early.
        let waiting = AnswerEvent::Status {
            phase: AnswerPhase::Waiting,
            progress: None,
        };
        let mut waited = vec![waiting.clone()];
        waited.extend(a.iter().cloned());
        assert_eq!(answer_order_violation(&waited, false), None);
        let mut late = a.clone();
        late.insert(1, waiting.clone());
        assert!(answer_order_violation(&late, false).is_some());
        let cancelled = AnswerEvent::Done {
            answer: Answer {
                text: String::new(),
                citations: Vec::new(),
                no_sources: false,
                level: AnswerLevel::Fast,
                outcome: AnswerOutcome::Cancelled,
            },
        };
        let empty = AnswerEvent::Sources { hits: Vec::new() };
        let starting = AnswerEvent::Status {
            phase: AnswerPhase::StartingModel,
            progress: None,
        };
        for (events, history) in [
            (
                vec![waiting.clone(), empty.clone(), cancelled.clone()],
                false,
            ),
            (
                vec![starting.clone(), empty.clone(), cancelled.clone()],
                true,
            ),
        ] {
            assert_eq!(answer_order_violation(&events, history), None, "{events:?}");
        }
        let full = a
            .iter()
            .find(|e| matches!(e, AnswerEvent::Sources { .. }))
            .unwrap();
        assert!(
            answer_order_violation(&[waiting, full.clone(), cancelled], false).is_some(),
            "only an empty Sources skips ahead"
        );
    });
}

// ---------------------------------------------------------------- answer levels

#[test]
fn a_question_in_a_level_without_its_model_is_not_asked() {
    block_on(async {
        let generator = Arc::new(
            MockGenerator::new("Jährlich [1].")
                .without_model(AnswerLevel::Balanced)
                .offering(&[AnswerLevel::Fast, AnswerLevel::Balanced]),
        );
        let (kb, store) = kb_with(
            Arc::new(MockEmbedder::default()),
            Arc::clone(&generator) as Arc<dyn Generator>,
            PipelineConfig::default(),
        )
        .await;
        let id = ChatId::new("chat-1");
        let ask = |level| kb.ask(QUESTION, Some(&id), level, CancellationToken::new());

        let error = ask(AnswerLevel::Balanced).await.err().unwrap();
        assert_eq!(
            error.kind(),
            ErrorKind::ModelMissing,
            "the download is offered"
        );
        let error = ask(AnswerLevel::Thorough).await.err().unwrap();
        assert_eq!(
            error.kind(),
            ErrorKind::InvalidInput,
            "not offered on the tier"
        );
        assert!(generator.requests().is_empty() && generator.counted().is_empty());
        assert!(store.chat(&id).await.unwrap().is_none(), "nothing saved");

        let events: Vec<AnswerEvent> = ask(AnswerLevel::Fast)
            .await
            .unwrap()
            .map(|e| e.unwrap())
            .collect()
            .await;
        assert_eq!(shape(&events), CASE_A);
    });
}

#[test]
fn every_call_of_an_answer_uses_its_level_and_the_answer_keeps_it() {
    block_on(async {
        let s = setup("Jedes Gerät wird jährlich geprüft [1].");
        s.ingest_all().await;
        let id = ChatId::new("chat-1");
        s.ask(QUESTION, Some(&id)).await;
        assert_eq!(s.generator.loaded(), Some(AnswerLevel::Fast));
        let first_requests = s.generator.requests().len();
        let first_counts = s.generator.counted().len();

        // A follow-up in another level: rewriting, counting and the answer call all
        // use it, and the loaded model is replaced.
        let events: Vec<AnswerEvent> =
            s.kb.ask(
                "Und wie oft?",
                Some(&id),
                AnswerLevel::Thorough,
                CancellationToken::new(),
            )
            .await
            .unwrap()
            .map(|e| e.unwrap())
            .collect()
            .await;
        assert_eq!(
            shape(&events),
            CASE_B,
            "the model change runs under StartingModel"
        );
        let requests = s.generator.requests();
        assert_eq!(
            requests.len() - first_requests,
            2,
            "rewriting and answer call"
        );
        assert!(
            requests[first_requests..]
                .iter()
                .all(|r| r.level == AnswerLevel::Thorough)
        );
        let counted = s.generator.counted();
        assert!(counted.len() > first_counts);
        assert!(
            counted[first_counts..]
                .iter()
                .all(|l| *l == AnswerLevel::Thorough)
        );
        assert_eq!(s.generator.loaded(), Some(AnswerLevel::Thorough));
        assert_eq!(done(&events).level, AnswerLevel::Thorough);

        let chat = s.store.chat(&id).await.unwrap().unwrap();
        let levels: Vec<AnswerLevel> = chat.turns.iter().map(|t| t.answer.level).collect();
        assert_eq!(levels, [AnswerLevel::Fast, AnswerLevel::Thorough]);

        // Also an answer without sources records its level; the facade does not change
        // the level in the settings (the user interface does).
        let none =
            s.kb.ask(
                "Quantenchromodynamik?",
                None,
                AnswerLevel::Balanced,
                CancellationToken::new(),
            )
            .await
            .unwrap()
            .map(|e| e.unwrap())
            .collect::<Vec<_>>()
            .await;
        assert_eq!(done(&none).level, AnswerLevel::Balanced);
        assert_eq!(
            s.store.settings().await.unwrap().answer_level,
            AnswerLevel::Fast
        );
    });
}

#[test]
fn a_model_change_waits_under_starting_model_and_can_be_cancelled() {
    block_on(async {
        // The model of the level takes time to load (counting waits for the gate).
        let loaded = Gate::new();
        let generator = Arc::new(MockGenerator::new("Jährlich [1]."));
        let (kb, _) = kb_with(
            Arc::new(MockEmbedder::default()),
            Arc::new(GatedGenerator::loading(
                Arc::clone(&generator) as Arc<dyn Generator>,
                loaded.clone(),
            )),
            PipelineConfig::default(),
        )
        .await;
        let cancel = CancellationToken::new();
        let mut stream = kb
            .ask(QUESTION, None, AnswerLevel::Balanced, cancel.clone())
            .await
            .unwrap();
        let mut events = Vec::new();
        ready_events(&mut stream, &mut events);
        assert_eq!(shape(&events), ["searching", "sources", "starting"]);
        cancel.cancel();
        rest_at_once(&mut stream, &mut events);
        assert_eq!(done(&events).outcome, AnswerOutcome::Cancelled);
        assert_eq!(done(&events).level, AnswerLevel::Balanced);
        assert_eq!(answer_order_violation(&events, false), None);
    });
}

#[test]
fn the_answer_order_checker_compares_the_final_text_with_the_deltas() {
    block_on(async {
        let s = setup("Jedes Gerät wird jährlich geprüft [1, 2] [9].");
        s.ingest_all().await;
        let a = s.ask(QUESTION, None).await;
        assert_eq!(
            answer_order_violation(&a, false),
            None,
            "checked text is accepted"
        );
        let with_text = |events: &[AnswerEvent], text: &str| {
            let mut events = events.to_vec();
            let Some(AnswerEvent::Done { answer }) = events.last_mut() else {
                unreachable!()
            };
            answer.text = text.into();
            events
        };
        assert!(
            answer_order_violation(&with_text(&a, "Etwas anderes."), false).is_some(),
            "a final text the deltas did not carry"
        );
        let without_deltas: Vec<AnswerEvent> = a
            .iter()
            .filter(|e| !matches!(e, AnswerEvent::Delta { .. }))
            .cloned()
            .collect();
        assert!(
            answer_order_violation(&without_deltas, false).is_some(),
            "text without deltas"
        );

        // Cancelled in the model start of case A: no delta, so no text.
        let cut: Vec<AnswerEvent> = a
            .iter()
            .take_while(|e| {
                !matches!(
                    e,
                    AnswerEvent::Status {
                        phase: AnswerPhase::ReadingSources,
                        ..
                    }
                )
            })
            .cloned()
            .chain([AnswerEvent::Done {
                answer: Answer {
                    text: String::new(),
                    citations: Vec::new(),
                    no_sources: false,
                    level: AnswerLevel::Fast,
                    outcome: AnswerOutcome::Cancelled,
                },
            }])
            .collect();
        assert_eq!(shape(&cut), ["searching", "sources", "starting", "done"]);
        assert_eq!(answer_order_violation(&cut, false), None);
        assert!(answer_order_violation(&with_text(&cut, "Jährlich"), false).is_some());
    });
}

// ---------------------------------------------------------------- queue order (FIFO)

/// A knowledge base over the fixtures whose answers do not wait for anything.
async fn plain_kb() -> KnowledgeBase {
    kb_with(
        Arc::new(MockEmbedder::default()),
        Arc::new(MockGenerator::new("Jährlich [1].")),
        PipelineConfig::default(),
    )
    .await
    .0
}

/// Starts an answer and reads it up to its sources, so that it holds the turn.
async fn running(kb: &KnowledgeBase) -> (AnswerStream, Vec<AnswerEvent>) {
    let mut stream = kb
        .ask(QUESTION, None, AnswerLevel::Fast, CancellationToken::new())
        .await
        .unwrap();
    let mut events = Vec::new();
    for _ in 0..2 {
        events.push(stream.next().now_or_never().unwrap().unwrap().unwrap());
    }
    assert_eq!(shape(&events), ["searching", "sources"]);
    (stream, events)
}

fn after_waiting() -> Vec<&'static str> {
    let mut expected = vec!["waiting"];
    expected.extend(CASE_A);
    expected
}

#[test]
fn a_new_question_does_not_overtake_a_waiting_one() {
    block_on(async {
        let kb = plain_kb().await;
        let ask = || kb.ask(QUESTION, None, AnswerLevel::Fast, CancellationToken::new());
        let (mut a, mut a_events) = running(&kb).await;
        let mut b = ask().await.unwrap();
        let mut b_events = Vec::new();
        ready_events(&mut b, &mut b_events);
        assert_eq!(shape(&b_events), ["waiting"]);

        // A ends: the turn goes to B, even before B is polled again.
        rest_at_once(&mut a, &mut a_events);
        // C is asked after that and polled first: it waits for B.
        let mut c = ask().await.unwrap();
        let mut c_events = Vec::new();
        ready_events(&mut c, &mut c_events);
        assert_eq!(shape(&c_events), ["waiting"], "C does not overtake B");

        rest_at_once(&mut b, &mut b_events);
        assert_eq!(shape(&b_events), after_waiting());
        rest_at_once(&mut c, &mut c_events);
        assert_eq!(shape(&c_events), after_waiting());
    });
}

#[test]
fn the_order_follows_the_calls_not_the_first_poll() {
    block_on(async {
        let kb = plain_kb().await;
        let ask = || kb.ask(QUESTION, None, AnswerLevel::Fast, CancellationToken::new());

        // Nothing runs: X is asked first and holds the turn, though Y is polled first.
        let mut x = ask().await.unwrap();
        let mut y = ask().await.unwrap();
        let mut y_events = Vec::new();
        ready_events(&mut y, &mut y_events);
        assert_eq!(shape(&y_events), ["waiting"]);
        let mut x_events = Vec::new();
        rest_at_once(&mut x, &mut x_events);
        assert_eq!(shape(&x_events), CASE_A, "X did not wait");
        rest_at_once(&mut y, &mut y_events);
        assert_eq!(shape(&y_events), after_waiting());

        // While A runs, X and Y are asked in this order and polled the other way round.
        let (mut a, mut a_events) = running(&kb).await;
        let mut x = ask().await.unwrap();
        let mut y = ask().await.unwrap();
        let (mut x_events, mut y_events) = (Vec::new(), Vec::new());
        ready_events(&mut y, &mut y_events);
        ready_events(&mut x, &mut x_events);
        rest_at_once(&mut a, &mut a_events);
        ready_events(&mut y, &mut y_events);
        assert_eq!(shape(&y_events), ["waiting"], "Y still waits for X");
        rest_at_once(&mut x, &mut x_events);
        assert_eq!(shape(&x_events), after_waiting());
        rest_at_once(&mut y, &mut y_events);
        assert_eq!(shape(&y_events), after_waiting());
    });
}

#[test]
fn a_cancelled_waiting_question_keeps_the_order_and_starts_nothing() {
    block_on(async {
        let kb = plain_kb().await;
        let ask = |cancel: CancellationToken| kb.ask(QUESTION, None, AnswerLevel::Fast, cancel);
        let (mut a, mut a_events) = running(&kb).await;
        let cancel_b = CancellationToken::new();
        let mut b = ask(cancel_b.clone()).await.unwrap();
        let mut c = ask(CancellationToken::new()).await.unwrap();
        let mut d = ask(CancellationToken::new()).await.unwrap();
        let (mut b_events, mut c_events, mut d_events) = (Vec::new(), Vec::new(), Vec::new());
        ready_events(&mut b, &mut b_events);
        ready_events(&mut c, &mut c_events);
        ready_events(&mut d, &mut d_events);

        cancel_b.cancel();
        rest_at_once(&mut b, &mut b_events);
        assert_eq!(shape(&b_events), ["waiting", "sources0", "done"]);
        // No second answer starts while A runs, and C stays ahead of D.
        ready_events(&mut c, &mut c_events);
        ready_events(&mut d, &mut d_events);
        assert_eq!(shape(&c_events), ["waiting"]);
        assert_eq!(shape(&d_events), ["waiting"]);

        rest_at_once(&mut a, &mut a_events);
        ready_events(&mut d, &mut d_events);
        assert_eq!(shape(&d_events), ["waiting"], "D waits for C");
        rest_at_once(&mut c, &mut c_events);
        assert_eq!(shape(&c_events), after_waiting());
        rest_at_once(&mut d, &mut d_events);
        assert_eq!(shape(&d_events), after_waiting());
    });
}

// ---------------------------------------------------------------- asking again

#[test]
fn asking_again_uses_only_the_turns_before_the_original_one() {
    block_on(async {
        let s = setup("Jedes Gerät wird jährlich geprüft [1].");
        s.ingest_all().await;
        let id = ChatId::new("chat-1");
        s.ask(QUESTION, Some(&id)).await;
        s.ask("Und wie oft?", Some(&id)).await;
        s.ask("Und wer prüft?", Some(&id)).await;
        let before = s.generator.requests().len();

        // Asking the second question again: only the first turn is history (case B),
        // neither the old answer to it nor the later turn.
        let events: Vec<AnswerEvent> =
            s.kb.ask_again(&id, 1, AnswerLevel::Fast, CancellationToken::new())
                .await
                .unwrap()
                .map(|e| e.unwrap())
                .collect()
                .await;
        assert_eq!(shape(&events), CASE_B);
        let rewrite = &s.generator.requests()[before];
        assert_eq!(rewrite.messages[0].content, MockAnswerer::REWRITE_PROMPT);
        let users: Vec<&str> = rewrite
            .messages
            .iter()
            .filter(|m| m.role == Role::User)
            .map(|m| m.content.as_str())
            .collect();
        assert_eq!(users, [QUESTION, "Und wie oft?"]);

        // Appended as a new turn; the old turn stays.
        let chat = s.store.chat(&id).await.unwrap().unwrap();
        let questions: Vec<&str> = chat.turns.iter().map(|t| t.question.as_str()).collect();
        assert_eq!(
            questions,
            [QUESTION, "Und wie oft?", "Und wer prüft?", "Und wie oft?"]
        );

        // The first question asked again runs as case A.
        let first: Vec<AnswerEvent> =
            s.kb.ask_again(&id, 0, AnswerLevel::Fast, CancellationToken::new())
                .await
                .unwrap()
                .map(|e| e.unwrap())
                .collect()
                .await;
        assert_eq!(shape(&first), CASE_A);

        // Unknown turn or chat.
        // `turn == len` is beyond the last turn as well.
        let len = s.store.chat(&id).await.unwrap().unwrap().turns.len();
        for (chat, turn) in [
            (id.clone(), len),
            (id.clone(), len + 4),
            (ChatId::new("other"), 0),
        ] {
            let error =
                s.kb.ask_again(&chat, turn, AnswerLevel::Fast, CancellationToken::new())
                    .await
                    .err()
                    .unwrap();
            assert_eq!(error.kind(), ErrorKind::InvalidInput);
        }
    });
}

#[test]
fn no_second_answer_starts_while_one_is_writing() {
    block_on(async {
        let kb = plain_kb().await;
        let mut a = kb
            .ask(QUESTION, None, AnswerLevel::Fast, CancellationToken::new())
            .await
            .unwrap();
        let mut a_events = Vec::new();
        while !matches!(a_events.last(), Some(AnswerEvent::Delta { .. })) {
            a_events.push(a.next().now_or_never().unwrap().unwrap().unwrap());
        }
        assert_eq!(
            shape(&a_events),
            [
                "searching",
                "sources",
                "starting",
                "reading",
                "writing",
                "delta"
            ]
        );

        // A is writing: B waits until A's `Done`, not until A started writing.
        let mut b = kb
            .ask(QUESTION, None, AnswerLevel::Fast, CancellationToken::new())
            .await
            .unwrap();
        let mut b_events = Vec::new();
        ready_events(&mut b, &mut b_events);
        assert_eq!(shape(&b_events), ["waiting"]);

        rest_at_once(&mut a, &mut a_events);
        assert_eq!(shape(&a_events), CASE_A);
        rest_at_once(&mut b, &mut b_events);
        assert_eq!(shape(&b_events), after_waiting());
    });
}

#[test]
fn a_running_answer_dropped_midway_passes_the_turn_on() {
    block_on(async {
        let kb = plain_kb().await;
        let (a, _) = running(&kb).await;
        let mut b = kb
            .ask(QUESTION, None, AnswerLevel::Fast, CancellationToken::new())
            .await
            .unwrap();
        let mut b_events = Vec::new();
        ready_events(&mut b, &mut b_events);
        assert_eq!(shape(&b_events), ["waiting"]);

        // Dropping A's stream before its `Done` frees the turn; B does not hang.
        drop(a);
        rest_at_once(&mut b, &mut b_events);
        assert_eq!(shape(&b_events), after_waiting());
    });
}

#[test]
fn a_citation_from_slide_notes_keeps_the_notes_anchor() {
    block_on(async {
        let s = setup("Das steht in den Notizen [1].");
        s.ingest_all().await;
        let events = s
            .ask("Notiz für die Vortragenden: Hausmeister monatlich", None)
            .await;
        let notes = Anchor::Slide {
            slide: 2,
            in_notes: true,
        };
        assert_eq!(
            sources_of(&events)[0].chunk.provenance.anchors,
            std::slice::from_ref(&notes)
        );
        let answer = done(&events);
        assert_eq!(answer.citations.len(), 1);
        assert_eq!(answer.citations[0].chunk.provenance.anchors, [notes]);
    });
}
