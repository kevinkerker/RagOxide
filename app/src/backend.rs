//! Command logic over [`KnowledgeBase`] and [`ModelHub`], independent of Tauri.
//!
//! The Tauri layer (`commands.rs`) only forwards to these methods, so they can be tested
//! with the mocks of `kb-core` without a window.
//!
//! The library mirrors the watched folders minus the exclusions (ADR 0025). The backend
//! keeps one folder watcher running over the scope in the settings and restarts it after
//! every change of folders or exclusions; the watcher's first synchronisation brings the
//! library in step.
//!
//! Paths that widen what the app reads (`add_folder`, the offline package of
//! `provide_models`) are accepted only if the native folder dialog returned them in this
//! session for that purpose ([`Backend::allow_picked`], [`FolderPurpose`]). A compromised
//! web view can thus not register an arbitrary folder and read its files through
//! `document_file`, and not reuse a folder the user chose as offline package or for an
//! exclusion. The command line (package N) uses the `KnowledgeBase` facade directly and
//! is not affected.
//!
//! Exclusions the folder watcher drops are kept here until the user interface
//! acknowledges them, because the watcher may drop them at start, before the web view
//! listens (`list_removed_exclusions`).

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError};

use futures::StreamExt;
use kb_core::{
    Answer, AnswerEvent, AnswerLevel, CancellationToken, Chat, ChatId, ChatSummary, DocumentId,
    DocumentRecord, ErrorKind, FileResult, HardwareInfo, HardwareTier, KnowledgeBase,
    MissingModelFile, ModelEntry, ModelHub, ModelManifest, ProgressCallback, ScopeCallback,
    ScopeEvent, SearchHit, SearchRequest, Settings, SyncMode, path_key,
};

use crate::api::{
    ApiError, AskAgainArgs, AskArgs, DocumentDetails, DocumentFile, DocumentFileInfo, Event,
    FolderPurpose, FolderStatus, IngestFinished, IngestProgress, LevelState, LibraryScope,
    ModelProgress, OperationId, ScopeNotice, SettingsChange, SetupState, WatchedFolder,
};

/// Receives the events the backend broadcasts. Must not block.
pub type EventSink = Arc<dyn Fn(Event) + Send + Sync>;

/// Result of a command.
pub type ApiResult<T> = Result<T, ApiError>;

/// Shared state behind all commands. Cloning is cheap.
#[derive(Clone)]
pub struct Backend {
    inner: Arc<Inner>,
}

struct Inner {
    kb: KnowledgeBase,
    hub: Arc<dyn ModelHub>,
    manifest: ModelManifest,
    emit: EventSink,
    operations: Mutex<HashMap<OperationId, CancellationToken>>,
    /// Last scope event per watched folder, by path key.
    folders: Arc<Mutex<HashMap<String, FolderStatus>>>,
    /// Token of the running folder watcher. Held while the watcher emits progress, so a
    /// restart cannot slip between a stopped watcher's last event and `ingest_finished`.
    watcher: Arc<Mutex<Option<CancellationToken>>>,
    /// Folders the folder dialog returned in this session, by purpose and path key.
    picked: Mutex<HashSet<(FolderPurpose, String)>>,
    /// Exclusions the folder watcher dropped, not yet acknowledged, oldest first.
    removed_exclusions: Arc<Mutex<Vec<PathBuf>>>,
}

fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Removes an operation from the registry when the command ends, also on error.
struct Registration<'a> {
    backend: &'a Backend,
    op: OperationId,
    cancel: CancellationToken,
}

impl Drop for Registration<'_> {
    fn drop(&mut self) {
        lock(&self.backend.inner.operations).remove(&self.op);
    }
}

impl Backend {
    /// Backend over a knowledge base and a model hub; `emit` receives all events.
    pub fn new(
        kb: KnowledgeBase,
        hub: Arc<dyn ModelHub>,
        manifest: ModelManifest,
        emit: EventSink,
    ) -> Self {
        Self {
            inner: Arc::new(Inner {
                kb,
                hub,
                manifest,
                emit,
                operations: Mutex::new(HashMap::new()),
                folders: Arc::new(Mutex::new(HashMap::new())),
                watcher: Arc::new(Mutex::new(None)),
                picked: Mutex::new(HashSet::new()),
                removed_exclusions: Arc::new(Mutex::new(Vec::new())),
            }),
        }
    }

    /// The knowledge base behind the commands.
    pub fn knowledge_base(&self) -> &KnowledgeBase {
        &self.inner.kb
    }

