//! The component contracts.
//!
//! Conventions that apply to every trait:
//!
//! - All traits are object safe and require `Send + Sync`, so they are held as
//!   `Arc<dyn Trait>` and real and mock implementations can be mixed freely.
//! - Asynchronous methods use [`async_trait`]; their futures are `Send`. CPU-heavy
//!   implementations must not block the async executor (move the work to a blocking
//!   thread pool or a worker process).
//! - Long-running methods take a [`CancellationToken`]. When it is cancelled, the method
//!   stops as soon as possible, leaves no partial result behind and returns
//!   [`Error::Cancelled`](crate::Error::Cancelled). Streams end with that error,
//!   except the [`AnswerStream`], which ends with a `Done` that records the cancel.
//! - Methods never panic on bad input; they return [`Error::InvalidInput`](crate::Error::InvalidInput).
//!   Invalid everywhere: a query or question text that is empty or only whitespace,
//!   and a [`SearchRequest`] with `limit` 0. Further cases are named at the method. A
//!   vector of the wrong dimension is `EmbeddingMismatch`, not `InvalidInput`.
//! - Paths passed in may be in any absolute form; paths handed out are in
//!   [`canonical_path`](crate::canonical_path) form.

use std::path::{Path, PathBuf};

use async_trait::async_trait;
use futures::stream::BoxStream;
use tokio_util::sync::CancellationToken;

use crate::answer::{
    AnswerEvent, Chat, ChatId, ChatSummary, ChatTurn, GenerationEvent, GenerationRequest,
};
use crate::chunk::{Chunk, Embedding, EmbeddingSpace, IndexEntry, SearchHit, SearchRequest};
use crate::config::{AnswerLevel, HardwareTier, Settings, UiLanguage};
use crate::document::{Document, DocumentId, SourceFile};
use crate::error::Result;
use crate::hub::{
    HardwareInfo, MissingModelFile, ModelProgressCallback, ModelStorage, ResolvedModel,
};
use crate::ingest::{DocumentRecord, FileResult, ProgressCallback, ScopeCallback, SyncMode};

/// A change to the settings, applied atomically by [`Store::update_settings`]. On
/// `Err` nothing is saved.
pub type SettingsUpdate = Box<dyn FnOnce(&mut Settings) -> Result<()> + Send>;

/// Stream of phases and text pieces from [`Generator::generate`].
pub type GenerationStream = BoxStream<'static, Result<GenerationEvent>>;

/// Stream of answer events from [`Answerer::ask`].
pub type AnswerStream = BoxStream<'static, Result<AnswerEvent>>;

/// Counts tokens as the embedding model sees them (`kb-models`), for the [`Chunker`]
/// and [`PipelineConfig::chunk_max_tokens`](crate::PipelineConfig).
///
/// Passed in when the chunker is built, so no domain crate depends on another. The
/// answer model counts with [`Generator::count_tokens`], because its count depends on
/// the level.
#[async_trait]
pub trait TokenCounter: Send + Sync {
    /// Number of tokens of `text` without special tokens or task prefixes.
    /// Deterministic for equal input; the empty text has 0 tokens.
    async fn count_tokens(&self, text: &str) -> Result<usize>;
}

/// Turns a source file into a structured [`Document`] (package A, `kb-parse`).
#[async_trait]
pub trait Parser: Send + Sync {
    /// Parses the file at `source.path` and returns its elements in reading order.
    ///
    /// The returned document has `source == *source` and the format given by
    /// [`DocumentFormat::from_path`](crate::DocumentFormat::from_path). Every element
    /// has at least one anchor of the variant that belongs to the format (ADR 0010).
    /// Boxes are in the frame of the displayed page (see [`Page`](crate::Page)); a
    /// multi-page TIFF yields one page per frame, each with its own page number
    /// (ADR 0097; the fork of docling.rs does so from the patch of ADR 0097 on, the
    /// revision pinned in `Cargo.toml` does not contain it yet). Text that continues in another place
    /// carries one anchor per place, in reading order: a paragraph continued on the next
    /// page one per page, a paragraph continued in the next column of the same page one
    /// per column. Each page says whether its text comes from a text recognition
    /// ([`Page::from_ocr`](crate::Page)). A text layer that a PDF already has is taken
    /// over; a damaged one is replaced by recognising the page again (ADR 0091). XLSX
    /// cells carry their displayed value in US notation
    /// ([`TableCell::text`](crate::TableCell), ADR 0092; for all number formats only
    /// from the patch of ADR 0093 on, which the pinned revision does not contain yet).
    ///
    /// What is read (ADR 0095, ADR 0096): the content the author wrote, including
    /// footnotes and the notes of a slide (with the anchor of their slide). Hidden
    /// sheets and slides and comments are not read. For HTML the main content of the
    /// page is read, including the text before the first heading, without menus,
    /// sidebars and footers.
    ///
    /// Errors: [`Error::UnsupportedFormat`](crate::Error::UnsupportedFormat) for
    /// unknown extensions, [`Error::Parse`](crate::Error::Parse) for corrupt or
    /// encrypted files and crashed worker processes, `Cancelled`.
    async fn parse(&self, source: &SourceFile, cancel: CancellationToken) -> Result<Document>;
}

