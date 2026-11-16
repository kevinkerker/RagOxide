//! Chunks, embeddings and search results.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::document::{DocumentId, Provenance};

/// Identifier of a chunk, unique inside one knowledge base.
///
/// Always built with [`ChunkId::new`], which yields `"<document id>#<ordinal>"`.
/// Other components treat it as opaque.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct ChunkId(pub String);

impl ChunkId {
    /// Builds the id of the chunk at position `ordinal` (0-based, reading order)
    /// within `document`.
    pub fn new(document: &DocumentId, ordinal: u32) -> Self {
        Self(format!("{document}#{ordinal}"))
    }

    /// Returns the identifier as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ChunkId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A retrievable piece of a document with complete provenance.
///
/// Produced by the [`crate::Chunker`]; embedded, indexed, reranked, given to the answer
/// model as a numbered source and returned inside hits and citations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct Chunk {
    /// `ChunkId::new(&document_id, ordinal)`, ordinals 0, 1, 2, ... in reading order.
    pub id: ChunkId,
    /// Content text, not empty. This exact text is indexed, shown and quoted.
    /// It contains neither the heading path nor any model prefix.
    pub text: String,
    /// Language of the text as ISO 639-1 code, `None` if unknown. Defaults to the
    /// document's language. The index chooses its stemmer by it and uses its default
    /// analyser for `None`.
    pub language: Option<String>,
    /// File, heading path and anchors of the content: the anchors of all elements (or,
    /// for a part of a split table, of all rows) the chunk was built from.
    pub provenance: Provenance,
}

impl Chunk {
    /// Document the chunk belongs to (`provenance.source.document_id`).
    pub fn document_id(&self) -> &DocumentId {
        &self.provenance.source.document_id
    }
}

/// Embedding vector: `f32`, L2-normalised, length [`EmbeddingSpace::dimension`].
pub type Embedding = Vec<f32>;

/// The model and dimension that produced a set of embeddings.
///
/// Vectors are only comparable within one space. An [`crate::Index`] is bound to the
/// space it was built with.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct EmbeddingSpace {
    /// `id` of the embedding model's entry in `models.toml`.
    pub model_id: String,
    /// Vector length after Matryoshka truncation, from the same entry.
    pub dimension: u32,
}

/// Search strategy of an [`crate::Index`] query.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(rename_all = "snake_case")]
pub enum SearchMode {
    /// Vector and full-text search fused by reciprocal rank fusion. The default.
    #[default]
    Hybrid,
    /// Vector search only. Used to measure hybrid against single searches.
    Vector,
    /// Full-text search only. Used to measure hybrid against single searches.
    FullText,
}

/// A search query.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct SearchRequest {
    /// Query text as typed by the user, not empty.
    pub text: String,
    /// Maximum number of hits to return, at least 1.
    pub limit: u32,
    /// Restricts the search to these documents. `None` means no filter; `Some` with an
    /// empty list matches no document and yields no hits.
    #[serde(default)]
    pub documents: Option<Vec<DocumentId>>,
    /// Search strategy.
    #[serde(default)]
    pub mode: SearchMode,
}

impl SearchRequest {
    /// Hybrid search over all documents.
    pub fn new(text: impl Into<String>, limit: u32) -> Self {
        Self {
            text: text.into(),
            limit,
            documents: None,
            mode: SearchMode::Hybrid,
        }
    }
}

/// A chunk found by a search, with its scores.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct SearchHit {
    /// The chunk that was found, including its provenance.
    pub chunk: Chunk,
    /// Score assigned by the [`crate::Index`] that orders its result list: higher is
    /// better, only comparable within one list (fusion score in hybrid mode, cosine
    /// similarity in vector mode, full-text score in full-text mode).
    pub retrieval_score: f32,
    /// Cosine similarity of query and chunk embedding in `[-1, 1]`. Set for every hit
    /// in vector and hybrid mode (also for hits found only by the full-text part),
    /// `None` in full-text mode.
    pub vector_score: Option<f32>,
    /// Score assigned by the [`crate::Reranker`] in `[0, 1]`, `None` if the hit was
    /// not reranked. If set, result lists are ordered by it.
    pub rerank_score: Option<f32>,
}

/// A chunk together with its embedding, as written into the [`crate::Index`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct IndexEntry {
    /// The chunk.
    pub chunk: Chunk,
    /// Its document embedding from [`crate::Embedder::embed_documents`].
    pub embedding: Embedding,
}
