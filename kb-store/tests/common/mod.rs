//! Helpers shared by the integration tests: runtimes, temporary folders, test data.
//!
//! Every test creates its database in a temporary folder of its own, which is removed
//! when the test ends.

#![allow(dead_code)]

use std::future::Future;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use kb_core::mock::{
    MockAnswerer, MockChunker, MockEmbedder, MockFiles, MockGenerator, MockIndex, MockIngestor,
    MockParser, MockReranker, MockTokenCounter, fixtures,
};
use kb_core::*;
use kb_store::SqliteStore;
use tempfile::TempDir;

/// Runs `future` on a runtime with several threads.
pub fn run<F: Future>(future: F) -> F::Output {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .expect("runtime")
        .block_on(future)
}

/// A temporary folder that is removed at the end of the test, also if it holds
/// read-only files.
pub struct Scratch {
    dir: Option<TempDir>,
}

impl Scratch {
    pub fn new() -> Self {
        Self {
            dir: Some(
                tempfile::Builder::new()
                    .prefix("kb-store-test")
                    .tempdir()
                    .expect("temp dir"),
            ),
        }
    }

    pub fn root(&self) -> &Path {
        self.dir.as_ref().expect("present").path()
    }

    /// Path of the database file, in a data folder that does not exist yet.
    pub fn db(&self) -> PathBuf {
        self.root().join("Daten").join("library.sqlite")
    }

    pub fn data_folder(&self) -> PathBuf {
        self.root().join("Daten")
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        if let Some(dir) = self.dir.take() {
            make_writable(dir.path());
            let _ = dir.close();
        }
    }
}

/// Clears the read-only flag of every file below `folder`.
pub fn make_writable(folder: &Path) {
    let Ok(entries) = std::fs::read_dir(folder) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            make_writable(&path);
        } else if let Ok(meta) = path.metadata() {
            let mut permissions = meta.permissions();
            #[allow(clippy::permissions_set_readonly_false)]
            permissions.set_readonly(false);
            let _ = std::fs::set_permissions(&path, permissions);
        }
    }
}

pub async fn open(path: &Path) -> SqliteStore {
    SqliteStore::open(path).await.expect("open store")
}

/// All files in `folder` (not recursive) with their bytes.
pub fn files_in(folder: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    let mut files: Vec<_> = std::fs::read_dir(folder)
        .expect("read folder")
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_file())
        .map(|p| {
            let bytes = std::fs::read(&p).expect("read file");
            (p, bytes)
        })
        .collect();
    files.sort();
    files
}

pub fn file_names(folder: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(folder)
        .expect("read folder")
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

pub fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty() && haystack.windows(needle.len()).any(|w| w == needle)
}

/// Files of `folder` whose bytes contain `needle`.
pub fn files_containing(folder: &Path, needle: &str) -> Vec<PathBuf> {
    files_in(folder)
        .into_iter()
        .filter(|(_, bytes)| contains(bytes, needle.as_bytes()))
        .map(|(p, _)| p)
        .collect()
}

pub fn record(id: &str, path: &str) -> DocumentRecord {
    DocumentRecord {
        id: DocumentId::new(id),
        path: PathBuf::from(path),
        format: DocumentFormat::from_path(Path::new(path)).unwrap_or(DocumentFormat::Pdf),
        title: None,
        content_hash: None,
        size_bytes: None,
        modified_at: None,
        status: DocumentStatus::Queued,
    }
}

/// The seven fixture documents as records (`Indexed`) and contents.
pub fn fixture_library() -> Vec<(DocumentRecord, Document)> {
    fixtures::documents()
        .into_values()
        .map(|document| {
            let record = DocumentRecord {
                id: document.source.document_id.clone(),
                path: document.source.path.clone(),
                format: document.format,
                title: document.title.clone(),
                content_hash: Some(document.source.content_hash.clone()),
                size_bytes: Some(4096),
                modified_at: Some(Timestamp(1_794_700_000_000)),
                status: DocumentStatus::Indexed {
                    chunks: u32::try_from(document.elements.len()).expect("small"),
                },
            };
            (record, document)
        })
        .collect()
}