/// Splits a [`Document`] into [`Chunk`]s (package B, `kb-chunk`).
///
/// Built with the [`TokenCounter`] of the embedding model and the token limit
/// `chunk_max_tokens`.
#[async_trait]
pub trait Chunker: Send + Sync {
    /// Returns the chunks of `document` in reading order.
    ///
    /// General guarantees: ids are `ChunkId::new(&document.source.document_id, 0..)`;
    /// every chunk has non-empty text, `provenance.source == document.source`, the
    /// complete heading path and at least one anchor; `language` is the document's
    /// unless the chunker detects another one. Furniture and pictures without text are
    /// left out. Consecutive elements under the same heading path may be merged into
    /// one chunk. A document without text yields an empty list.
    ///
    /// Token limit (ADR 0014): no chunk text has more tokens than the limit, without
    /// exception.
    ///
    /// - Running text longer than the limit is split at sentence boundaries, failing
    ///   that at word boundaries, failing that at token boundaries.
    /// - A table whose text fits stays in one chunk.
    /// - A larger table is split at row boundaries into consecutive parts. Each part is
    ///   a Markdown table that repeats the header rows (see
    ///   [`TableCell`](crate::TableCell)) followed by as many further rows as fit. Its
    ///   anchors are those of its own rows (the header rows count as rows of the first
    ///   part unless they form a part of their own, see below): per page the smallest
    ///   box around their cells ([`Anchor::Page`](crate::Anchor)), or the smallest
    ///   range around their cells ([`Anchor::Sheet`](crate::Anchor)); without cell
    ///   anchors the table's anchors.
    /// - A split never cuts through a cell with `row_span > 1`: rows joined by such
    ///   cells form one row group, and "row" in these rules means row group.
    /// - If header rows plus one row exceed the limit, that row is emitted without
    ///   header. If the row alone exceeds the limit, its cell texts are joined with
    ///   `" | "` in row and column order and split like running text; every piece keeps
    ///   the row's anchors.
    /// - If no row fits together with the header rows (ADR 0136), the header rows are
    ///   emitted once as a part of their own before the parts of the rows: the Markdown
    ///   table of only the header rows and the separator line. If that text exceeds the
    ///   limit, the cell texts of the header rows are joined with `" | "` in row and
    ///   column order and split like running text, as for a row over the limit. The
    ///   header part and each of its pieces carry the anchors of the header rows, which
    ///   then do not count as rows of the first part of the rows. If at least one row
    ///   fits together with the header rows, there is no header part of its own. No
    ///   text of a table is lost.
    async fn chunk(&self, document: &Document) -> Result<Vec<Chunk>>;
}

/// Computes embeddings (package C, `kb-models`).
///
/// The embedder owns the model-specific input format: it adds the task prefixes for
/// queries and documents that the model requires and truncates to the dimension of its
/// [`EmbeddingSpace`] (Matryoshka) before normalising. Output is always `f32`, never
/// computed in float16.
#[async_trait]
pub trait Embedder: Send + Sync {
    /// Model and dimension of every returned vector (from the model's entry in
    /// `models.toml`).
    fn space(&self) -> EmbeddingSpace;

