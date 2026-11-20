//! [`LanceIndex`]: the [`Index`] over one LanceDB table in a local folder.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use async_trait::async_trait;
use futures::TryStreamExt;
use kb_core::{
    Chunk, DocumentId, EmbeddingSpace, Error, Index, IndexEntry, Result, SearchHit, SearchMode,
    SearchRequest,
};
use lancedb::arrow::arrow_array::{
    Array, FixedSizeListArray, Float32Array, RecordBatch, RecordBatchIterator, StringArray,
};
use lancedb::arrow::arrow_schema::{DataType, Field, Schema, SchemaRef};
use lancedb::expr::{DfExpr, col, lit};
use lancedb::index::scalar::{FtsIndexBuilder, FullTextSearchQuery};
use lancedb::query::{ExecutableQuery, QueryBase, Select};
use lancedb::table::{Duration, OptimizeAction};
use lancedb::{Connection, DistanceType, Table};
use serde::{Deserialize, Serialize};

use crate::analysis::{Analysis, Analyzer};
use crate::fusion;

/// Version of the stored format: table layout, analysis and word list.
///
/// It rises whenever the columns of the table, the own text analysis
/// ([`crate::analysis`]) or the word list ([`crate::wordlist`]) change, because the
/// stored tokens would no longer match the analysis of the query.
pub const FORMAT_VERSION: u32 = 2;

/// Name of the file that holds space, fusion weight and format version.
pub const META_FILE: &str = "kb-index.json";

/// Number of table versions above which the table is compacted and old versions are
/// removed. Every change (`add` with entries, `delete_document` that deleted rows)
/// adds one version, so this counts changes since the last maintenance, also across
/// sessions: the count is read from the table, not kept in memory.
pub const MAINTENANCE_EVERY: u64 = 20;

/// Each list is read to at least this depth before the fusion.
pub const MIN_FUSION_DEPTH: usize = 100;

const TABLE: &str = "chunks";
const COL_ID: &str = "id";
const COL_DOCUMENT: &str = "document";
const COL_CHUNK: &str = "chunk";
const COL_TOKENS: &str = "tokens";
const COL_VECTOR: &str = "vector";
/// Extra rows read past the wanted number before checking for ties at the cut.
const TIE_MARGIN: usize = 16;
/// Scores closer than this count as equal when deciding whether a list must be read
/// further to resolve ties at the cut (covers rounding between LanceDB's f32 distance
/// and the recomputed cosine).
const TIE_EPSILON: f32 = 1e-6;
/// The only entries an incompletely created index can leave behind.
const OWN_ENTRIES: [&str; 2] = ["chunks.lance", "kb-index.json.tmp"];
/// Folder of the table inside the index folder.
const TABLE_DIR: &str = "chunks.lance";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Meta {
    format_version: u32,
    model_id: String,
    dimension: u32,
    vector_weight: f32,
}

/// A candidate row read from the table.
struct Row {
    id: String,
    chunk: Chunk,
    cosine: f32,
    text_score: f32,
}

/// Write state, guarded so that changes to the table run one after another.
struct WriteState;

/// The search index of a knowledge base: one LanceDB table in a local folder, with
/// exact vector search (cosine), full-text search over the own analysis and
/// reciprocal rank fusion. See the README of `kb-index`.
pub struct LanceIndex {
    dir: PathBuf,
    space: EmbeddingSpace,
    vector_weight: f32,
    _db: Connection,
    table: Table,
    analyzer: Arc<Analyzer>,
    has_text_index: AtomicBool,
    write: tokio::sync::Mutex<WriteState>,
}

fn index_err(dir: &Path, what: &str, e: impl std::fmt::Display) -> Error {
    Error::Index(format!("{what} in {}: {e}", dir.display()))
}

/// Rejects anything that is not a local folder path (ADR 0015).
fn check_local(dir: &Path) -> Result<()> {
    let s = dir.to_string_lossy();
    if s.contains("://") {
        return Err(Error::InvalidInput(format!(
            "index folder must be a local path, not a URL: {s}"
        )));
    }
    let verbatim_unc =
        s.len() >= 8 && s.is_char_boundary(8) && s[..8].eq_ignore_ascii_case("\\\\?\\UNC\\");
    let unc = ((s.starts_with("\\\\") || s.starts_with("//")) && !s.starts_with("\\\\?\\"))
        || verbatim_unc;
    if unc {
        return Err(Error::InvalidInput(format!(
            "index folder must be on a local disk, not a network share: {s}"
        )));
    }
    Ok(())
}

