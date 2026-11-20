//! Parser of the knowledge base (package A): turns a source file into a
//! [`kb_core::Document`] with elements in reading order and one anchor per format.
//!
//! [`DoclingParser`] implements [`kb_core::Parser`] over docling.rs (the fork pinned in the
//! workspace, ADR 0011, ADR 0032). Every conversion runs in its own process, the worker
//! program `kb-parse-worker` of this package, so that a crash or a runaway conversion does
//! not take the caller down and a cancel can stop it at once. The worker maps the docling
//! JSON and conversion report onto the contract types; the parser starts it, waits for it
//! without blocking the executor and kills it on cancel.
//!
//! The parser models (layout, table structure, text detection and recognition) come from
//! `models.toml` via [`kb_core::ModelManifest::select`] and the model directory; the
//! library names no model or model file itself. See `README.md` for the mapping per
//! format and the rules of the package.

mod convert;
mod parser;
pub mod worker;

pub use convert::PageOrigin;
pub use parser::{
    DoclingParser, ParserConfig, TracedDocument, worker_file_name, worker_next_to_executable,
};
