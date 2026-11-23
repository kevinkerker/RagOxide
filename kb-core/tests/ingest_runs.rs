//! Overlapping synchronisations of the `MockIngestor` run one after another (K48), so
//! that a new file gets exactly one record.

use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use async_trait::async_trait;
use futures::executor::block_on;
use kb_core::mock::*;
use kb_core::*;

/// Returns `Pending` once and wakes itself, so that another future gets its turn.
struct YieldOnce(bool);

impl Future for YieldOnce {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        if self.0 {
            Poll::Ready(())
        } else {
            self.0 = true;
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

/// A [`MockStore`] whose lookup by path gives way to other tasks after it has read,
/// like a store on another thread: two runs can both see a file as new before either
/// has written its record.
struct YieldingStore(MockStore);

#[async_trait]
impl Store for YieldingStore {
    async fn upsert_document(&self, record: &DocumentRecord) -> Result<()> {
        self.0.upsert_document(record).await
    }
    async fn document(&self, id: &DocumentId) -> Result<Option<DocumentRecord>> {
        self.0.document(id).await
    }
    async fn document_by_path(&self, path: &Path) -> Result<Option<DocumentRecord>> {
        let found = self.0.document_by_path(path).await;
        YieldOnce(false).await;
        found
    }
    async fn documents(&self) -> Result<Vec<DocumentRecord>> {
        self.0.documents().await
    }
    async fn delete_document(&self, id: &DocumentId) -> Result<()> {
        self.0.delete_document(id).await
    }
    async fn save_content(&self, document: &Document) -> Result<()> {
        self.0.save_content(document).await
    }
    async fn content(&self, id: &DocumentId) -> Result<Option<Document>> {
        self.0.content(id).await
    }
    async fn save_chat(&self, chat: &Chat) -> Result<()> {
        self.0.save_chat(chat).await
    }
    async fn chat(&self, id: &ChatId) -> Result<Option<Chat>> {
        self.0.chat(id).await
    }
    async fn chats(&self) -> Result<Vec<ChatSummary>> {
        self.0.chats().await
    }
    async fn delete_chat(&self, id: &ChatId) -> Result<()> {
        self.0.delete_chat(id).await
    }
    async fn settings(&self) -> Result<Settings> {
        self.0.settings().await
    }
    async fn save_settings(&self, settings: &Settings) -> Result<()> {
        self.0.save_settings(settings).await
    }
    async fn update_settings(&self, update: SettingsUpdate) -> Result<Settings> {
        self.0.update_settings(update).await
    }
}

#[test]
fn two_overlapping_runs_give_a_new_file_exactly_one_record() {
    block_on(async {
        let folder = PathBuf::from(MockFiles::FIXTURE_FOLDER);
        let file = folder.join("notizen.md");
        let files = Arc::new(MockFiles::new());
        files.add_file(&file).unwrap();
        let store = Arc::new(YieldingStore(MockStore::new()));
        store
            .save_settings(&Settings {
                watched_folders: vec![folder],
                ..Settings::default()
            })
            .await
            .unwrap();
        let embedder: Arc<dyn Embedder> = Arc::new(MockEmbedder::default());
        let tokens: Arc<dyn TokenCounter> = Arc::new(MockTokenCounter);
        let ingestor = MockIngestor::new(
            Arc::new(MockParser::new()),
            Arc::new(MockChunker::new(tokens, 400)),
            Arc::clone(&embedder),
            Arc::new(MockIndex::new(embedder.space(), 0.5)),
            Arc::clone(&store) as Arc<dyn Store>,
            files,
        );
        let run = || {
            ingestor.sync(
                Arc::new(|_| {}),
                Arc::new(|_| {}),
                SyncMode::Normal,
                CancellationToken::new(),
            )
        };
        // A watcher and a command, at the same time.
        let (first, second) = futures::join!(run(), run());
        let first = first.unwrap();
        let second = second.unwrap();
        let records = store.documents().await.unwrap();
        assert_eq!(records.len(), 1, "{records:?}");
        assert_eq!(records[0].path, file);
        assert!(matches!(records[0].status, DocumentStatus::Indexed { .. }));
        // One run indexed the file, the other found it unchanged.
        let statuses: Vec<&FileStatus> = first.iter().chain(&second).map(|r| &r.status).collect();
        assert_eq!(statuses.len(), 2, "{statuses:?}");
        assert!(
            statuses
                .iter()
                .any(|s| matches!(s, FileStatus::Indexed { .. })),
            "{statuses:?}"
        );
        assert!(
            statuses
                .iter()
                .all(|s| !matches!(s, FileStatus::Failed { .. })),
            "{statuses:?}"
        );
    });
}
