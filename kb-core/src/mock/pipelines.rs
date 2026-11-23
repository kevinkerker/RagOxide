//! Mocks of the pipeline traits, built on the component traits.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, PoisonError};

use async_trait::async_trait;
use futures::{StreamExt, future, stream};
use tokio_util::sync::CancellationToken;

use super::files::MockFiles;
use super::queue::{Ticket, TurnQueue};
use super::{check_request, or_cancel};
use crate::document::key_within;
use crate::{
    Answer, AnswerEvent, AnswerLevel, AnswerOutcome, AnswerPhase, AnswerStream, Answerer, ChatTurn,
    Chunker, Citation, DocumentFormat, DocumentId, DocumentRecord, DocumentStatus, Embedder, Error,
    ErrorKind, FileResult, FileStatus, GenerationEvent, GenerationRequest, Generator, Index,
    IndexEntry, IngestEvent, Ingestor, Message, Parser, PipelineConfig, ProgressCallback, Reranker,
    Result, Role, ScopeCallback, ScopeEvent, SearchHit, SearchRequest, SourceFile, Store, SyncMode,
    Timestamp, UiLanguage, canonical_path, path_key,
};

/// [`Ingestor`] over a [`MockFiles`] file system, processing files one after the other.
///
/// Follows the synchronisation rules of the trait. Simplifications: files are never
/// read (the [`Parser`] decides, the hash comes from [`MockFiles`]), runs that overlap
/// (a watcher and a command, for example) run one after another as a whole instead of
/// sharing one queue of files, and `watch` synchronises once and then only waits for
/// the cancel. A run waiting for its turn ends at once when it is cancelled. New document ids are `doc-<file stem>` (with a number appended if taken)
/// so that the fixture files keep the ids from `documents.json`; this scheme is for the
/// mock only.
pub struct MockIngestor {
    parser: Arc<dyn Parser>,
    chunker: Arc<dyn Chunker>,
    embedder: Arc<dyn Embedder>,
    index: Arc<dyn Index>,
    store: Arc<dyn Store>,
    files: Arc<MockFiles>,
    /// Held for the whole of a synchronisation, so that runs do not overlap.
    runs: futures::lock::Mutex<()>,
}

struct Run<'a> {
    progress: &'a ProgressCallback,
    completed: u32,
    total: u32,
}

impl Run<'_> {
    fn emit(&self, path: &Path, status: FileStatus) {
        (self.progress)(IngestEvent {
            path: path.to_path_buf(),
            status,
            completed: self.completed,
            total: self.total,
        });
    }
}

impl MockIngestor {
    /// Pipeline over the given components and file system.
    pub fn new(
        parser: Arc<dyn Parser>,
        chunker: Arc<dyn Chunker>,
        embedder: Arc<dyn Embedder>,
        index: Arc<dyn Index>,
        store: Arc<dyn Store>,
        files: Arc<MockFiles>,
    ) -> Self {
        Self {
            parser,
            chunker,
            embedder,
            index,
            store,
            files,
            runs: futures::lock::Mutex::new(()),
        }
    }

    /// `doc-<file stem>`, with `-2`, `-3`, ... appended if another file has it.
    async fn new_id(&self, path: &Path) -> Result<DocumentId> {
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unnamed");
        let mut id = DocumentId::new(format!("doc-{stem}"));
        let mut n = 1;
        while self.store.document(&id).await?.is_some() {
            n += 1;
            id = DocumentId::new(format!("doc-{stem}-{n}"));
        }
        Ok(id)
    }

    async fn remove_document(&self, record: &DocumentRecord) -> Result<FileStatus> {
        self.index.delete_document(&record.id).await?;
        self.store.delete_document(&record.id).await?;
        Ok(FileStatus::Removed {
            document_id: record.id.clone(),
        })
    }