fn schema(dimension: u32) -> Result<SchemaRef> {
    let dim = i32::try_from(dimension)
        .map_err(|_| Error::InvalidInput(format!("dimension {dimension} is too large")))?;
    Ok(Arc::new(Schema::new(vec![
        Field::new(COL_ID, DataType::Utf8, false),
        Field::new(COL_DOCUMENT, DataType::Utf8, false),
        Field::new(COL_CHUNK, DataType::Utf8, false),
        Field::new(COL_TOKENS, DataType::Utf8, false),
        Field::new(
            COL_VECTOR,
            DataType::FixedSizeList(Arc::new(Field::new("item", DataType::Float32, true)), dim),
            false,
        ),
    ])))
}

fn text_index_params() -> FtsIndexBuilder {
    // The tokens are produced by the own analysis; LanceDB only splits at spaces.
    FtsIndexBuilder::default()
        .base_tokenizer("whitespace".to_string())
        .lower_case(false)
        .stem(false)
        .remove_stop_words(false)
        .ascii_folding(false)
        .max_token_length(None)
        .with_position(false)
}

fn cosine(a: &[f32], b: &[f32]) -> f32 {
    let (mut dot, mut na, mut nb) = (0.0_f64, 0.0_f64, 0.0_f64);
    for (x, y) in a.iter().zip(b) {
        let (x, y) = (f64::from(*x), f64::from(*y));
        dot += x * y;
        na += x * x;
        nb += y * y;
    }
    if na == 0.0 || nb == 0.0 {
        return 0.0;
    }
    ((dot / (na.sqrt() * nb.sqrt())).clamp(-1.0, 1.0)) as f32
}

/// A vector the cosine is defined for: all values finite, not all zero.
fn usable_vector(v: &[f32]) -> bool {
    v.iter().all(|x| x.is_finite()) && v.iter().any(|x| *x != 0.0)
}

/// Whether a list cut after `want` rows may hide rows that tie with the row at the
/// cut: the list was cut at `n` rows (not exhausted) and its last row still ties
/// with row `want`.
fn ties_beyond(scores: &[f32], want: usize, n: usize, total: usize) -> bool {
    if scores.len() < n || n >= total || want == 0 || scores.len() <= want {
        return false;
    }
    (scores[want - 1] - scores[scores.len() - 1]).abs() <= TIE_EPSILON
}

