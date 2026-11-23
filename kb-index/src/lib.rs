//! Vector and full-text search with rank fusion (package E, implements
//! [`kb_core::Index`]).
//!
//! [`LanceIndex`] keeps the chunks of a knowledge base with their embeddings in one
//! LanceDB table in a local folder (ADR 0129): exact vector search by cosine
//! similarity, full-text search over tokens of the own analysis in [`analysis`]
//! (ADR 0130) and weighted reciprocal rank fusion of both ([`fusion`]). The README
//! of the crate describes storage, analysis, fusion and the measurements.

pub mod analysis;
pub mod fusion;
mod index;
pub mod wordlist;

pub use index::{FORMAT_VERSION, LanceIndex, MAINTENANCE_EVERY, META_FILE, MIN_FUSION_DEPTH};