    async fn ingest_file(
        &self,
        path: &Path,
        mode: SyncMode,
        run: &Run<'_>,
        cancel: &CancellationToken,
    ) -> Result<FileStatus> {
        let format = DocumentFormat::from_path(path)
            .ok_or_else(|| Error::UnsupportedFormat(path.display().to_string()))?;
        let not_found = || Error::NotFound(path.display().to_string());
        let (size, modified) = self.files.metadata(path)?.ok_or_else(not_found)?;
        let existing = self.store.document_by_path(path).await?;
        let indexed = existing
            .as_ref()
            .filter(|r| matches!(r.status, DocumentStatus::Indexed { .. }));
        // Shortcut: same size and modification time as at indexing, no hashing.
        if let Some(record) = indexed
            && mode != SyncMode::Full
            && record.size_bytes == Some(size)
            && record.modified_at == Some(modified)
        {
            return Ok(FileStatus::Unchanged {
                document_id: record.id.clone(),
            });
        }
        let hash = self.files.hash(path)?.ok_or_else(not_found)?;
        if let Some(record) = indexed
            && record.content_hash.as_deref() == Some(hash.as_str())
        {
            let mut record = record.clone();
            record.size_bytes = Some(size);
            record.modified_at = Some(modified);
            self.store.upsert_document(&record).await?;
            return Ok(FileStatus::Unchanged {
                document_id: record.id,
            });
        }
        let id = match existing.as_ref() {
            Some(record) => record.id.clone(),
            None => self.new_id(path).await?,
        };
        let source = SourceFile::new(id.clone(), path, hash.clone())?;
        // While processing, the record no longer vouches for an indexed version.
        let mut record = DocumentRecord {
            id: id.clone(),
            path: source.path.clone(),
            format,
            title: existing.as_ref().and_then(|r| r.title.clone()),
            content_hash: None,
            size_bytes: Some(size),
            modified_at: Some(modified),
            status: DocumentStatus::Queued,
        };
        self.store.upsert_document(&record).await?;

        match self.process(&source, run, cancel).await {
            Ok((title, chunks)) => {
                record.title = title;
                record.content_hash = Some(hash);
                record.size_bytes = Some(size);
                record.modified_at = Some(modified);
                record.status = DocumentStatus::Indexed { chunks };
                self.store.upsert_document(&record).await?;
                Ok(FileStatus::Indexed {
                    document_id: id,
                    chunks,
                })
            }
            Err(Error::Cancelled) => {
                // State after a cancel: record `Queued` without hash, no chunks, no content.
                self.index.delete_document(&id).await?;
                self.store.delete_document(&id).await?;
                self.store.upsert_document(&record).await?;
                Err(Error::Cancelled)
            }
            Err(error) => {
                // State after a failure: record `Failed` without hash, no chunks, no content.
                self.index.delete_document(&id).await?;
                self.store.delete_document(&id).await?;
                let (kind, message) = (error.kind(), error.to_string());
                record.status = DocumentStatus::Failed {
                    kind,
                    message: message.clone(),
                };
                self.store.upsert_document(&record).await?;
                Ok(FileStatus::Failed { kind, message })
            }
        }
    }

    /// Parses, chunks, embeds and indexes one file. Returns title and chunk count.
    async fn process(
        &self,
        source: &SourceFile,
        run: &Run<'_>,
        cancel: &CancellationToken,
    ) -> Result<(Option<String>, u32)> {
        let path = source.path.as_path();
        run.emit(path, FileStatus::Parsing);
        let document = self.parser.parse(source, cancel.clone()).await?;
        run.emit(path, FileStatus::Chunking);
        let chunks = self.chunker.chunk(&document).await?;
        run.emit(path, FileStatus::Embedding);
        let embeddings = self
            .embedder
            .embed_documents(&chunks, cancel.clone())
            .await?;
        if embeddings.len() != chunks.len() {
            return Err(Error::Model(
                "embedder returned a wrong number of vectors".into(),
            ));
        }
        if cancel.is_cancelled() {
            return Err(Error::Cancelled);
        }
        run.emit(path, FileStatus::Indexing);
        let entries: Vec<IndexEntry> = chunks
            .into_iter()
            .zip(embeddings)
            .map(|(chunk, embedding)| IndexEntry { chunk, embedding })
            .collect();
        self.index.delete_document(&source.document_id).await?;
        self.index.add(&entries).await?;
        self.store.save_content(&document).await?;
        let count =
            u32::try_from(entries.len()).map_err(|_| Error::Index("too many chunks".into()))?;
        Ok((document.title, count))
    }

    /// `true` if the absence of `path` could be seen: a watched folder holding it is an
    /// available directory and the directory that should hold `path` can be read.
    fn absence_established(&self, folder_keys: &[String], path: &Path) -> bool {
        let Ok(key) = path_key(path) else {
            return false;
        };
        let folder_ok = folder_keys.iter().any(|f| {
            key_within(&key, f) && key != *f && {
                let folder = Path::new(f);
                self.files.is_dir(folder) && self.files.readable(folder)
            }
        });
        folder_ok && path.parent().is_some_and(|dir| self.files.readable(dir))
    }

