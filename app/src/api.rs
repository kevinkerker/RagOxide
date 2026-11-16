//! Arguments, results, errors and events of the interface to the user interface.
//!
//! Together with the command list in `commands.rs` this is the contract the user
//! interface is built against. The data types come from `kb-core` (feature `ts`); this
//! module adds only what the app itself defines. The TypeScript bindings in
//! `ui/api/bindings.ts` are generated from both (see `bindings.rs`).

use std::path::PathBuf;

use kb_core::{
    AnswerLevel, ChatId, ColorScheme, Document, DocumentId, DocumentRecord, ErrorKind,
    HardwareInfo, HardwareTier, IngestEvent, MissingModelFile, ModelEntry, ModelEvent,
    ModelStorage, ScopeEvent, Settings, SyncMode, UiLanguage,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Identifier of a running operation (`ask`, `sync`, `provide_models`), created by the
/// user interface, e.g. with `crypto.randomUUID()`. `cancel` stops the operation.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
pub struct OperationId(pub String);

/// Arguments of `ask`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct AskArgs {
    /// Operation id for `cancel`, unique among running operations.
    pub op: OperationId,
    /// The question, not empty.
    pub question: String,
    /// Chat the turn is saved to. A chat id that does not exist yet creates the chat
    /// with the question as title. `null`: nothing is saved. Only one question per chat
    /// may run at a time.
    pub chat: Option<ChatId>,
    /// Answer level of this question (ADR 0055). Not written to the settings: the user
    /// interface does that with `set_answer_level` when the user changes it (ADR 0060).
    /// A level whose model is missing is `model_missing`, one the hardware tier does not
    /// offer `invalid_input`; the question is then not asked.
    pub level: AnswerLevel,
}

/// Arguments of `ask_again`: the question of a saved turn asked again (ADR 0071, "Erneut
/// fragen in …", "Gründlicher beantworten").
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct AskAgainArgs {
    /// Operation id for `cancel`, unique among running operations.
    pub op: OperationId,
    /// The chat of the turn.
    pub chat: ChatId,
    /// Index of the turn in `Chat.turns`, from 0. Its question is asked again; the
    /// history is only the complete turns before it.
    pub turn: u32,
    /// Answer level of this question; like `ask`, not written to the settings.
    pub level: AnswerLevel,
}

/// Arguments of `cancel`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct OperationArgs {
    /// The operation.
    pub op: OperationId,
}

/// Arguments of `sync`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct SyncArgs {
    /// Operation id for `cancel` and for matching the progress events.
    pub op: OperationId,
    /// Which files are processed again: `normal` (changed files), `retry_failed`
    /// ("Erneut versuchen" in the error list: also failed files that did not change),
    /// `full` ("Integrität prüfen", every file by content, ADR 0053, 0072).
    pub mode: SyncMode,
}

/// Arguments of `provide_models`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct ProvideModelsArgs {
    /// Operation id for `cancel` and for matching `model_progress`.
    pub op: OperationId,
    /// Level to provide (ADR 0058): `fast` is the base equipment of the setup (parser
    /// models, embedding, reranker, fast answer model); a higher level adds its answer
    /// model. A level the hardware tier does not offer is `invalid_input`.
    pub level: AnswerLevel,
    /// Folder of an offline package chosen by the user (ADR 0049): files found there are
    /// checked and copied, the rest is downloaded. `null`: download only. Must be a
    /// folder `pick_folder` returned in this session for `offline_package`, otherwise
    /// `invalid_input`.
    pub source: Option<PathBuf>,
}

/// Arguments of `rename_chat`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct RenameChatArgs {
    /// The chat.
    pub id: ChatId,
    /// New title, trimmed; empty is `invalid_input` (ADR 0044).
    pub title: String,
}

/// Arguments of `get_chat` and `delete_chat`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct ChatArgs {
    /// The chat.
    pub id: ChatId,
}