impl LanceIndex {
    /// Opens the index in `dir`, or creates an empty one if `dir` is missing or empty.
    ///
    /// `space` and `vector_weight` (from [`kb_core::PipelineConfig::vector_weight`],
    /// in `[0, 1]`) are stored in the folder. Errors: `InvalidInput` for a weight
    /// outside `[0, 1]` or NaN, a dimension of 0 or a path that is not a local folder;
    /// `EmbeddingMismatch` if the folder holds an index of another space or format
    /// version; `Index` for unreadable or foreign files in the folder.
    pub async fn open(
        dir: impl AsRef<Path>,
        space: EmbeddingSpace,
        vector_weight: f32,
    ) -> Result<Self> {
        if !(0.0..=1.0).contains(&vector_weight) {
            return Err(Error::InvalidInput(format!(
                "vector_weight must be in [0, 1], got {vector_weight}"
            )));
        }
        if space.dimension == 0 {
            return Err(Error::InvalidInput("embedding dimension 0".into()));
        }
        let dir = dir.as_ref();
        check_local(dir)?;
        let dir = std::path::absolute(dir)
            .map_err(|e| Error::InvalidInput(format!("index folder {}: {e}", dir.display())))?;
        if dir.exists() && !dir.is_dir() {
            return Err(Error::Index(format!(
                "{} is a file, not a folder for an index",
                dir.display()
            )));
        }
        std::fs::create_dir_all(&dir).map_err(|e| index_err(&dir, "cannot create folder", e))?;
        let meta_path = dir.join(META_FILE);
        let wanted = Meta {
            format_version: FORMAT_VERSION,
            model_id: space.model_id.clone(),
            dimension: space.dimension,
            vector_weight,
        };
        let existing = if meta_path.exists() {
            let text = std::fs::read_to_string(&meta_path)
                .map_err(|e| index_err(&dir, "cannot read index metadata", e))?;
            let meta: Meta = serde_json::from_str(&text)
                .map_err(|e| index_err(&dir, "damaged index metadata", e))?;
            if meta.format_version != FORMAT_VERSION {
                return Err(Error::EmbeddingMismatch(format!(
                    "index in {} has format version {}, this program reads format version {}; \
                     the index must be rebuilt",
                    dir.display(),
                    meta.format_version,
                    FORMAT_VERSION
                )));
            }
            if meta.model_id != space.model_id || meta.dimension != space.dimension {
                return Err(Error::EmbeddingMismatch(format!(
                    "index in {} was built for model {} with dimension {}, not for model {} \
                     with dimension {}",
                    dir.display(),
                    meta.model_id,
                    meta.dimension,
                    space.model_id,
                    space.dimension
                )));
            }
            true
        } else {
            // Without metadata only what an interrupted first opening leaves behind is
            // accepted: the own temporary metadata file and a table that is checked
            // below (own layout, no rows). Anything else is foreign; nothing is deleted.
            let names: Vec<String> = std::fs::read_dir(&dir)
                .map_err(|e| index_err(&dir, "cannot read folder", e))?
                .flatten()
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .collect();
            if names.iter().any(|n| !OWN_ENTRIES.contains(&n.as_str())) {
                return Err(Error::Index(format!(
                    "{} is not empty and holds no index (no {META_FILE})",
                    dir.display()
                )));
            }
            false
        };
        let half_created = !existing && dir.join(TABLE_DIR).exists();

        // The word list is parsed once per process; keep it off the executor.
        let analyzer = tokio::task::spawn_blocking(|| {
            let a = Analyzer::new();
            // Forces parsing of the embedded word list now.
            let _ = a.text_tokens("Bundestag", Analysis::German);
            a
        })
        .await
        .map_err(|e| Error::Index(format!("text analysis failed to start: {e}")))?;

        let uri = dir.to_string_lossy().to_string();
        let db = lancedb::connect(&uri)
            .execute()
            .await
            .map_err(|e| index_err(&dir, "cannot open index", e))?;
        let table = if existing || half_created {
            let table = db.open_table(TABLE).execute().await.map_err(|e| {
                if half_created {
                    index_err(&dir, "no index (no kb-index.json) and no readable table", e)
                } else {
                    index_err(&dir, "cannot open index table", e)
                }
            })?;
            let found = table
                .schema()
                .await
                .map_err(|e| index_err(&dir, "cannot read index table", e))?;
            if found.as_ref() != schema(space.dimension)?.as_ref() {
                return Err(Error::Index(format!(
                    "index table in {} has an unexpected layout",
                    dir.display()
                )));
            }
            if half_created {
                // Only an empty table of the own layout is the rest of an interrupted
                // first opening; a filled one lost its metadata and must be rebuilt.
                let rows = table
                    .count_rows(None)
                    .await
                    .map_err(|e| index_err(&dir, "cannot read index table", e))?;
                if rows > 0 {
                    return Err(Error::Index(format!(
                        "{} holds an index table with {rows} rows but no {META_FILE}; \
                         the index must be rebuilt",
                        dir.display()
                    )));
                }
            }
            table
        } else {
            db.create_empty_table(TABLE, schema(space.dimension)?)
                .execute()
                .await
                .map_err(|e| index_err(&dir, "cannot create index table", e))?
        };
        // The metadata is written last: it marks a complete index, and a changed
        // fusion weight replaces the stored one.
        let stored = serde_json::to_string_pretty(&wanted)
            .map_err(|e| Error::Index(format!("cannot write index metadata: {e}")))?;
        let tmp = dir.join(format!("{META_FILE}.tmp"));
        std::fs::write(&tmp, stored).map_err(|e| index_err(&dir, "cannot write metadata", e))?;
        std::fs::rename(&tmp, &meta_path)
            .map_err(|e| index_err(&dir, "cannot write metadata", e))?;

        let has_text_index = table
            .list_indices()
            .await
            .map_err(|e| index_err(&dir, "cannot list indices", e))?
            .iter()
            .any(|i| i.columns.iter().any(|c| c == COL_TOKENS));
        let index = Self {
            dir,
            space,
            vector_weight,
            _db: db,
            table,
            analyzer: Arc::new(analyzer),
            has_text_index: AtomicBool::new(has_text_index),
            write: tokio::sync::Mutex::new(WriteState),
        };
        // Changes of earlier sessions count: maintenance that is due runs now. If it
        // fails, the index still opens; the next change tries again, because the
        // versions are still there.
        {
            let mut state = index.write.lock().await;
            let _ = index.maintain_if_due(&mut state).await;
        }
        Ok(index)
    }