    /// `true` if a failed file is not to be tried again: same size and modification
    /// time as at the failed attempt, not an I/O error, and a `Normal` run.
    fn keep_failure(record: &DocumentRecord, metadata: (u64, Timestamp), mode: SyncMode) -> bool {
        matches!(&record.status, DocumentStatus::Failed { kind, .. } if *kind != ErrorKind::Io)
            && mode == SyncMode::Normal
            && record.size_bytes == Some(metadata.0)
            && record.modified_at == Some(metadata.1)
    }

    /// Steps 1 and 2 of a run: folder events and dropping stale exclusions.
    async fn check_scope(&self, scope: &ScopeCallback) -> Result<crate::Settings> {
        let settings = self.store.settings().await?;
        let mut seen = BTreeSet::new();
        for folder in &settings.watched_folders {
            let Ok(folder) = canonical_path(folder) else {
                continue;
            };
            if !seen.insert(path_key(&folder)?) {
                continue;
            }
            scope(if !self.files.readable(&folder) {
                ScopeEvent::FolderUnavailable {
                    message: format!("cannot read folder {}", folder.display()),
                    kind: ErrorKind::Io,
                    folder,
                }
            } else if !self.files.exists(&folder) {
                ScopeEvent::FolderUnavailable {
                    message: format!("folder does not exist: {}", folder.display()),
                    kind: ErrorKind::NotFound,
                    folder,
                }
            } else if !self.files.is_dir(&folder) {
                ScopeEvent::FolderUnavailable {
                    message: format!("not a directory: {}", folder.display()),
                    kind: ErrorKind::InvalidInput,
                    folder,
                }
            } else {
                ScopeEvent::FolderAvailable { folder }
            });
        }
        let folders: Vec<String> = seen.into_iter().collect();
        let stale: Vec<PathBuf> = settings
            .excluded
            .iter()
            .filter(|e| {
                let inside =
                    path_key(e).is_ok_and(|k| folders.iter().any(|f| key_within(&k, f) && k != *f));
                !inside || (self.absence_established(&folders, e) && !self.files.exists(e))
            })
            .cloned()
            .collect();
        if stale.is_empty() {
            return Ok(settings);
        }
        let drop: Vec<String> = stale.iter().filter_map(|p| path_key(p).ok()).collect();
        let settings = self
            .store
            .update_settings(Box::new(move |s| {
                s.excluded
                    .retain(|e| path_key(e).is_ok_and(|k| !drop.contains(&k)));
                Ok(())
            }))
            .await?;
        for path in stale {
            scope(ScopeEvent::ExclusionRemoved { path });
        }
        Ok(settings)
    }
}

#[async_trait]
impl Ingestor for MockIngestor {
    async fn sync(
        &self,
        progress: ProgressCallback,
        scope: ScopeCallback,
        mode: SyncMode,
        cancel: CancellationToken,
    ) -> Result<Vec<FileResult>> {
        if cancel.is_cancelled() {
            return Err(Error::Cancelled);
        }
        let _turn = or_cancel(&cancel, async { Ok(self.runs.lock().await) }).await?;
        let settings = self.check_scope(&scope).await?;
        let folder_keys: Vec<String> = settings
            .watched_folders
            .iter()
            .filter_map(|f| path_key(f).ok())
            .collect();

        // Supported files in scope, each once.
        let mut present: Vec<PathBuf> = Vec::new();
        let mut keys = BTreeSet::new();
        for folder in &settings.watched_folders {
            if !self.files.readable(folder) || !self.files.is_dir(folder) {
                continue;
            }
            for file in self.files.files_below(folder) {
                if DocumentFormat::from_path(&file).is_some()
                    && settings.in_scope(&file)
                    && keys.insert(path_key(&file)?)
                {
                    present.push(file);
                }
            }
        }
        present.sort_by_cached_key(|p| path_key(p).unwrap_or_default());

        // Failed files that are not tried again take no part in the run.
        let mut to_process = Vec::with_capacity(present.len());
        for path in present {
            let record = self.store.document_by_path(&path).await?;
            let metadata = self.files.metadata(&path)?;
            let keep = record
                .zip(metadata)
                .is_some_and(|(r, m)| Self::keep_failure(&r, m, mode));
            if !keep {
                to_process.push(path);
            }
        }

        let mut removals = Vec::new();
        for record in self.store.documents().await? {
            let gone = !settings.in_scope(&record.path)
                || (self.absence_established(&folder_keys, &record.path)
                    && !keys.contains(&path_key(&record.path)?));
            if gone {
                removals.push(record);
            }
        }

        let mut run = Run {
            progress: &progress,
            completed: 0,
            total: u32::try_from(removals.len() + to_process.len())
                .map_err(|_| Error::InvalidInput("too many files".into()))?,
        };
        let mut results = Vec::with_capacity(removals.len() + to_process.len());
        for record in removals {
            if cancel.is_cancelled() {
                return Err(Error::Cancelled);
            }
            let status = self.remove_document(&record).await?;
            run.completed += 1;
            run.emit(&record.path, status.clone());
            results.push(FileResult {
                path: record.path,
                status,
            });
        }
        for path in &to_process {
            run.emit(path, FileStatus::Queued);
        }
        for path in to_process {
            if cancel.is_cancelled() {
                return Err(Error::Cancelled);
            }
            let status = self.ingest_file(&path, mode, &run, &cancel).await?;
            run.completed += 1;
            run.emit(&path, status.clone());
            results.push(FileResult { path, status });
        }
        Ok(results)
    }

