//! The Tauri commands: the single list the user interface is built against.
//!
//! `commands!` expands every entry into a Tauri command that forwards to [`Backend`] (or,
//! for the folder dialog, to the dialog plugin), registers all of them in
//! [`invoke_handler`] and lists them for the generated TypeScript bindings. A command is
//! therefore registered and typed if and only if it is in this list.
//!
//! Wire convention: a command takes at most one data argument named `args` and at most
//! one channel named `on_event`; it resolves with its result or rejects with an
//! [`ApiError`].

use std::path::PathBuf;

use tauri::ipc::{Channel, InvokeResponseBody, IpcResponse};
use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;

use kb_core::{
    Answer, AnswerEvent, AnswerLevel, Chat, ChatSummary, DocumentRecord, ErrorKind, FileResult,
    SearchHit, SearchRequest, Settings,
};

use crate::api::{
    AnswerLevelArgs, ApiError, AskAgainArgs, AskArgs, ChatArgs, DocumentArgs, DocumentDetails,
    DocumentFile, FolderPurpose, LibraryScope, OperationArgs, PathArgs, PathsArgs, PickFolderArgs,
    ProvideModelsArgs, RenameChatArgs, SettingsChange, SetupState, Signature, SyncArgs,
};
use crate::backend::{ApiResult, Backend};

impl IpcResponse for DocumentFile {
    fn body(self) -> tauri::Result<InvokeResponseBody> {
        Ok(InvokeResponseBody::Raw(self.encode()))
    }
}

macro_rules! command_fn {
    ($name:ident () -> $ret:ty) => {
        #[tauri::command(rename_all = "snake_case")]
        async fn $name(app: AppHandle, backend: State<'_, Backend>) -> ApiResult<$ret> {
            handlers::$name(&app, &backend).await
        }
    };
    ($name:ident (args: $args:ty) -> $ret:ty) => {
        #[tauri::command(rename_all = "snake_case")]
        async fn $name(
            app: AppHandle,
            backend: State<'_, Backend>,
            args: $args,
        ) -> ApiResult<$ret> {
            handlers::$name(&app, &backend, args).await
        }
    };
    ($name:ident (args: $args:ty, event: $event:ty) -> $ret:ty) => {
        #[tauri::command(rename_all = "snake_case")]
        async fn $name(
            app: AppHandle,
            backend: State<'_, Backend>,
            args: $args,
            on_event: Channel<$event>,
        ) -> ApiResult<$ret> {
            handlers::$name(&app, &backend, args, on_event).await
        }
    };
}

macro_rules! command_signature {
    ($cfg:ident, $name:ident, $docs:expr, () -> $ret:ty) => {
        Signature {
            name: stringify!($name),
            docs: $docs,
            args: None,
            event: None,
            result: <$ret as ts_rs::TS>::name($cfg),
        }
    };
    ($cfg:ident, $name:ident, $docs:expr, (args: $args:ty) -> $ret:ty) => {
        Signature {
            name: stringify!($name),
            docs: $docs,
            args: Some(<$args as ts_rs::TS>::name($cfg)),
            event: None,
            result: <$ret as ts_rs::TS>::name($cfg),
        }
    };
    ($cfg:ident, $name:ident, $docs:expr, (args: $args:ty, event: $event:ty) -> $ret:ty) => {
        Signature {
            name: stringify!($name),
            docs: $docs,
            args: Some(<$args as ts_rs::TS>::name($cfg)),
            event: Some(<$event as ts_rs::TS>::name($cfg)),
            result: <$ret as ts_rs::TS>::name($cfg),
        }
    };
}

