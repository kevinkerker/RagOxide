//! Configuration: user settings, pipeline parameters and the shared model manifest.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::document::{canonical_path, key_within as within, path_key};
use crate::error::{Error, Result};

const GIB: u64 = 1024 * 1024 * 1024;

/// Hardware tier: which models and answer levels a machine gets (ADR 0056, ADR 0050,
/// ADR 0080).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(rename_all = "snake_case")]
pub enum HardwareTier {
    /// At least the minimum requirement of 16 GB RAM: the supported and tested tier, all
    /// answer levels, with reranker.
    Supported,
    /// Below the minimum requirement: the setup warns and allows the attempt, with the
    /// fast answer level only and without reranker (which models a tier gets is fixed
    /// in `models.toml`). Target values do not apply.
    BelowMinimum,
}

impl HardwareTier {
    /// Minimum total physical RAM for [`HardwareTier::Supported`]: 15 GiB, so that a
    /// nominal 16 GB machine with memory reserved by firmware or graphics still counts.
    /// Provisional until the measurement on the reference device (ADR 0017).
    pub const MIN_TOTAL_BYTES: u64 = 15 * GIB;

    /// The tier of a machine with `total_ram_bytes` of physical RAM. There is no lower
    /// limit and no setting that overrides it: below the minimum the attempt is allowed
    /// (ADR 0050, ADR 0080).
    pub fn from_total_ram(total_ram_bytes: u64) -> Self {
        if total_ram_bytes >= Self::MIN_TOTAL_BYTES {
            Self::Supported
        } else {
            Self::BelowMinimum
        }
    }
}

/// Level of the answer model, chosen by the user per question (ADR 0055, ADR 0060). The
/// names follow ADR 0067: fast, balanced, thorough (shown as Schnell, Ausgewogen,
/// Gründlich).
///
/// Each level has its own answer model in `models.toml`; only one answer model is loaded
/// at a time. Which levels a hardware tier offers is fixed there too
/// ([`ModelManifest::levels`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(rename_all = "snake_case")]
pub enum AnswerLevel {
    /// Fast: the smallest model, part of the base equipment. The default.
    #[default]
    Fast,
    /// Balanced: a larger model, provided on demand.
    Balanced,
    /// Thorough: the largest model, provided on demand; answers take minutes.
    Thorough,
}

impl AnswerLevel {
    /// All levels, from fast to thorough.
    pub const ALL: [AnswerLevel; 3] = [Self::Fast, Self::Balanced, Self::Thorough];
}

/// Language of the user interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum UiLanguage {
    /// German.
    #[default]
    #[serde(rename = "de")]
    German,
    /// English.
    #[serde(rename = "en")]
    English,
}

/// Colour scheme of the user interface (ADR 0024).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(rename_all = "snake_case")]
pub enum ColorScheme {
    /// Follow the operating system. The default.
    #[default]
    System,
    /// Always light.
    Light,
    /// Always dark.
    Dark,
}

/// User settings, persisted by the [`crate::Store`] and edited in the settings view.
///
/// Contains no secrets: the answer model always runs locally (ADR 0015).
///
/// The library scope (ADR 0025) is `watched_folders` minus `excluded`: a file belongs
/// to the library if [`Settings::in_scope`] holds and its format is supported. Both
/// lists hold canonical paths without duplicates. The user changes them through
/// [`Settings::add_folder`], [`Settings::remove_folder`], [`Settings::exclude`] and
/// [`Settings::include`] (via [`KnowledgeBase`](crate::KnowledgeBase)), which keep these
/// invariants; the [`Ingestor`](crate::Ingestor) only drops stale exclusions. There
/// are no patterns, only concrete paths. A watched folder must be a directory; a path
/// that is missing, a file or unreadable counts as unavailable in a synchronisation.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(default)]
pub struct Settings {
    /// Watched folders. Nesting is allowed: a folder inside another watched folder
    /// changes nothing about the files below it, but keeps them in scope when the outer
    /// folder is deregistered.
    pub watched_folders: Vec<PathBuf>,
    /// Excluded files and subfolders, each strictly inside a watched folder. An
    /// excluded subfolder excludes everything below it. Shown in the library so that
    /// the user can take entries off the list again.
    pub excluded: Vec<PathBuf>,
    /// The answer level chosen last; the user interface preselects it for the next
    /// question and writes it back when the user changes it (ADR 0060). Default fast.
    /// If the hardware tier does not offer it, fast applies without changing this
    /// field, see [`ModelManifest::effective_level`].
    pub answer_level: AnswerLevel,
    /// Language of the user interface and of fixed texts in answers.
    pub ui_language: UiLanguage,
    /// Colour scheme of the user interface.
    pub color_scheme: ColorScheme,
}

fn keys(paths: &[PathBuf]) -> Vec<String> {
    paths.iter().filter_map(|p| path_key(p).ok()).collect()
}

impl Settings {
    /// `true` if `path` lies strictly below a watched folder and is not excluded. A
    /// watched folder itself is never in scope, so a file registered as a folder keeps
    /// no document in the library.
    pub fn in_scope(&self, path: &Path) -> bool {
        let Ok(key) = path_key(path) else {
            return false;
        };
        keys(&self.watched_folders)
            .iter()
            .any(|f| within(&key, f) && key != *f)
            && !keys(&self.excluded).iter().any(|e| within(&key, e))
    }