    /// Embeds a search query with the model's query prefix. An empty or whitespace-only
    /// query is `InvalidInput`.
    async fn embed_query(&self, query: &str) -> Result<Embedding>;

    /// Embeds chunks for the index with the model's document prefix. The heading path
    /// is used as the document title where the model supports one. Returns one vector
    /// per chunk in the same order. Batching is up to the implementation, but it keeps
    /// batches small: memory grows with the batch (about 0.3 GB per 400-token chunk,
    /// `docs/analyse/embedding-test.md`). Checks `cancel` at least between batches.
    async fn embed_documents(
        &self,
        chunks: &[Chunk],
        cancel: CancellationToken,
    ) -> Result<Vec<Embedding>>;
}

/// Scores how well chunks answer a query (package C, `kb-models`).
///
/// Optional: a tier without a reranker entry in `models.toml` runs without one, see
/// [`Answerer`].
#[async_trait]
pub trait Reranker: Send + Sync {
    /// Returns one relevance score per chunk, same order, each in `[0, 1]`
    /// (higher is more relevant). Scores are comparable across calls, so a fixed
    /// threshold ([`PipelineConfig::min_relevance`](crate::PipelineConfig)) works.
    /// Checks `cancel` at least between batches. An empty or whitespace-only query is
    /// `InvalidInput`.
    ///
    /// - The heading of a chunk (its heading path from
    ///   [`Provenance`](crate::Provenance)) goes into the score before its text.
    /// - The score of a chunk does not depend on which other chunks are in the same
    ///   call. An implementation whose model file makes it depend on the batch (dynamic
    ///   quantisation) scores the chunks one by one.
    /// - The reranker has its own tokenizer and token limit, unlike
    ///   [`PipelineConfig::chunk_max_tokens`](crate::PipelineConfig), which counts in
    ///   tokens of the embedding model. A chunk that does not fit with query and
    ///   heading is cut at its end.
    async fn rerank(
        &self,
        query: &str,
        chunks: &[Chunk],
        cancel: CancellationToken,
    ) -> Result<Vec<f32>>;
}

/// Stores chunks with embeddings and searches them (package E, `kb-index`).
///
/// An index is bound to one [`EmbeddingSpace`], given when it is built and persisted
/// with it. Opening an index that was built for another space fails with
/// [`Error::EmbeddingMismatch`](crate::Error::EmbeddingMismatch); the app then rebuilds
/// it by ingesting all documents again.
#[async_trait]
pub trait Index: Send + Sync {
    /// The embedding space the index is bound to.
    fn space(&self) -> EmbeddingSpace;

    /// Adds entries. An entry whose chunk id already exists replaces the old one.
    /// An embedding whose length differs from the space's dimension is
    /// `EmbeddingMismatch`, and nothing is added.
    async fn add(&self, entries: &[IndexEntry]) -> Result<()>;

    /// Removes all chunks of a document. Succeeds if there are none.
    async fn delete_document(&self, id: &DocumentId) -> Result<()>;

    /// Searches according to `request` and returns at most `request.limit` hits, best
    /// first by `retrieval_score`, with `rerank_score: None`.
    ///
    /// `embedding` is the query embedding from [`Embedder::embed_query`] (wrong length:
    /// `EmbeddingMismatch`); it is ignored in [`SearchMode::FullText`](crate::SearchMode).
    /// Vector mode returns the nearest chunks whatever their similarity. Full-text mode
    /// returns only chunks that contain at least one query term and must find German
    /// inflected forms and compounds. Hybrid mode fuses both lists, so it also returns
    /// chunks with a low score; judging relevance is the [`Answerer`]'s job. No hits
    /// come back only if no chunk passes the document filter or, in full-text mode,
    /// none matches.
    ///
    /// The request carries no language: how the query text is analysed (for example
    /// with the stemmers of all languages present in the index) is up to the
    /// implementation. Empty query text and `limit` 0 are `InvalidInput`.
    async fn search(&self, request: &SearchRequest, embedding: &[f32]) -> Result<Vec<SearchHit>>;
}