    /// Starts the folder watcher over the scope in the settings. Called once at
    /// application start.
    pub fn start(&self) {
        self.restart_watcher();
    }

    fn emit(&self, event: Event) {
        (self.inner.emit)(event);
    }

    /// Records a folder the native folder dialog returned for `purpose`, so that
    /// `add_folder` (`Watch`) or `provide_models` (`OfflinePackage`) accept it. Called only
    /// by the `pick_folder` command (and tests); the web view cannot reach it.
    pub fn allow_picked(&self, purpose: FolderPurpose, path: &Path) {
        if let Ok(key) = path_key(path) {
            lock(&self.inner.picked).insert((purpose, key));
        }
    }

    fn ensure_picked(&self, purpose: FolderPurpose, path: &Path) -> ApiResult<()> {
        let picked =
            path_key(path).is_ok_and(|key| lock(&self.inner.picked).contains(&(purpose, key)));
        if picked {
            Ok(())
        } else {
            Err(ApiError::new(
                ErrorKind::InvalidInput,
                format!(
                    "{} was not chosen in the folder dialog of this session for {purpose:?}",
                    path.display()
                ),
            ))
        }
    }

    fn register(&self, op: OperationId) -> ApiResult<Registration<'_>> {
        let cancel = CancellationToken::new();
        let mut operations = lock(&self.inner.operations);
        if operations.contains_key(&op) {
            return Err(ApiError::new(
                ErrorKind::InvalidInput,
                format!("operation {} is already running", op.0),
            ));
        }
        operations.insert(op.clone(), cancel.clone());
        Ok(Registration {
            backend: self,
            op,
            cancel,
        })
    }

    /// Cancels a running operation. Unknown or finished operations are ignored.
    pub fn cancel(&self, op: &OperationId) {
        if let Some(cancel) = lock(&self.inner.operations).get(op) {
            cancel.cancel();
        }
    }

    // ------------------------------------------------------------------ chat

    /// Answers a question; events go to `on_event`, the result is the final answer
    /// (the same as in `done`). A cancelled or failed answer also ends with `done` and
    /// is saved (`Answer.outcome`, ADR 0045); only an empty question, a duplicate
    /// operation id or a failed save are errors.
    pub async fn ask(
        &self,
        args: AskArgs,
        on_event: impl Fn(AnswerEvent) + Send,
    ) -> ApiResult<Answer> {
        let AskArgs {
            op,
            question,
            chat,
            level,
        } = args;
        let registration = self.register(op)?;
        let stream = self
            .inner
            .kb
            .ask(&question, chat.as_ref(), level, registration.cancel.clone())
            .await?;
        Self::forward(stream, on_event).await
    }

    /// Asks the question of a saved turn again (ADR 0071): history only the complete
    /// turns before it, saved as a new turn at the end. Events and errors as `ask`; an
    /// unknown chat or turn is `invalid_input`.
    pub async fn ask_again(
        &self,
        args: AskAgainArgs,
        on_event: impl Fn(AnswerEvent) + Send,
    ) -> ApiResult<Answer> {
        let AskAgainArgs {
            op,
            chat,
            turn,
            level,
        } = args;
        let registration = self.register(op)?;
        let turn = usize::try_from(turn).unwrap_or(usize::MAX);
        let stream = self
            .inner
            .kb
            .ask_again(&chat, turn, level, registration.cancel.clone())
            .await?;
        Self::forward(stream, on_event).await
    }

    /// Passes the events of an answer on and returns the answer of `done`.
    async fn forward(
        mut stream: kb_core::AnswerStream,
        on_event: impl Fn(AnswerEvent) + Send,
    ) -> ApiResult<Answer> {
        let mut answer = None;
        while let Some(event) = stream.next().await {
            let event = event?;
            if let AnswerEvent::Done { answer: done } = &event {
                answer = Some(done.clone());
            }
            on_event(event);
        }
        answer
            .ok_or_else(|| ApiError::new(ErrorKind::Generator, "answer stream ended without done"))
    }

    /// Lists the stored chats, most recent first.
    pub async fn list_chats(&self) -> ApiResult<Vec<ChatSummary>> {
        Ok(self.inner.kb.store().chats().await?)
    }

    /// Returns a stored chat.
    pub async fn get_chat(&self, id: &ChatId) -> ApiResult<Option<Chat>> {
        Ok(self.inner.kb.store().chat(id).await?)
    }

    /// Renames a chat (ADR 0044); the title is trimmed, empty is `invalid_input`.
    pub async fn rename_chat(&self, id: &ChatId, title: &str) -> ApiResult<()> {
        Ok(self.inner.kb.rename_chat(id, title).await?)
    }

    /// Deletes a chat without undo (ADR 0023).
    pub async fn delete_chat(&self, id: &ChatId) -> ApiResult<()> {
        Ok(self.inner.kb.delete_chat(id).await?)
    }

    /// Searches and ranks.
    pub async fn search(&self, request: &SearchRequest) -> ApiResult<Vec<SearchHit>> {
        Ok(self
            .inner
            .kb
            .search(request, CancellationToken::new())
            .await?)
    }

    // ------------------------------------------------------------------ documents

    /// Lists the library, sorted by path.
    pub async fn list_documents(&self) -> ApiResult<Vec<DocumentRecord>> {
        Ok(self.inner.kb.documents().await?)
    }

    /// Returns library entry and parsed content of a document.
    pub async fn get_document(&self, id: &DocumentId) -> ApiResult<Option<DocumentDetails>> {
        let Some(record) = self.inner.kb.document(id).await? else {
            return Ok(None);
        };
        let content = self.inner.kb.document_content(id).await?;
        Ok(Some(DocumentDetails { record, content }))
    }

    /// The library record of a document whose file may be handed out or opened: the
    /// document must be in the library and its path still in scope (not excluded or
    /// deregistered since the last synchronisation). The path always comes from the
    /// record, never from the caller, so no command reaches files outside the library.
    pub async fn library_file(&self, id: &DocumentId) -> ApiResult<DocumentRecord> {
        let record = self
            .inner
            .kb
            .document(id)
            .await?
            .ok_or_else(|| ApiError::new(ErrorKind::NotFound, format!("document {id}")))?;
        let settings = self.inner.kb.store().settings().await?;
        if !settings.in_scope(&record.path) {
            return Err(ApiError::new(
                ErrorKind::NotFound,
                format!("document {id} is no longer in the library scope"),
            ));
        }
        Ok(record)
    }

    /// Reads the source file of a document, with the hash of the bytes and whether the
    /// file changed since it was ingested.
    pub async fn document_file(&self, id: &DocumentId) -> ApiResult<DocumentFile> {
        let record = self.library_file(id).await?;
        let bytes = tauri::async_runtime::spawn_blocking(move || std::fs::read(record.path))
            .await
            .map_err(|e| ApiError::new(ErrorKind::Io, e.to_string()))?
            .map_err(|e| ApiError::from(kb_core::Error::Io(e)))?;
        let content_hash = kb_core::content_hash(bytes.as_slice()).map_err(kb_core::Error::Io)?;
        let changed = record.content_hash.as_deref() != Some(content_hash.as_str());
        Ok(DocumentFile {
            info: DocumentFileInfo {
                content_hash,
                changed,
            },
            bytes,
        })
    }

    // ------------------------------------------------------------------ library scope

    fn progress_callback(&self, op: Option<OperationId>) -> ProgressCallback {
        let emit = Arc::clone(&self.inner.emit);
        Arc::new(move |event| {
            emit(Event::IngestProgress(IngestProgress {
                op: op.clone(),
                event,
            }));
        })
    }

    fn scope_callback(&self, op: Option<OperationId>) -> ScopeCallback {
        let emit = Arc::clone(&self.inner.emit);
        let folders = Arc::clone(&self.inner.folders);
        let removed = Arc::clone(&self.inner.removed_exclusions);
        let kb = self.inner.kb.clone();
        Arc::new(move |event| {
            match &event {
                ScopeEvent::FolderAvailable { folder } => {
                    if let Ok(key) = path_key(folder) {
                        lock(&folders).insert(key, FolderStatus::Available);
                    }
                }
                ScopeEvent::FolderUnavailable {
                    folder,
                    kind,
                    message,
                } => {
                    if let Ok(key) = path_key(folder) {
                        let status = FolderStatus::Unavailable {
                            kind: *kind,
                            message: message.clone(),
                        };
                        lock(&folders).insert(key, status);
                    }
                }
                ScopeEvent::ExclusionRemoved { path } => {
                    // Kept until acknowledged: the user interface may not listen yet.
                    let mut removed = lock(&removed);
                    if !removed.contains(path) {
                        removed.push(path.clone());
                    }
                    drop(removed);
                    // The ingestor changed the settings; tell the user interface.
                    let kb = kb.clone();
                    let emit = Arc::clone(&emit);
                    tauri::async_runtime::spawn(async move {
                        if let Ok(settings) = kb.store().settings().await {
                            emit(Event::SettingsChanged(settings));
                        }
                    });
                }
            }
            emit(Event::ScopeChanged(ScopeNotice {
                op: op.clone(),
                event,
            }));
        })
    }

    /// (Re)starts the folder watcher over the current scope. Its first synchronisation
    /// also removes what is no longer in scope, so it runs even without folders. A
    /// stopped watcher's run ends with `ingest_finished` (`op` null); its late progress
    /// events are dropped, so the status bar never keeps a run nobody finishes.
    fn restart_watcher(&self) {
        let cancel = CancellationToken::new();
        {
            let mut current = lock(&self.inner.watcher);
            if let Some(previous) = current.replace(cancel.clone()) {
                previous.cancel();
                self.emit(Event::IngestFinished(IngestFinished { op: None }));
            }
        }
        let kb = self.inner.kb.clone();
        let emit = Arc::clone(&self.inner.emit);
        let watcher = Arc::clone(&self.inner.watcher);
        let progress: ProgressCallback = {
            let emit = Arc::clone(&emit);
            let watcher = Arc::clone(&watcher);
            let cancel = cancel.clone();
            Arc::new(move |event| {
                let _current = lock(&watcher);
                if !cancel.is_cancelled() {
                    emit(Event::IngestProgress(IngestProgress { op: None, event }));
                }
            })
        };
        // Scope events of a stopped watcher are dropped too: a late `folder_unavailable`
        // of a folder just deregistered would otherwise stay in the status bar.
        let scope: ScopeCallback = {
            let inner = self.scope_callback(None);
            let watcher = Arc::clone(&watcher);
            let cancel = cancel.clone();
            Arc::new(move |event| {
                let _current = lock(&watcher);
                if !cancel.is_cancelled() {
                    inner(event);
                }
            })
        };
        tauri::async_runtime::spawn(async move {
            let result = kb.watch(progress, scope, cancel.clone()).await;
            let _current = lock(&watcher);
            if cancel.is_cancelled() {
                // Restarted: `restart_watcher` already reported the end of the run.
                return;
            }
            if let Err(error) = result {
                emit(Event::WatchFailed(error.into()));
            }
            emit(Event::IngestFinished(IngestFinished { op: None }));
        });
    }

    fn scope_of(&self, settings: &Settings) -> LibraryScope {
        let statuses = lock(&self.inner.folders);
        LibraryScope {
            folders: settings
                .watched_folders
                .iter()
                .map(|path| WatchedFolder {
                    path: path.clone(),
                    status: path_key(path)
                        .ok()
                        .and_then(|key| statuses.get(&key).cloned())
                        .unwrap_or(FolderStatus::Unknown),
                })
                .collect(),
            excluded: settings.excluded.clone(),
        }
    }

    /// Watched folders with their state and the exclusion list.
    pub async fn get_scope(&self) -> ApiResult<LibraryScope> {
        let settings = self.inner.kb.store().settings().await?;
        Ok(self.scope_of(&settings))
    }

    /// Applies a scope change: broadcasts the settings, restarts the watcher and returns
    /// the new scope.
    fn scope_changed(&self, settings: Settings) -> LibraryScope {
        let scope = self.scope_of(&settings);
        self.emit(Event::SettingsChanged(settings));
        self.restart_watcher();
        scope
    }

    /// Registers a watched folder chosen in the folder dialog; its files are ingested by
    /// the restarted watcher.
    pub async fn add_folder(&self, folder: &Path) -> ApiResult<LibraryScope> {
        self.ensure_picked(FolderPurpose::Watch, folder)?;
        let settings = self.inner.kb.add_folder(folder).await?;
        Ok(self.scope_changed(settings))
    }

    /// Deregisters a watched folder; its documents disappear unless another watched
    /// folder contains them.
    pub async fn remove_folder(&self, folder: &Path) -> ApiResult<LibraryScope> {
        let settings = self.inner.kb.remove_folder(folder).await?;
        if let Ok(key) = path_key(folder) {
            lock(&self.inner.folders).remove(&key);
        }
        Ok(self.scope_changed(settings))
    }

    /// Excludes a file or subfolder inside a watched folder.
    pub async fn exclude(&self, path: &Path) -> ApiResult<LibraryScope> {
        let settings = self.inner.kb.exclude(path).await?;
        Ok(self.scope_changed(settings))
    }

    /// Exclusions the folder watcher dropped because their path is gone, not yet
    /// acknowledged; also those dropped before the user interface listened.
    pub fn removed_exclusions(&self) -> Vec<PathBuf> {
        lock(&self.inner.removed_exclusions).clone()
    }

    /// The user has seen these dropped exclusions; they are no longer listed.
    pub fn acknowledge_removed_exclusions(&self, paths: &[PathBuf]) {
        let keys: Vec<String> = paths.iter().filter_map(|p| path_key(p).ok()).collect();
        lock(&self.inner.removed_exclusions)
            .retain(|p| path_key(p).map_or(true, |key| !keys.contains(&key)));
    }

    /// Takes an entry off the exclusion list.
    pub async fn include(&self, path: &Path) -> ApiResult<LibraryScope> {
        let settings = self.inner.kb.include(path).await?;
        Ok(self.scope_changed(settings))
    }

    /// Runs one synchronisation now; progress and scope events carry `op`. `mode` decides
    /// which files are processed again (`RetryFailed` for "Erneut versuchen", `Full` for
    /// "Integrität prüfen", ADR 0053, 0072). Ends with `ingest_finished`, also when
    /// cancelled or failed.
    pub async fn sync(&self, op: OperationId, mode: SyncMode) -> ApiResult<Vec<FileResult>> {
        let registration = self.register(op.clone())?;
        let progress = self.progress_callback(Some(op.clone()));
        let scope = self.scope_callback(Some(op.clone()));
        let result = self
            .inner
            .kb
            .sync(progress, scope, mode, registration.cancel.clone())
            .await;
        self.emit(Event::IngestFinished(IngestFinished { op: Some(op) }));
        Ok(result?)
    }

    // ------------------------------------------------------------------ settings

    /// Returns the settings.
    pub async fn get_settings(&self) -> ApiResult<Settings> {
        Ok(self.inner.kb.store().settings().await?)
    }

    /// Changes language and colour scheme atomically, leaving the library scope and the
    /// answer level untouched, and broadcasts `settings_changed`.
    pub async fn update_settings(&self, change: SettingsChange) -> ApiResult<Settings> {
        let settings = self
            .inner
            .kb
            .store()
            .update_settings(Box::new(move |settings| {
                let SettingsChange {
                    ui_language,
                    color_scheme,
                } = change;
                settings.ui_language = ui_language;
                settings.color_scheme = color_scheme;
                Ok(())
            }))
            .await?;
        self.emit(Event::SettingsChanged(settings.clone()));
        Ok(settings)
    }

    /// Remembers the level the user chose (ADR 0060) and broadcasts `settings_changed`.
    /// A level the hardware tier does not offer is `invalid_input`. A question does not
    /// change it: the facade leaves it alone, so the MCP server cannot either.
    pub async fn set_answer_level(&self, level: AnswerLevel) -> ApiResult<Settings> {
        let (_, tier) = self.tier()?;
        if !self.inner.manifest.levels(tier).contains(&level) {
            return Err(ApiError::new(
                ErrorKind::InvalidInput,
                format!("tier {tier:?} does not offer level {level:?}"),
            ));
        }
        let settings = self
            .inner
            .kb
            .store()
            .update_settings(Box::new(move |settings| {
                settings.answer_level = level;
                Ok(())
            }))
            .await?;
        self.emit(Event::SettingsChanged(settings.clone()));
        Ok(settings)
    }

    // ------------------------------------------------------------------ models

    fn tier(&self) -> ApiResult<(HardwareInfo, HardwareTier)> {
        let hardware = self.inner.hub.hardware()?;
        Ok((
            hardware,
            HardwareTier::from_total_ram(hardware.total_ram_bytes),
        ))
    }

    /// Sizes of `missing` files: the sum of the known ones and the number of unknown ones.
    fn download_size(&self, missing: &[MissingModelFile]) -> (u64, u32) {
        let sizes: Vec<Option<u64>> = missing
            .iter()
            .map(|m| {
                self.inner
                    .manifest
                    .models
                    .iter()
                    .find(|entry| entry.id == m.model_id)
                    .and_then(|entry| entry.files.iter().find(|f| f.path == m.file))
                    .and_then(|file| file.size_bytes)
            })
            .collect();
        let unknown =
            u32::try_from(sizes.iter().filter(|s| s.is_none()).count()).unwrap_or(u32::MAX);
        (sizes.iter().flatten().sum(), unknown)
    }

    /// Hardware, the base equipment (ingestion and the fast level, ADR 0058) with what is
    /// missing, and the state of every answer level, for the setup and the level choice.
    pub async fn setup_state(&self) -> ApiResult<SetupState> {
        let (hardware, tier) = self.tier()?;
        let manifest = &self.inner.manifest;
        let base: Vec<ModelEntry> = manifest.models_for(tier, AnswerLevel::Fast)?;
        let missing = self.inner.hub.missing(tier, AnswerLevel::Fast).await?;
        let (download_bytes, unknown_sizes) = self.download_size(&missing);
        let mut levels = Vec::new();
        for level in AnswerLevel::ALL {
            let model = manifest.answer_model(tier, level);
            let level_missing = match model {
                Some(_) => self.inner.hub.missing(tier, level).await?,
                None => Vec::new(),
            };
            let (bytes, unknown) = self.download_size(&level_missing);
            levels.push(LevelState {
                level,
                offered: model.is_some(),
                ready: model.is_some() && level_missing.is_empty(),
                model,
                missing: level_missing,
                download_bytes: bytes,
                unknown_sizes: unknown,
            });
        }
        let settings = self.inner.kb.store().settings().await?;
        // A level the machine does not offer counts as fast (ADR 0074); the setting stays.
        let answer_level = manifest.effective_level(settings.answer_level, tier);
        Ok(SetupState {
            hardware,
            tier,
            models_ready: missing.is_empty(),
            models: base,
            missing,
            download_bytes,
            unknown_sizes,
            levels,
            answer_level,
            storage: self.inner.hub.storage()?,
        })
    }

    /// The levels whose files all lie in the offline package `folder` (ADR 0076), from
    /// fast to thorough, among those this machine offers: every file of
    /// `ModelManifest::models_for` at `<folder>/<entry id>/<file path>`, with the size of
    /// `models.toml` where it states one. Nothing is hashed; `provide_models` verifies the
    /// checksums when it copies. `folder` must have been chosen for `offline_package`.
    pub async fn offline_package_levels(&self, folder: PathBuf) -> ApiResult<Vec<AnswerLevel>> {
        self.ensure_picked(FolderPurpose::OfflinePackage, &folder)?;
        let (_, tier) = self.tier()?;
        let manifest = self.inner.manifest.clone();
        tauri::async_runtime::spawn_blocking(move || {
            let complete = |level: AnswerLevel| -> bool {
                manifest.models_for(tier, level).is_ok_and(|models| {
                    models.iter().all(|entry| {
                        entry.files.iter().all(|file| {
                            std::fs::metadata(folder.join(&entry.id).join(&file.path)).is_ok_and(
                                |meta| {
                                    meta.is_file()
                                        && file.size_bytes.is_none_or(|size| meta.len() == size)
                                },
                            )
                        })
                    })
                })
            };
            manifest
                .levels(tier)
                .into_iter()
                .filter(|level| complete(*level))
                .collect()
        })
        .await
        .map_err(|e| ApiError::new(ErrorKind::Io, e.to_string()))
    }

    /// The model folder, for "Speicherort zeigen" (ADR 0049). The path comes from the
    /// model hub, never from the caller.
    pub fn model_folder(&self) -> ApiResult<PathBuf> {
        Ok(self.inner.hub.storage()?.dir)
    }

    /// Provides the models of `level` on this machine's tier (ADR 0058): `fast` is the
    /// base equipment of the setup, a higher level adds its answer model. From `source`
    /// (an offline package chosen in the folder dialog, ADR 0049) where it has them,
    /// otherwise by download; progress goes out as `model_progress` with the level.
    pub async fn provide_models(
        &self,
        op: OperationId,
        level: AnswerLevel,
        source: Option<PathBuf>,
    ) -> ApiResult<()> {
        if let Some(source) = &source {
            self.ensure_picked(FolderPurpose::OfflinePackage, source)?;
        }
        let (_, tier) = self.tier()?;
        let registration = self.register(op.clone())?;
        let emit = Arc::clone(&self.inner.emit);
        let progress: kb_core::ModelProgressCallback = Arc::new(move |event| {
            emit(Event::ModelProgress(ModelProgress {
                op: op.clone(),
                level,
                event,
            }));
        });
        self.inner
            .hub
            .provide(tier, level, source, progress, registration.cancel.clone())
            .await?;
        Ok(())
    }
}