/// Arguments of `get_document`, `document_file`, `open_document` and `reveal_document`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct DocumentArgs {
    /// The document.
    pub id: DocumentId,
}

/// Arguments of `add_folder`, `remove_folder`, `exclude` and `include`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct PathArgs {
    /// Absolute path of a folder, file or subfolder. For `add_folder` it must be a folder
    /// `pick_folder` returned in this session for `watch`, otherwise `invalid_input`.
    pub path: PathBuf,
}

/// Arguments of `acknowledge_removed_exclusions`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct PathsArgs {
    /// The paths, as `list_removed_exclusions` returned them.
    pub paths: Vec<PathBuf>,
}

/// What a folder is chosen for in `pick_folder`. The dialog names it in its title, and
/// the chosen folder is released for this purpose only.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum FolderPurpose {
    /// A folder to watch (`add_folder`).
    Watch,
    /// A subfolder to exclude (`exclude` needs no release, it only narrows the scope).
    Exclude,
    /// An offline package (`provide_models.source`, ADR 0049).
    OfflinePackage,
}

/// Arguments of `pick_folder`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct PickFolderArgs {
    /// What the folder is for; named in the dialog title.
    pub purpose: FolderPurpose,
    /// Folder the dialog opens in, e.g. a watched folder when choosing a subfolder to
    /// exclude; `null` for the system default.
    pub start: Option<PathBuf>,
}

/// Arguments of `set_answer_level`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct AnswerLevelArgs {
    /// The level the user chose.
    pub level: AnswerLevel,
}

/// Arguments of `update_settings`: the settings of the user interface. The library scope
/// changes only through the folder and exclusion commands, the answer level only
/// through `set_answer_level`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct SettingsChange {
    /// Language of the user interface.
    pub ui_language: UiLanguage,
    /// Colour scheme of the user interface.
    pub color_scheme: ColorScheme,
}

/// A document for the viewer: library entry and parsed content.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct DocumentDetails {
    /// Library entry, with the current content hash.
    pub record: DocumentRecord,
    /// Parsed content with pages, elements and anchors; `null` while the document has
    /// not been indexed successfully.
    pub content: Option<Document>,
}

/// Last known state of a watched folder.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FolderStatus {
    /// Not checked yet since the start or since it was registered.
    Unknown,
    /// The folder could be read at the last check.
    Available,
    /// The folder could not be read at the last check; its documents are kept.
    Unavailable {
        /// Category of the error.
        kind: ErrorKind,
        /// Technical error text (English).
        message: String,
    },
}

/// A watched folder with its state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct WatchedFolder {
    /// The folder, canonical.
    pub path: PathBuf,
    /// Its last known state.
    pub status: FolderStatus,
}

/// The library scope (ADR 0025): watched folders with their state, minus the
/// exclusion list.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct LibraryScope {
    /// Watched folders in the order they were registered.
    pub folders: Vec<WatchedFolder>,
    /// Excluded files and subfolders, each inside a watched folder.
    pub excluded: Vec<PathBuf>,
}

/// State of one answer level on this machine (ADR 0055, 0058).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct LevelState {
    /// The level.
    pub level: AnswerLevel,
    /// `true` if the hardware tier offers the level (below the minimum only `fast`,
    /// ADR 0056).
    pub offered: bool,
    /// Its answer model, `null` if not offered.
    pub model: Option<ModelEntry>,
    /// `true` if asking in this level is possible: offered and nothing missing.
    pub ready: bool,
    /// Files `provide_models` for this level would still fetch: the base equipment if
    /// it is incomplete, and the answer model of the level. Empty if not offered.
    pub missing: Vec<MissingModelFile>,
    /// Sum of the known sizes of `missing`, the download size of the level.
    pub download_bytes: u64,
    /// Files in `missing` whose size `models.toml` does not state.
    pub unknown_sizes: u32,
}