/// Generates text with the local answer models (package F, `kb-llm`).
///
/// Text only, no tool calling (ADR 0004); no external endpoint (ADR 0015). Built with
/// the [`ModelManifest`](crate::ModelManifest), the model directory, the
/// [`HardwareTier`] and [`PipelineConfig::context_tokens`](crate::PipelineConfig). Each
/// [`AnswerLevel`] the tier offers has its model
/// ([`ModelManifest::answer_model`](crate::ModelManifest::answer_model) for the
/// hardware tier), started with that context and, if that entry names a drafter, with
/// multi-token prediction. Whether it does follows the manifest per hardware tier
/// (`mtp_tiers`): below the minimum the fast model starts without it (ADR 0084).
///
/// One answer model is loaded at a time (ADR 0055). A call for another level than the
/// loaded one stops that model and loads the model of the level; in `generate` this is
/// reported as `StartingModel`, in `count_tokens` it is silent (the [`Answerer`] has
/// announced it).
#[async_trait]
pub trait Generator: Send + Sync {
    /// Whether `level` can be used: `Ok` if its model is provided,
    /// [`Error::ModelMissing`](crate::Error::ModelMissing) if the tier offers the level
    /// but its model files are not in the model directory (ADR 0058), `InvalidInput` if
    /// the tier does not offer it.
    ///
    /// Synchronous on purpose: it looks only at the presence of the model files in the
    /// model directory (file metadata, no hashing, no network, no backend), which is
    /// quick enough to call before an answer starts, like [`ModelHub::storage`]. The
    /// checksums were verified by [`ModelHub::provide`]; the asynchronous
    /// [`ModelHub::missing`] is the full check, which may hash files. A file under its
    /// target name is complete, because partial downloads never lie there (see
    /// [`ModelHub::provide`]); a file damaged later passes here and fails in the stream.
    fn check_level(&self, level: AnswerLevel) -> Result<()>;

    /// Number of tokens of `text` for the model of `level`, without special tokens or
    /// chat template. Deterministic for equal input; the empty text has 0 tokens.
    ///
    /// Served by the backend of the answer model: the call starts it and loads the model
    /// of `level` if needed and waits for that; the sequence at [`AnswerEvent`] relies
    /// on this. There is no token: the caller cancels by dropping the future, so the
    /// [`Answerer`] races every call against its cancel token. Fails like
    /// [`Generator::check_level`] for a level that cannot be used, and with
    /// `Error::Generator` if the backend cannot be started.
    async fn count_tokens(&self, level: AnswerLevel, text: &str) -> Result<usize>;

    /// Starts generation with the model of `request.level` and returns its phases and
    /// text as a stream (order at [`GenerationEvent`]).
    ///
    /// Returns the stream without waiting for the model: starting the backend if it is
    /// not running and loading the model happen inside the stream and are reported as
    /// phases (`StartingModel`), as is reading the prompt (`ReadingSources`, at least
    /// once, with progress if the backend reports it, without otherwise). If the backend
    /// cannot be started, the stream yields `Err(Generator)`. The stream ends when
    /// generation is complete. Cancelling `cancel` ends the stream at once in every
    /// phase (starting, loading, reading the prompt, writing) with `Err(Cancelled)` as
    /// its last item, without waiting for output of the backend, and aborts the request
    /// to the backend; dropping the stream aborts it too. If the backend crashes during
    /// streaming, the stream yields `Err(Generator)` as its last item, and the next call
    /// starts the backend again. Invalid requests and levels that cannot be used (see
    /// [`Generator::check_level`]) are errors before streaming.
    ///
    /// The backend runs with its default batch size (ADR 0101,
    /// `docs/analyse/spike-llama-server-fortschritt.md`): the progress of
    /// `ReadingSources` may be coarse, a few reports per prompt, one per batch of the
    /// server. After a cancel the stream ends at once as above, but the model may stay
    /// busy until the batch it is reading ends; the next call to the backend waits that
    /// long.
    async fn generate(
        &self,
        request: GenerationRequest,
        cancel: CancellationToken,
    ) -> Result<GenerationStream>;
}

/// Persists documents, chats and settings (package G, `kb-store`).
///
/// Delete methods succeed if the object does not exist.
#[async_trait]
pub trait Store: Send + Sync {
    /// Inserts or replaces the record with `record.id`.
    async fn upsert_document(&self, record: &DocumentRecord) -> Result<()>;

