//! Mocks of the component traits.

use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use async_trait::async_trait;
use futures::future::{BoxFuture, Shared};
use futures::{FutureExt, StreamExt};
use tokio_util::sync::CancellationToken;

use super::{check_request, fixtures, words};
use crate::{
    Anchor, BoundingBox, CellRange, Chat, ChatId, ChatSummary, Chunk, ChunkId, Chunker, Clock,
    Document, DocumentFormat, DocumentId, DocumentRecord, Element, ElementKind, Embedder,
    Embedding, EmbeddingSpace, Error, GenerationRequest, Generator, HardwareInfo, HardwareTier,
    Index, IndexEntry, MissingModelFile, ModelEntry, ModelEvent, ModelFileStatus, ModelHub,
    ModelManifest, ModelProgressCallback, Parser, Provenance, Reranker, ResolvedModel, Result,
    SearchHit, SearchMode, SearchRequest, Settings, SettingsUpdate, SourceFile, Store, Table,
    TableCell, Timestamp, TokenCounter, path_key,
};
use crate::{
    AnswerLevel, AnswerPhase, GenerationEvent, GenerationStream, ModelStorage, PhaseProgress,
};

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

fn check(cancel: &CancellationToken) -> Result<()> {
    if cancel.is_cancelled() {
        Err(Error::Cancelled)
    } else {
        Ok(())
    }
}

/// [`TokenCounter`] that counts whitespace-separated words.
#[derive(Debug, Default)]
pub struct MockTokenCounter;

#[async_trait]
impl TokenCounter for MockTokenCounter {
    async fn count_tokens(&self, text: &str) -> Result<usize> {
        Ok(text.split_whitespace().count())
    }
}

/// [`Parser`] that returns prepared documents instead of reading files.
///
/// `source.name` selects the document; its `source` is replaced by the requested one.
/// The file itself is never opened.
pub struct MockParser {
    documents: BTreeMap<String, Document>,
}

impl MockParser {
    /// Parser over the documents of `testdata/documents.json`.
    pub fn new() -> Self {
        Self::with_documents(fixtures::documents())
    }

    /// Parser over the given documents, keyed by file name.
    pub fn with_documents(documents: BTreeMap<String, Document>) -> Self {
        Self { documents }
    }
}

impl Default for MockParser {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Parser for MockParser {
    async fn parse(&self, source: &SourceFile, cancel: CancellationToken) -> Result<Document> {
        check(&cancel)?;
        if DocumentFormat::from_path(&source.path).is_none() {
            return Err(Error::UnsupportedFormat(source.path.display().to_string()));
        }
        let mut document = self
            .documents
            .get(&source.name)
            .cloned()
            .ok_or_else(|| Error::Parse(format!("no mock document for {}", source.name)))?;
        document.source = source.clone();
        Ok(document)
    }
}

/// [`Chunker`] that makes one chunk per element and splits what exceeds the limit.
///
/// Headings form the heading path and are not chunks themselves. Text over the limit is
/// split at word boundaries (the mock skips sentence detection), tables follow the
/// splitting rules of the trait. Never merges elements.
pub struct MockChunker {
    tokens: Arc<dyn TokenCounter>,
    max_tokens: usize,
}

impl MockChunker {
    /// Chunker with the given counter and limit (at least 1).
    pub fn new(tokens: Arc<dyn TokenCounter>, max_tokens: usize) -> Self {
        Self {
            tokens,
            max_tokens: max_tokens.max(1),
        }
    }

    async fn fits(&self, text: &str) -> Result<bool> {
        Ok(self.tokens.count_tokens(text).await? <= self.max_tokens)
    }

    /// Greedy split at word boundaries.
    async fn split_text(&self, text: &str) -> Result<Vec<String>> {
        let mut pieces = Vec::new();
        let mut current = String::new();
        for word in text.split_whitespace() {
            let candidate = if current.is_empty() {
                word.to_owned()
            } else {
                format!("{current} {word}")
            };
            if !current.is_empty() && !self.fits(&candidate).await? {
                pieces.push(std::mem::replace(&mut current, word.to_owned()));
            } else {
                current = candidate;
            }
        }
        if !current.is_empty() {
            pieces.push(current);
        }
        Ok(pieces)
    }

