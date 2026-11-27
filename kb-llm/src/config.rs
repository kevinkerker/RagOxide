//! Construction parameters and the model files of a level.

use std::path::{Path, PathBuf};
use std::time::Duration;

use kb_core::{AnswerLevel, Error, HardwareTier, ModelManifest, Result, canonical_path};

/// Default time after which the model of `Balanced` or `Thorough` is unloaded when no
/// call uses it (ADR 0170).
pub const DEFAULT_IDLE_TIMEOUT: Duration = Duration::from_secs(5 * 60);

/// Everything [`crate::LlamaGenerator::new`] needs (F-F1).
///
/// The package reads no environment variable and knows no built-in path: the app passes
/// the server and the number of threads (package O, phase 2).
#[derive(Debug, Clone)]
pub struct GeneratorConfig {
    /// The model manifest (`models.toml`); answer models and drafters come from
    /// [`ModelManifest::answer_model`].
    pub manifest: ModelManifest,
    /// Folder holding the model files as `<model_dir>/<id>/<path>`. Any absolute form;
    /// a relative path is `Error::InvalidInput`.
    pub model_dir: PathBuf,
    /// Hardware tier of the machine: which levels exist and whether the drafter is used.
    pub tier: HardwareTier,
    /// Context size the server starts with (`PipelineConfig::context_tokens`); 0 is
    /// `Error::Config`.
    pub context_tokens: u32,
    /// Path of `llama-server.exe` in its folder of the release (the server needs the
    /// libraries next to it).
    pub server: PathBuf,
    /// Threads for generation and prompt processing (`-t`, `-tb`); 0 is `Error::Config`.
    pub threads: u32,
    /// Idle time after which the model of `Balanced` or `Thorough` is unloaded, counted
    /// from the end of the last call (ADR 0170). `Fast` stays loaded.
    pub idle_timeout: Duration,
}

impl GeneratorConfig {
    /// Configuration with the default idle time ([`DEFAULT_IDLE_TIMEOUT`]).
    pub fn new(
        manifest: ModelManifest,
        model_dir: impl Into<PathBuf>,
        tier: HardwareTier,
        context_tokens: u32,
        server: impl Into<PathBuf>,
        threads: u32,
    ) -> Self {
        Self {
            manifest,
            model_dir: model_dir.into(),
            tier,
            context_tokens,
            server: server.into(),
            threads,
            idle_timeout: DEFAULT_IDLE_TIMEOUT,
        }
    }
}

/// The checked configuration.
#[derive(Debug)]
pub(crate) struct Resolved {
    pub(crate) manifest: ModelManifest,
    pub(crate) model_dir: PathBuf,
    pub(crate) tier: HardwareTier,
    pub(crate) context_tokens: u32,
    pub(crate) server: PathBuf,
    pub(crate) threads: u32,
    pub(crate) idle_timeout: Duration,
}

/// Model and drafter of one level, as absolute paths in `canonical_path` form.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ModelFiles {
    pub(crate) model: PathBuf,
    pub(crate) draft: Option<PathBuf>,
}

impl Resolved {
    pub(crate) fn new(config: GeneratorConfig) -> Result<Self> {
        let model_dir = canonical_path(&config.model_dir)?;
        if config.context_tokens == 0 {
            return Err(Error::Config("context_tokens must not be 0".into()));
        }
        if config.threads == 0 {
            return Err(Error::Config("threads must not be 0".into()));
        }
        Ok(Self {
            manifest: config.manifest,
            model_dir,
            tier: config.tier,
            context_tokens: config.context_tokens,
            server: config.server,
            threads: config.threads,
            idle_timeout: config.idle_timeout,
        })
    }

    /// The files of `level` if the tier offers it and every file is present: only file
    /// metadata, no content, no process, no connection (contract of `check_level`).
    pub(crate) fn model_files(&self, level: AnswerLevel) -> Result<ModelFiles> {
        let Some(entry) = self.manifest.answer_model(self.tier, level) else {
            return Err(Error::InvalidInput(format!(
                "level {level:?} is not offered on tier {:?}",
                self.tier
            )));
        };
        let folder = self.model_dir.join(&entry.id);
        let mut model = None;
        let mut draft = None;
        for file in &entry.files {
            let path = join_relative(&folder, &file.path);
            if !path.is_file() {
                return Err(Error::ModelMissing(format!(
                    "level {level:?}: file {}/{} is missing",
                    entry.id, file.path
                )));
            }
            if entry.mtp_draft.as_deref() == Some(file.path.as_str()) {
                draft = Some(path);
            } else {
                model = Some(path);
            }
        }
        let Some(model) = model else {
            return Err(Error::Config(format!(
                "answer model {} has no model file",
                entry.id
            )));
        };
        Ok(ModelFiles {
            model: canonical_path(&model)?,
            draft: draft.map(|d| canonical_path(&d)).transpose()?,
        })
    }
}

/// `folder` joined with a manifest path whose parts are separated by `/`.
fn join_relative(folder: &Path, relative: &str) -> PathBuf {
    relative
        .split('/')
        .fold(folder.to_path_buf(), |path, part| path.join(part))
}
