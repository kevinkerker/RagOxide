//! [`SqliteStore`], the implementation of [`Store`].

use std::path::{Path, PathBuf};

use kb_core::{
    Chat, ChatId, ChatSummary, Document, DocumentId, DocumentRecord, Result, Settings,
    SettingsUpdate, Store, async_trait,
};

use crate::ops;
use crate::schema::{self, Schema};
use crate::worker::Worker;

/// [`Store`] in one SQLite database file (package G).
///
/// Built with [`SqliteStore::open`]. Usable as `Arc<dyn Store>`. All methods behave like
/// `kb_core::mock::MockStore`, durably, with one addition: a record for a path that
/// another record already has (equal [`path_key`](kb_core::path_key)) is refused with
/// `InvalidInput`, because the contract allows exactly one record per file.
///
/// The database stays open, and locked against every other opening, until the store is
/// dropped. Dropping waits until the calls already sent have finished and the file is
/// closed.
pub struct SqliteStore {
    worker: Worker,
    path: PathBuf,
}

impl std::fmt::Debug for SqliteStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SqliteStore")
            .field("path", &self.path)
            .finish_non_exhaustive()
    }
}

impl SqliteStore {
    /// Opens the database file at `path`, an absolute path chosen by the caller (the app
    /// passes a file in its local data folder). Missing folders on the way are created.
    ///
    /// - No file yet (or an empty one): the database is created with the current schema
    ///   ([`SCHEMA_VERSION`](crate::SCHEMA_VERSION)); no backup is made.
    /// - Current schema: opened as it is. If a backup from a migration lies next to it,
    ///   every table is read and one write is made; if both succeed, the backup is
    ///   deleted (ADR 0128).
    /// - Older schema of this package: a backup `<file name>.v<old version>.bak` is
    ///   written next to the database, older backups are removed, then the database is
    ///   migrated in one transaction (ADR 0119).
    ///
    /// Errors, all leaving an existing file byte for byte unchanged:
    /// [`Error::InvalidInput`](kb_core::Error::InvalidInput) for a relative path, and
    /// [`Error::Store`](kb_core::Error::Store) if the folder cannot be created, the file
    /// is read-only (it is not opened for reading instead, ADR 0144), is not an SQLite
    /// database or belongs to another application, its schema is newer than this build
    /// supports, another process (or another store in this process) has it open, the
    /// backup fails, or the migration fails.
    ///
    /// After a write that was cut off, the free pages are overwritten before `open`
    /// returns (ADR 0142). If that fails (a full disk, for example), the database is
    /// opened all the same and every later opening tries again.
    pub async fn open(path: impl AsRef<Path>) -> Result<Self> {
        Self::open_with(path.as_ref(), &schema::CURRENT).await
    }

    /// Opens with a given schema (the tests migrate to schemas of their own).
    pub(crate) async fn open_with(path: &Path, schema: &'static Schema) -> Result<Self> {
        let path = path.to_path_buf();
        let worker = Worker::start(path.clone(), schema).await?;
        Ok(Self { worker, path })
    }

    /// The path of the database file, as passed to [`SqliteStore::open`].
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Runs `work` on the database thread (for the tests of this crate).
    #[cfg(test)]
    pub(crate) async fn with_connection<T, F>(&self, work: F) -> Result<T>
    where
        T: Send + 'static,
        F: FnOnce(&mut rusqlite::Connection) -> Result<T> + Send + 'static,
    {
        self.worker.call(work).await
    }
}

#[async_trait]
impl Store for SqliteStore {
    async fn upsert_document(&self, record: &DocumentRecord) -> Result<()> {
        let record = record.clone();
        self.worker
            .call(move |conn| ops::upsert_document(conn, &record))
            .await
    }

    async fn document(&self, id: &DocumentId) -> Result<Option<DocumentRecord>> {
        let id = id.clone();
        self.worker.call(move |conn| ops::document(conn, &id)).await
    }

    async fn document_by_path(&self, path: &Path) -> Result<Option<DocumentRecord>> {
        let path = path.to_path_buf();
        self.worker
            .call(move |conn| ops::document_by_path(conn, &path))
            .await
    }

    async fn documents(&self) -> Result<Vec<DocumentRecord>> {
        self.worker.call(|conn| ops::documents(conn)).await
    }

    async fn delete_document(&self, id: &DocumentId) -> Result<()> {
        let id = id.clone();
        self.worker
            .call(move |conn| ops::delete_document(conn, &id))
            .await
    }

    async fn save_content(&self, document: &Document) -> Result<()> {
        let document = document.clone();
        self.worker
            .call(move |conn| ops::save_content(conn, &document))
            .await
    }

    async fn content(&self, id: &DocumentId) -> Result<Option<Document>> {
        let id = id.clone();
        self.worker.call(move |conn| ops::content(conn, &id)).await
    }

    async fn save_chat(&self, chat: &Chat) -> Result<()> {
        let chat = chat.clone();
        self.worker
            .call(move |conn| ops::save_chat(conn, &chat))
            .await
    }

    async fn chat(&self, id: &ChatId) -> Result<Option<Chat>> {
        let id = id.clone();
        self.worker.call(move |conn| ops::chat(conn, &id)).await
    }

    async fn chats(&self) -> Result<Vec<ChatSummary>> {
        self.worker.call(|conn| ops::chats(conn)).await
    }

    async fn delete_chat(&self, id: &ChatId) -> Result<()> {
        let id = id.clone();
        self.worker
            .call(move |conn| ops::delete_chat(conn, &id))
            .await
    }

    async fn settings(&self) -> Result<Settings> {
        self.worker.call(|conn| ops::settings(conn)).await
    }

    async fn save_settings(&self, settings: &Settings) -> Result<()> {
        let settings = settings.clone();
        self.worker
            .call(move |conn| ops::save_settings(conn, &settings))
            .await
    }

    async fn update_settings(&self, update: SettingsUpdate) -> Result<Settings> {
        self.worker
            .call(move |conn| ops::update_settings(conn, update))
            .await
    }
}