    /// Registers a watched folder. The path must be absolute (`InvalidInput`
    /// otherwise); it need not exist yet (a network drive may be offline). A folder
    /// already on the list (equal [`path_key`]) changes nothing. An exclusion equal to
    /// the folder is lifted, because registering it explicitly brings it into scope. A
    /// folder strictly below an exclusion is `InvalidInput`: the user takes the
    /// exclusion off first, so that its other contents do not come back unnoticed.
    pub fn add_folder(&mut self, folder: &Path) -> Result<()> {
        let folder = canonical_path(folder)?;
        let key = path_key(&folder)?;
        if keys(&self.watched_folders).contains(&key) {
            return Ok(());
        }
        if keys(&self.excluded)
            .iter()
            .any(|e| within(&key, e) && key != *e)
        {
            return Err(Error::InvalidInput(format!(
                "{} lies in an excluded folder; take the exclusion off first",
                folder.display()
            )));
        }
        self.excluded
            .retain(|e| path_key(e).is_ok_and(|e| e != key));
        self.watched_folders.push(folder);
        Ok(())
    }

    /// Deregisters a watched folder (no change if it is not on the list). Exclusions
    /// that no longer lie in any watched folder are dropped with it.
    pub fn remove_folder(&mut self, folder: &Path) -> Result<()> {
        let key = path_key(folder)?;
        self.watched_folders
            .retain(|f| path_key(f).is_ok_and(|f| f != key));
        let folders = keys(&self.watched_folders);
        self.excluded.retain(|e| {
            path_key(e).is_ok_and(|e| folders.iter().any(|f| within(&e, f) && e != *f))
        });
        Ok(())
    }

    /// Excludes a file or subfolder. It must lie strictly inside a watched folder:
    /// a path outside every watched folder, or equal to a watched folder (deregister it
    /// instead), is `InvalidInput`. A path already covered by an exclusion changes
    /// nothing; exclusions below the new one are dropped as covered by it. A file with
    /// an unsupported extension may be excluded; the exclusion has no effect until the
    /// format is supported (ADR 0085).
    pub fn exclude(&mut self, path: &Path) -> Result<()> {
        let path = canonical_path(path)?;
        let key = path_key(&path)?;
        let folders = keys(&self.watched_folders);
        if folders.contains(&key) {
            return Err(Error::InvalidInput(format!(
                "{} is a watched folder; deregister it instead of excluding it",
                path.display()
            )));
        }
        if !folders.iter().any(|f| within(&key, f)) {
            return Err(Error::InvalidInput(format!(
                "{} is not inside a watched folder",
                path.display()
            )));
        }
        if keys(&self.excluded).iter().any(|e| within(&key, e)) {
            return Ok(());
        }
        self.excluded
            .retain(|e| path_key(e).is_ok_and(|e| !within(&e, &key)));
        self.excluded.push(path);
        Ok(())
    }

    /// Takes an entry off the exclusion list (no change if it is not on the list). The
    /// file or subfolder is ingested again at the next synchronisation.
    pub fn include(&mut self, path: &Path) -> Result<()> {
        let key = path_key(path)?;
        self.excluded
            .retain(|e| path_key(e).is_ok_and(|e| e != key));
        Ok(())
    }
}