    /// The folder of the index.
    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// The fusion weight of the vector list.
    pub fn vector_weight(&self) -> f32 {
        self.vector_weight
    }

    /// Number of table versions kept on disk.
    pub async fn version_count(&self) -> Result<usize> {
        Ok(self
            .table
            .list_versions()
            .await
            .map_err(|e| self.err("cannot list versions", e))?
            .len())
    }

    /// Compacts the table, rebuilds the full-text index and removes all old
    /// versions. Runs by itself as soon as more than [`MAINTENANCE_EVERY`] versions
    /// exist, after a change or when the index is opened.
    pub async fn optimize(&self) -> Result<()> {
        let mut state = self.write.lock().await;
        self.optimize_locked(&mut state).await
    }

    fn err(&self, what: &str, e: impl std::fmt::Display) -> Error {
        index_err(&self.dir, what, e)
    }

    async fn optimize_locked(&self, _state: &mut WriteState) -> Result<()> {
        self.table
            .optimize(OptimizeAction::Compact {
                options: Default::default(),
                remap_options: None,
            })
            .await
            .map_err(|e| self.err("compaction failed", e))?;
        if self.has_text_index.load(Ordering::SeqCst) {
            // Rebuilt rather than merged: the BM25 statistics of a merged index still
            // count replaced and deleted rows, so scores would depend on the history.
            self.table
                .create_index(
                    &[COL_TOKENS],
                    lancedb::index::Index::FTS(text_index_params()),
                )
                .replace(true)
                .execute()
                .await
                .map_err(|e| self.err("rebuilding the full-text index failed", e))?;
        }
        // One process opens the index (ADR 0122), so no other transaction can be
        // running and old versions may go at once.
        self.table
            .optimize(OptimizeAction::Prune {
                older_than: Some(Duration::zero()),
                delete_unverified: Some(true),
                error_if_tagged_old_versions: Some(false),
            })
            .await
            .map_err(|e| self.err("removing old versions failed", e))?;
        Ok(())
    }

    /// Runs the maintenance if more than [`MAINTENANCE_EVERY`] versions exist.
    async fn maintain_if_due(&self, state: &mut WriteState) -> Result<()> {
        if self.version_count().await? as u64 > MAINTENANCE_EVERY {
            self.optimize_locked(state).await?;
        }
        Ok(())
    }

    fn filter(documents: &Option<Vec<DocumentId>>) -> Option<DfExpr> {
        documents.as_ref().map(|ids| {
            col(COL_DOCUMENT).in_list(ids.iter().map(|d| lit(d.0.clone())).collect(), false)
        })
    }

    /// Reads rows of a result stream into candidates.
    fn rows(&self, batches: Vec<RecordBatch>, query: Option<&[f32]>) -> Result<Vec<Row>> {
        let mut out = Vec::new();
        for b in batches {
            let ids = string_col(&b, COL_ID).map_err(|e| self.err("bad result", e))?;
            let chunks = string_col(&b, COL_CHUNK).map_err(|e| self.err("bad result", e))?;
            let vectors = b
                .column_by_name(COL_VECTOR)
                .and_then(|c| c.as_any().downcast_ref::<FixedSizeListArray>())
                .ok_or_else(|| self.err("bad result", "no vector column"))?;
            let scores = b
                .column_by_name("_score")
                .and_then(|c| c.as_any().downcast_ref::<Float32Array>());
            for i in 0..b.num_rows() {
                let chunk: Chunk = serde_json::from_str(chunks.value(i))
                    .map_err(|e| self.err("damaged chunk", e))?;
                let cos = match query {
                    Some(q) => {
                        let v = vectors.value(i);
                        let v = v
                            .as_any()
                            .downcast_ref::<Float32Array>()
                            .ok_or_else(|| self.err("bad result", "vector is not f32"))?;
                        cosine(q, v.values())
                    }
                    None => 0.0,
                };
                out.push(Row {
                    id: ids.value(i).to_string(),
                    chunk,
                    cosine: cos,
                    text_score: scores.map(|s| s.value(i)).unwrap_or(0.0),
                });
            }
        }
        Ok(out)
    }