    /// Splits a table at row boundaries. Returns text and anchors of each part.
    async fn split_table(
        &self,
        table: &Table,
        element: &Element,
    ) -> Result<Vec<(String, Vec<Anchor>)>> {
        let rows: Vec<Vec<&TableCell>> = (0..table.num_rows)
            .map(|r| table.cells.iter().filter(|c| c.row == r).collect())
            .collect();
        let mut header = rows
            .iter()
            .take_while(|row| !row.is_empty() && row.iter().all(|c| c.column_header))
            .count();
        if header == rows.len() {
            header = 0;
        }
        let line = |row: &[&TableCell]| {
            let mut cols = vec![""; table.num_cols as usize];
            for cell in row {
                if let Some(slot) = cols.get_mut(cell.col as usize) {
                    *slot = &cell.text;
                }
            }
            format!("| {} |", cols.join(" | "))
        };
        let header_text = if header > 0 {
            let mut lines: Vec<String> = rows[..header].iter().map(|r| line(r)).collect();
            lines.push(format!(
                "| {} |",
                vec!["---"; table.num_cols as usize].join(" | ")
            ));
            lines.join("\n")
        } else {
            String::new()
        };
        let render = |body: &[usize], with_header: bool| {
            let mut lines = Vec::new();
            if with_header && header > 0 {
                lines.push(header_text.clone());
            }
            lines.extend(body.iter().map(|&r| line(&rows[r])));
            lines.join("\n")
        };
        let anchors = |row_indices: &[usize]| {
            let cells: Vec<&TableCell> =
                row_indices.iter().flat_map(|&r| rows[r].clone()).collect();
            if cells.iter().any(|c| c.anchor.is_none()) {
                return element.anchors.clone();
            }
            merge_anchors(cells.iter().filter_map(|c| c.anchor.clone()))
        };
        // Rows a part is anchored to: its body rows, plus the header rows for the first part.
        let own_rows = |body: &[usize], first: bool| {
            let mut own: Vec<usize> = if first {
                (0..header).collect()
            } else {
                Vec::new()
            };
            own.extend_from_slice(body);
            own
        };

        // Row groups: rows joined by cells with `row_span > 1` are never separated.
        let mut groups: Vec<Vec<usize>> = Vec::new();
        let mut start = header;
        while start < rows.len() {
            let mut end = start;
            let mut row = start;
            while row <= end && row < rows.len() {
                for cell in &rows[row] {
                    end = end.max((cell.row + cell.row_span.max(1) - 1) as usize);
                }
                row += 1;
            }
            let end = end.min(rows.len() - 1);
            groups.push((start..=end).collect());
            start = end + 1;
        }

        let mut parts = Vec::new();
        let mut current: Vec<usize> = Vec::new();
        for group in groups {
            let mut candidate = current.clone();
            candidate.extend_from_slice(&group);
            if self.fits(&render(&candidate, true)).await? {
                current = candidate;
                continue;
            }
            if !current.is_empty() {
                let first = parts.is_empty();
                parts.push((render(&current, true), anchors(&own_rows(&current, first))));
                current.clear();
                if self.fits(&render(&group, true)).await? {
                    current = group;
                    continue;
                }
            }
            let first = parts.is_empty();
            let group_anchors = anchors(&own_rows(&group, first));
            if self.fits(&render(&group, false)).await? {
                parts.push((render(&group, false), group_anchors));
            } else {
                let joined: Vec<&str> = group
                    .iter()
                    .flat_map(|&r| rows[r].iter().map(|c| c.text.as_str()))
                    .collect();
                for piece in self.split_text(&joined.join(" | ")).await? {
                    parts.push((piece, group_anchors.clone()));
                }
            }
        }
        if !current.is_empty() {
            let first = parts.is_empty();
            parts.push((render(&current, true), anchors(&own_rows(&current, first))));
        }
        Ok(parts)
    }
}

impl Default for MockChunker {
    /// [`MockTokenCounter`] and the default limit of
    /// [`PipelineConfig::chunk_max_tokens`](crate::PipelineConfig).
    fn default() -> Self {
        Self::new(
            Arc::new(MockTokenCounter),
            crate::PipelineConfig::default().chunk_max_tokens as usize,
        )
    }
}

/// Merges anchors: one box per page around all boxes on it, one range per sheet around
/// all ranges on it, other anchors once each, in order of first appearance.
fn merge_anchors(anchors: impl Iterator<Item = Anchor>) -> Vec<Anchor> {
    let mut merged: Vec<Anchor> = Vec::new();
    for anchor in anchors {
        let target = merged.iter_mut().find(|m| match (&**m, &anchor) {
            (Anchor::Page { page: a, .. }, Anchor::Page { page: b, .. }) => a == b,
            (Anchor::Sheet { sheet: a, .. }, Anchor::Sheet { sheet: b, .. }) => a == b,
            (a, b) => a == b,
        });
        match (target, anchor) {
            (Some(Anchor::Page { bbox, .. }), Anchor::Page { bbox: b, .. }) => {
                *bbox = BoundingBox {
                    left: bbox.left.min(b.left),
                    top: bbox.top.min(b.top),
                    right: bbox.right.max(b.right),
                    bottom: bbox.bottom.max(b.bottom),
                };
            }
            (Some(Anchor::Sheet { range, .. }), Anchor::Sheet { range: r, .. }) => {
                *range = CellRange {
                    start_row: range.start_row.min(r.start_row),
                    start_col: range.start_col.min(r.start_col),
                    end_row: range.end_row.max(r.end_row),
                    end_col: range.end_col.max(r.end_col),
                };
            }
            (Some(_), _) => {}
            (None, anchor) => merged.push(anchor),
        }
    }
    merged
}

#[async_trait]
impl Chunker for MockChunker {
    async fn chunk(&self, document: &Document) -> Result<Vec<Chunk>> {
        let mut heading_path: Vec<String> = Vec::new();
        let mut parts: Vec<(String, Vec<Anchor>, Vec<String>)> = Vec::new();
        for element in &document.elements {
            match &element.kind {
                ElementKind::Heading { level } => {
                    heading_path.truncate(usize::from(level.saturating_sub(1)));
                    heading_path.push(element.text.clone());
                    continue;
                }
                ElementKind::Furniture => continue,
                _ if element.text.trim().is_empty() => continue,
                _ => {}
            }
            let pieces = if self.fits(&element.text).await? {
                vec![(element.text.clone(), element.anchors.clone())]
            } else if let ElementKind::Table(table) = &element.kind
                && !table.cells.is_empty()
            {
                self.split_table(table, element).await?
            } else {
                self.split_text(&element.text)
                    .await?
                    .into_iter()
                    .map(|text| (text, element.anchors.clone()))
                    .collect()
            };
            parts.extend(
                pieces
                    .into_iter()
                    .map(|(text, anchors)| (text, anchors, heading_path.clone())),
            );
        }
        parts
            .into_iter()
            .enumerate()
            .map(|(ordinal, (text, anchors, heading_path))| {
                let ordinal =
                    u32::try_from(ordinal).map_err(|_| Error::Chunk("too many chunks".into()))?;
                Ok(Chunk {
                    id: ChunkId::new(&document.source.document_id, ordinal),
                    text,
                    language: document.language.clone(),
                    provenance: Provenance {
                        source: document.source.clone(),
                        heading_path,
                        anchors,
                    },
                })
            })
            .collect()
    }
}

/// [`Embedder`] that hashes words into a vector (FNV-1a), then L2-normalises it.
///
/// Texts sharing words get similar vectors. Queries and documents are treated alike.
/// Its space has the model id `mock-embedding`.
#[derive(Debug)]
pub struct MockEmbedder {
    dimension: usize,
}

impl MockEmbedder {
    /// Embedder with vectors of length `dimension` (at least 1).
    pub fn new(dimension: usize) -> Self {
        Self {
            dimension: dimension.max(1),
        }
    }