    /// Returns the record with `id`, if any.
    async fn document(&self, id: &DocumentId) -> Result<Option<DocumentRecord>>;

    /// Returns the record whose path has the same [`path_key`](crate::path_key) as
    /// `path`, if any.
    async fn document_by_path(&self, path: &Path) -> Result<Option<DocumentRecord>>;

    /// Returns all records, sorted by [`path_key`](crate::path_key).
    async fn documents(&self) -> Result<Vec<DocumentRecord>>;

    /// Deletes the record and the stored content of a document.
    async fn delete_document(&self, id: &DocumentId) -> Result<()>;

    /// Stores the parsed document (for the text view and `get_document`), replacing
    /// earlier content of the same document.
    async fn save_content(&self, document: &Document) -> Result<()>;

    /// Returns the parsed document with `id`, if stored.
    async fn content(&self, id: &DocumentId) -> Result<Option<Document>>;

    /// Inserts or replaces a chat, with its timestamps as given (the
    /// [`KnowledgeBase`](crate::KnowledgeBase) sets them).
    async fn save_chat(&self, chat: &Chat) -> Result<()>;

    /// Returns the chat with `id`, if any.
    async fn chat(&self, id: &ChatId) -> Result<Option<Chat>>;

    /// Returns all chats ordered by `updated_at`, newest first; chats with equal
    /// `updated_at` by id ascending.
    async fn chats(&self) -> Result<Vec<ChatSummary>>;

    /// Deletes a chat.
    async fn delete_chat(&self, id: &ChatId) -> Result<()>;

    /// Returns the settings, or `Settings::default()` if none were saved.
    async fn settings(&self) -> Result<Settings>;

    /// Replaces the settings. To change one field, prefer [`Store::update_settings`].
    /// The user changes the library scope (`watched_folders`, `excluded`) through the
    /// [`KnowledgeBase`](crate::KnowledgeBase) methods, the [`Ingestor`] drops stale
    /// exclusions; a caller of `save_settings` keeps both lists as read.
    async fn save_settings(&self, settings: &Settings) -> Result<()>;

    /// Reads the settings, applies `update` and saves the result as one atomic step
    /// (no other settings write in between), then returns the saved settings. If
    /// `update` fails, nothing is saved and its error is returned. Used for the library
    /// scope, which both the user and the [`Ingestor`] change.
    async fn update_settings(&self, update: SettingsUpdate) -> Result<Settings>;
}