    async fn vector_list(
        &self,
        embedding: &[f32],
        filter: Option<DfExpr>,
        n: usize,
    ) -> Result<Vec<Row>> {
        let mut q = self
            .table
            .query()
            .nearest_to(embedding)
            .map_err(|e| self.err("bad query vector", e))?
            .distance_type(DistanceType::Cosine)
            .bypass_vector_index()
            .select(Select::columns(&[COL_ID, COL_CHUNK, COL_VECTOR]))
            .limit(n);
        if let Some(f) = filter {
            q = q.only_if_expr(f);
        }
        let batches: Vec<RecordBatch> = q
            .execute()
            .await
            .map_err(|e| self.err("vector search failed", e))?
            .try_collect()
            .await
            .map_err(|e| self.err("vector search failed", e))?;
        let mut rows = self.rows(batches, Some(embedding))?;
        rows.sort_by(|a, b| b.cosine.total_cmp(&a.cosine).then_with(|| a.id.cmp(&b.id)));
        Ok(rows)
    }

    async fn text_list(
        &self,
        tokens: &[String],
        embedding: Option<&[f32]>,
        filter: Option<DfExpr>,
        n: usize,
    ) -> Result<Vec<Row>> {
        let fts = FullTextSearchQuery::new(tokens.join(" "))
            .with_column(COL_TOKENS.to_string())
            .map_err(|e| self.err("bad full-text query", e))?;
        let mut q = self
            .table
            .query()
            .full_text_search(fts)
            .select(Select::columns(&[COL_ID, COL_CHUNK, COL_VECTOR]))
            .limit(n);
        if let Some(f) = filter {
            q = q.only_if_expr(f);
        }
        let batches: Vec<RecordBatch> = q
            .execute()
            .await
            .map_err(|e| self.err("full-text search failed", e))?
            .try_collect()
            .await
            .map_err(|e| self.err("full-text search failed", e))?;
        let mut rows = self.rows(batches, embedding)?;
        rows.sort_by(|a, b| {
            b.text_score
                .total_cmp(&a.text_score)
                .then_with(|| a.id.cmp(&b.id))
        });
        Ok(rows)
    }

    /// The best `want` rows of the vector list, ties at the cut ordered by chunk id.
    async fn vector_ranked(
        &self,
        embedding: &[f32],
        filter: Option<DfExpr>,
        want: usize,
        total: usize,
    ) -> Result<Vec<Row>> {
        let mut n = want.saturating_add(TIE_MARGIN).min(total);
        loop {
            let mut rows = self.vector_list(embedding, filter.clone(), n).await?;
            let scores: Vec<f32> = rows.iter().map(|r| r.cosine).collect();
            if !ties_beyond(&scores, want, n, total) {
                rows.truncate(want);
                return Ok(rows);
            }
            n = n.saturating_mul(2).min(total);
        }
    }

    /// The best `want` rows of the full-text list, ties at the cut ordered by chunk id.
    async fn text_ranked(
        &self,
        text: &str,
        embedding: Option<&[f32]>,
        filter: Option<DfExpr>,
        want: usize,
        total: usize,
    ) -> Result<Vec<Row>> {
        if !self.has_text_index.load(Ordering::SeqCst) {
            return Ok(Vec::new());
        }
        let analyzer = self.analyzer.clone();
        let text_owned = text.to_string();
        let tokens = tokio::task::spawn_blocking(move || analyzer.query_tokens(&text_owned))
            .await
            .map_err(|e| Error::Index(format!("query analysis failed: {e}")))?;
        if tokens.is_empty() {
            return Ok(Vec::new());
        }
        let mut n = want.saturating_add(TIE_MARGIN).min(total);
        loop {
            let mut rows = self
                .text_list(&tokens, embedding, filter.clone(), n)
                .await?;
            let scores: Vec<f32> = rows.iter().map(|r| r.text_score).collect();
            if !ties_beyond(&scores, want, n, total) {
                rows.truncate(want);
                return Ok(rows);
            }
            n = n.saturating_mul(2).min(total);
        }
    }