/// Tuning parameters of the pipeline.
///
/// Not a user setting: the values are fixed in code and tuned in Phase 2 with
/// `kb-eval`. The defaults are provisional starting points.
///
/// How the defaults fit together (ADR 0081, ADR 0082, `docs/analyse/tempomessung.md`):
///
/// - Answer prompt: eight sources of at most 400 tokens (3200), the source headers
///   (`[n] file | heading path`, about 50 to 80 tokens each, 400 to 640), the system
///   instruction (about 100) and the question with chat template (about 50) need about
///   3750 to 4000 tokens and use up `prompt_token_budget` 4000. If not everything fits
///   (longer headers or chunks), the answerer drops the trailing sources: from the
///   first source that does not fit on, all are left out (see
///   [`AnswerEvent::Sources`](crate::AnswerEvent)). The chat history is not part of the
///   answer prompt (ADR 0100); it is only used to rewrite a follow-up question and has
///   its own limit, `rewrite_history_tokens`. The chunker counts with
///   the embedding model's tokenizer, the budget with the answer model's, so the 400 are
///   approximate. A prompt of about 3500 tokens took 14.5 s to the first token with the
///   fast model on the development machine (`tempomessung.md`).
/// - Context: `prompt_token_budget + max_answer_tokens` = 4000 + 1024 = 5024, rounded
///   up to `context_tokens` 5120 (20 × 256).
/// - Rewriting call: at most `rewrite_history_tokens` 1000 of history plus
///   [`PipelineConfig::REWRITE_FIXED_TOKENS`] 150 (instruction, question, template) and
///   `max_answer_tokens` 1024 make 2174, which also fits into 5120.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(default)]
pub struct PipelineConfig {
    /// Upper bound of tokens per chunk, counted over [`crate::Chunk::text`] with the
    /// [`crate::TokenCounter`] of the embedding model. Default 400, provisional: the
    /// chunk size the speed measurement assumes.
    pub chunk_max_tokens: u32,
    /// Weight of the vector list in the rank fusion of hybrid search, in `[0, 1]`;
    /// the full-text list gets `1 - vector_weight`. Default 0.5.
    pub vector_weight: f32,
    /// Number of index hits passed to the reranker. Default 20, provisional (ADR 0078:
    /// about 0.22 s per candidate, sources list within about 10 s).
    pub rerank_candidates: u32,
    /// Maximum number of numbered sources in the prompt. Default 8, provisional
    /// (ADR 0082); trailing sources that do not fit `prompt_token_budget` are dropped.
    pub max_sources: u32,
    /// With reranker: minimum `rerank_score` for a hit to be used as a source.
    /// Default 0.1.
    pub min_relevance: f32,
    /// Without reranker: minimum `vector_score` for a hit to be used as a source.
    /// Default 0.3, provisional: with EmbeddingGemma 2 it filters nothing (every cosine
    /// in the test was above 0.6); Phase 2 sets it with the question catalogue.
    pub min_similarity: f32,
    /// Upper bound of answer prompt tokens (instruction, sources, question), counted
    /// with [`Generator::count_tokens`](crate::Generator::count_tokens) for the level of
    /// the question. The same for all levels. Default 4000, provisional (ADR 0081):
    /// eight full sources with their headers, instruction and question need about 3750
    /// to 4000.
    pub prompt_token_budget: u32,
    /// Upper bound of generated tokens per answer. The [`crate::Answerer`] always sets
    /// it as [`GenerationRequest::max_tokens`](crate::GenerationRequest). Default 1024.
    pub max_answer_tokens: u32,
    /// Context size of the answer model in tokens. `kb-llm` starts the model of every
    /// level with exactly this value (`llama-server --ctx-size`). Must hold the answer
    /// call and the rewriting call, see [`PipelineConfig::validate`]. Default 5120
    /// (4000 + 1024, rounded up to a multiple of 256), provisional.
    pub context_tokens: u32,
    /// Upper bound of history tokens used to rewrite a follow-up question, counted like
    /// the prompt; how history is cut is fixed at [`crate::Answerer::ask`]. Default
    /// 1000, provisional.
    pub rewrite_history_tokens: u32,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            chunk_max_tokens: 400,
            vector_weight: 0.5,
            rerank_candidates: 20,
            max_sources: 8,
            min_relevance: 0.1,
            min_similarity: 0.3,
            prompt_token_budget: 4000,
            max_answer_tokens: 1024,
            context_tokens: 5120,
            rewrite_history_tokens: 1000,
        }
    }
}

impl PipelineConfig {
    /// Tokens of the rewriting call besides the history: instruction, question and chat
    /// template. An estimate, provisional; [`PipelineConfig::validate`] reserves it.
    pub const REWRITE_FIXED_TOKENS: u32 = 150;

    /// Checks that both calls fit into the context: the answer call
    /// (`prompt_token_budget + max_answer_tokens`) and the rewriting call
    /// (`rewrite_history_tokens + REWRITE_FIXED_TOKENS + max_answer_tokens`), and that
    /// the rewriting history is not larger than the prompt budget.
    pub fn validate(&self) -> Result<()> {
        if self.rewrite_history_tokens > self.prompt_token_budget {
            return Err(Error::Config(format!(
                "rewrite_history_tokens = {} exceeds prompt_token_budget = {}",
                self.rewrite_history_tokens, self.prompt_token_budget
            )));
        }
        let needed = u64::from(self.prompt_token_budget) + u64::from(self.max_answer_tokens);
        if needed > u64::from(self.context_tokens) {
            return Err(Error::Config(format!(
                "prompt_token_budget + max_answer_tokens = {needed} exceeds context_tokens = {}",
                self.context_tokens
            )));
        }
        let rewriting = u64::from(self.rewrite_history_tokens)
            + u64::from(Self::REWRITE_FIXED_TOKENS)
            + u64::from(self.max_answer_tokens);
        if rewriting > u64::from(self.context_tokens) {
            return Err(Error::Config(format!(
                "rewriting call ({rewriting} tokens with answer) exceeds context_tokens = {}",
                self.context_tokens
            )));
        }
        Ok(())
    }
}

/// Contents of the shared model file `models.toml` in the repository root.
///
/// Single place for model names, sources, checksums, tiers and levels and the versions
/// of the bundled runtimes; no crate hard-codes them (plan, rules for agents). Read by
/// `kb-hub` and `kb-llm`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(deny_unknown_fields)]
pub struct ModelManifest {
    /// Format version of the file, currently 4.
    pub schema_version: u32,
    /// Bundled runtimes (not downloaded, shipped with the installer).
    #[serde(default)]
    pub runtimes: Vec<RuntimeEntry>,
    /// All models as written in the file: answer models list their drafter for every
    /// hardware tier. For one hardware tier use [`ModelManifest::models_for`] and
    /// [`ModelManifest::answer_model`], which return the entries in their form for that
    /// tier ([`ModelEntry::on_tier`]).
    pub models: Vec<ModelEntry>,
}