    /// Synchronises once (see the type docs), then waits for `cancel`.
    async fn watch(
        &self,
        progress: ProgressCallback,
        scope: ScopeCallback,
        cancel: CancellationToken,
    ) -> Result<()> {
        match self
            .sync(progress, scope, SyncMode::Normal, cancel.clone())
            .await
        {
            Ok(_) | Err(Error::Cancelled) => {}
            Err(error) => return Err(error),
        }
        cancel.cancelled().await;
        Ok(())
    }
}

/// [`Answerer`] with a straightforward pipeline.
///
/// `ask` searches with `max_sources` hits, keeps the relevant ones (rules at the trait),
/// numbers them as sources as long as the prompt fits the token budget, sends one system
/// and one user message to the [`Generator`], streams its output and finally keeps only
/// markers `[n]` that refer to a source. Retrieval and generation run inside the
/// returned stream, in the sequence of case A or B at [`AnswerEvent`]. The wait for the
/// turn, search, counting, rewriting and the answer call are raced against the cancel
/// token, and a cancel or failure ends the stream with a `Done` that keeps the verified
/// text written so far. Answers run one after another in the order of the `ask` calls
/// (a ticket queue); one that has to wait sends `Waiting` first. A follow-up is "rewritten" with a real call to the generator
/// (history cut to `rewrite_history_tokens`), but the mock searches with the original
/// question. Every call to the generator uses the level of the question.
#[derive(Clone)]
pub struct MockAnswerer {
    embedder: Arc<dyn Embedder>,
    index: Arc<dyn Index>,
    reranker: Option<Arc<dyn Reranker>>,
    generator: Arc<dyn Generator>,
    config: PipelineConfig,
    /// Queue of answers; the running one holds the turn.
    queue: Arc<TurnQueue>,
    /// Level of the question this clone answers.
    level: AnswerLevel,
}

impl MockAnswerer {
    /// Answer text without relevant source, German.
    pub const NO_SOURCES_DE: &'static str = "Keine passenden Quellen gefunden.";

    /// Answer text without relevant source, English.
    pub const NO_SOURCES_EN: &'static str = "No relevant sources found.";

    /// System prompt of the rewriting call for a follow-up question.
    pub const REWRITE_PROMPT: &'static str =
        "Rewrite the last question as a standalone question, using the conversation before it.";

    /// System prompt sent with every question.
    pub const SYSTEM_PROMPT: &'static str = "Answer only from the numbered sources. Cite every \
        statement with the number of its source in square brackets, e.g. [1]. If the sources \
        do not contain the answer, say so.";

    /// Pipeline over the given components; `reranker` is `None` for a tier without one.
    pub fn new(
        embedder: Arc<dyn Embedder>,
        index: Arc<dyn Index>,
        reranker: Option<Arc<dyn Reranker>>,
        generator: Arc<dyn Generator>,
        config: PipelineConfig,
    ) -> Self {
        Self {
            embedder,
            index,
            reranker,
            generator,
            config,
            queue: Arc::default(),
            level: AnswerLevel::Fast,
        }
    }

