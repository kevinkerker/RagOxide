//! Local answer model via the bundled `llama-server` (package F): [`LlamaGenerator`]
//! implements [`kb_core::Generator`].
//!
//! The generator starts and watches one `llama-server` process, loads the answer model
//! of the requested level (from [`kb_core::ModelManifest::answer_model`], with its
//! drafter where the manifest names one for the hardware tier), streams phases and text,
//! cancels at once and catches crashes. The answer model counts tokens through
//! [`kb_core::Generator::count_tokens`]; this package has no `TokenCounter`. Only the
//! CPU is used (`-ngl 0`, ADR 0160), reasoning is off (ADR 0169), and `Balanced` and
//! `Thorough` are unloaded after an idle time (ADR 0170).
//!
//! Depends only on `kb-core` from the workspace. See `README.md` for the switches of the
//! server, the mapping onto phases and the known limits.

mod config;
mod generator;
mod manager;
mod protocol;
mod server;
mod sse;
mod tail;

pub use config::{DEFAULT_IDLE_TIMEOUT, GeneratorConfig};
pub use generator::LlamaGenerator;