    fn embed(&self, text: &str) -> Embedding {
        let mut vector = vec![0.0_f32; self.dimension];
        for word in words(text) {
            let hash = word.bytes().fold(0xcbf2_9ce4_8422_2325_u64, |h, b| {
                (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3)
            });
            let slot = usize::try_from(hash % self.dimension as u64).unwrap_or(0);
            vector[slot] += 1.0;
        }
        let norm = vector.iter().map(|v| v * v).sum::<f32>().sqrt();
        if norm > 0.0 {
            vector.iter_mut().for_each(|v| *v /= norm);
        } else {
            // Text without words: a fixed unit vector, so the result stays normalised.
            vector[0] = 1.0;
        }
        vector
    }
}

impl Default for MockEmbedder {
    /// 64 dimensions.
    fn default() -> Self {
        Self::new(64)
    }
}

#[async_trait]
impl Embedder for MockEmbedder {
    fn space(&self) -> EmbeddingSpace {
        EmbeddingSpace {
            model_id: "mock-embedding".into(),
            dimension: u32::try_from(self.dimension).unwrap_or(u32::MAX),
        }
    }

    async fn embed_query(&self, query: &str) -> Result<Embedding> {
        if query.trim().is_empty() {
            return Err(Error::InvalidInput("empty query".into()));
        }
        Ok(self.embed(query))
    }