fn answer(text: &str, outcome: AnswerOutcome, level: AnswerLevel) -> Answer {
    let hits = fixtures::hits();
    Answer {
        text: text.to_owned(),
        citations: hits
            .iter()
            .take(2)
            .enumerate()
            .map(|(i, hit)| Citation {
                number: u32::try_from(i + 1).expect("small"),
                chunk: hit.chunk.clone(),
            })
            .collect(),
        no_sources: false,
        level,
        outcome,
    }
}

/// Three chats that together hold every outcome, citations with chunk and provenance,
/// an answer without sources and every level.
pub fn sample_chats() -> Vec<Chat> {
    vec![
        Chat {
            id: ChatId::new("chat-complete"),
            title: "Wie oft wird ein Gerät geprüft?".into(),
            created_at: Timestamp(1_794_700_800_000),
            updated_at: Timestamp(1_794_700_900_000),
            turns: vec![
                ChatTurn {
                    question: "Wie oft wird ein Gerät geprüft?".into(),
                    answer: answer(
                        "Jährlich [1], bei Bedarf öfter [2].",
                        AnswerOutcome::Complete,
                        AnswerLevel::Fast,
                    ),
                },
                ChatTurn {
                    question: "Und wer prüft?".into(),
                    answer: Answer {
                        text: "Dazu habe ich in der Bibliothek nichts gefunden.".into(),
                        citations: Vec::new(),
                        no_sources: true,
                        level: AnswerLevel::Balanced,
                        outcome: AnswerOutcome::Complete,
                    },
                },
            ],
        },
        Chat {
            id: ChatId::new("chat-cancelled"),
            title: "Abgebrochen".into(),
            created_at: Timestamp(1_794_700_100_000),
            updated_at: Timestamp(1_794_700_200_000),
            turns: vec![ChatTurn {
                question: "Was steht in der Prüfordnung über Fristen?".into(),
                answer: answer(
                    "Die Frist beträgt [1] zwölf",
                    AnswerOutcome::Cancelled,
                    AnswerLevel::Thorough,
                ),
            }],
        },
        Chat {
            id: ChatId::new("chat-failed"),
            title: "Fehlgeschlagen".into(),
            created_at: Timestamp(1_794_700_300_000),
            updated_at: Timestamp(1_794_700_300_000),
            turns: vec![ChatTurn {
                question: "Welche Geräte stehen im Budget?".into(),
                answer: answer(
                    "Im Budget stehen [1] und [2] bis",
                    AnswerOutcome::Failed {
                        kind: ErrorKind::Generator,
                        message: "llama-server exited with code 3".into(),
                    },
                    AnswerLevel::Fast,
                ),
            }],
        },
    ]
}

/// Settings with every field away from its default.
pub fn sample_settings() -> Settings {
    Settings {
        watched_folders: vec![
            PathBuf::from(r"C:\Ablage"),
            PathBuf::from(r"\\nas\Freigabe\Prüfberichte"),
            PathBuf::from(r"D:\Übergabe\Größen"),
        ],
        excluded: vec![
            PathBuf::from(r"C:\Ablage\Entwürfe"),
            PathBuf::from(r"\\nas\Freigabe\Prüfberichte\alt.xyz"),
        ],
        answer_level: AnswerLevel::Thorough,
        ui_language: UiLanguage::English,
        color_scheme: ColorScheme::Dark,
    }
}