    /// Search and rerank, without the checks of [`Answerer::search`].
    async fn ranked(
        &self,
        request: &SearchRequest,
        cancel: CancellationToken,
    ) -> Result<Vec<SearchHit>> {
        let embedding = self.embedder.embed_query(&request.text).await?;
        let Some(reranker) = &self.reranker else {
            return self.index.search(request, &embedding).await;
        };
        let candidates = SearchRequest {
            limit: request.limit.max(self.config.rerank_candidates),
            ..request.clone()
        };
        let mut hits = self.index.search(&candidates, &embedding).await?;
        let chunks: Vec<_> = hits.iter().map(|h| h.chunk.clone()).collect();
        let scores = reranker.rerank(&request.text, &chunks, cancel).await?;
        if scores.len() != hits.len() {
            return Err(Error::Model(
                "reranker returned a wrong number of scores".into(),
            ));
        }
        for (hit, score) in hits.iter_mut().zip(scores) {
            hit.rerank_score = Some(score);
        }
        hits.sort_by(|a, b| {
            b.rerank_score
                .unwrap_or(0.0)
                .total_cmp(&a.rerank_score.unwrap_or(0.0))
        });
        hits.truncate(request.limit as usize);
        Ok(hits)
    }

    fn is_relevant(&self, hit: &SearchHit) -> bool {
        match (&self.reranker, hit.rerank_score, hit.vector_score) {
            (Some(_), Some(score), _) => score >= self.config.min_relevance,
            (None, _, Some(similarity)) => similarity >= self.config.min_similarity,
            _ => false,
        }
    }
}

/// Builds the user message with numbered sources.
fn user_prompt(question: &str, hits: &[SearchHit]) -> String {
    let mut prompt = String::from("Sources:\n");
    for (i, hit) in hits.iter().enumerate() {
        let provenance = &hit.chunk.provenance;
        let heading = provenance.heading_path.join(" > ");
        prompt.push_str(&format!(
            "\n[{}] {} | {heading}\n{}\n",
            i + 1,
            provenance.source.name,
            hit.chunk.text
        ));
    }
    prompt.push_str(&format!("\nQuestion: {question}"));
    prompt
}

/// Numbers listed in one pair of brackets: `1`, `1, 2`, `1-3` or `1–3`, combined with
/// commas. `None` if `inner` is anything else. Ranges are capped at `max`, because
/// larger numbers are dropped anyway.
fn marker_numbers(inner: &str, max: usize) -> Option<Vec<u32>> {
    let mut numbers = Vec::new();
    for item in inner.split(',') {
        let item = item.trim();
        let (from, to) = match item.split_once(['-', '\u{2013}']) {
            Some((a, b)) => (a.trim(), b.trim()),
            None => (item, item),
        };
        let digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
        if !digits(from) || !digits(to) {
            return None;
        }
        let (from, to) = (from.parse::<u32>().ok()?, to.parse::<u32>().ok()?);
        if from > to {
            return None;
        }
        let capped = to.min(u32::try_from(max).unwrap_or(u32::MAX).max(from));
        numbers.extend(from..=capped);
    }
    Some(numbers)
}

/// Removes a marker cut off at the end of an incomplete text: an `[` followed only by
/// digits, commas, dashes and spaces.
fn trim_cut_marker(text: &str) -> &str {
    match text.rfind('[') {
        Some(start)
            if text[start + 1..]
                .chars()
                .all(|c| c.is_ascii_digit() || matches!(c, ',' | '-' | '\u{2013}' | ' ')) =>
        {
            &text[..start]
        }
        _ => text,
    }
}

/// The outcome for an error that ended an answer.
fn outcome_of(error: &Error) -> AnswerOutcome {
    match error {
        Error::Cancelled => AnswerOutcome::Cancelled,
        error => AnswerOutcome::Failed {
            kind: error.kind(),
            message: error.to_string(),
        },
    }
}

