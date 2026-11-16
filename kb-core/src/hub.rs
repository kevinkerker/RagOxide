//! Types of the model management (package D, `kb-hub`).

use std::path::PathBuf;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::config::{ModelEntry, ModelRole};

/// Result of the hardware detection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct HardwareInfo {
    /// Total physical RAM in bytes, as reported by the operating system. Input of
    /// [`HardwareTier::from_total_ram`](crate::HardwareTier::from_total_ram).
    pub total_ram_bytes: u64,
    /// Number of logical CPUs. Basis for worker and thread counts of the parser.
    pub logical_cpus: u32,
}

/// A model of the active tier, ready to use.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct ResolvedModel {
    /// The manifest entry in its form for the hardware tier
    /// ([`ModelEntry::on_tier`](crate::ModelEntry::on_tier)): below the minimum the
    /// fast model without its drafter.
    pub entry: ModelEntry,
    /// Absolute folder of the model; file `entry.files[i]` is at
    /// `dir.join(&entry.files[i].path)`. All files exist and match their checksum.
    pub dir: PathBuf,
}

/// A model file of a tier and level that is not ready: absent, not matching its checksum, or
/// without a checksum to verify it against.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct MissingModelFile {
    /// `id` of the model entry.
    pub model_id: String,
    /// Role of the model; tells whether ingestion or asking still waits for it (see
    /// [`ModelRole::is_needed_for_ingestion`]).
    pub role: ModelRole,
    /// `path` of the file within the model entry.
    pub file: String,
}

/// Where models are stored, for the first-run setup.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct ModelStorage {
    /// Absolute model directory; model `id` lives in `<dir>/<id>`.
    pub dir: PathBuf,
    /// Free bytes on the volume of `dir`, for a warning before the download.
    pub free_bytes: u64,
}

/// State of one model file while models are provided.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ModelFileStatus {
    /// The checksum of a file found in the model folder or the offline package folder
    /// is being checked.
    Verifying,
    /// The file is being copied from an offline package folder.
    Copying {
        /// Bytes copied so far.
        done_bytes: u64,
        /// Total size if known.
        total_bytes: Option<u64>,
    },
    /// The file is being downloaded (resumed if a partial download exists).
    Downloading {
        /// Bytes on disk so far, including a resumed part.
        done_bytes: u64,
        /// Total size if known.
        total_bytes: Option<u64>,
    },
    /// Terminal: the file is present and its checksum matches.
    Ready,
}

/// Progress event while models are provided.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct ModelEvent {
    /// `id` of the model entry.
    pub model_id: String,
    /// Role of the model, see [`ModelRole::PROVIDE_ORDER`].
    pub role: ModelRole,
    /// `path` of the file within the model entry.
    pub file: String,
    /// New state of that file.
    pub status: ModelFileStatus,
}

/// Receives progress of [`ModelHub::provide`](crate::ModelHub::provide). Same rules as
/// [`ProgressCallback`](crate::ProgressCallback): called from worker tasks, must not block.
pub type ModelProgressCallback = Arc<dyn Fn(ModelEvent) + Send + Sync>;