    async fn row_count(&self) -> Result<usize> {
        self.table
            .count_rows(None)
            .await
            .map_err(|e| self.err("cannot count rows", e))
    }
}

/// Builds the rows to write: id, document, chunk as JSON, tokens, vector.
fn build_batch(dimension: u32, rows: Vec<(IndexEntry, String)>) -> Result<RecordBatch> {
    let schema = schema(dimension)?;
    let mut ids = Vec::with_capacity(rows.len());
    let mut docs = Vec::with_capacity(rows.len());
    let mut chunks = Vec::with_capacity(rows.len());
    let mut tokens = Vec::with_capacity(rows.len());
    let mut values = Vec::with_capacity(rows.len() * dimension as usize);
    for (entry, toks) in rows {
        ids.push(entry.chunk.id.0.clone());
        docs.push(entry.chunk.document_id().0.clone());
        chunks.push(
            serde_json::to_string(&entry.chunk)
                .map_err(|e| Error::Index(format!("cannot store chunk: {e}")))?,
        );
        tokens.push(toks);
        values.extend_from_slice(&entry.embedding);
    }
    let dim = i32::try_from(dimension).unwrap_or(i32::MAX);
    let vectors = FixedSizeListArray::try_new(
        Arc::new(Field::new("item", DataType::Float32, true)),
        dim,
        Arc::new(Float32Array::from(values)),
        None,
    )
    .map_err(|e| Error::Index(format!("cannot build vectors: {e}")))?;
    RecordBatch::try_new(
        schema,
        vec![
            Arc::new(StringArray::from(ids)),
            Arc::new(StringArray::from(docs)),
            Arc::new(StringArray::from(chunks)),
            Arc::new(StringArray::from(tokens)),
            Arc::new(vectors),
        ],
    )
    .map_err(|e| Error::Index(format!("cannot build rows: {e}")))
}

fn string_col<'a>(b: &'a RecordBatch, name: &str) -> std::result::Result<&'a StringArray, String> {
    b.column_by_name(name)
        .and_then(|c| c.as_any().downcast_ref::<StringArray>())
        .ok_or_else(|| format!("no column {name}"))
}

#[async_trait]
impl Index for LanceIndex {
    fn space(&self) -> EmbeddingSpace {
        self.space.clone()
    }

    async fn add(&self, entries: &[IndexEntry]) -> Result<()> {
        if entries.is_empty() {
            return Ok(());
        }
        let dim = self.space.dimension as usize;
        if let Some(bad) = entries.iter().find(|e| e.embedding.len() != dim) {
            return Err(Error::EmbeddingMismatch(format!(
                "embedding of chunk {} has length {}, the index expects {dim}; nothing was added",
                bad.chunk.id,
                bad.embedding.len()
            )));
        }
        if let Some(bad) = entries.iter().find(|e| !usable_vector(&e.embedding)) {
            return Err(Error::InvalidInput(format!(
                "embedding of chunk {} is zero or has NaN or infinite values; nothing was added",
                bad.chunk.id
            )));
        }
        // A chunk id given twice in one call: the last entry wins.
        let mut seen = HashSet::new();
        let mut unique: Vec<IndexEntry> = entries
            .iter()
            .rev()
            .filter(|e| seen.insert(e.chunk.id.clone()))
            .cloned()
            .collect();
        unique.reverse();
        let analyzer = self.analyzer.clone();
        let dimension = self.space.dimension;
        let batch = tokio::task::spawn_blocking(move || {
            let rows = unique
                .into_iter()
                .map(|e| {
                    let analysis = Analysis::for_language(e.chunk.language.as_deref());
                    let tokens = analyzer.text_tokens(&e.chunk.text, analysis).join(" ");
                    (e, tokens)
                })
                .collect::<Vec<_>>();
            build_batch(dimension, rows)
        })
        .await
        .map_err(|e| Error::Index(format!("text analysis failed: {e}")))??;
        let schema = batch.schema();
        let reader = Box::new(RecordBatchIterator::new(vec![Ok(batch)], schema));

        let mut state = self.write.lock().await;
        let mut merge = self.table.merge_insert(&[COL_ID]);
        merge
            .when_matched_update_all(None)
            .when_not_matched_insert_all();
        merge
            .execute(reader)
            .await
            .map_err(|e| self.err("cannot add entries", e))?;
        if !self.has_text_index.load(Ordering::SeqCst) {
            self.table
                .create_index(
                    &[COL_TOKENS],
                    lancedb::index::Index::FTS(text_index_params()),
                )
                .execute()
                .await
                .map_err(|e| self.err("cannot create the full-text index", e))?;
            self.has_text_index.store(true, Ordering::SeqCst);
        }
        self.maintain_if_due(&mut state).await
    }