/// Rewrites lists and ranges into single markers, keeps markers `[n]` with
/// `1 <= n <= hits.len()`, removes all other markers and returns the answer with one
/// citation per used number. An incomplete text first loses a marker cut off at its
/// end.
pub(crate) fn verify_citations(
    raw: &str,
    hits: &[SearchHit],
    outcome: AnswerOutcome,
    level: AnswerLevel,
) -> Answer {
    let raw = if outcome == AnswerOutcome::Complete {
        raw
    } else {
        trim_cut_marker(raw)
    };
    let mut text = String::with_capacity(raw.len());
    let mut numbers = BTreeSet::new();
    let mut rest = raw;
    while let Some(start) = rest.find('[') {
        text.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        let parsed = after.find(']').and_then(|close| {
            let inner = &after[..close];
            (!inner.contains('[')).then_some(())?;
            marker_numbers(inner, hits.len()).map(|n| (n, close))
        });
        match parsed {
            Some((list, close)) => {
                let mut seen = BTreeSet::new();
                for n in list {
                    if (1..=hits.len()).contains(&(n as usize)) && seen.insert(n) {
                        text.push_str(&format!("[{n}]"));
                        numbers.insert(n);
                    }
                }
                rest = &after[close + 1..];
            }
            None => {
                text.push('[');
                rest = after;
            }
        }
    }
    text.push_str(rest);
    let citations = numbers
        .into_iter()
        .map(|number| Citation {
            number,
            chunk: hits[number as usize - 1].chunk.clone(),
        })
        .collect();
    Answer {
        text,
        citations,
        no_sources: false,
        level,
        outcome,
    }
}

/// A stream of the given events.
fn events(items: Vec<AnswerEvent>) -> AnswerStream {
    stream::iter(items.into_iter().map(Ok)).boxed()
}

/// A `Status` without progress.
fn status(phase: AnswerPhase) -> AnswerEvent {
    AnswerEvent::Status {
        phase,
        progress: None,
    }
}

/// An empty `Sources` and a `Done` with the outcome of `error`: the end before the
/// search is done.
fn ended_before_sources(error: &Error, level: AnswerLevel) -> AnswerStream {
    let answer = verify_citations("", &[], outcome_of(error), level);
    events(vec![
        AnswerEvent::Sources { hits: Vec::new() },
        AnswerEvent::Done { answer },
    ])
}

impl MockAnswerer {
    /// Relevant hits, before the prompt budget is applied.
    async fn relevant(&self, question: &str, cancel: &CancellationToken) -> Result<Vec<SearchHit>> {
        Ok(self
            .search(
                &SearchRequest::new(question, self.config.max_sources),
                cancel.clone(),
            )
            .await?
            .into_iter()
            .filter(|h| self.is_relevant(h))
            .collect())
    }

    /// The leading sources whose prompt fits the token budget, counted by the
    /// generator for the level (which may have to load its model).
    async fn within_budget(&self, question: &str, hits: &[SearchHit]) -> Result<Vec<SearchHit>> {
        let mut prompt_hits = Vec::new();
        for hit in hits {
            prompt_hits.push(hit.clone());
            let prompt = format!(
                "{}\n{}",
                Self::SYSTEM_PROMPT,
                user_prompt(question, &prompt_hits)
            );
            let tokens = self.generator.count_tokens(self.level, &prompt).await?;
            if tokens > self.config.prompt_token_budget as usize {
                prompt_hits.pop();
                break;
            }
        }
        Ok(prompt_hits)
    }

    /// The history used for rewriting: newest whole turns first, within
    /// `rewrite_history_tokens`; returned in chronological order.
    async fn rewrite_history(&self, history: &[ChatTurn]) -> Result<Vec<ChatTurn>> {
        let mut selected = Vec::new();
        let mut used = 0;
        for turn in history.iter().rev() {
            let text = format!("{}\n{}", turn.question, turn.answer.text);
            let tokens = self.generator.count_tokens(self.level, &text).await?;
            if used + tokens > self.config.rewrite_history_tokens as usize {
                break;
            }
            used += tokens;
            selected.push(turn.clone());
        }
        selected.reverse();
        Ok(selected)
    }

    /// The rewriting call: sends the selected history and the question to the
    /// generator and drains its stream. The mock keeps the original question.
    async fn rewrite(
        &self,
        question: &str,
        history: &[ChatTurn],
        cancel: &CancellationToken,
    ) -> Result<String> {
        if history.is_empty() {
            return Ok(question.to_owned());
        }
        let mut messages = vec![Message {
            role: Role::System,
            content: Self::REWRITE_PROMPT.into(),
        }];
        for turn in history {
            messages.push(Message {
                role: Role::User,
                content: turn.question.clone(),
            });
            messages.push(Message {
                role: Role::Assistant,
                content: turn.answer.text.clone(),
            });
        }
        messages.push(Message {
            role: Role::User,
            content: question.to_owned(),
        });
        let request = GenerationRequest {
            level: self.level,
            messages,
            max_tokens: Some(self.config.max_answer_tokens),
            temperature: None,
        };
        let mut generation = self.generator.generate(request, cancel.clone()).await?;
        while let Some(item) = generation.next().await {
            item?;
        }
        Ok(question.to_owned())
    }