    async fn embed_documents(
        &self,
        chunks: &[Chunk],
        cancel: CancellationToken,
    ) -> Result<Vec<Embedding>> {
        check(&cancel)?;
        Ok(chunks.iter().map(|c| self.embed(&c.text)).collect())
    }
}

/// [`Reranker`] whose score is the share of distinct query words found in the heading
/// path and text of the chunk. Each chunk is scored on its own, so its score does not
/// depend on the other chunks of the call.
#[derive(Debug, Default)]
pub struct MockReranker;

#[async_trait]
impl Reranker for MockReranker {
    async fn rerank(
        &self,
        query: &str,
        chunks: &[Chunk],
        cancel: CancellationToken,
    ) -> Result<Vec<f32>> {
        check(&cancel)?;
        if query.trim().is_empty() {
            return Err(Error::InvalidInput("empty query".into()));
        }
        let query_words: HashSet<String> = words(query).into_iter().collect();
        Ok(chunks
            .iter()
            .map(|chunk| {
                let heading = chunk.provenance.heading_path.join(" ");
                let chunk_words: HashSet<String> = words(&format!("{heading}\n{}", chunk.text))
                    .into_iter()
                    .collect();
                query_words.intersection(&chunk_words).count() as f32
                    / query_words.len().max(1) as f32
            })
            .collect())
    }
}

/// In-memory [`Index`].
///
/// Vector score is the dot product (cosine for normalised vectors), full-text score the
/// share of query words in the chunk (no stemming), hybrid is reciprocal rank fusion
/// with k = 60 and `vector_weight` for the vector list.
pub struct MockIndex {
    space: EmbeddingSpace,
    vector_weight: f32,
    entries: Mutex<Vec<IndexEntry>>,
}

impl MockIndex {
    /// Empty index bound to `space`.
    pub fn new(space: EmbeddingSpace, vector_weight: f32) -> Self {
        Self {
            space,
            vector_weight,
            entries: Mutex::new(Vec::new()),
        }
    }

    /// Number of stored chunks.
    pub fn len(&self) -> usize {
        lock(&self.entries).len()
    }

    /// Returns `true` if no chunk is stored.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn check_dimension(&self, length: usize, what: &str) -> Result<()> {
        if length == self.space.dimension as usize {
            Ok(())
        } else {
            Err(Error::EmbeddingMismatch(format!(
                "{what} has length {length}, index {} expects {}",
                self.space.model_id, self.space.dimension
            )))
        }
    }
}

fn ranked(mut scores: Vec<(usize, f32)>) -> Vec<(usize, f32)> {
    scores.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
    scores
}

#[async_trait]
impl Index for MockIndex {
    fn space(&self) -> EmbeddingSpace {
        self.space.clone()
    }

    async fn add(&self, entries: &[IndexEntry]) -> Result<()> {
        for entry in entries {
            self.check_dimension(
                entry.embedding.len(),
                &format!("embedding of {}", entry.chunk.id),
            )?;
        }
        let mut stored = lock(&self.entries);
        for entry in entries {
            match stored.iter_mut().find(|e| e.chunk.id == entry.chunk.id) {
                Some(existing) => *existing = entry.clone(),
                None => stored.push(entry.clone()),
            }
        }
        Ok(())
    }

    async fn delete_document(&self, id: &DocumentId) -> Result<()> {
        lock(&self.entries).retain(|e| e.chunk.document_id() != id);
        Ok(())
    }