/// State of the model setup.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct SetupState {
    /// Detected hardware.
    pub hardware: HardwareInfo,
    /// Tier of the hardware: `supported` from the minimum of 16 GB, `below_minimum`
    /// under it (setup warns and allows the attempt, fast level only, ADR 0056, 0050).
    pub tier: HardwareTier,
    /// The base equipment: what ingestion and the fast level need (ADR 0058), in the
    /// order it is provided.
    pub models: Vec<ModelEntry>,
    /// Files of the base equipment that are absent or do not match their checksum.
    pub missing: Vec<MissingModelFile>,
    /// Sum of the known sizes of `missing`, the download of the setup.
    pub download_bytes: u64,
    /// Files in `missing` whose size `models.toml` does not state; if not 0, the download
    /// is somewhat larger than `download_bytes`.
    pub unknown_sizes: u32,
    /// `true` if nothing of the base equipment is missing: ingestion and the fast level
    /// work. The shell starts the first-run setup otherwise.
    pub models_ready: bool,
    /// All three levels, from fast to thorough, with what each one needs.
    pub levels: Vec<LevelState>,
    /// The level to preselect: the one chosen last (`Settings::answer_level`, ADR 0060)
    /// if this machine offers it, otherwise `fast` (ADR 0074). The setting itself stays.
    pub answer_level: AnswerLevel,
    /// Model directory and free space on its drive, for the warning before the download
    /// and "show storage location" (ADR 0049).
    pub storage: ModelStorage,
}

/// What `document_file` reports about the bytes it delivers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct DocumentFileInfo {
    /// SHA-256 of the delivered bytes.
    pub content_hash: String,
    /// `true` if the file changed since it was ingested: `content_hash` differs from
    /// `DocumentRecord.content_hash`. The viewer then shows the passage as outdated.
    pub changed: bool,
}

/// Source file of a document with what is known about its version. On the wire a raw
/// response (`u32` little-endian length, the JSON of [`DocumentFileInfo`], the file
/// bytes); `ui/api/tauri.ts` turns it into `DocumentFileInfo & { bytes: ArrayBuffer }`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentFile {
    /// Hash and change flag.
    pub info: DocumentFileInfo,
    /// File content.
    pub bytes: Vec<u8>,
}

impl DocumentFile {
    /// The raw response body.
    pub fn encode(&self) -> Vec<u8> {
        let header = serde_json::to_vec(&self.info).unwrap_or_default();
        let length = u32::try_from(header.len()).unwrap_or(u32::MAX);
        let mut body = Vec::with_capacity(4 + header.len() + self.bytes.len());
        body.extend_from_slice(&length.to_le_bytes());
        body.extend_from_slice(&header);
        body.extend_from_slice(&self.bytes);
        body
    }
}

impl TS for DocumentFile {
    type WithoutGenerics = Self;
    type OptionInnerType = Self;

    fn docs() -> Option<String> {
        Some(
            [
                "/**",
                " * Source file of a document: the bytes for pdf.js and the image display, with",
                " * the hash of these bytes and whether the file changed since it was ingested.",
                " */",
                "",
            ]
            .join("\n"),
        )
    }

    fn name(_: &ts_rs::Config) -> String {
        "DocumentFile".to_owned()
    }

    fn inline(cfg: &ts_rs::Config) -> String {
        Self::name(cfg)
    }

    fn decl(_: &ts_rs::Config) -> String {
        "type DocumentFile = DocumentFileInfo & { bytes: ArrayBuffer };".to_owned()
    }
}

/// Error of a command. Commands reject their promise with this value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct ApiError {
    /// Category to react on; `cancelled` is not a failure and must not be reported.
    pub kind: ErrorKind,
    /// Technical detail in English, for logs and an expandable detail line.
    pub message: String,
}

impl ApiError {
    /// Error of the given kind.
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}

impl From<kb_core::Error> for ApiError {
    fn from(error: kb_core::Error) -> Self {
        Self::new(error.kind(), error.to_string())
    }
}