    async fn delete_document(&self, id: &DocumentId) -> Result<()> {
        let mut state = self.write.lock().await;
        let predicate = col(COL_DOCUMENT).eq(lit(id.0.clone()));
        self.table
            .delete(&predicate)
            .await
            .map_err(|e| self.err("cannot delete document", e))?;
        // LanceDB adds a version even when nothing matched, so this counts as well.
        self.maintain_if_due(&mut state).await
    }

    async fn search(&self, request: &SearchRequest, embedding: &[f32]) -> Result<Vec<SearchHit>> {
        if request.text.trim().is_empty() {
            return Err(Error::InvalidInput("empty query text".into()));
        }
        if request.limit == 0 {
            return Err(Error::InvalidInput("limit 0".into()));
        }
        let dim = self.space.dimension as usize;
        if request.mode != SearchMode::FullText && embedding.len() != dim {
            return Err(Error::EmbeddingMismatch(format!(
                "query embedding has length {}, the index expects {dim}",
                embedding.len()
            )));
        }
        if request.mode != SearchMode::FullText && !usable_vector(embedding) {
            return Err(Error::InvalidInput(
                "query embedding is zero or has NaN or infinite values".into(),
            ));
        }
        if request.documents.as_ref().is_some_and(|d| d.is_empty()) {
            return Ok(Vec::new());
        }
        let total = self.row_count().await?;
        if total == 0 {
            return Ok(Vec::new());
        }
        let limit = usize::try_from(request.limit)
            .unwrap_or(usize::MAX)
            .min(total);
        let filter = Self::filter(&request.documents);
        let hits = match request.mode {
            SearchMode::Vector => self
                .vector_ranked(embedding, filter, limit, total)
                .await?
                .into_iter()
                .take(limit)
                .map(|r| SearchHit {
                    chunk: r.chunk,
                    retrieval_score: r.cosine,
                    vector_score: Some(r.cosine),
                    rerank_score: None,
                })
                .collect(),
            SearchMode::FullText => self
                .text_ranked(&request.text, None, filter, limit, total)
                .await?
                .into_iter()
                .take(limit)
                .map(|r| SearchHit {
                    chunk: r.chunk,
                    retrieval_score: r.text_score,
                    vector_score: None,
                    rerank_score: None,
                })
                .collect(),
            SearchMode::Hybrid => {
                let depth = limit.saturating_mul(2).max(MIN_FUSION_DEPTH).min(total);
                let vector = self
                    .vector_ranked(embedding, filter.clone(), depth, total)
                    .await?;
                let text = self
                    .text_ranked(&request.text, Some(embedding), filter, depth, total)
                    .await?;
                let vector_ids: Vec<String> = vector.iter().map(|r| r.id.clone()).collect();
                let text_ids: Vec<String> = text.iter().map(|r| r.id.clone()).collect();
                let fused = fusion::fuse(f64::from(self.vector_weight), &vector_ids, &text_ids);
                let mut by_id: HashMap<String, Row> = HashMap::new();
                for r in text.into_iter().chain(vector) {
                    by_id.insert(r.id.clone(), r);
                }
                fused
                    .into_iter()
                    .take(limit)
                    .filter_map(|(id, score)| {
                        by_id.remove(&id).map(|r| SearchHit {
                            chunk: r.chunk,
                            retrieval_score: score as f32,
                            vector_score: Some(r.cosine),
                            rerank_score: None,
                        })
                    })
                    .collect()
            }
        };
        Ok(hits)
    }
}
