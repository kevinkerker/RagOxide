//! Shared contract of the knowledge base.
//!
//! Every other crate of the workspace depends only on this crate. It defines
//!
//! - the data types that flow through the pipeline: [`Document`] (parsed file with
//!   [`Anchor`]s), [`Chunk`] with [`Provenance`], [`SearchHit`], [`Answer`] with
//!   [`Citation`]s,
//! - the component traits [`Parser`], [`Chunker`], [`TokenCounter`], [`Embedder`],
//!   [`Reranker`], [`Index`], [`Generator`], [`Store`], [`ModelHub`] and the pipeline
//!   traits [`Ingestor`] and [`Answerer`],
//! - the facade [`KnowledgeBase`] used by the app, the MCP server, the CLI and the
//!   evaluation,
//! - the [`Error`] type, user [`Settings`], [`PipelineConfig`], the [`ModelManifest`]
//!   read from `models.toml`, and the canonical path form ([`canonical_path`],
//!   [`path_key`]),
//! - with the `mock` feature: one mock per trait and JSON fixtures in the module
//!   `mock`; with the `ts` feature: `ts_rs::TS` on every serialised type.
//!
//! Async traits use `async-trait` so that they work as `Arc<dyn Trait>`. Streaming uses
//! [`futures::stream::BoxStream`], cancellation uses [`CancellationToken`], both
//! re-exported here so that all crates use the same versions.

mod answer;
mod chunk;
mod config;
mod document;
mod error;
mod hub;
mod ingest;
mod knowledge_base;
mod time;
mod traits;

#[cfg(feature = "mock")]
pub mod mock;

pub use answer::{
    Answer, AnswerEvent, AnswerOutcome, AnswerPhase, Chat, ChatId, ChatSummary, ChatTurn, Citation,
    GenerationEvent, GenerationRequest, Message, PhaseProgress, Role,
};
pub use chunk::{
    Chunk, ChunkId, Embedding, EmbeddingSpace, IndexEntry, SearchHit, SearchMode, SearchRequest,
};
pub use config::{
    AnswerLevel, ColorScheme, FileSource, HardwareTier, HuggingFaceSource, ModelEntry, ModelFile,
    ModelManifest, ModelRole, PipelineConfig, ReleaseSource, RuntimeEntry, RuntimeFile, Settings,
    UiLanguage,
};
pub use document::{
    Anchor, BoundingBox, CellRange, Document, DocumentFormat, DocumentId, Element, ElementKind,
    Page, Provenance, SourceFile, Table, TableCell, canonical_path, content_hash, path_key,
};
pub use error::{Error, ErrorKind, Result};
pub use hub::{
    HardwareInfo, MissingModelFile, ModelEvent, ModelFileStatus, ModelProgressCallback,
    ModelStorage, ResolvedModel,
};
pub use ingest::{
    DocumentRecord, DocumentStatus, FileResult, FileStatus, IngestEvent, ProgressCallback,
    ScopeCallback, ScopeEvent, SyncMode,
};
pub use knowledge_base::KnowledgeBase;
pub use time::{Clock, SystemClock, Timestamp};
pub use traits::{
    AnswerStream, Answerer, Chunker, Embedder, GenerationStream, Generator, Index, Ingestor,
    ModelHub, Parser, Reranker, SettingsUpdate, Store, TokenCounter,
};

pub use async_trait::async_trait;
pub use futures::stream::BoxStream;
pub use tokio_util::sync::CancellationToken;
