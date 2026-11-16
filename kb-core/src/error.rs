//! The single error type shared by all traits.

use serde::{Deserialize, Serialize};

/// Result type used by every fallible function of the contract.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Error returned by every trait of the contract.
///
/// Each implementing crate maps its internal errors onto the variant of its own
/// component (for example `kb-index` onto [`Error::Index`]) and puts the original
/// error text into the message. [`Error::Cancelled`], [`Error::NotFound`],
/// [`Error::UnsupportedFormat`] and [`Error::InvalidInput`] are shared by all
/// components, because callers react to them specifically.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The operation was stopped through its [`crate::CancellationToken`] or because
    /// its stream was dropped. Not a failure: callers must not report it as one.
    #[error("operation cancelled")]
    Cancelled,

    /// A referenced object (document, chat, model file) does not exist.
    /// The message names the object, e.g. `"document doc-123"`.
    #[error("not found: {0}")]
    NotFound(String),

    /// The file type is not supported by the parser. The message names the file.
    #[error("unsupported format: {0}")]
    UnsupportedFormat(String),

    /// The caller passed arguments that violate the contract, for example an empty
    /// query text or a `limit` of 0 (see the method docs). A vector of the wrong
    /// dimension is never `InvalidInput` but [`Error::EmbeddingMismatch`].
    #[error("invalid input: {0}")]
    InvalidInput(String),

    /// The parser failed on a file (corrupt, encrypted, worker process crashed).
    #[error("parser error: {0}")]
    Parse(String),

    /// The chunker failed, for example because its tokenizer could not be loaded.
    #[error("chunker error: {0}")]
    Chunk(String),

    /// Embedding, reranking or model management failed (missing model file, no
    /// checksum or download source in `models.toml`, ONNX Runtime error).
    #[error("model error: {0}")]
    Model(String),

    /// Embeddings and index belong to different embedding spaces (model or dimension):
    /// the index was built with another model, or a vector has the wrong length. The
    /// index has to be rebuilt by ingesting all documents again.
    #[error("embedding space mismatch: {0}")]
    EmbeddingMismatch(String),

    /// The search index failed.
    #[error("index error: {0}")]
    Index(String),

    /// A download failed because of the network (no connection, timeout, server error).
    #[error("network error: {0}")]
    Network(String),

    /// The disk holding the target folder is full.
    #[error("storage full: {0}")]
    StorageFull(String),

    /// A file does not match its SHA-256 from `models.toml` (after download or copy).
    #[error("checksum mismatch: {0}")]
    ChecksumMismatch(String),

    /// The answer model of the requested [`AnswerLevel`](crate::AnswerLevel) is offered on
    /// the hardware tier but not provided yet (ADR 0058). The question is not asked; the
    /// user interface offers the download of that level. The message names the level.
    #[error("answer model missing: {0}")]
    ModelMissing(String),

    /// The answer model failed (server not reachable, crashed, invalid response).
    #[error("generator error: {0}")]
    Generator(String),

    /// The persistent store failed (SQLite error, failed migration).
    #[error("store error: {0}")]
    Store(String),

    /// A configuration or model manifest is invalid.
    #[error("configuration error: {0}")]
    Config(String),

    /// An operating system I/O error that no component-specific variant describes better.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Machine-readable category of an [`Error`], one per variant.
///
/// Stored with failed files and documents and sent to the user interface, which shows
/// a translated text per kind; the technical message is kept next to it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(rename_all = "snake_case")]
pub enum ErrorKind {
    /// [`Error::Cancelled`].
    Cancelled,
    /// [`Error::NotFound`].
    NotFound,
    /// [`Error::UnsupportedFormat`].
    UnsupportedFormat,
    /// [`Error::InvalidInput`].
    InvalidInput,
    /// [`Error::Parse`].
    Parse,
    /// [`Error::Chunk`].
    Chunk,
    /// [`Error::Model`].
    Model,
    /// [`Error::EmbeddingMismatch`].
    EmbeddingMismatch,
    /// [`Error::Index`].
    Index,
    /// [`Error::Network`].
    Network,
    /// [`Error::StorageFull`].
    StorageFull,
    /// [`Error::ChecksumMismatch`].
    ChecksumMismatch,
    /// [`Error::ModelMissing`].
    ModelMissing,
    /// [`Error::Generator`].
    Generator,
    /// [`Error::Store`].
    Store,
    /// [`Error::Config`].
    Config,
    /// [`Error::Io`].
    Io,
}

impl Error {
    /// The category of this error.
    pub fn kind(&self) -> ErrorKind {
        match self {
            Self::Cancelled => ErrorKind::Cancelled,
            Self::NotFound(_) => ErrorKind::NotFound,
            Self::UnsupportedFormat(_) => ErrorKind::UnsupportedFormat,
            Self::InvalidInput(_) => ErrorKind::InvalidInput,
            Self::Parse(_) => ErrorKind::Parse,
            Self::Chunk(_) => ErrorKind::Chunk,
            Self::Model(_) => ErrorKind::Model,
            Self::EmbeddingMismatch(_) => ErrorKind::EmbeddingMismatch,
            Self::Index(_) => ErrorKind::Index,
            Self::Network(_) => ErrorKind::Network,
            Self::StorageFull(_) => ErrorKind::StorageFull,
            Self::ChecksumMismatch(_) => ErrorKind::ChecksumMismatch,
            Self::ModelMissing(_) => ErrorKind::ModelMissing,
            Self::Generator(_) => ErrorKind::Generator,
            Self::Store(_) => ErrorKind::Store,
            Self::Config(_) => ErrorKind::Config,
            Self::Io(_) => ErrorKind::Io,
        }
    }
}