/// A runtime shipped with the installer, pinned by release.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(deny_unknown_fields)]
pub struct RuntimeEntry {
    /// Stable key, e.g. `llama-server`.
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Pinned release. `None` is a placeholder: not pinned yet.
    pub version: Option<String>,
    /// SPDX license expression.
    pub license: String,
    /// Page of the pinned release that holds the files.
    pub source: String,
    /// Files of the release that the installer takes, with what is known about them.
    #[serde(default)]
    pub files: Vec<RuntimeFile>,
}

/// One file of a runtime release, or a file inside an archive of that release.
///
/// `path` follows the rules of [`ModelFile::path`] (names as in [`FileSource`]);
/// `(archive, path)` is unique within the runtime, also ignoring case. `archive` names a file of the same runtime that is
/// not itself in an archive, and a file in an archive has no source of its own. Unknown
/// keys are an error.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(deny_unknown_fields)]
pub struct RuntimeFile {
    /// Asset name in the release, or with `archive` the path inside that archive.
    pub path: String,
    /// Asset name of the archive that contains the file; `None` for a release asset.
    #[serde(default)]
    pub archive: Option<String>,
    /// Where a release asset is downloaded from (package O); `None` for a file inside
    /// an archive and while open.
    #[serde(default)]
    pub source: Option<FileSource>,
    /// SHA-256, 64 lowercase hex digits. `None` is a placeholder: not verified yet.
    pub sha256: Option<String>,
    /// Size in bytes. `None` if unknown.
    pub size_bytes: Option<u64>,
}

/// What a model is used for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(rename_all = "snake_case")]
pub enum ModelRole {
    /// Text embedding for vector search.
    Embedding,
    /// Cross-encoder or LLM reranker.
    Reranker,
    /// Answer model of one [`AnswerLevel`], run by `llama-server`.
    Generator,
    /// Parser: layout detection.
    Layout,
    /// Parser: table structure recognition.
    TableStructure,
    /// Parser: OCR text line detection.
    OcrDetection,
    /// Parser: OCR text recognition.
    OcrRecognition,
}

impl ModelRole {
    /// Order in which [`ModelHub::provide`](crate::ModelHub::provide) makes the roles
    /// ready: first what ingestion needs (parser models, embedding), then the reranker,
    /// last the answer models. So ingestion can start while an answer model is still
    /// downloading (ADR 0036, ADR 0048).
    pub const PROVIDE_ORDER: [ModelRole; 7] = [
        ModelRole::Layout,
        ModelRole::TableStructure,
        ModelRole::OcrDetection,
        ModelRole::OcrRecognition,
        ModelRole::Embedding,
        ModelRole::Reranker,
        ModelRole::Generator,
    ];

    /// `true` for the roles ingestion needs: parser models and embedding. Ingestion can
    /// start once no file of these roles is missing.
    pub fn is_needed_for_ingestion(self) -> bool {
        !matches!(self, ModelRole::Reranker | ModelRole::Generator)
    }
}

/// One model with its files.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(deny_unknown_fields)]
pub struct ModelEntry {
    /// Stable key, also the name of the model's folder below the model directory.
    pub id: String,
    /// Human-readable name including format and quantisation.
    pub name: String,
    /// Use of the model.
    pub role: ModelRole,
    /// Answer models only: the level the model serves. Required for
    /// [`ModelRole::Generator`], absent otherwise.
    #[serde(default)]
    pub level: Option<AnswerLevel>,
    /// Tiers that use this model, at least one. A tier without a model of a role runs
    /// without it; this is allowed for [`ModelRole::Reranker`] only. A tier offers the
    /// levels that have an answer model for it, the fast level always.
    pub tiers: Vec<HardwareTier>,
    /// 1 for the planned model, 2 for the first fallback, 3 for the second. For each
    /// role, tier and (answer models) level the entry with the lowest value is used.
    pub preference: u8,
    /// Embedding models only: vector length after Matryoshka truncation, which fixes
    /// the [`crate::EmbeddingSpace`]. Required for embedding entries, absent otherwise.
    #[serde(default)]
    pub dimension: Option<u32>,
    /// Answer models only: `path` of the file in `files` that is the drafter for
    /// multi-token prediction. On the tiers in `mtp_tiers` `llama-server` is started
    /// with it (MTP on); this can only be switched when the server starts, not per
    /// request (ADR 0063). An answer model has exactly one file besides the drafter: the
    /// model itself.
    #[serde(default)]
    pub mtp_draft: Option<String>,
    /// The hardware tiers on which the drafter is used, a non-empty subset of `tiers`;
    /// required with `mtp_draft`, empty without. On the other tiers the model runs
    /// without it, and the drafter is neither provided nor loaded there (ADR 0084). See
    /// [`ModelEntry::on_tier`].
    #[serde(default)]
    pub mtp_tiers: Vec<HardwareTier>,
    /// SPDX license expression of the model weights.
    pub license: String,
    /// Upstream page of the model.
    pub source: String,
    /// Files of the model, at least one.
    pub files: Vec<ModelFile>,
}