    /// Everything after the checks of `ask`, with the ticket taken there; never fails.
    /// Runs at once if the ticket holds the turn when the stream is first polled,
    /// otherwise sends `Waiting` and waits for it.
    fn answer(
        self,
        ticket: Ticket,
        question: String,
        history: Vec<ChatTurn>,
        language: UiLanguage,
        cancel: CancellationToken,
    ) -> AnswerStream {
        stream::once(async move {
            if ticket.runs() {
                return self.in_turn(ticket, question, history, language, cancel);
            }
            let level = self.level;
            let rest = stream::once(async move {
                match or_cancel(&cancel, futures::FutureExt::map(ticket.turn(), Ok)).await {
                    Ok(()) => self.in_turn(ticket, question, history, language, cancel),
                    // The ticket leaves the queue here; the order of the others stays.
                    Err(error) => ended_before_sources(&error, level),
                }
            })
            .flatten();
            events(vec![status(AnswerPhase::Waiting)])
                .chain(rest)
                .boxed()
        })
        .flatten()
        .boxed()
    }

    /// The answer while it holds the turn, which ends with `Done` or when the stream is
    /// dropped.
    fn in_turn(
        self,
        ticket: Ticket,
        question: String,
        history: Vec<ChatTurn>,
        language: UiLanguage,
        cancel: CancellationToken,
    ) -> AnswerStream {
        let first = if history.is_empty() {
            AnswerPhase::Searching
        } else {
            AnswerPhase::StartingModel
        };
        let mut ticket = Some(ticket);
        events(vec![status(first)])
            .chain(self.answer_in_turn(question, history, language, cancel))
            .map(move |item| {
                if matches!(item, Ok(AnswerEvent::Done { .. })) {
                    ticket.take();
                }
                item
            })
            .boxed()
    }

    /// The answer once it has its turn, without its first event.
    fn answer_in_turn(
        self,
        question: String,
        history: Vec<ChatTurn>,
        language: UiLanguage,
        cancel: CancellationToken,
    ) -> AnswerStream {
        if history.is_empty() {
            // Case A: search, sources, then the model.
            return stream::once(async move {
                match self.relevant(&question, &cancel).await {
                    Err(error) => ended_before_sources(&error, self.level),
                    Ok(hits) => self.after_search(question, hits, true, language, cancel),
                }
            })
            .flatten()
            .boxed();
        }
        // Case B: the model first (counting the history waits for it), then rewriting
        // and search, then the sources.
        stream::once(async move {
            let selected = match or_cancel(&cancel, self.rewrite_history(&history)).await {
                Ok(selected) => selected,
                Err(error) => return ended_before_sources(&error, self.level),
            };
            let search = stream::once(async move {
                let searched = async {
                    let rewritten = self.rewrite(&question, &selected, &cancel).await?;
                    self.relevant(&rewritten, &cancel).await
                };
                match or_cancel(&cancel, searched).await {
                    Err(error) => ended_before_sources(&error, self.level),
                    Ok(hits) => self.after_search(question, hits, false, language, cancel),
                }
            })
            .flatten();
            events(vec![status(AnswerPhase::Searching)])
                .chain(search)
                .boxed()
        })
        .flatten()
        .boxed()
    }

    /// `Sources`, in case A the model start, then the answer call.
    fn after_search(
        self,
        question: String,
        hits: Vec<SearchHit>,
        announce_start: bool,
        language: UiLanguage,
        cancel: CancellationToken,
    ) -> AnswerStream {
        if hits.is_empty() {
            let text = match language {
                UiLanguage::German => Self::NO_SOURCES_DE,
                UiLanguage::English => Self::NO_SOURCES_EN,
            };
            let answer = Answer {
                text: text.into(),
                citations: Vec::new(),
                no_sources: true,
                level: self.level,
                outcome: AnswerOutcome::Complete,
            };
            return events(vec![
                AnswerEvent::Sources { hits: Vec::new() },
                AnswerEvent::Done { answer },
            ]);
        }
        let mut head = vec![AnswerEvent::Sources { hits: hits.clone() }];
        if announce_start {
            head.push(status(AnswerPhase::StartingModel));
        }
        let generation = stream::once(self.generate(question, hits, cancel)).flatten();
        events(head).chain(generation).boxed()
    }