macro_rules! commands {
    ($( $(#[doc = $doc:literal])* $name:ident ( $($params:tt)* ) -> $ret:ty; )*) => {
        $( command_fn!($name ( $($params)* ) -> $ret); )*

        /// Handler that registers every command of the list.
        pub(crate) fn invoke_handler(
        ) -> impl Fn(tauri::ipc::Invoke<tauri::Wry>) -> bool + Send + Sync + 'static {
            tauri::generate_handler![$($name),*]
        }

        /// The command list for the generated bindings.
        pub(crate) fn command_signatures(cfg: &ts_rs::Config) -> Vec<Signature> {
            vec![$( command_signature!(cfg, $name, vec![$($doc),*], ( $($params)* ) -> $ret) ),*]
        }
    };
}

commands! {
    /// Asks a question in the level of `args.level` (ADR 0055). Streams `status` (five
    /// phases), `sources`, `delta` and `done` through `on_event` in the order of ADR 0054
    /// and 0062: `waiting` first if another answer runs; then a first question
    /// `searching`, `sources`, `starting_model`, `reading_sources`, `writing`; a follow-up
    /// `starting_model`, `searching`, `sources`, `reading_sources`, `writing` (details in
    /// `AnswerEvent`). It resolves with the final answer (the same as in `done`, with
    /// its level) after the chat turn was saved. A cancelled or failed answer also ends
    /// with `done` and is saved, with `outcome` saying how it ended (ADR 0045). A level
    /// whose model is missing rejects with `model_missing` before anything is asked
    /// (ADR 0058), one the hardware tier does not offer with `invalid_input`. Package K.
    ask(args: AskArgs, event: AnswerEvent) -> Answer;
    /// Asks the question of turn `args.turn` of a saved chat again in `args.level`
    /// ("Erneut fragen in …", "Gründlicher beantworten", ADR 0065, 0070, 0071). The
    /// history is only the complete turns before that turn: neither the old answer nor
    /// later turns go in. The answer is saved as a new turn at the end, the old one stays.
    /// Events, cancel and errors as `ask`; an unknown chat or turn is `invalid_input`.
    /// Package K.
    ask_again(args: AskAgainArgs, event: AnswerEvent) -> Answer;
    /// Cancels a running `ask`, `sync` or `provide_models`. Unknown or finished
    /// operations are ignored. Packages K and L.
    cancel(args: OperationArgs) -> ();
    /// Lists the stored chats, most recent first. Package K.
    list_chats() -> Vec<ChatSummary>;
    /// Returns a stored chat with all turns, `null` if it does not exist. Package K.
    get_chat(args: ChatArgs) -> Option<Chat>;
    /// Renames a chat (ADR 0044). The title is trimmed; empty is `invalid_input`, an
    /// unknown chat `not_found`. The order of the history does not change. Package K.
    rename_chat(args: RenameChatArgs) -> ();
    /// Deletes a chat without undo (ADR 0023); succeeds if it does not exist. Package K.
    delete_chat(args: ChatArgs) -> ();
    /// Searches the library without generating an answer; hits are ranked and carry
    /// their provenance. Package K.
    search(args: SearchRequest) -> Vec<SearchHit>;
    /// Lists the library sorted by path, including queued and failed entries (the
    /// error list). Packages L and M.
    list_documents() -> Vec<DocumentRecord>;
    /// Returns library entry and parsed content (pages, elements, anchors) of a document,
    /// `null` if it does not exist. Package M.
    get_document(args: DocumentArgs) -> Option<DocumentDetails>;
    /// Returns the source file of a document for the PDF and image display, with the
    /// SHA-256 of the delivered bytes and `changed` if the file differs from the ingested
    /// version (outdated citation). Only documents in the library. Package M.
    document_file(args: DocumentArgs) -> DocumentFile;
    /// Opens the source file of a document in its default program (ADR 0039). Takes a
    /// document id, never a path: only documents in the library. Packages L and M.
    open_document(args: DocumentArgs) -> ();
    /// Shows the source file of a document in Explorer or Finder (ADR 0039). Takes a
    /// document id, never a path: only documents in the library. Packages L and M.
    reveal_document(args: DocumentArgs) -> ();
    /// Returns the watched folders with their state and the exclusion list. Package L.
    get_scope() -> LibraryScope;
    /// Opens the native dialog to choose a folder for `purpose`, named in its title;
    /// resolves with the path, `null` if the dialog was closed. The folder is released
    /// for this purpose only: `watch` for `add_folder`, `offline_package` for the
    /// `source` of `provide_models`; `exclude` needs no release. Package L.
    pick_folder(args: PickFolderArgs) -> Option<PathBuf>;
    /// Registers a watched folder chosen with `pick_folder` for `watch` (any other path
    /// is `invalid_input`). Everything in it is ingested by the folder watcher; progress goes
    /// out as `ingest_progress` with `op` null. A folder inside an exclusion is
    /// `invalid_input`; a missing folder or a file comes as unavailable. Package L.
    add_folder(args: PathArgs) -> LibraryScope;
    /// Deregisters a watched folder. Its documents and exclusions disappear unless
    /// another watched folder contains them. Package L.
    remove_folder(args: PathArgs) -> LibraryScope;
    /// Excludes a file or subfolder strictly inside a watched folder; its documents
    /// disappear, the files on disk stay. Otherwise `invalid_input`. Package L.
    exclude(args: PathArgs) -> LibraryScope;
    /// Takes an entry off the exclusion list; it is ingested again. Package L.
    include(args: PathArgs) -> LibraryScope;
    /// Exclusions the folder watcher dropped because their path is gone, not yet
    /// acknowledged. Kept by the backend, so that those dropped at start, before the user
    /// interface listened to `scope_changed`, are not lost. Shell and package L.
    list_removed_exclusions() -> Vec<PathBuf>;
    /// The user has seen these dropped exclusions; they are no longer listed. Package L
    /// through the shell (`removedExclusions.acknowledge`).
    acknowledge_removed_exclusions(args: PathsArgs) -> ();
    /// Runs one synchronisation now and resolves with one result per touched file.
    /// `mode`: `normal` ("Erneut prüfen" for an unavailable folder), `retry_failed`
    /// ("Erneut versuchen" in the error list: failed files again even if unchanged),
    /// `full` ("Integrität prüfen", every file by content, ADR 0053, 0072).
    /// Progress as `ingest_progress` and `scope_changed` with the same `op`, the end as
    /// `ingest_finished`. Package L.
    sync(args: SyncArgs) -> Vec<FileResult>;
    /// Returns the settings: scope, answer level, language, colour scheme. Package L and
    /// shell.
    get_settings() -> Settings;
    /// Changes language and colour scheme without touching scope and answer level, and
    /// broadcasts `settings_changed`. Package L.
    update_settings(args: SettingsChange) -> Settings;
    /// Remembers the answer level the user chose (ADR 0060) and broadcasts
    /// `settings_changed`; a level the hardware tier does not offer is `invalid_input`.
    /// Package K through the shell (`answerLevel.choose`).
    set_answer_level(args: AnswerLevelArgs) -> Settings;
    /// Returns hardware and tier (ADR 0056), the base equipment (ingestion and the fast
    /// level) with the files still missing and their size, every answer level with
    /// whether it is offered and ready and what it still needs, the level chosen last,
    /// and model folder and free space. Packages L, K and shell.
    setup_state() -> SetupState;
    /// Provides the models of `level` (ADR 0058): `fast` is the base equipment of the
    /// setup, a higher level adds its answer model. Found in the model folder, copied
    /// from an offline package in `source` (chosen with `pick_folder` for
    /// `offline_package`, ADR 0049) or downloaded, resuming a partial download, one role
    /// after the other (parser models, embedding, reranker, fast answer model, the
    /// level's answer model last). Progress goes out as `model_progress` with role and
    /// level. Fails with `network`, `checksum_mismatch` or `storage_full` where that is
    /// the cause. Packages L and K through the shell (`setup.provide`).
    provide_models(args: ProvideModelsArgs) -> ();
    /// The answer levels whose files all lie in an offline package folder chosen with
    /// `pick_folder` for `offline_package` (`<folder>/<entry id>/<file path>`, size as in
    /// `models.toml`, not hashed), from fast to thorough; only levels this machine offers
    /// (ADR 0076). L provides only these, so an incomplete package starts no download.
    /// Package L through the shell (`setup.provideFromPackage`).
    offline_package_levels(args: PathArgs) -> Vec<AnswerLevel>;
    /// Shows the model folder in Explorer or Finder ("Speicherort zeigen" when the disk is
    /// full, ADR 0049). Takes no path: only this folder. Package L.
    reveal_model_folder() -> ();
}

/// Implementations behind the commands.
mod handlers {
    use super::*;

    pub async fn ask(
        _: &AppHandle,
        backend: &Backend,
        args: AskArgs,
        on_event: Channel<AnswerEvent>,
    ) -> ApiResult<Answer> {
        backend
            .ask(args, move |event| {
                // A closed channel means the view is gone; the answer still completes
                // so that the chat turn is saved.
                let _ = on_event.send(event);
            })
            .await
    }

    pub async fn ask_again(
        _: &AppHandle,
        backend: &Backend,
        args: AskAgainArgs,
        on_event: Channel<AnswerEvent>,
    ) -> ApiResult<Answer> {
        backend
            .ask_again(args, move |event| {
                let _ = on_event.send(event);
            })
            .await
    }

    pub async fn offline_package_levels(
        _: &AppHandle,
        backend: &Backend,
        args: PathArgs,
    ) -> ApiResult<Vec<AnswerLevel>> {
        backend.offline_package_levels(args.path).await
    }

    pub async fn cancel(_: &AppHandle, backend: &Backend, args: OperationArgs) -> ApiResult<()> {
        backend.cancel(&args.op);
        Ok(())
    }

    pub async fn list_chats(_: &AppHandle, backend: &Backend) -> ApiResult<Vec<ChatSummary>> {
        backend.list_chats().await
    }

    pub async fn get_chat(
        _: &AppHandle,
        backend: &Backend,
        args: ChatArgs,
    ) -> ApiResult<Option<Chat>> {
        backend.get_chat(&args.id).await
    }

    pub async fn rename_chat(
        _: &AppHandle,
        backend: &Backend,
        args: RenameChatArgs,
    ) -> ApiResult<()> {
        backend.rename_chat(&args.id, &args.title).await
    }

    pub async fn delete_chat(_: &AppHandle, backend: &Backend, args: ChatArgs) -> ApiResult<()> {
        backend.delete_chat(&args.id).await
    }

    pub async fn search(
        _: &AppHandle,
        backend: &Backend,
        args: SearchRequest,
    ) -> ApiResult<Vec<SearchHit>> {
        backend.search(&args).await
    }

    pub async fn list_documents(
        _: &AppHandle,
        backend: &Backend,
    ) -> ApiResult<Vec<DocumentRecord>> {
        backend.list_documents().await
    }

    pub async fn get_document(
        _: &AppHandle,
        backend: &Backend,
        args: DocumentArgs,
    ) -> ApiResult<Option<DocumentDetails>> {
        backend.get_document(&args.id).await
    }

    pub async fn document_file(
        _: &AppHandle,
        backend: &Backend,
        args: DocumentArgs,
    ) -> ApiResult<DocumentFile> {
        backend.document_file(&args.id).await
    }

    /// Runs an opener function on the path of a library document, off the async runtime.
    async fn with_library_path(
        backend: &Backend,
        args: DocumentArgs,
        action: fn(&std::path::Path) -> Result<(), tauri_plugin_opener::Error>,
    ) -> ApiResult<()> {
        let record = backend.library_file(&args.id).await?;
        tauri::async_runtime::spawn_blocking(move || action(&record.path))
            .await
            .map_err(|e| ApiError::new(ErrorKind::Io, e.to_string()))?
            .map_err(|e| ApiError::new(ErrorKind::Io, e.to_string()))
    }

    pub async fn open_document(
        _: &AppHandle,
        backend: &Backend,
        args: DocumentArgs,
    ) -> ApiResult<()> {
        with_library_path(backend, args, |path| {
            tauri_plugin_opener::open_path(path, None::<&str>)
        })
        .await
    }

    pub async fn reveal_document(
        _: &AppHandle,
        backend: &Backend,
        args: DocumentArgs,
    ) -> ApiResult<()> {
        with_library_path(backend, args, |path| {
            tauri_plugin_opener::reveal_item_in_dir(path)
        })
        .await
    }

    pub async fn get_scope(_: &AppHandle, backend: &Backend) -> ApiResult<LibraryScope> {
        backend.get_scope().await
    }

    fn dialog_error(error: impl std::fmt::Display) -> ApiError {
        ApiError::new(ErrorKind::Io, error.to_string())
    }

    /// Dialog titles per language and purpose (`app/src/i18n.json`).
    fn dialog_title(language: kb_core::UiLanguage, purpose: FolderPurpose) -> String {
        let titles: serde_json::Value =
            serde_json::from_str(include_str!("i18n.json")).unwrap_or_default();
        let language = serde_json::to_value(language).unwrap_or_default();
        let purpose = serde_json::to_value(purpose).unwrap_or_default();
        titles[language.as_str().unwrap_or("de")][purpose.as_str().unwrap_or("watch")]
            .as_str()
            .unwrap_or_default()
            .to_owned()
    }

    pub async fn pick_folder(
        app: &AppHandle,
        backend: &Backend,
        args: PickFolderArgs,
    ) -> ApiResult<Option<PathBuf>> {
        let language = backend.get_settings().await?.ui_language;
        let (sender, receiver) = futures::channel::oneshot::channel();
        let mut dialog = app
            .dialog()
            .file()
            .set_title(dialog_title(language, args.purpose));
        if let Some(start) = args.start {
            dialog = dialog.set_directory(start);
        }
        dialog.pick_folder(move |folder| {
            let _ = sender.send(folder);
        });
        let folder = receiver.await.map_err(dialog_error)?;
        let folder = folder
            .map(|f| f.into_path().map_err(dialog_error))
            .transpose()?;
        if let Some(folder) = &folder {
            backend.allow_picked(args.purpose, folder);
        }
        Ok(folder)
    }

    pub async fn add_folder(
        _: &AppHandle,
        backend: &Backend,
        args: PathArgs,
    ) -> ApiResult<LibraryScope> {
        backend.add_folder(&args.path).await
    }

    pub async fn remove_folder(
        _: &AppHandle,
        backend: &Backend,
        args: PathArgs,
    ) -> ApiResult<LibraryScope> {
        backend.remove_folder(&args.path).await
    }

    pub async fn exclude(
        _: &AppHandle,
        backend: &Backend,
        args: PathArgs,
    ) -> ApiResult<LibraryScope> {
        backend.exclude(&args.path).await
    }

    pub async fn include(
        _: &AppHandle,
        backend: &Backend,
        args: PathArgs,
    ) -> ApiResult<LibraryScope> {
        backend.include(&args.path).await
    }

    pub async fn list_removed_exclusions(
        _: &AppHandle,
        backend: &Backend,
    ) -> ApiResult<Vec<PathBuf>> {
        Ok(backend.removed_exclusions())
    }

    pub async fn acknowledge_removed_exclusions(
        _: &AppHandle,
        backend: &Backend,
        args: PathsArgs,
    ) -> ApiResult<()> {
        backend.acknowledge_removed_exclusions(&args.paths);
        Ok(())
    }

    pub async fn sync(
        _: &AppHandle,
        backend: &Backend,
        args: SyncArgs,
    ) -> ApiResult<Vec<FileResult>> {
        backend.sync(args.op, args.mode).await
    }

    pub async fn get_settings(_: &AppHandle, backend: &Backend) -> ApiResult<Settings> {
        backend.get_settings().await
    }

    pub async fn update_settings(
        _: &AppHandle,
        backend: &Backend,
        args: SettingsChange,
    ) -> ApiResult<Settings> {
        backend.update_settings(args).await
    }

    pub async fn set_answer_level(
        _: &AppHandle,
        backend: &Backend,
        args: AnswerLevelArgs,
    ) -> ApiResult<Settings> {
        backend.set_answer_level(args.level).await
    }

    pub async fn setup_state(_: &AppHandle, backend: &Backend) -> ApiResult<SetupState> {
        backend.setup_state().await
    }

    pub async fn provide_models(
        _: &AppHandle,
        backend: &Backend,
        args: ProvideModelsArgs,
    ) -> ApiResult<()> {
        backend
            .provide_models(args.op, args.level, args.source)
            .await
    }

    pub async fn reveal_model_folder(_: &AppHandle, backend: &Backend) -> ApiResult<()> {
        let folder = backend.model_folder()?;
        tauri::async_runtime::spawn_blocking(move || {
            tauri_plugin_opener::reveal_item_in_dir(&folder)
        })
        .await
        .map_err(|e| ApiError::new(ErrorKind::Io, e.to_string()))?
        .map_err(|e| ApiError::new(ErrorKind::Io, e.to_string()))
    }
}
