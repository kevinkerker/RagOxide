//! Tests that need the internals of the crate: migrations to schemas that exist only
//! here, the connection itself, and the files the store writes.

mod crash;
mod files;
mod full;
mod migration;
mod read_only;

use std::future::Future;
use std::path::{Path, PathBuf};

use kb_core::mock::fixtures;
use kb_core::*;
use rusqlite::Transaction;

use crate::SqliteStore;
use crate::schema::{self, Migration, Schema};

/// Runs `future` on a runtime with several threads.
fn run<F: Future>(future: F) -> F::Output {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .expect("runtime")
        .block_on(future)
}

/// A temporary folder removed at the end of the test, also with read-only files in it.
struct Scratch(Option<tempfile::TempDir>);

impl Scratch {
    fn new() -> Self {
        Self(Some(
            tempfile::Builder::new()
                .prefix("kb-store-unit")
                .tempdir()
                .expect("temp dir"),
        ))
    }

    fn root(&self) -> &Path {
        self.0.as_ref().expect("present").path()
    }

    fn folder(&self) -> PathBuf {
        self.root().join("Daten")
    }

    fn db(&self) -> PathBuf {
        self.folder().join("library.sqlite")
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        if let Some(dir) = self.0.take() {
            for entry in walk(dir.path()) {
                set_read_only(&entry, false);
            }
            let _ = dir.close();
        }
    }
}

/// All files below `folder`.
fn walk(folder: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(folder) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                files.extend(walk(&path));
            } else {
                files.push(path);
            }
        }
    }
    files
}

fn set_read_only(path: &Path, read_only: bool) {
    if let Ok(meta) = path.metadata() {
        let mut permissions = meta.permissions();
        #[allow(clippy::permissions_set_readonly_false)]
        permissions.set_readonly(read_only);
        let _ = std::fs::set_permissions(path, permissions);
    }
}