/// Where a file is downloaded from: exactly one place, without a fallback (ADR 0077).
///
/// A field that is `None` is a placeholder: the source is not fixed yet, the file has no
/// address ([`FileSource::url`] is `None`), and `kb-hub` cannot download it.
///
/// [`ModelManifest::validate`] checks the fields that are set: `repo` is `owner/name`,
/// two parts; `revision` is a full commit hash; `tag` and `asset` are single names.
///
/// A name (also an id or a path part) is a valid file name on Windows: not empty, at
/// most 100 characters, contains no `..` and does not end with `.`; it has no whitespace,
/// no control characters, none of the format characters U+00AD, U+061C, U+180E,
/// U+200B to U+200F, U+202A to U+202E, U+2060 to U+2064, U+2066 to U+206F, U+FEFF and
/// U+FFF9 to U+FFFB (other invisible characters are not checked), and none of
/// `/ \ : ? # % < > | * "`; and it is no reserved device name (`CON`, `PRN`, `AUX`,
/// `NUL`, `COM1` to `COM9`, `LPT1` to `LPT9`, `CONIN$`, `CONOUT$`, also with an
/// extension such as `LPT1.onnx`). Unknown keys are an error, so a misspelt key cannot
/// leave a file open unnoticed. Every complete address occurs only once in the
/// manifest, comparing repositories without regard to case.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FileSource {
    /// An unchanged file from a Hugging Face repository; its name in the repository is
    /// the file's `path`.
    HuggingFace(HuggingFaceSource),
    /// A file made or changed by the project, an asset of a GitHub release with a fixed
    /// tag.
    GithubRelease(ReleaseSource),
}

/// The fields of [`FileSource::HuggingFace`]. Unknown keys are an error.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(deny_unknown_fields)]
pub struct HuggingFaceSource {
    /// Repository, `owner/name`.
    #[serde(default)]
    pub repo: Option<String>,
    /// Full commit hash (40 lowercase hex digits).
    #[serde(default)]
    pub revision: Option<String>,
}

/// The fields of [`FileSource::GithubRelease`]. Unknown keys are an error.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(deny_unknown_fields)]
pub struct ReleaseSource {
    /// Repository, `owner/name`.
    #[serde(default)]
    pub repo: Option<String>,
    /// Fixed release tag.
    #[serde(default)]
    pub tag: Option<String>,
    /// Asset name in the release, a single name.
    pub asset: String,
}

impl FileSource {
    /// The address of the file at `path` (the path matters for Hugging Face only):
    /// `https://huggingface.co/<repo>/resolve/<revision>/<path>` or
    /// `https://github.com/<repo>/releases/download/<tag>/<asset>`. `None` while a part
    /// of the source is open.
    pub fn url(&self, path: &str) -> Option<String> {
        match self {
            Self::HuggingFace(HuggingFaceSource { repo, revision }) => Some(format!(
                "https://huggingface.co/{}/resolve/{}/{path}",
                repo.as_ref()?,
                revision.as_ref()?
            )),
            Self::GithubRelease(ReleaseSource { repo, tag, asset }) => Some(format!(
                "https://github.com/{}/releases/download/{}/{asset}",
                repo.as_ref()?,
                tag.as_ref()?
            )),
        }
    }

    /// The form of the fields that are set; open fields are fine.
    fn check(&self) -> std::result::Result<(), &'static str> {
        let repo = match self {
            Self::HuggingFace(HuggingFaceSource { repo, revision }) => {
                if revision.as_deref().is_some_and(|r| !is_hex(r, 40)) {
                    return Err("revision is not a full commit hash");
                }
                repo
            }
            Self::GithubRelease(ReleaseSource { repo, tag, asset }) => {
                if tag.as_deref().is_some_and(|t| !is_name(t)) {
                    return Err("tag is not a single name");
                }
                if !is_name(asset) {
                    return Err("asset is not a single name");
                }
                repo
            }
        };
        if repo.as_deref().is_some_and(|r| !is_repo(r)) {
            return Err("repo is not owner/name");
        }
        Ok(())
    }
}

/// One file of a model, downloaded from exactly one place (ADR 0077).
///
/// `path` is relative: parts joined by `/`, each a single name as in [`FileSource`],
/// at most 200 characters in all, so it cannot leave the model's folder (no leading
/// `/`, no drive, no `.` or `..`). The limits hold for each name and for this relative
/// path; that the full path including the model directory fits the system is the
/// concern of package D. Paths are unique within a model, also ignoring case, and no
/// path is the folder of another (`a` next to `a/b`). Unknown keys are an error.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(deny_unknown_fields)]
pub struct ModelFile {
    /// Path relative to the model's folder, with `/` as separator; for a Hugging Face
    /// file also its name in the repository.
    pub path: String,
    /// Where the file comes from. `None` is a placeholder: no source yet.
    #[serde(default)]
    pub source: Option<FileSource>,
    /// SHA-256 of the file, 64 lowercase hex digits. `None` is a placeholder: not
    /// verified yet. `kb-hub` must reject files without a checksum.
    pub sha256: Option<String>,
    /// Size in bytes, for showing the download size before the download and for
    /// progress. May be rounded while unverified (marked in `models.toml`); never used
    /// to verify a file, only `sha256` does that. `None` if unknown.
    pub size_bytes: Option<u64>,
}

impl ModelFile {
    /// The only download address, see [`FileSource::url`]. `None` while the source is
    /// missing or incomplete.
    pub fn url(&self) -> Option<String> {
        self.source.as_ref()?.url(&self.path)
    }
}