    async fn search(&self, request: &SearchRequest, embedding: &[f32]) -> Result<Vec<SearchHit>> {
        check_request(request)?;
        if request.mode != SearchMode::FullText {
            self.check_dimension(embedding.len(), "query embedding")?;
        }
        let stored = lock(&self.entries);
        let candidates: Vec<(usize, &IndexEntry)> = stored
            .iter()
            .enumerate()
            .filter(|(_, e)| {
                request
                    .documents
                    .as_ref()
                    .is_none_or(|docs| docs.contains(e.chunk.document_id()))
            })
            .collect();

        let cosine: HashMap<usize, f32> = candidates
            .iter()
            .map(|(i, e)| {
                (
                    *i,
                    e.embedding.iter().zip(embedding).map(|(a, b)| a * b).sum(),
                )
            })
            .collect();
        let query_words: HashSet<String> = words(&request.text).into_iter().collect();
        let vector = || ranked(cosine.iter().map(|(i, s)| (*i, *s)).collect());
        let full_text = || {
            let mut scores: Vec<(usize, f32)> = candidates
                .iter()
                .map(|(i, e)| {
                    let chunk_words: HashSet<String> = words(&e.chunk.text).into_iter().collect();
                    let found = query_words.intersection(&chunk_words).count();
                    (*i, found as f32 / query_words.len().max(1) as f32)
                })
                .collect();
            scores.retain(|(_, s)| *s > 0.0);
            ranked(scores)
        };
        let results = match request.mode {
            SearchMode::Vector => vector(),
            SearchMode::FullText => full_text(),
            SearchMode::Hybrid => {
                let mut fused: HashMap<usize, f32> = HashMap::new();
                for (weight, list) in [
                    (self.vector_weight, vector()),
                    (1.0 - self.vector_weight, full_text()),
                ] {
                    for (rank, (i, _)) in list.into_iter().enumerate() {
                        *fused.entry(i).or_default() += weight / (60.0 + rank as f32 + 1.0);
                    }
                }
                ranked(fused.into_iter().collect())
            }
        };
        Ok(results
            .into_iter()
            .take(request.limit as usize)
            .map(|(i, score)| SearchHit {
                chunk: stored[i].chunk.clone(),
                retrieval_score: score,
                vector_score: (request.mode != SearchMode::FullText).then(|| cosine[&i]),
                rerank_score: None,
            })
            .collect())
    }
}

/// [`Generator`] that streams a fixed reply, one word (with its trailing space) per item.
///
/// Offers all levels with their models provided, unless [`MockGenerator::offering`] or
/// [`MockGenerator::without_model`] say otherwise. Holds one model at a time:
/// `count_tokens` and `generate` load the model of their level (counting one token per
/// whitespace-separated word), and `generate` reports `StartingModel` only when it has
/// to load. Then `ReadingSources` twice with progress over the prompt words (half,
/// then all), and `Writing` before the first word. Every request is recorded and
/// available through [`MockGenerator::requests`]. [`MockGenerator::failing_after`]
/// simulates a backend crash.
pub struct MockGenerator {
    reply: String,
    fail_after: Option<usize>,
    offered: Vec<AnswerLevel>,
    missing: Vec<AnswerLevel>,
    loaded: Mutex<Option<AnswerLevel>>,
    counted: Mutex<Vec<AnswerLevel>>,
    requests: Mutex<Vec<GenerationRequest>>,
}

impl MockGenerator {
    /// Generator that always answers `reply`.
    pub fn new(reply: impl Into<String>) -> Self {
        Self {
            reply: reply.into(),
            fail_after: None,
            offered: AnswerLevel::ALL.to_vec(),
            missing: Vec::new(),
            loaded: Mutex::new(None),
            counted: Mutex::new(Vec::new()),
            requests: Mutex::new(Vec::new()),
        }
    }

    /// Generator that yields the first `words` words of `reply` and then fails with
    /// `Error::Generator`, like a crashed backend.
    pub fn failing_after(reply: impl Into<String>, words: usize) -> Self {
        Self {
            fail_after: Some(words),
            ..Self::new(reply)
        }
    }

    /// Offers only `levels`, like the generator of a tier below the minimum.
    pub fn offering(mut self, levels: &[AnswerLevel]) -> Self {
        self.offered = levels.to_vec();
        self
    }

    /// Treats the model of `level` as not provided yet.
    pub fn without_model(mut self, level: AnswerLevel) -> Self {
        self.missing.push(level);
        self
    }

    /// All requests received so far, oldest first.
    pub fn requests(&self) -> Vec<GenerationRequest> {
        lock(&self.requests).clone()
    }

    /// The levels of all `count_tokens` calls so far, oldest first.
    pub fn counted(&self) -> Vec<AnswerLevel> {
        lock(&self.counted).clone()
    }

    /// The level whose model is loaded, `None` before the first call.
    pub fn loaded(&self) -> Option<AnswerLevel> {
        *lock(&self.loaded)
    }

    /// Loads the model of `level`; `true` if another one (or none) was loaded.
    fn load(&self, level: AnswerLevel) -> bool {
        lock(&self.loaded).replace(level) != Some(level)
    }
}

#[async_trait]
impl Generator for MockGenerator {
    fn check_level(&self, level: AnswerLevel) -> Result<()> {
        if !self.offered.contains(&level) {
            return Err(Error::InvalidInput(format!(
                "level {level:?} is not offered"
            )));
        }
        if self.missing.contains(&level) {
            return Err(Error::ModelMissing(format!("{level:?}")));
        }
        Ok(())
    }

    async fn count_tokens(&self, level: AnswerLevel, text: &str) -> Result<usize> {
        self.check_level(level)?;
        self.load(level);
        lock(&self.counted).push(level);
        Ok(text.split_whitespace().count())
    }