/// Keeps the library in step with its scope: hash, parse, chunk, embed, index, record
/// (package H, `kb-ingest`, ADR 0025).
///
/// Built from a [`Parser`], [`Chunker`], [`Embedder`], [`Index`] and [`Store`]. The
/// scope is read from [`Store::settings`] when a run starts: the watched folders minus
/// the exclusions ([`Settings::in_scope`]). A running `watch` keeps the scope it started
/// with; after the user changes folders or exclusions, the caller restarts it.
///
/// A synchronisation run does, in this order:
///
/// 1. Checks every watched folder (duplicates by [`path_key`](crate::path_key) once).
///    A watched folder must be a directory that can be listed. One that is missing,
///    is a file or cannot be read is reported as
///    [`ScopeEvent::FolderUnavailable`](crate::ScopeEvent), nothing in it is ingested,
///    and its documents are kept unchanged; a listable one is reported as
///    `FolderAvailable`. The library is never emptied because a folder (for example a
///    network drive) is temporarily unreachable.
/// 2. Drops exclusions from the settings (with [`Store::update_settings`], reported as
///    `ExclusionRemoved`) that lie in no watched folder or whose file or subfolder no
///    longer exists. Absence counts only if it was seen: a watched folder holding the
///    path was listed and the directory that should hold it could be read.
/// 3. Removes every document (chunks, stored content and record, whatever its status,
///    including `Failed`) whose path is out of scope (folder deregistered or path
///    excluded) or whose file is gone. Gone means: a watched folder holding the file
///    was listed, the directory that should hold the file could be read, and the file
///    is not there. Documents below a directory that could not be read stay, unless
///    they are out of scope. Each removal is a file event with status `Removed`.
/// 4. Processes every supported file in scope, each once even if it lies in nested
///    watched folders. Files with unsupported extensions are skipped without an event.
///    A file is `Unchanged` only if its record is `Indexed` and its content is the
///    indexed one; otherwise it is processed again. Shortcut: if size and modification
///    time of the file equal `size_bytes` and `modified_at` of the `Indexed` record,
///    the file counts as unchanged without hashing. If either differs (or is unknown),
///    the content hash decides; if the hash matches, the file is `Unchanged` and the
///    record takes the new size and modification time. Residual risk: a file whose
///    content changed while its size stayed the same and its modification time was
///    set back (copy tools that keep timestamps, clock changes) is not re-indexed. A
///    complete comparison by content is a full run ([`SyncMode::Full`]). A
///    changed file replaces its old chunks. Records left in `Queued` by an earlier run
///    (crash, restart) are processed as part of this run.
/// 5. A failing file is `Failed` (with [`ErrorKind`](crate::ErrorKind)), and the run
///    continues. Its record keeps size and modification time of the failed attempt but
///    no content hash, and its chunks and stored content are removed, so a citation of
///    an earlier version counts as outdated. A `Failed` file is tried again only if its
///    size or modification time changed, if the failure was an I/O error (typically
///    the file was open in another program), or in a run with
///    [`SyncMode::RetryFailed`] or [`SyncMode::Full`]; otherwise it takes no part
///    in the run (no event, no result).
///
/// Queue: all runs share one queue. A file is processed once at a time; a file that
/// two runs need is processed once and reported to both. Counters and results of a
/// call cover only its own files.
///
/// State after a cancel: files finished before stay as they are. The file that was in
/// progress has its record in `Queued` without content hash, no chunks of its document
/// in the index and no stored content (its earlier version, if any, is no longer
/// searchable), so the next run picks it up again. A run cancelled before it starts
/// changes nothing (no settings, no removals).
///
/// Errors of single files and folders are events, not errors of the call. A call
/// fails only on errors that concern no single file or folder (store or index
/// failing), or with `Cancelled`.
#[async_trait]
pub trait Ingestor: Send + Sync {
    /// Runs one synchronisation and returns one result per file it touched (processed
    /// or removed) once all are done. Progress of files goes to `progress`, folder and
    /// exclusion events go to `scope`. On cancel the method returns `Err(Cancelled)` and
    /// leaves the state described at the trait.
    ///
    /// `mode` decides which files are processed again (see [`SyncMode`]). In a full run
    /// every file is compared by its content hash (ADR 0053); only files whose
    /// content changed are processed again, the others stay `Unchanged`.
    async fn sync(
        &self,
        progress: ProgressCallback,
        scope: ScopeCallback,
        mode: SyncMode,
        cancel: CancellationToken,
    ) -> Result<Vec<FileResult>>;

    /// Runs one synchronisation and then keeps the library in step: new, changed,
    /// deleted, excluded and re-included files are handled as they appear. Folders that
    /// were unavailable are checked again at least once a minute and synchronised when
    /// readable. Runs until `cancel` is cancelled and then returns `Ok(())`.
    async fn watch(
        &self,
        progress: ProgressCallback,
        scope: ScopeCallback,
        cancel: CancellationToken,
    ) -> Result<()>;
}

/// Retrieval and answering with citations (package I, `kb-rag`).
///
/// Built from an [`Embedder`], [`Index`], optionally a [`Reranker`], and a
/// [`Generator`]. Never stores chats; that is done by
/// [`KnowledgeBase::ask`](crate::KnowledgeBase::ask).
///
/// Ranking and relevance: with a reranker, hits are ordered by `rerank_score` and a hit
/// is relevant if `rerank_score >= min_relevance`. Without a reranker, hits keep the
/// index order, `rerank_score` stays `None`, and a hit is relevant if
/// `vector_score >= min_similarity` (see [`PipelineConfig`](crate::PipelineConfig)).
#[async_trait]
pub trait Answerer: Send + Sync {
    /// Searches and ranks: with a reranker, fetches `max(request.limit,
    /// rerank_candidates)` hits from the index, reranks them and returns the best
    /// `request.limit`; without one, returns the index hits. Empty query text and
    /// `limit` 0 are `InvalidInput`. Cancelling `cancel` ends the search at once with
    /// `Cancelled`; embedding and index search are raced against the token.
    async fn search(
        &self,
        request: &SearchRequest,
        cancel: CancellationToken,
    ) -> Result<Vec<SearchHit>>;