impl RuntimeFile {
    /// The download address of a release asset, see [`FileSource::url`]; `None` for a
    /// file inside an archive and while the source is open.
    pub fn url(&self) -> Option<String> {
        self.source.as_ref()?.url(&self.path)
    }
}

/// Longest name (id, path part, asset, tag), in characters.
const MAX_NAME_CHARS: usize = 100;

/// Longest path, in characters.
const MAX_PATH_CHARS: usize = 200;

/// Device names Windows reserves, also with an extension (`LPT1.onnx`).
const RESERVED_NAMES: [&str; 24] = [
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
    "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9", "CONIN$",
    "CONOUT$",
];

/// Invisible format characters (bidirectional controls, zero-width characters and the
/// like), which would make two names look equal.
fn is_format_char(c: char) -> bool {
    matches!(
        c,
        '\u{00AD}'
            | '\u{061C}'
            | '\u{180E}'
            | '\u{200B}'..='\u{200F}'
            | '\u{202A}'..='\u{202E}'
            | '\u{2060}'..='\u{2064}'
            | '\u{2066}'..='\u{206F}'
            | '\u{FEFF}'
            | '\u{FFF9}'..='\u{FFFB}'
    )
}

/// A single name that is also a valid file name on Windows: not empty, at most
/// [`MAX_NAME_CHARS`] characters, not `.`, without `..`, not ending with `.`, without
/// whitespace, control or invisible format characters and without any of
/// `/ \ : ? # % < > | * "`, and not a reserved device name.
fn is_name(text: &str) -> bool {
    let stem = text.split('.').next().unwrap_or_default().to_uppercase();
    !text.is_empty()
        && text.chars().count() <= MAX_NAME_CHARS
        && !text.contains("..")
        && !text.ends_with('.')
        && !RESERVED_NAMES.contains(&stem.as_str())
        && !text.chars().any(|c| {
            c.is_whitespace()
                || c.is_control()
                || is_format_char(c)
                || matches!(
                    c,
                    '/' | '\\' | ':' | '?' | '#' | '%' | '<' | '>' | '|' | '*' | '"'
                )
        })
}

/// A relative path: names joined by `/`, at most [`MAX_PATH_CHARS`] characters.
fn is_relative_path(text: &str) -> bool {
    text.chars().count() <= MAX_PATH_CHARS && text.split('/').all(is_name)
}

/// The first path of `paths` (lower case) that is also the folder of another one, as
/// in `a` next to `a/b`.
fn file_as_folder<'a>(paths: impl IntoIterator<Item = &'a str> + Clone) -> Option<String> {
    let all: HashSet<String> = paths.clone().into_iter().map(str::to_lowercase).collect();
    paths.into_iter().find_map(|path| {
        let lower = path.to_lowercase();
        let parts: Vec<&str> = lower.split('/').collect();
        (1..parts.len())
            .map(|n| parts[..n].join("/"))
            .find(|folder| all.contains(folder))
    })
}

/// The address of a file for finding duplicates: [`ModelFile::url`] with the repository
/// in lower case, because both hosts resolve `owner/name` without regard to case.
fn address_key(source: Option<&FileSource>, path: &str) -> Option<String> {
    let lower = |repo: &Option<String>| repo.as_ref().map(|r| r.to_lowercase());
    let source = match source? {
        FileSource::HuggingFace(s) => FileSource::HuggingFace(HuggingFaceSource {
            repo: lower(&s.repo),
            revision: s.revision.clone(),
        }),
        FileSource::GithubRelease(s) => FileSource::GithubRelease(ReleaseSource {
            repo: lower(&s.repo),
            tag: s.tag.clone(),
            asset: s.asset.clone(),
        }),
    };
    source.url(path)
}

/// `owner/name`, two names.
fn is_repo(text: &str) -> bool {
    let parts: Vec<&str> = text.split('/').collect();
    parts.len() == 2 && parts.iter().all(|p| is_name(p))
}

impl ModelEntry {
    /// The entry as it is used on `tier`: on a tier without multi-token prediction
    /// (not in `mtp_tiers`) without `mtp_draft` and without the drafter in `files`;
    /// otherwise unchanged. [`ModelManifest::answer_model`] and
    /// [`ModelManifest::models_for`] return entries in this form.
    pub fn on_tier(&self, tier: HardwareTier) -> ModelEntry {
        let mut entry = self.clone();
        if let Some(draft) = &self.mtp_draft
            && !self.mtp_tiers.contains(&tier)
        {
            entry.files.retain(|f| f.path != *draft);
            entry.mtp_draft = None;
            entry.mtp_tiers.clear();
        }
        entry
    }
}