    async fn generate(
        &self,
        request: GenerationRequest,
        cancel: CancellationToken,
    ) -> Result<GenerationStream> {
        self.check_level(request.level)?;
        if request.messages.is_empty() {
            return Err(Error::InvalidInput("no messages".into()));
        }
        let prompt_words: usize = request
            .messages
            .iter()
            .map(|m| m.content.split_whitespace().count())
            .sum();
        let total = u32::try_from(prompt_words).unwrap_or(u32::MAX);
        let loads = self.load(request.level);
        lock(&self.requests).push(request);

        let mut items: VecDeque<Result<GenerationEvent>> = VecDeque::new();
        let phase = |phase, progress| Ok(GenerationEvent::Phase { phase, progress });
        if loads {
            items.push_back(phase(AnswerPhase::StartingModel, None));
        }
        for done in [total / 2, total] {
            items.push_back(phase(
                AnswerPhase::ReadingSources,
                Some(PhaseProgress { done, total }),
            ));
        }
        items.push_back(phase(AnswerPhase::Writing, None));
        for (i, word) in self.reply.split_inclusive(' ').enumerate() {
            if self.fail_after == Some(i) {
                break;
            }
            items.push_back(Ok(GenerationEvent::Text(word.to_owned())));
        }
        if self.fail_after.is_some() {
            items.push_back(Err(Error::Generator("backend crashed".into())));
        }
        let stream = futures::stream::unfold(
            (items, cancel, false),
            |(mut items, cancel, finished)| async move {
                if finished {
                    return None;
                }
                if cancel.is_cancelled() {
                    return Some((Err(Error::Cancelled), (items, cancel, true)));
                }
                let item = items.pop_front()?;
                let last = item.is_err();
                Some((item, (items, cancel, last)))
            },
        );
        Ok(stream.boxed())
    }
}

#[derive(Default)]
struct StoreState {
    documents: BTreeMap<DocumentId, DocumentRecord>,
    contents: HashMap<DocumentId, Document>,
    chats: Vec<Chat>,
    settings: Option<Settings>,
}

/// In-memory [`Store`].
#[derive(Default)]
pub struct MockStore {
    state: Mutex<StoreState>,
}

impl MockStore {
    /// Empty store.
    pub fn new() -> Self {
        Self::default()
    }
}

fn key(path: &Path) -> String {
    path_key(path).unwrap_or_else(|_| path.to_string_lossy().to_lowercase())
}

#[async_trait]
impl Store for MockStore {
    async fn upsert_document(&self, record: &DocumentRecord) -> Result<()> {
        lock(&self.state)
            .documents
            .insert(record.id.clone(), record.clone());
        Ok(())
    }

    async fn document(&self, id: &DocumentId) -> Result<Option<DocumentRecord>> {
        Ok(lock(&self.state).documents.get(id).cloned())
    }

    async fn document_by_path(&self, path: &Path) -> Result<Option<DocumentRecord>> {
        let wanted = path_key(path)?;
        Ok(lock(&self.state)
            .documents
            .values()
            .find(|r| key(&r.path) == wanted)
            .cloned())
    }

    async fn documents(&self) -> Result<Vec<DocumentRecord>> {
        let mut records: Vec<DocumentRecord> =
            lock(&self.state).documents.values().cloned().collect();
        records.sort_by_cached_key(|r| key(&r.path));
        Ok(records)
    }

    async fn delete_document(&self, id: &DocumentId) -> Result<()> {
        let mut state = lock(&self.state);
        state.documents.remove(id);
        state.contents.remove(id);
        Ok(())
    }

    async fn save_content(&self, document: &Document) -> Result<()> {
        lock(&self.state)
            .contents
            .insert(document.source.document_id.clone(), document.clone());
        Ok(())
    }

    async fn content(&self, id: &DocumentId) -> Result<Option<Document>> {
        Ok(lock(&self.state).contents.get(id).cloned())
    }

    async fn save_chat(&self, chat: &Chat) -> Result<()> {
        let mut state = lock(&self.state);
        state.chats.retain(|c| c.id != chat.id);
        state.chats.insert(0, chat.clone());
        Ok(())
    }

    async fn chat(&self, id: &ChatId) -> Result<Option<Chat>> {
        Ok(lock(&self.state)
            .chats
            .iter()
            .find(|c| &c.id == id)
            .cloned())
    }