    /// Counting, the answer call and the final `Done`.
    async fn generate(
        self,
        question: String,
        hits: Vec<SearchHit>,
        cancel: CancellationToken,
    ) -> AnswerStream {
        let level = self.level;
        let done_early = |error: Error, hits: &[SearchHit]| {
            let answer = verify_citations("", hits, outcome_of(&error), level);
            events(vec![AnswerEvent::Done { answer }])
        };
        let prompt_hits = match or_cancel(&cancel, self.within_budget(&question, &hits)).await {
            Ok(prompt_hits) => prompt_hits,
            Err(error) => return done_early(error, &hits),
        };
        let request = GenerationRequest {
            level: self.level,
            messages: vec![
                Message {
                    role: Role::System,
                    content: Self::SYSTEM_PROMPT.into(),
                },
                Message {
                    role: Role::User,
                    content: user_prompt(&question, &prompt_hits),
                },
            ],
            max_tokens: Some(self.config.max_answer_tokens),
            temperature: None,
        };
        let started = self.generator.generate(request, cancel.clone());
        let generation = match or_cancel(&cancel, started).await {
            Ok(generation) => generation,
            Err(error) => return done_early(error, &prompt_hits),
        };
        // Raced against the token: the stream is cut at a cancel even if the generator
        // does not end it, and `cut` records that it was.
        let cut = Arc::new(AtomicBool::new(false));
        let generation = {
            let cut = Arc::clone(&cut);
            generation.take_until(async move {
                cancel.cancelled().await;
                cut.store(true, Ordering::SeqCst);
            })
        };

        // (accumulated text, outcome)
        let state = Arc::new(Mutex::new((String::new(), AnswerOutcome::Complete)));
        let items = {
            let state = Arc::clone(&state);
            generation.filter_map(move |item| {
                let mut state = state.lock().unwrap_or_else(PoisonError::into_inner);
                let event = match item {
                    // Already announced by the answerer.
                    Ok(GenerationEvent::Phase {
                        phase: AnswerPhase::StartingModel,
                        ..
                    }) => None,
                    Ok(GenerationEvent::Phase { phase, progress }) => {
                        Some(Ok(AnswerEvent::Status { phase, progress }))
                    }
                    Ok(GenerationEvent::Text(piece)) => {
                        state.0.push_str(&piece);
                        Some(Ok(AnswerEvent::Delta { text: piece }))
                    }
                    Err(error) => {
                        state.1 = outcome_of(&error);
                        None
                    }
                };
                future::ready(event)
            })
        };
        let done = stream::once(async move {
            let state = state.lock().unwrap_or_else(PoisonError::into_inner);
            let outcome = if cut.load(Ordering::SeqCst) {
                AnswerOutcome::Cancelled
            } else {
                state.1.clone()
            };
            Ok(AnswerEvent::Done {
                answer: verify_citations(&state.0, &prompt_hits, outcome, level),
            })
        });
        items.chain(done).boxed()
    }
}

#[async_trait]
impl Answerer for MockAnswerer {
    async fn search(
        &self,
        request: &SearchRequest,
        cancel: CancellationToken,
    ) -> Result<Vec<SearchHit>> {
        check_request(request)?;
        or_cancel(&cancel, self.ranked(request, cancel.clone())).await
    }

    async fn ask(
        &self,
        question: &str,
        history: &[ChatTurn],
        level: AnswerLevel,
        language: UiLanguage,
        cancel: CancellationToken,
    ) -> Result<AnswerStream> {
        if question.trim().is_empty() {
            return Err(Error::InvalidInput("empty question".into()));
        }
        self.generator.check_level(level)?;
        // Joins the queue now, so the order follows the calls, not the first poll.
        let ticket = TurnQueue::join(&self.queue);
        let this = Self {
            level,
            ..self.clone()
        };
        Ok(this.answer(
            ticket,
            question.to_owned(),
            history.to_vec(),
            language,
            cancel,
        ))
    }
}