fn is_hex(text: &str, len: usize) -> bool {
    text.len() == len && text.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

impl ModelManifest {
    /// Parses and validates a manifest in TOML format.
    pub fn from_toml_str(text: &str) -> Result<Self> {
        let manifest: Self = toml::from_str(text).map_err(|e| Error::Config(e.to_string()))?;
        manifest.validate()?;
        Ok(manifest)
    }

    /// Checks the invariants documented on the fields: schema version 4, ids that are
    /// single names (see [`FileSource`]) and unique across models and runtimes also
    /// ignoring case (an id is a folder name), no unknown keys (checked when parsing),
    /// at least one tier and
    /// file per model, relative file paths unique per model and runtime (also ignoring
    /// case), well-formed checksums and sources (open parts allowed, see
    /// [`FileSource`]), every complete address only once, `archive` naming a release file
    /// of the same runtime, `dimension` exactly on embedding entries,
    /// `level` exactly on answer models, `mtp_draft` only on answer models and naming a
    /// file besides exactly one model file, for every tier in use a model of every role
    /// except the reranker and an answer model of the fast level, and a model with
    /// preference 1 for every role, tier and level in use.
    pub fn validate(&self) -> Result<()> {
        let fail = |message: String| Err(Error::Config(message));
        if self.schema_version != 4 {
            return fail(format!(
                "unsupported schema_version {}",
                self.schema_version
            ));
        }
        let mut ids = HashSet::new();
        let mut addresses = HashSet::new();
        for runtime in &self.runtimes {
            if !ids.insert(runtime.id.to_lowercase()) {
                return fail(format!("duplicate id {}", runtime.id));
            }
            if !is_name(&runtime.id) {
                return fail(format!("runtime id {} is not a single name", runtime.id));
            }
            let mut archives: Vec<Option<&String>> =
                runtime.files.iter().map(|f| f.archive.as_ref()).collect();
            archives.dedup();
            for archive in archives {
                let group = runtime
                    .files
                    .iter()
                    .filter(|f| f.archive.as_ref() == archive)
                    .map(|f| f.path.as_str());
                if let Some(folder) = file_as_folder(group) {
                    return fail(format!(
                        "runtime {}: {folder} is a file and the folder of another file",
                        runtime.id
                    ));
                }
            }
            let mut paths = HashSet::new();
            for file in &runtime.files {
                if !is_relative_path(&file.path) {
                    return fail(format!(
                        "runtime {}: path {} is not a relative path",
                        runtime.id, file.path
                    ));
                }
                if !paths.insert((file.archive.clone(), file.path.to_lowercase())) {
                    return fail(format!(
                        "runtime {}: duplicate file {}",
                        runtime.id, file.path
                    ));
                }
                if let Some(archive) = &file.archive {
                    let found = runtime
                        .files
                        .iter()
                        .any(|f| f.archive.is_none() && f.path == *archive);
                    if !found {
                        return fail(format!(
                            "runtime {}: archive {archive} of {} is no release file of it",
                            runtime.id, file.path
                        ));
                    }
                }
                if let Some(url) = address_key(file.source.as_ref(), &file.path)
                    && !addresses.insert(url.clone())
                {
                    return fail(format!("address {url} occurs twice"));
                }
                if file.sha256.as_deref().is_some_and(|h| !is_hex(h, 64)) {
                    return fail(format!(
                        "runtime {}: sha256 of {} is not 64 lowercase hex digits",
                        runtime.id, file.path
                    ));
                }
                if file.archive.is_some() && file.source.is_some() {
                    return fail(format!(
                        "runtime {}: {} lies in an archive and has no source of its own",
                        runtime.id, file.path
                    ));
                }
                if let Some(Err(message)) = file.source.as_ref().map(FileSource::check) {
                    return fail(format!("runtime {}: {}: {message}", runtime.id, file.path));
                }
            }
        }
        for model in &self.models {
            let id = &model.id;
            if !ids.insert(id.to_lowercase()) {
                return fail(format!("duplicate id {id}"));
            }
            if !is_name(id) {
                return fail(format!("model id {id} is not a single name"));
            }
            if model.tiers.is_empty() {
                return fail(format!("model {id} has no tier"));
            }
            if model.files.is_empty() {
                return fail(format!("model {id} has no file"));
            }
            if (model.role == ModelRole::Embedding) != model.dimension.is_some() {
                return fail(format!(
                    "model {id}: dimension is required for embedding models and only for them"
                ));
            }
            let answer = model.role == ModelRole::Generator;
            if answer != model.level.is_some() {
                return fail(format!(
                    "model {id}: level is required for answer models and only for them"
                ));
            }
            if let Some(draft) = &model.mtp_draft {
                let others = model.files.iter().filter(|f| f.path != *draft).count();
                if !answer || others + 1 != model.files.len() || others != 1 {
                    return fail(format!(
                        "model {id}: mtp_draft must name a file of an answer model besides its one model file"
                    ));
                }
            } else if answer && model.files.len() != 1 {
                return fail(format!("model {id}: an answer model has one model file"));
            }
            let mtp_tiers: HashSet<&HardwareTier> = model.mtp_tiers.iter().collect();
            let mtp_ok = if model.mtp_draft.is_some() {
                !model.mtp_tiers.is_empty()
                    && mtp_tiers.len() == model.mtp_tiers.len()
                    && model.mtp_tiers.iter().all(|t| model.tiers.contains(t))
            } else {
                model.mtp_tiers.is_empty()
            };
            if !mtp_ok {
                return fail(format!(
                    "model {id}: mtp_tiers must be a non-empty subset of tiers with mtp_draft and empty without"
                ));
            }
            if let Some(folder) = file_as_folder(model.files.iter().map(|f| f.path.as_str())) {
                return fail(format!(
                    "model {id}: {folder} is a file and the folder of another file"
                ));
            }
            let mut paths = HashSet::new();
            for file in &model.files {
                let path = &file.path;
                if !is_relative_path(path) {
                    return fail(format!("model {id}: path {path} is not a relative path"));
                }
                if !paths.insert(path.to_lowercase()) {
                    return fail(format!("model {id}: duplicate file {path}"));
                }
                if let Some(url) = address_key(file.source.as_ref(), &file.path)
                    && !addresses.insert(url.clone())
                {
                    return fail(format!("address {url} occurs twice"));
                }
                if file.sha256.as_deref().is_some_and(|h| !is_hex(h, 64)) {
                    return fail(format!(
                        "model {id}: sha256 of {path} is not 64 lowercase hex digits"
                    ));
                }
                if let Some(Err(message)) = file.source.as_ref().map(FileSource::check) {
                    return fail(format!("model {id}: source of {path}: {message}"));
                }
            }
        }
        let tiers: HashSet<HardwareTier> =
            self.models.iter().flat_map(|m| m.tiers.clone()).collect();
        for tier in tiers {
            for role in ModelRole::PROVIDE_ORDER {
                let present = match role {
                    ModelRole::Reranker => true,
                    ModelRole::Generator => self.answer_entry(tier, AnswerLevel::Fast).is_some(),
                    _ => self.select(role, tier).is_some(),
                };
                if !present {
                    return fail(format!("no {role:?} model for tier {tier:?}"));
                }
            }
        }
        for model in &self.models {
            for tier in &model.tiers {
                let chosen = match model.level {
                    Some(level) => self.answer_entry(*tier, level),
                    None => self.select(model.role, *tier),
                };
                if chosen.map(|m| m.preference) != Some(1) {
                    return fail(format!(
                        "no preferred model (preference 1) for role {:?}, level {:?} and tier {tier:?}",
                        model.role, model.level
                    ));
                }
            }
        }
        Ok(())
    }

    /// Returns the model to use for `role` on `tier`: the entry with the lowest
    /// preference among those listing the tier. `None` if the tier runs without it, and
    /// always for [`ModelRole::Generator`], whose model depends on the level (see
    /// [`ModelManifest::answer_model`]).
    pub fn select(&self, role: ModelRole, tier: HardwareTier) -> Option<&ModelEntry> {
        self.models
            .iter()
            .filter(|m| m.role == role && m.level.is_none() && m.tiers.contains(&tier))
            .min_by_key(|m| m.preference)
    }

    /// Returns the answer model of `level` on `tier`, chosen like in
    /// [`ModelManifest::select`] and in its form for the tier ([`ModelEntry::on_tier`]:
    /// without the drafter where multi-token prediction is off there). `None` if the
    /// tier does not offer the level.
    pub fn answer_model(&self, tier: HardwareTier, level: AnswerLevel) -> Option<ModelEntry> {
        self.answer_entry(tier, level).map(|m| m.on_tier(tier))
    }

    /// The manifest entry behind [`ModelManifest::answer_model`].
    fn answer_entry(&self, tier: HardwareTier, level: AnswerLevel) -> Option<&ModelEntry> {
        self.models
            .iter()
            .filter(|m| m.level == Some(level) && m.tiers.contains(&tier))
            .min_by_key(|m| m.preference)
    }

    /// The level to preselect and use for `saved` (usually
    /// [`Settings::answer_level`]) on `tier`: `saved` if the tier offers it, otherwise
    /// [`AnswerLevel::Fast`] (settings from another machine, less memory). The saved
    /// setting is not changed; it applies again on a tier that offers it.
    pub fn effective_level(&self, saved: AnswerLevel, tier: HardwareTier) -> AnswerLevel {
        if self.answer_entry(tier, saved).is_some() {
            saved
        } else {
            AnswerLevel::Fast
        }
    }

    /// The levels `tier` offers, from fast to thorough.
    pub fn levels(&self, tier: HardwareTier) -> Vec<AnswerLevel> {
        AnswerLevel::ALL
            .into_iter()
            .filter(|l| self.answer_entry(tier, *l).is_some())
            .collect()
    }

    /// The models [`ModelHub::provide`](crate::ModelHub::provide) makes ready for
    /// `level` on `tier`, in [`ModelRole::PROVIDE_ORDER`]: the base equipment (one model
    /// of every role the tier uses except the answer model, and the answer model of the
    /// fast level, ADR 0058) and the answer model of `level` after it, each in its form
    /// for the tier ([`ModelEntry::on_tier`]: below the minimum the fast model comes
    /// without its drafter, ADR 0084). `InvalidInput` if the tier does not offer
    /// `level`.
    pub fn models_for(&self, tier: HardwareTier, level: AnswerLevel) -> Result<Vec<ModelEntry>> {
        let Some(answer) = self.answer_model(tier, level) else {
            return Err(Error::InvalidInput(format!(
                "tier {tier:?} does not offer level {level:?}"
            )));
        };
        let mut models: Vec<ModelEntry> = ModelRole::PROVIDE_ORDER
            .iter()
            .filter_map(|r| self.select(*r, tier))
            .map(|m| m.on_tier(tier))
            .collect();
        models.extend(self.answer_model(tier, AnswerLevel::Fast));
        if level != AnswerLevel::Fast {
            models.push(answer);
        }
        Ok(models)
    }
}