    /// Answers `question` with the answer model of `level`, using `history` (earlier
    /// complete turns of the same chat) to rewrite a follow-up question into a
    /// standalone one. The events follow case A (empty history) or case B (non-empty
    /// history) at [`AnswerEvent`]. Counting, rewriting and generating all use `level`,
    /// and the answer records it.
    ///
    /// Before returning the stream, `ask` checks the level with
    /// [`Generator::check_level`]: a level whose model is missing is
    /// [`Error::ModelMissing`](crate::Error::ModelMissing), one the tier does not offer
    /// `InvalidInput`, and the question is not asked.
    ///
    /// The history is used only to rewrite a follow-up question; it is not part of the
    /// answer prompt, whose `prompt_token_budget` counts instruction, sources and
    /// question (ADR 0100).
    ///
    /// History for rewriting: the newest turns first, whole turns only (question and
    /// answer text), as long as their tokens, counted with
    /// [`Generator::count_tokens`], stay within [`PipelineConfig::rewrite_history_tokens`](crate::PipelineConfig);
    /// the first turn that does not fit and all older ones are left out. If not even the
    /// newest turn fits, the question is not rewritten. The rewriting call's phases are
    /// not passed on.
    ///
    /// Returns the stream without waiting for retrieval or the model; everything after
    /// the validation of the question happens inside it, and every failure there ends
    /// in a `Done` with [`AnswerOutcome::Failed`](crate::AnswerOutcome) instead of an
    /// error item, keeping the text written so far. Cancelling `cancel` ends in a `Done`
    /// with `Cancelled` at once, in every phase: the wait for the turn, the search
    /// (embedder and index take no token), counting, rewriting and the answer call are
    /// raced against the token, so a cancel does not wait for a component, also not
    /// for a generator that is slow to end its stream. Dropping the stream discards the
    /// answer and frees the turn.
    ///
    /// Answers run one after another, first in, first out, a waiting one in phase
    /// `Waiting` (see [`AnswerEvent`]). The question joins the queue when the future of
    /// this method runs (after the checks above), not when the future is created and not
    /// when the stream is first polled; two futures awaited in reverse order join in
    /// the order they are awaited. So the order follows the calls. The queue covers
    /// every answer of this answerer; the app and the MCP server must share one
    /// (ADR 0062).
    ///
    /// Only relevant hits become sources, at most `max_sources`, and the prompt stays
    /// within `prompt_token_budget`. Without a relevant source the answer model is not
    /// called and the answer has `no_sources: true` and a fixed notice in `language`.
    /// Every generation request sets `max_tokens` to
    /// [`PipelineConfig::max_answer_tokens`](crate::PipelineConfig). The answer call's
    /// phases `ReadingSources` (with its progress unchanged) and `Writing` are passed on
    /// as `Status` events. Errors of this method: an empty question (`InvalidInput`) and
    /// the level check above. Citation rules, including the rewriting of `[1, 2]` and `[1-3]`,
    /// are described at [`Answer`](crate::Answer).
    async fn ask(
        &self,
        question: &str,
        history: &[ChatTurn],
        level: AnswerLevel,
        language: UiLanguage,
        cancel: CancellationToken,
    ) -> Result<AnswerStream>;
}

/// Model management: hardware detection and providing the models of a tier and level
/// (package D, `kb-hub`).
///
/// Built with the [`ModelManifest`](crate::ModelManifest) and the model directory. The
/// tier is [`HardwareTier::from_total_ram`] of [`ModelHub::hardware`]; below the
/// minimum it is `BelowMinimum`, which the setup shows as a warning, not a lock
/// (ADR 0050, ADR 0056); no setting raises it (ADR 0080). `provide` and `missing` work
/// for both tiers whatever the
/// hardware.
///
/// Models are provided per [`AnswerLevel`] (ADR 0058): the setup provides the fast
/// level, which is the base equipment (parser models, embedding, reranker if the tier
/// has one, answer model of the fast level); a higher level adds its answer model when
/// the user chooses it. Which models that are is
/// [`ModelManifest::models_for`](crate::ModelManifest::models_for). An offline package
/// holds the models of all levels.
///
/// Failures carry a category the user interface can tell apart:
/// [`Error::Network`](crate::Error::Network), [`Error::StorageFull`](crate::Error::StorageFull),
/// [`Error::ChecksumMismatch`](crate::Error::ChecksumMismatch); other problems are
/// `Model` or `Io`. A level the tier does not offer is `InvalidInput`.
#[async_trait]
pub trait ModelHub: Send + Sync {
    /// Detects the hardware.
    fn hardware(&self) -> Result<HardwareInfo>;