/// A large parsed document: `elements` text elements of `chars` characters each.
pub fn big_document(id: &str, elements: usize, chars: usize) -> Document {
    let line: String = "Prüfbericht Größe Maß äöü ß 0123456789 "
        .chars()
        .cycle()
        .take(chars)
        .collect();
    Document {
        source: SourceFile {
            document_id: DocumentId::new(id),
            path: PathBuf::from(format!(r"C:\Ablage\{id}.xlsx")),
            name: format!("{id}.xlsx"),
            content_hash: "0".repeat(64),
        },
        format: DocumentFormat::Xlsx,
        title: Some(format!("Große Mappe {id}")),
        language: Some("de".into()),
        pages: Vec::new(),
        elements: (0..elements)
            .map(|i| Element {
                kind: ElementKind::Text,
                text: format!("{i:07} {line}"),
                anchors: vec![Anchor::Sheet {
                    sheet: "Tabelle1".into(),
                    range: CellRange {
                        start_row: u32::try_from(i).expect("fits"),
                        start_col: 0,
                        end_row: u32::try_from(i).expect("fits"),
                        end_col: 3,
                    },
                }],
            })
            .collect(),
    }
}

/// A knowledge base on the given store with the mocks of `kb-core` around it (fixture
/// files in `C:\Ablage`, mock parser, chunker, embedder, index, reranker, generator).
pub fn knowledge_base(store: Arc<dyn Store>, clock: Arc<dyn Clock>) -> KnowledgeBase {
    let embedder: Arc<dyn Embedder> = Arc::new(MockEmbedder::new(64));
    let index: Arc<dyn Index> = Arc::new(MockIndex::new(embedder.space(), 0.5));
    let tokens: Arc<dyn TokenCounter> = Arc::new(MockTokenCounter);
    let ingestor = Arc::new(MockIngestor::new(
        Arc::new(MockParser::new()),
        Arc::new(MockChunker::new(tokens, 400)),
        Arc::clone(&embedder),
        Arc::clone(&index),
        Arc::clone(&store),
        Arc::new(MockFiles::fixtures()),
    ));
    let answerer = Arc::new(MockAnswerer::new(
        embedder,
        index,
        Some(Arc::new(MockReranker)),
        Arc::new(MockGenerator::new(
            "Geräte werden jährlich geprüft [1]. Die Frist steht in der Prüfordnung [2].",
        )) as Arc<dyn Generator>,
        PipelineConfig::default(),
    ));
    KnowledgeBase::new(ingestor, answerer, store).with_clock(clock)
}

/// Everything a store returns through its reading methods.
#[derive(Debug, PartialEq)]
pub struct Snapshot {
    pub documents: Vec<DocumentRecord>,
    pub by_id: Vec<Option<DocumentRecord>>,
    pub by_path: Vec<Option<DocumentRecord>>,
    pub contents: Vec<Option<Document>>,
    pub chats: Vec<ChatSummary>,
    pub chat_contents: Vec<Option<Chat>>,
    pub settings: Settings,
}

/// Reads everything through the reading methods, also by id and by path.
pub async fn snapshot(store: &dyn Store, extra_ids: &[&str]) -> Snapshot {
    let documents = store.documents().await.expect("documents");
    let mut ids: Vec<DocumentId> = documents.iter().map(|r| r.id.clone()).collect();
    ids.extend(extra_ids.iter().map(|id| DocumentId::new(*id)));
    let mut by_id = Vec::new();
    let mut contents = Vec::new();
    for id in &ids {
        by_id.push(store.document(id).await.expect("document"));
        contents.push(store.content(id).await.expect("content"));
    }
    let mut by_path = Vec::new();
    for record in &documents {
        if record.path.is_absolute() {
            by_path.push(store.document_by_path(&record.path).await.expect("by path"));
        }
    }
    let chats = store.chats().await.expect("chats");
    let mut chat_contents = Vec::new();
    for summary in &chats {
        chat_contents.push(store.chat(&summary.id).await.expect("chat"));
    }
    Snapshot {
        documents,
        by_id,
        by_path,
        contents,
        chats,
        chat_contents,
        settings: store.settings().await.expect("settings"),
    }
}
