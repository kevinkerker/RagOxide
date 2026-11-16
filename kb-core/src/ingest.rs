//! Ingestion progress and the per-file document record.

use std::path::PathBuf;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::document::{DocumentFormat, DocumentId};
use crate::error::ErrorKind;
use crate::time::Timestamp;

/// Library entry for one source file, persisted by the [`crate::Store`].
///
/// There is exactly one record per supported file in the library scope (see
/// [`Settings::in_scope`](crate::Settings::in_scope); paths with equal
/// [`path_key`](crate::path_key) are the same). The record also serves as the
/// ingestion job: a record in [`DocumentStatus::Queued`] is work that is still to be
/// done, also after a restart, and a record in [`DocumentStatus::Failed`] appears in the
/// error list. Files with an unsupported extension never get a record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct DocumentRecord {
    /// Document identifier.
    pub id: DocumentId,
    /// Absolute path of the source file in [`canonical_path`](crate::canonical_path) form.
    pub path: PathBuf,
    /// Format derived from the path with [`DocumentFormat::from_path`].
    pub format: DocumentFormat,
    /// Title from the parsed document, if any.
    pub title: Option<String>,
    /// SHA-256 of the indexed content (lowercase hex, [`content_hash`](crate::content_hash)).
    /// Set exactly when the status is `Indexed`; `None` while queued and after a
    /// failure, so a citation of an earlier version is recognised as outdated.
    pub content_hash: Option<String>,
    /// Size of the file in bytes at the last attempt (indexed or failed), `None`
    /// before the first one. With `modified_at` the shortcut of the synchronisation and
    /// the rule for retrying failed files (see [`crate::Ingestor`]).
    pub size_bytes: Option<u64>,
    /// Modification time of the file at the last attempt, as reported by the file
    /// system, `None` before the first one.
    pub modified_at: Option<Timestamp>,
    /// Ingestion state.
    pub status: DocumentStatus,
}

/// Ingestion state of a [`DocumentRecord`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DocumentStatus {
    /// Waiting for (re-)ingestion.
    Queued,
    /// Parsed, chunked and searchable.
    Indexed {
        /// Number of chunks in the index.
        chunks: u32,
    },
    /// The last ingestion attempt failed. Chunks and stored content are removed.
    Failed {
        /// Category for a translated text in the error list.
        kind: ErrorKind,
        /// Technical error text (English), shown as details.
        message: String,
    },
}

/// State of one file during an ingestion run.
///
/// Stages appear in this order: `Queued`, `Parsing`, `Chunking`, `Embedding`,
/// `Indexing`, then exactly one terminal state (`Indexed`, `Unchanged`, `Removed` or
/// `Failed`). Stages may be skipped, terminal states end the file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FileStatus {
    /// Accepted, waiting for a worker.
    Queued,
    /// The parser is running.
    Parsing,
    /// The chunker is running.
    Chunking,
    /// Embeddings are being computed.
    Embedding,
    /// Chunks are being written to the index and the store.
    Indexing,
    /// Terminal: the file is searchable.
    Indexed {
        /// Document the file belongs to.
        document_id: DocumentId,
        /// Number of chunks written.
        chunks: u32,
    },
    /// Terminal: the content is the indexed one (by size and modification time, or by
    /// hash); nothing was re-indexed, at most the record took a new modification time.
    Unchanged {
        /// Document the file belongs to.
        document_id: DocumentId,
    },
    /// Terminal: the document was removed because its file is gone, its folder was
    /// deregistered or it was excluded.
    Removed {
        /// The removed document.
        document_id: DocumentId,
    },
    /// Terminal: this file failed; other files are not affected.
    Failed {
        /// Category for a translated text in the error list.
        kind: ErrorKind,
        /// Technical error text (English), shown as details.
        message: String,
    },
}

impl FileStatus {
    /// Returns `true` for `Indexed`, `Unchanged`, `Removed` and `Failed`.
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Indexed { .. }
                | Self::Unchanged { .. }
                | Self::Removed { .. }
                | Self::Failed { .. }
        )
    }
}

/// Progress event of a synchronisation run, sent through a [`ProgressCallback`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct IngestEvent {
    /// File the event is about, in canonical form.
    pub path: PathBuf,
    /// New state of that file.
    pub status: FileStatus,
    /// Files of this run that have reached a terminal state, including this one.
    pub completed: u32,
    /// Files in this run so far. Grows while a watched folder reports new files.
    pub total: u32,
}

/// Which files a synchronisation run processes again (rules 4 and 5 at
/// [`crate::Ingestor`], ADR 0053).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(rename_all = "snake_case")]
pub enum SyncMode {
    /// The shortcut over size and modification time applies, and a `Failed` file is
    /// tried again only if it changed or failed with an I/O error.
    #[default]
    Normal,
    /// Like `Normal`, but every `Failed` file in scope is tried again ("try again" in
    /// the list of failed files). No other file is compared by its content.
    RetryFailed,
    /// No shortcut: every file is compared by its content hash, and every `Failed`
    /// file is tried again. In the user interface "Integrität prüfen" (ADR 0072).
    Full,
}

/// Final state of one file of a synchronisation run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct FileResult {
    /// The file, in canonical form.
    pub path: PathBuf,
    /// Its terminal state.
    pub status: FileStatus,
}

/// Receives ingestion progress.
///
/// Called from worker tasks, possibly concurrently; must return quickly and must not
/// block (forward into a channel if the receiver is slow).
pub type ProgressCallback = Arc<dyn Fn(IngestEvent) + Send + Sync>;

/// Event about the library scope during synchronisation, sent through a
/// [`ScopeCallback`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ScopeEvent {
    /// A watched folder can be read (again).
    FolderAvailable {
        /// The watched folder, canonical.
        folder: PathBuf,
    },
    /// A watched folder cannot be listed. Its documents are kept unchanged until it can
    /// be listed again, and nothing in it is ingested. The kind tells why: `NotFound`
    /// if nothing exists at the path, `InvalidInput` if the path is a file (not a
    /// directory), `Io` if it exists but cannot be read (network drive offline, access
    /// denied).
    FolderUnavailable {
        /// The watched folder, canonical.
        folder: PathBuf,
        /// Category of the error.
        kind: ErrorKind,
        /// Technical error text (English).
        message: String,
    },
    /// An exclusion was dropped from the settings because its file or subfolder no
    /// longer exists or it lies in no watched folder any more.
    ExclusionRemoved {
        /// The dropped exclusion, canonical.
        path: PathBuf,
    },
}

/// Receives scope events. Same rules as [`ProgressCallback`].
pub type ScopeCallback = Arc<dyn Fn(ScopeEvent) + Send + Sync>;