    async fn chats(&self) -> Result<Vec<ChatSummary>> {
        let mut chats: Vec<ChatSummary> = lock(&self.state)
            .chats
            .iter()
            .map(|c| ChatSummary {
                id: c.id.clone(),
                title: c.title.clone(),
                created_at: c.created_at,
                updated_at: c.updated_at,
            })
            .collect();
        chats.sort_by(|a, b| b.updated_at.cmp(&a.updated_at).then(a.id.cmp(&b.id)));
        Ok(chats)
    }

    async fn delete_chat(&self, id: &ChatId) -> Result<()> {
        lock(&self.state).chats.retain(|c| &c.id != id);
        Ok(())
    }

    async fn settings(&self) -> Result<Settings> {
        Ok(lock(&self.state).settings.clone().unwrap_or_default())
    }

    async fn save_settings(&self, settings: &Settings) -> Result<()> {
        lock(&self.state).settings = Some(settings.clone());
        Ok(())
    }

    async fn update_settings(&self, update: SettingsUpdate) -> Result<Settings> {
        let mut state = lock(&self.state);
        let mut settings = state.settings.clone().unwrap_or_default();
        update(&mut settings)?;
        state.settings = Some(settings.clone());
        Ok(settings)
    }
}

/// Files of a [`MockModelHub`]: ready ones, fetches in flight, partial files with the
/// bytes already there, and how often each file was fetched, keyed by
/// `<model id>/<file path>`.
#[derive(Default)]
struct HubFiles {
    ready: HashSet<String>,
    in_flight: HashMap<String, Shared<BoxFuture<'static, ()>>>,
    partial: HashMap<String, u64>,
    fetched: HashMap<String, u32>,
}

/// [`ModelHub`] with fixed hardware that keeps the provided files in memory.
///
/// `provide` takes the models of [`ModelManifest::models_for`] in their order and
/// reports `Verifying` for every file, then for a file that is not ready `Copying` (with
/// an offline package) or `Downloading`, then `Ready`; it resolves to
/// `<models_dir>/<id>`. A file is fetched once even if several calls need it at the
/// same time: they all wait for the same fetch and report it, and a fetch goes on as
/// long as a call waits for it (one that nobody waits for any more is resumed by the
/// next call). A fetch that has started counts half of the file as present, kept as a
/// partial file when its call is cancelled; the next call continues it, copying or
/// downloading, and names that half in its first `Copying` or `Downloading` event.
/// With [`MockModelHub::with_fetch_gate`] every fetch waits for a gate. It
/// ignores the hardware, neither touches files nor checks checksums, so it also works
/// with the placeholders in `models.toml`. `missing` lists the files not ready yet.
pub struct MockModelHub {
    manifest: ModelManifest,
    hardware: HardwareInfo,
    models_dir: PathBuf,
    free_bytes: u64,
    gate: Option<super::Gate>,
    files: Arc<Mutex<HubFiles>>,
}

impl MockModelHub {
    /// Hub over `manifest` reporting `hardware`, with no model provided and 100 GB free.
    pub fn new(manifest: ModelManifest, hardware: HardwareInfo, models_dir: PathBuf) -> Self {
        Self {
            manifest,
            hardware,
            models_dir,
            free_bytes: 100_000_000_000,
            gate: None,
            files: Arc::default(),
        }
    }

    /// Every fetch (download or copy) waits for `gate` before the file is ready.
    pub fn with_fetch_gate(mut self, gate: super::Gate) -> Self {
        self.gate = Some(gate);
        self
    }

    /// How often the file `path` of model `model_id` was fetched so far.
    pub fn fetches(&self, model_id: &str, path: &str) -> u32 {
        lock(&self.files)
            .fetched
            .get(&format!("{model_id}/{path}"))
            .copied()
            .unwrap_or(0)
    }

    /// The fetch of `key`, shared with calls that already wait for it.
    fn fetch(&self, key: &str, size: Option<u64>) -> Shared<BoxFuture<'static, ()>> {
        let mut files = lock(&self.files);
        if let Some(fetch) = files.in_flight.get(key) {
            return fetch.clone();
        }
        let state = Arc::clone(&self.files);
        let gate = self.gate.clone();
        let done = key.to_owned();
        let fetch = async move {
            // The first half arrives at once, as a partial file.
            lock(&state)
                .partial
                .insert(done.clone(), size.unwrap_or(0) / 2);
            if let Some(gate) = gate {
                gate.passed().await;
            }
            // Only now the file counts as present under its target name.
            let mut files = lock(&state);
            files.in_flight.remove(&done);
            files.partial.remove(&done);
            *files.fetched.entry(done.clone()).or_default() += 1;
            files.ready.insert(done);
        }
        .boxed()
        .shared();
        files.in_flight.insert(key.to_owned(), fetch.clone());
        fetch
    }

    /// Reports `free_bytes` of free space; `provide` fails with `StorageFull` when the
    /// known sizes of the models still to provide exceed it.
    pub fn with_free_bytes(mut self, free_bytes: u64) -> Self {
        self.free_bytes = free_bytes;
        self
    }

    fn entries(&self, tier: HardwareTier, level: AnswerLevel) -> Result<Vec<ModelEntry>> {
        self.manifest.models_for(tier, level)
    }
}