/// Payload of `ingest_progress`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct IngestProgress {
    /// Operation of the `sync` call, `null` for the folder watcher.
    pub op: Option<OperationId>,
    /// Progress of one file.
    pub event: IngestEvent,
}

/// Payload of `ingest_finished`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct IngestFinished {
    /// Operation of the `sync` call, `null` for the folder watcher.
    pub op: Option<OperationId>,
}

/// Payload of `scope_changed`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct ScopeNotice {
    /// Operation of the `sync` call, `null` for the folder watcher.
    pub op: Option<OperationId>,
    /// Folder availability or a dropped exclusion.
    pub event: ScopeEvent,
}

/// Payload of `model_progress`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct ModelProgress {
    /// Operation of the `provide_models` call.
    pub op: OperationId,
    /// Level the call provides, so the status bar and the chat can tell the download of
    /// the setup (`fast`) from that of a higher level.
    pub level: AnswerLevel,
    /// Progress of one model file.
    pub event: ModelEvent,
}

/// One entry of the command or event list, for the generated bindings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Signature {
    /// Command or event name as used on the wire.
    pub name: &'static str,
    /// Doc comment lines.
    pub docs: Vec<&'static str>,
    /// TypeScript type of the `args` parameter, if any.
    pub args: Option<String>,
    /// TypeScript type of the `on_event` channel messages, if any.
    pub event: Option<String>,
    /// TypeScript type of the result (commands) or payload (events).
    pub result: String,
}

macro_rules! events {
    ($( $(#[doc = $doc:literal])* $variant:ident($payload:ty) => $name:literal; )*) => {
        /// An event the backend broadcasts to the user interface.
        #[derive(Debug, Clone, PartialEq)]
        pub enum Event {
            $( $(#[doc = $doc])* $variant($payload), )*
        }

        impl Event {
            /// Event name on the wire.
            pub fn name(&self) -> &'static str {
                match self {
                    $( Self::$variant(_) => $name, )*
                }
            }

            /// Serialised payload.
            pub fn payload(&self) -> serde_json::Result<serde_json::Value> {
                match self {
                    $( Self::$variant(payload) => serde_json::to_value(payload), )*
                }
            }
        }

        /// The event list for the generated bindings.
        pub(crate) fn event_signatures(cfg: &ts_rs::Config) -> Vec<Signature> {
            vec![$(
                Signature {
                    name: $name,
                    docs: vec![$($doc),*],
                    args: None,
                    event: None,
                    result: <$payload as TS>::name(cfg),
                },
            )*]
        }
    };
}

events! {
    /// Progress of one file of a synchronisation, from `sync` (with `op`) or from the
    /// folder watcher (`op` is `null`). A terminal state (`indexed`, `unchanged`,
    /// `removed`, `failed`) means `list_documents` returns new data. Package L.
    IngestProgress(IngestProgress) => "ingest_progress";
    /// A synchronisation run ended: a `sync` call (with `op`) returned, completed,
    /// cancelled or failed, or a run of the folder watcher (`op` is `null`) was stopped
    /// because the watcher restarted or failed. A cancelled run sends no terminal
    /// `ingest_progress` for the files it did not reach; this event tells the status bar
    /// that the run is over. Shell.
    IngestFinished(IngestFinished) => "ingest_finished";
    /// A watched folder became readable or unreadable, or an exclusion whose path is gone
    /// was dropped. `get_scope` returns the new state. Package L.
    ScopeChanged(ScopeNotice) => "scope_changed";
    /// Progress of one model file during `provide_models`, with the level provided.
    /// Package L (setup), K (download of a higher level) and shell (status bar).
    ModelProgress(ModelProgress) => "model_progress";
    /// The settings changed (any command that changes them, or a dropped exclusion);
    /// carries the new settings. The shell switches language and colour scheme on it.
    SettingsChanged(Settings) => "settings_changed";
    /// The folder watcher stopped with an error that concerns no single folder (store or
    /// index failing); folder problems come as `scope_changed`. Package L.
    WatchFailed(ApiError) => "watch_failed";
}