    /// The model directory and the free space on its volume.
    fn storage(&self) -> Result<ModelStorage>;

    /// Makes the models of `level` on `tier` available
    /// ([`ModelManifest::models_for`](crate::ModelManifest::models_for)) and returns
    /// them. They are made ready one after the other in that order (roles in
    /// [`ModelRole::PROVIDE_ORDER`](crate::ModelRole::PROVIDE_ORDER), the fast answer
    /// model before that of `level`); every [`ModelEvent`](crate::ModelEvent) names its
    /// model and role, so the user interface sees when ingestion can start (all roles
    /// with [`ModelRole::is_needed_for_ingestion`](crate::ModelRole::is_needed_for_ingestion)
    /// `Ready`) and when asking in `level` can (all `Ready`); after a restart `missing`
    /// tells the same.
    ///
    /// Files are expected at `<model directory>/<entry id>/<file path>`. A file found
    /// there with a matching checksum is used as is; this is how the pre-filled folder
    /// of the offline installer works (ADR 0003). Otherwise, if `source` is given (an
    /// offline package chosen by the user, ADR 0049, same layout
    /// `<source>/<entry id>/<file path>`), a file found there is verified and copied
    /// into the model directory (`Copying`); the source is never changed. What is still
    /// missing is downloaded from [`ModelFile::url`](crate::ModelFile::url), its only
    /// source (a Hugging Face repository or a GitHub release, ADR 0077), and verified. A
    /// file without `sha256`, or missing without a complete source, is `Error::Model`.
    /// On cancel the method returns `Err(Cancelled)`.
    ///
    /// What stays after a cancel or failure: an incomplete copy or download stays as a
    /// partial file after a cancel and after `Network`, `StorageFull` and I/O errors. A
    /// later call continues it, whether that call copies or downloads, and its first
    /// `Copying` or `Downloading` event for the file names the part already there as
    /// `done_bytes`. After `ChecksumMismatch` the file is deleted and starts at zero.
    ///
    /// Partial downloads and copies never lie under the target name
    /// `<model directory>/<entry id>/<file path>`: they are written under another name
    /// and moved there only after the checksum matches. So a file under its target name
    /// is always complete, which [`Generator::check_level`] relies on; a question in a
    /// level whose model is still loading ends with `ModelMissing` before the stream.
    ///
    /// Concurrent calls (the setup provides the fast level while the user chooses a
    /// higher one; both need the base equipment) share their files like the queue of the
    /// [`Ingestor`]: a file is fetched once, and every call that needs it waits for that
    /// fetch and reports its progress. Cancelling one call ends only that call; the fetch
    /// goes on as long as another call needs the file.
    ///
    /// The result of a checksum check may be cached; the cached result is valid as long
    /// as size and modification time of the file are unchanged.
    async fn provide(
        &self,
        tier: HardwareTier,
        level: AnswerLevel,
        source: Option<PathBuf>,
        progress: ModelProgressCallback,
        cancel: CancellationToken,
    ) -> Result<Vec<ResolvedModel>>;

    /// Lists the files of `level` on `tier` (the same models as `provide`) that
    /// `provide` would still have to fetch or repair, without downloading anything:
    /// files that are absent, do not match their checksum, or have no checksum to verify
    /// against. Empty means asking in `level` is possible. Checksum results may be
    /// cached like in `provide`, so the app can call this at every start for the fast
    /// level (is the setup needed?) and when the user chooses a level (offer the
    /// download?).
    async fn missing(
        &self,
        tier: HardwareTier,
        level: AnswerLevel,
    ) -> Result<Vec<MissingModelFile>>;
}