fn names(folder: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(folder)
        .expect("folder")
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

/// Files of `folder` whose bytes contain `needle`.
fn files_containing(folder: &Path, needle: &str) -> Vec<PathBuf> {
    walk(folder)
        .into_iter()
        .filter(|p| {
            let bytes = std::fs::read(p).expect("read");
            bytes.windows(needle.len()).any(|w| w == needle.as_bytes())
        })
        .collect()
}

fn user_version(db: &Path) -> u32 {
    let conn =
        rusqlite::Connection::open_with_flags(db, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
            .expect("open");
    conn.pragma_query_value(None, "user_version", |r| r.get(0))
        .expect("version")
}

// ---- Schemas that exist only in the tests -------------------------------------------

fn migrate_1_to_2(tx: &Transaction<'_>) -> rusqlite::Result<()> {
    tx.execute_batch(
        "ALTER TABLE chats ADD COLUMN pinned INTEGER NOT NULL DEFAULT 0;
         CREATE TABLE chat_tags (chat_id TEXT NOT NULL, tag TEXT NOT NULL) STRICT;
         INSERT INTO chat_tags (chat_id, tag) SELECT id, 'migriert' FROM chats;",
    )
}

fn migrate_2_to_3(tx: &Transaction<'_>) -> rusqlite::Result<()> {
    tx.execute_batch(
        "CREATE TABLE notes (id TEXT PRIMARY KEY NOT NULL, note TEXT NOT NULL) STRICT;",
    )
}

fn fail_in_the_middle(tx: &Transaction<'_>) -> rusqlite::Result<()> {
    migrate_1_to_2(tx)?;
    tx.execute_batch(
        "UPDATE chats SET title = 'halb migriert';
         DELETE FROM contents;
         SELECT * FROM table_that_does_not_exist;",
    )
}

fn create_v2(tx: &Transaction<'_>) -> rusqlite::Result<()> {
    (schema::CURRENT.create)(tx)?;
    migrate_1_to_2(tx)
}

fn create_v3(tx: &Transaction<'_>) -> rusqlite::Result<()> {
    create_v2(tx)?;
    migrate_2_to_3(tx)
}

const TABLES_V2: &[(&str, &[&str])] = &[
    ("store_meta", &["key", "value"]),
    ("documents", &["id", "path_key", "record"]),
    ("contents", &["document_id", "document"]),
    (
        "chats",
        &["id", "title", "created_at", "updated_at", "turns", "pinned"],
    ),
    ("settings", &["id", "settings"]),
    ("chat_tags", &["chat_id", "tag"]),
];

const TABLES_V3: &[(&str, &[&str])] = &[
    ("store_meta", &["key", "value"]),
    ("documents", &["id", "path_key", "record"]),
    ("contents", &["document_id", "document"]),
    (
        "chats",
        &["id", "title", "created_at", "updated_at", "turns", "pinned"],
    ),
    ("settings", &["id", "settings"]),
    ("chat_tags", &["chat_id", "tag"]),
    ("notes", &["id", "note"]),
];

/// Schema 2: schema 1 plus a column and a table.
static SCHEMA_V2: Schema = Schema {
    version: 2,
    create: create_v2,
    migrations: &[Migration {
        from: 1,
        apply: migrate_1_to_2,
    }],
    tables: TABLES_V2,
};

/// Schema 3: schema 2 plus a table.
static SCHEMA_V3: Schema = Schema {
    version: 3,
    create: create_v3,
    migrations: &[
        Migration {
            from: 1,
            apply: migrate_1_to_2,
        },
        Migration {
            from: 2,
            apply: migrate_2_to_3,
        },
    ],
    tables: TABLES_V3,
};

/// Schema 2 whose migration fails after it has changed tables and data.
static SCHEMA_V2_FAILING: Schema = Schema {
    version: 2,
    create: create_v2,
    migrations: &[Migration {
        from: 1,
        apply: fail_in_the_middle,
    }],
    tables: TABLES_V2,
};

// ---- Test data ----------------------------------------------------------------------

fn record_of(document: &Document) -> DocumentRecord {
    DocumentRecord {
        id: document.source.document_id.clone(),
        path: document.source.path.clone(),
        format: document.format,
        title: document.title.clone(),
        content_hash: Some(document.source.content_hash.clone()),
        size_bytes: Some(1234),
        modified_at: Some(Timestamp(1_794_700_000_000)),
        status: DocumentStatus::Indexed { chunks: 4 },
    }
}

fn chat(id: &str, text: &str, outcome: AnswerOutcome) -> Chat {
    let hit = fixtures::hits().remove(0);
    Chat {
        id: ChatId::new(id),
        title: format!("Titel {text}"),
        created_at: Timestamp(1_794_700_800_000),
        updated_at: Timestamp(1_794_700_900_000),
        turns: vec![ChatTurn {
            question: format!("Frage {text}"),
            answer: Answer {
                text: format!("Antwort {text} [1]"),
                citations: vec![Citation {
                    number: 1,
                    chunk: hit.chunk,
                }],
                no_sources: false,
                level: AnswerLevel::Balanced,
                outcome,
            },
        }],
    }
}

fn settings() -> Settings {
    Settings {
        watched_folders: vec![PathBuf::from(r"C:\Ablage"), PathBuf::from(r"\\nas\Prüfung")],
        excluded: vec![PathBuf::from(r"C:\Ablage\Entwürfe")],
        answer_level: AnswerLevel::Thorough,
        ui_language: UiLanguage::English,
        color_scheme: ColorScheme::Dark,
    }
}

/// Fills every table: the seven fixture documents with records (one of them failed),
/// chats in all outcomes, settings; `store_meta` has its row from the creation.
async fn fill(store: &SqliteStore) {
    for document in fixtures::documents().into_values() {
        let mut record = record_of(&document);
        if document.format == DocumentFormat::Html {
            record.content_hash = None;
            record.status = DocumentStatus::Failed {
                kind: ErrorKind::Parse,
                message: "corrupt".into(),
            };
        }
        store.upsert_document(&record).await.expect("record");
        store.save_content(&document).await.expect("content");
    }
    store
        .save_chat(&chat("c1", "eins", AnswerOutcome::Complete))
        .await
        .expect("chat");
    store
        .save_chat(&chat("c2", "zwei", AnswerOutcome::Cancelled))
        .await
        .expect("chat");
    store
        .save_chat(&chat(
            "c3",
            "drei",
            AnswerOutcome::Failed {
                kind: ErrorKind::Generator,
                message: "crashed".into(),
            },
        ))
        .await
        .expect("chat");
    store.save_settings(&settings()).await.expect("settings");
}

/// What the reading methods return.
#[derive(Debug, PartialEq)]
struct Snapshot {
    documents: Vec<DocumentRecord>,
    contents: Vec<Option<Document>>,
    chats: Vec<ChatSummary>,
    chat_contents: Vec<Option<Chat>>,
    settings: Settings,
}

async fn snapshot(store: &SqliteStore) -> Snapshot {
    let documents = store.documents().await.expect("documents");
    let mut contents = Vec::new();
    for record in &documents {
        contents.push(store.content(&record.id).await.expect("content"));
    }
    let chats = store.chats().await.expect("chats");
    let mut chat_contents = Vec::new();
    for summary in &chats {
        chat_contents.push(store.chat(&summary.id).await.expect("chat"));
    }
    Snapshot {
        documents,
        contents,
        chats,
        chat_contents,
        settings: store.settings().await.expect("settings"),
    }
}

/// A large content as in G38: `elements` text elements of about 210 characters.
fn big_document(id: &str, elements: usize) -> Document {
    let mut document = fixtures::documents().remove("budget.xlsx").expect("budget");
    document.source.document_id = DocumentId::new(id);
    let line: String = "Größe Maß Prüfung äöü ß 0123456789 "
        .chars()
        .cycle()
        .take(200)
        .collect();
    document.elements = (0..elements)
        .map(|i| Element {
            kind: ElementKind::Text,
            text: format!("{i:09} {line}"),
            anchors: vec![Anchor::Paragraph {
                element: u32::try_from(i).expect("fits"),
            }],
        })
        .collect();
    document
}