#[async_trait]
impl ModelHub for MockModelHub {
    fn hardware(&self) -> Result<HardwareInfo> {
        Ok(self.hardware)
    }

    fn storage(&self) -> Result<ModelStorage> {
        Ok(ModelStorage {
            dir: self.models_dir.clone(),
            free_bytes: self.free_bytes,
        })
    }

    async fn provide(
        &self,
        tier: HardwareTier,
        level: AnswerLevel,
        source: Option<PathBuf>,
        progress: ModelProgressCallback,
        cancel: CancellationToken,
    ) -> Result<Vec<ResolvedModel>> {
        let entries = self.entries(tier, level)?;
        let key =
            |entry: &ModelEntry, file: &crate::ModelFile| format!("{}/{}", entry.id, file.path);
        let needed: u64 = {
            let files = lock(&self.files);
            entries
                .iter()
                .flat_map(|e| e.files.iter().map(move |f| (e, f)))
                .filter(|(e, f)| !files.ready.contains(&key(e, f)))
                .filter_map(|(_, f)| f.size_bytes)
                .sum()
        };
        if needed > self.free_bytes {
            return Err(Error::StorageFull(format!(
                "{needed} bytes needed, {} free in {}",
                self.free_bytes,
                self.models_dir.display()
            )));
        }
        let mut resolved = Vec::new();
        for entry in entries {
            for file in &entry.files {
                check(&cancel)?;
                let report = |status| {
                    progress(ModelEvent {
                        model_id: entry.id.clone(),
                        role: entry.role,
                        file: file.path.clone(),
                        status,
                    });
                };
                report(ModelFileStatus::Verifying);
                let key = key(&entry, file);
                if !lock(&self.files).ready.contains(&key) {
                    let fetch = self.fetch(&key, file.size_bytes);
                    let done_bytes = lock(&self.files).partial.get(&key).copied().unwrap_or(0);
                    report(if source.is_some() {
                        ModelFileStatus::Copying {
                            done_bytes,
                            total_bytes: file.size_bytes,
                        }
                    } else {
                        ModelFileStatus::Downloading {
                            done_bytes,
                            total_bytes: file.size_bytes,
                        }
                    });
                    super::or_cancel(&cancel, fetch.map(Ok)).await?;
                }
                report(ModelFileStatus::Ready);
            }
            resolved.push(ResolvedModel {
                entry: entry.clone(),
                dir: self.models_dir.join(&entry.id),
            });
        }
        Ok(resolved)
    }

    async fn missing(
        &self,
        tier: HardwareTier,
        level: AnswerLevel,
    ) -> Result<Vec<MissingModelFile>> {
        let entries = self.entries(tier, level)?;
        let files = lock(&self.files);
        Ok(entries
            .iter()
            .flat_map(|entry| entry.files.iter().map(move |file| (entry, file)))
            .filter(|(entry, file)| !files.ready.contains(&format!("{}/{}", entry.id, file.path)))
            .map(|(entry, file)| MissingModelFile {
                model_id: entry.id.clone(),
                role: entry.role,
                file: file.path.clone(),
            })
            .collect())
    }
}

/// [`Clock`] whose time the test sets.
#[derive(Debug, Default)]
pub struct MockClock {
    now: AtomicU64,
}

impl MockClock {
    /// Clock standing at `start`.
    pub fn new(start: Timestamp) -> Self {
        Self {
            now: AtomicU64::new(start.0),
        }
    }

    /// Sets the time.
    pub fn set(&self, time: Timestamp) {
        self.now.store(time.0, Ordering::SeqCst);
    }

    /// Moves the time forward by `millis`.
    pub fn advance(&self, millis: u64) {
        self.now.fetch_add(millis, Ordering::SeqCst);
    }
}

impl Clock for MockClock {
    fn now(&self) -> Timestamp {
        Timestamp(self.now.load(Ordering::SeqCst))
    }
}
