//! Command logic over the mocks of `kb-core`, without a window.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

use kb_core::mock::MockFiles;
use kb_core::{
    AnswerEvent, AnswerLevel, AnswerOutcome, AnswerPhase, ChatId, ColorScheme, DocumentId,
    DocumentStatus, ErrorKind, FileStatus, HardwareTier, ModelFileStatus, ModelRole, ScopeEvent,
    SearchMode, SearchRequest, SyncMode, UiLanguage,
};
use ragoxide_app::api::{
    AskAgainArgs, AskArgs, Event, FolderPurpose, FolderStatus, IngestFinished, OperationId,
    SettingsChange,
};
use ragoxide_app::backend::{Backend, EventSink};
use ragoxide_app::mock::{self, MockSetup};
use tauri::async_runtime::block_on;

struct Setup {
    backend: Backend,
    files: Arc<MockFiles>,
    events: Arc<Mutex<Vec<Event>>>,
}

fn setup() -> Setup {
    setup_with(&mock::MockConfig::default())
}

/// A seeded backend; tests start with the models of `config.provided` (default: fast).
fn setup_with(config: &mock::MockConfig) -> Setup {
    let events = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&events);
    let emit: EventSink = Arc::new(move |event| sink.lock().unwrap().push(event));
    let MockSetup { backend, files } = block_on(mock::seeded_backend(emit, config)).unwrap();
    Setup {
        backend,
        files,
        events,
    }
}

impl Setup {
    fn events(&self) -> Vec<Event> {
        self.events.lock().unwrap().clone()
    }

    /// Waits until the restarted folder watcher has emitted an event matching `found`.
    fn wait_for(&self, found: impl Fn(&Event) -> bool) {
        for _ in 0..200 {
            if self.events().iter().any(&found) {
                return;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        panic!("event not emitted: {:#?}", self.events());
    }

    fn document_paths(&self) -> Vec<PathBuf> {
        block_on(self.backend.list_documents())
            .unwrap()
            .into_iter()
            .map(|d| d.path)
            .collect()
    }
}

fn op(id: &str) -> OperationId {
    OperationId(id.into())
}

const QUESTION: &str = "Wie oft wird ein Gerät geprüft?";
const FOLDER: &str = MockFiles::FIXTURE_FOLDER;

#[test]
fn ask_streams_events_returns_the_answer_and_saves_the_chat() {
    let Setup { backend, .. } = setup();
    let received = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&received);
    let answer = block_on(backend.ask(
        AskArgs {
            op: op("a1"),
            question: QUESTION.into(),
            chat: Some(ChatId::new("chat-1")),
            level: AnswerLevel::Fast,
        },
        move |event| sink.lock().unwrap().push(event),
    ))
    .unwrap();

    let events = received.lock().unwrap();
    // A first question (ADR 0054, case A): searching, sources, starting_model,
    // reading_sources one or more times, writing, deltas, done.
    let mut kinds: Vec<String> = events
        .iter()
        .map(|e| match e {
            AnswerEvent::Status { phase, .. } => format!("{phase:?}"),
            AnswerEvent::Sources { .. } => "Sources".into(),
            AnswerEvent::Delta { .. } => "Delta".into(),
            AnswerEvent::Done { .. } => "Done".into(),
        })
        .collect();
    kinds.dedup();
    assert_eq!(
        kinds,
        [
            "Searching",
            "Sources",
            "StartingModel",
            "ReadingSources",
            "Writing",
            "Delta",
            "Done"
        ]
    );
    assert!(matches!(&events[1], AnswerEvent::Sources { hits } if !hits.is_empty()));
    assert_eq!(
        events.last(),
        Some(&AnswerEvent::Done {
            answer: answer.clone()
        })
    );
    assert!(!answer.citations.is_empty());

    let chats = block_on(backend.list_chats()).unwrap();
    assert_eq!(chats.len(), 1);
    assert_eq!(chats[0].title, QUESTION);
    let chat = block_on(backend.get_chat(&ChatId::new("chat-1")))
        .unwrap()
        .unwrap();
    assert_eq!(chat.turns[0].answer, answer);
}

#[test]
fn delete_chat_removes_it_and_tolerates_unknown_ids() {
    let Setup { backend, .. } = setup();
    block_on(backend.ask(
        AskArgs {
            op: op("a"),
            question: QUESTION.into(),
            chat: Some(ChatId::new("c")),
            level: AnswerLevel::Fast,
        },
        |_| {},
    ))
    .unwrap();
    block_on(backend.delete_chat(&ChatId::new("c"))).unwrap();
    assert!(block_on(backend.list_chats()).unwrap().is_empty());
    block_on(backend.delete_chat(&ChatId::new("never"))).unwrap();
}

#[test]
fn a_cancelled_answer_ends_with_done_is_saved_and_frees_the_id() {
    let Setup { backend, .. } = setup();
    let canceller = backend.clone();
    let received = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&received);
    let answer = block_on(backend.ask(
        AskArgs {
            op: op("a2"),
            question: QUESTION.into(),
            chat: Some(ChatId::new("chat-2")),
            level: AnswerLevel::Fast,
        },
        move |event| {
            // Cancel as soon as the answer starts writing.
            if matches!(event, AnswerEvent::Delta { .. }) {
                canceller.cancel(&OperationId("a2".into()));
            }
            sink.lock().unwrap().push(event);
        },
    ))
    .unwrap();
    assert_eq!(answer.outcome, AnswerOutcome::Cancelled);
    assert!(matches!(
        received.lock().unwrap().last(),
        Some(AnswerEvent::Done { .. })
    ));
    // ADR 0045: the question and the text so far stay in the chat.
    let chat = block_on(backend.get_chat(&ChatId::new("chat-2")))
        .unwrap()
        .unwrap();
    assert_eq!(chat.turns[0].answer.outcome, AnswerOutcome::Cancelled);
    block_on(backend.ask(
        AskArgs {
            op: op("a2"),
            question: QUESTION.into(),
            chat: None,
            level: AnswerLevel::Fast,
        },
        |_| {},
    ))
    .unwrap();
}

#[test]
fn duplicate_operation_ids_and_empty_questions_are_invalid_input() {
    let Setup { backend, .. } = setup();
    let error = block_on(backend.ask(
        AskArgs {
            op: op("a3"),
            question: "  ".into(),
            chat: None,
            level: AnswerLevel::Fast,
        },
        |_| {},
    ))
    .unwrap_err();
    assert_eq!(error.kind, ErrorKind::InvalidInput);

    let inner = backend.clone();
    block_on(backend.ask(
        AskArgs {
            op: op("same"),
            question: QUESTION.into(),
            chat: None,
            level: AnswerLevel::Fast,
        },
        // The mock futures need no runtime, so a plain executor can run them here.
        move |_| {
            let result = futures::executor::block_on(inner.sync(op("same"), SyncMode::Normal));
            assert_eq!(result.unwrap_err().kind, ErrorKind::InvalidInput);
        },
    ))
    .unwrap();
}

#[test]
fn library_search_and_document() {
    let Setup { backend, .. } = setup();
    let documents = block_on(backend.list_documents()).unwrap();
    assert_eq!(documents.len(), 7);
    assert!(
        documents
            .iter()
            .all(|d| matches!(d.status, DocumentStatus::Indexed { .. }))
    );
    let hits = block_on(backend.search(&SearchRequest {
        text: QUESTION.into(),
        limit: 3,
        documents: None,
        mode: SearchMode::Hybrid,
    }))
    .unwrap();
    assert!(!hits.is_empty() && hits.len() <= 3);

    let id = DocumentId::new("doc-pruefordnung");
    let details = block_on(backend.get_document(&id)).unwrap().unwrap();
    assert_eq!(details.record.id, id);
    assert_eq!(details.content.unwrap().pages.len(), 2);
    assert!(
        block_on(backend.get_document(&DocumentId::new("nope")))
            .unwrap()
            .is_none()
    );
    // The fixture files exist only on the mock disk.
    assert_eq!(
        block_on(backend.document_file(&id)).unwrap_err().kind,
        ErrorKind::Io
    );
    assert_eq!(
        block_on(backend.document_file(&DocumentId::new("nope")))
            .unwrap_err()
            .kind,
        ErrorKind::NotFound
    );
}

#[test]
fn exclude_and_include_change_the_library_through_the_watcher() {
    let setup = setup();
    let backend = &setup.backend;
    let excluded = PathBuf::from(r"C:\Ablage\notizen.md");
    let scope = block_on(backend.exclude(&excluded)).unwrap();
    assert_eq!(scope.excluded, vec![excluded.clone()]);
    setup.wait_for(|e| {
        matches!(e, Event::IngestProgress(p)
            if p.op.is_none() && matches!(p.event.status, FileStatus::Removed { .. }))
    });
    assert!(!setup.document_paths().contains(&excluded));
    assert!(
        setup.events().iter().any(
            |e| matches!(e, Event::SettingsChanged(s) if s.excluded == vec![excluded.clone()])
        )
    );

    let scope = block_on(backend.include(&excluded)).unwrap();
    assert!(scope.excluded.is_empty());
    setup.wait_for(|e| {
        matches!(e, Event::IngestProgress(p)
            if p.event.path == excluded && matches!(p.event.status, FileStatus::Indexed { .. }))
    });
    assert!(setup.document_paths().contains(&excluded));

    // Outside every watched folder, or the folder itself: invalid input.
    for path in [r"C:\Anderswo\x.pdf", FOLDER] {
        let error = block_on(backend.exclude(Path::new(path))).unwrap_err();
        assert_eq!(error.kind, ErrorKind::InvalidInput);
    }
}

#[test]
fn folders_are_added_and_removed_and_report_their_state() {
    let setup = setup();
    let backend = &setup.backend;
    let other = PathBuf::from(r"D:\Projekte");
    setup.files.add_file(&other.join("faq.html")).unwrap();
    backend.allow_picked(FolderPurpose::Watch, &other);
    let scope = block_on(backend.add_folder(&other)).unwrap();
    assert_eq!(scope.folders.len(), 2);
    setup.wait_for(|e| {
        matches!(e, Event::IngestProgress(p)
            if p.event.path == other.join("faq.html")
                && matches!(p.event.status, FileStatus::Indexed { .. }))
    });
    let scope = block_on(backend.get_scope()).unwrap();
    assert!(
        scope
            .folders
            .iter()
            .all(|f| f.status == FolderStatus::Available)
    );

    block_on(backend.remove_folder(Path::new(FOLDER))).unwrap();
    setup.wait_for(|e| {
        matches!(e, Event::IngestProgress(p)
            if p.event.path == Path::new(FOLDER).join("budget.xlsx")
                && matches!(p.event.status, FileStatus::Removed { .. }))
    });
    assert_eq!(setup.document_paths(), vec![other.join("faq.html")]);
    assert_eq!(block_on(backend.get_scope()).unwrap().folders.len(), 1);
}

#[test]
fn an_unreachable_folder_is_reported_and_keeps_its_documents() {
    let setup = setup();
    setup.files.set_reachable(Path::new(FOLDER), false).unwrap();
    let results = block_on(setup.backend.sync(op("s1"), SyncMode::Normal)).unwrap();
    assert!(results.is_empty());
    assert!(setup.events().iter().any(|e| matches!(e,
        Event::ScopeChanged(n) if n.op == Some(op("s1"))
            && matches!(&n.event, ScopeEvent::FolderUnavailable { folder, .. }
                if folder == Path::new(FOLDER)))));
    let scope = block_on(setup.backend.get_scope()).unwrap();
    assert!(matches!(
        scope.folders[0].status,
        FolderStatus::Unavailable { .. }
    ));
    assert_eq!(setup.document_paths().len(), 7);
}

#[test]
fn update_settings_keeps_the_scope_and_broadcasts() {
    let setup = setup();
    let backend = &setup.backend;
    block_on(backend.exclude(Path::new(r"C:\Ablage\faq.html"))).unwrap();
    let settings = block_on(backend.update_settings(SettingsChange {
        ui_language: UiLanguage::English,
        color_scheme: ColorScheme::Dark,
    }))
    .unwrap();
    assert_eq!(settings.answer_level, AnswerLevel::Fast);
    assert_eq!(settings.watched_folders, vec![PathBuf::from(FOLDER)]);
    assert_eq!(settings.excluded.len(), 1);
    assert_eq!(settings.color_scheme, ColorScheme::Dark);
    assert_eq!(block_on(backend.get_settings()).unwrap(), settings);
    assert!(setup.events().contains(&Event::SettingsChanged(settings)));
}

#[test]
fn setup_state_names_the_base_equipment_and_every_level() {
    let setup = setup_with(&mock::MockConfig {
        provided: Vec::new(),
        ..mock::MockConfig::default()
    });
    let backend = &setup.backend;
    let state = block_on(backend.setup_state()).unwrap();
    assert_eq!(state.tier, HardwareTier::Supported);
    assert!(!state.models_ready);
    let files: usize = state.models.iter().map(|m| m.files.len()).sum();
    assert_eq!(state.missing.len(), files);
    // The base equipment in the order it is provided, the fast answer model last.
    let roles: Vec<ModelRole> = state.models.iter().map(|m| m.role).collect();
    let expected: Vec<ModelRole> = ModelRole::PROVIDE_ORDER
        .into_iter()
        .filter(|r| roles.contains(r))
        .collect();
    assert_eq!(roles, expected);
    assert_eq!(state.models.last().unwrap().level, Some(AnswerLevel::Fast));
    let known: u64 = state
        .models
        .iter()
        .flat_map(|m| &m.files)
        .filter_map(|f| f.size_bytes)
        .sum();
    assert_eq!(state.download_bytes, known);
    // Three levels, all offered on a supported machine, none ready.
    let levels: Vec<AnswerLevel> = state.levels.iter().map(|l| l.level).collect();
    assert_eq!(levels, AnswerLevel::ALL);
    assert!(state.levels.iter().all(|l| l.offered && !l.ready));
    assert_eq!(state.answer_level, AnswerLevel::Fast);

    // The setup provides the fast level; the higher levels need only their answer model.
    block_on(backend.provide_models(op("m1"), AnswerLevel::Fast, None)).unwrap();
    assert!(setup.events().iter().any(|e| matches!(e,
        Event::ModelProgress(p) if p.op == op("m1") && p.level == AnswerLevel::Fast
            && p.event.status == ModelFileStatus::Ready)));
    let state = block_on(backend.setup_state()).unwrap();
    assert!(state.models_ready);
    assert!(state.missing.is_empty());
    assert!(state.levels[0].ready);
    for level in &state.levels[1..] {
        assert!(!level.ready);
        assert!(!level.missing.is_empty());
        assert!(level.missing.iter().all(|m| m.role == ModelRole::Generator
            && Some(&m.model_id) == level.model.as_ref().map(|e| &e.id)));
        assert!(level.download_bytes > 0);
    }
}

#[test]
fn a_level_without_its_model_is_refused_with_its_own_error() {
    let Setup { backend, .. } = setup();
    let ask = |level| {
        block_on(backend.ask(
            AskArgs {
                op: op("level"),
                question: QUESTION.into(),
                chat: None,
                level,
            },
            |_| {},
        ))
    };
    let error = ask(AnswerLevel::Balanced).unwrap_err();
    assert_eq!(error.kind, ErrorKind::ModelMissing);
    // Provided on demand (ADR 0058), the same question goes through and records its level.
    block_on(backend.provide_models(op("m"), AnswerLevel::Balanced, None)).unwrap();
    assert_eq!(
        ask(AnswerLevel::Balanced).unwrap().level,
        AnswerLevel::Balanced
    );
    assert_eq!(ask(AnswerLevel::Fast).unwrap().level, AnswerLevel::Fast);
    assert_eq!(
        ask(AnswerLevel::Thorough).unwrap_err().kind,
        ErrorKind::ModelMissing
    );
}

#[test]
fn the_chosen_level_is_remembered_and_a_question_does_not_change_it() {
    let setup = setup();
    let backend = &setup.backend;
    let settings = block_on(backend.set_answer_level(AnswerLevel::Thorough)).unwrap();
    assert_eq!(settings.answer_level, AnswerLevel::Thorough);
    assert!(setup.events().contains(&Event::SettingsChanged(settings)));
    block_on(backend.ask(
        AskArgs {
            op: op("q"),
            question: QUESTION.into(),
            chat: None,
            level: AnswerLevel::Fast,
        },
        |_| {},
    ))
    .unwrap();
    assert_eq!(
        block_on(backend.get_settings()).unwrap().answer_level,
        AnswerLevel::Thorough
    );
    assert_eq!(
        block_on(backend.setup_state()).unwrap().answer_level,
        AnswerLevel::Thorough
    );
}

#[test]
fn dropped_exclusions_are_kept_until_acknowledged() {
    // The watcher drops an exclusion whose path is gone, here before anyone listens.
    let setup = setup();
    let gone = PathBuf::from(r"C:\Ablage\entfernt");
    block_on(setup.backend.exclude(&gone)).unwrap();
    setup.wait_for(|e| {
        matches!(e, Event::ScopeChanged(n)
            if matches!(&n.event, ScopeEvent::ExclusionRemoved { path } if *path == gone))
    });
    assert_eq!(setup.backend.removed_exclusions(), vec![gone.clone()]);
    // Acknowledged in another spelling of the same path.
    setup
        .backend
        .acknowledge_removed_exclusions(&[PathBuf::from(r"c:\ablage\.\Entfernt")]);
    assert!(setup.backend.removed_exclusions().is_empty());
}

#[test]
fn the_window_theme_follows_the_colour_scheme() {
    use ragoxide_app::window_theme;
    use tauri::Theme;
    assert_eq!(window_theme(ColorScheme::System), None);
    assert_eq!(window_theme(ColorScheme::Light), Some(Theme::Light));
    assert_eq!(window_theme(ColorScheme::Dark), Some(Theme::Dark));
}

#[test]
fn rename_chat_trims_and_rejects_empty_titles() {
    let Setup { backend, .. } = setup();
    block_on(backend.ask(
        AskArgs {
            op: op("r"),
            question: QUESTION.into(),
            chat: Some(ChatId::new("c")),
            level: AnswerLevel::Fast,
        },
        |_| {},
    ))
    .unwrap();
    block_on(backend.rename_chat(&ChatId::new("c"), "  Prüffristen  ")).unwrap();
    assert_eq!(
        block_on(backend.list_chats()).unwrap()[0].title,
        "Prüffristen"
    );
    let empty = block_on(backend.rename_chat(&ChatId::new("c"), "   ")).unwrap_err();
    assert_eq!(empty.kind, ErrorKind::InvalidInput);
    let unknown = block_on(backend.rename_chat(&ChatId::new("nope"), "x")).unwrap_err();
    assert_eq!(unknown.kind, ErrorKind::NotFound);
}

#[test]
fn only_documents_in_the_library_scope_are_handed_out() {
    let setup = setup();
    let backend = &setup.backend;
    let notes = DocumentId::new("doc-notizen");
    assert!(block_on(backend.library_file(&notes)).is_ok());
    // An id that is a path, or any unknown id, is not a library document.
    for id in [r"C:\Windows\notepad.exe", "doc-nope"] {
        let error = block_on(backend.library_file(&DocumentId::new(id))).unwrap_err();
        assert_eq!(error.kind, ErrorKind::NotFound);
    }
    // Excluded but not yet synchronised: the record still exists, the file is refused.
    let path = block_on(backend.library_file(&notes)).unwrap().path;
    // Stop the watcher from removing the record before the check.
    setup.files.set_reachable(Path::new(FOLDER), false).unwrap();
    block_on(backend.exclude(&path)).unwrap();
    let error = block_on(backend.library_file(&notes)).unwrap_err();
    assert_eq!(error.kind, ErrorKind::NotFound);
}

#[test]
fn document_file_reports_the_hash_and_a_change_since_ingestion() {
    let setup = setup();
    let dir = std::env::temp_dir().join(format!("ragoxide-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    // The mock parser knows documents by file name; the mock disk lists the real file.
    let file = dir.join("notizen.md");
    std::fs::write(&file, "# Notizen\n").unwrap();
    setup.files.add_file(&file).unwrap();
    setup.backend.allow_picked(FolderPurpose::Watch, &dir);
    block_on(setup.backend.add_folder(&dir)).unwrap();
    block_on(setup.backend.sync(op("s"), SyncMode::Normal)).unwrap();
    let record = block_on(setup.backend.list_documents())
        .unwrap()
        .into_iter()
        .find(|d| d.path == file)
        .unwrap();
    let delivered = block_on(setup.backend.document_file(&record.id)).unwrap();
    assert_eq!(delivered.bytes, b"# Notizen\n");
    assert_eq!(
        delivered.info.content_hash,
        kb_core::content_hash(&b"# Notizen\n"[..]).unwrap()
    );
    // The mock ingestor stores a placeholder hash, so the real bytes count as changed.
    assert!(delivered.info.changed);
    let encoded = delivered.encode();
    let length = u32::from_le_bytes(encoded[..4].try_into().unwrap()) as usize;
    let header: serde_json::Value = serde_json::from_slice(&encoded[4..4 + length]).unwrap();
    assert_eq!(header["changed"], true);
    assert_eq!(&encoded[4 + length..], b"# Notizen\n");
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn setup_state_names_the_model_folder_and_free_space() {
    let Setup { backend, .. } = setup();
    let state = block_on(backend.setup_state()).unwrap();
    assert!(state.storage.dir.ends_with("ragoxide-models"));
    assert!(state.storage.free_bytes > 0);
    assert_eq!(backend.model_folder().unwrap(), state.storage.dir);
}

#[test]
fn only_folders_from_the_dialog_for_that_purpose_are_accepted() {
    let setup = setup();
    let backend = &setup.backend;
    let other = PathBuf::from(r"D:\Projekte");
    let error = block_on(backend.add_folder(&other)).unwrap_err();
    assert_eq!(error.kind, ErrorKind::InvalidInput);
    // Chosen as offline package or for an exclusion: not released for watching.
    backend.allow_picked(FolderPurpose::OfflinePackage, &other);
    backend.allow_picked(FolderPurpose::Exclude, &other);
    let error = block_on(backend.add_folder(&other)).unwrap_err();
    assert_eq!(error.kind, ErrorKind::InvalidInput);
    // The same folder, written differently, counts as chosen once the dialog returned it.
    backend.allow_picked(FolderPurpose::Watch, &other);
    block_on(backend.add_folder(Path::new(r"d:\projekte\."))).unwrap();

    let package = PathBuf::from(r"E:\RagOxide-Offline");
    backend.allow_picked(FolderPurpose::Watch, &package);
    let provide = |op_id, source| {
        block_on(backend.provide_models(op(op_id), AnswerLevel::Fast, Some(source)))
    };
    let error = provide("m", package.clone()).unwrap_err();
    assert_eq!(error.kind, ErrorKind::InvalidInput);
    backend.allow_picked(FolderPurpose::OfflinePackage, &package);
    provide("m2", package).unwrap();
}

#[test]
fn every_synchronisation_run_ends_with_ingest_finished() {
    let setup = setup();
    // An explicit run, completed.
    block_on(setup.backend.sync(op("s"), SyncMode::Normal)).unwrap();
    assert!(
        setup
            .events()
            .contains(&Event::IngestFinished(IngestFinished { op: Some(op("s")) }))
    );
    // A restarted watcher ends the run of the previous one.
    setup.backend.start();
    let before = setup.events().len();
    block_on(setup.backend.exclude(Path::new(r"C:\Ablage\faq.html"))).unwrap();
    assert!(setup.events()[before..].contains(&Event::IngestFinished(IngestFinished { op: None })));
}

#[test]
fn a_cancelled_synchronisation_ends_with_ingest_finished() {
    // Cancelled from the progress callback of its first file, so the cancel surely comes
    // after the operation is registered and before the run is over.
    let events = Arc::new(Mutex::new(Vec::new()));
    let backend_cell: Arc<OnceLock<Backend>> = Arc::new(OnceLock::new());
    let sink = Arc::clone(&events);
    let cell = Arc::clone(&backend_cell);
    let emit: EventSink = Arc::new(move |event| {
        if let Event::IngestProgress(p) = &event
            && p.op == Some(op("c"))
            && let Some(backend) = cell.get()
        {
            backend.cancel(&op("c"));
        }
        sink.lock().unwrap().push(event);
    });
    let MockSetup { backend, .. } =
        block_on(mock::seeded_backend(emit, &mock::MockConfig::default())).unwrap();
    let _ = backend_cell.set(backend.clone());
    let error = block_on(backend.sync(op("c"), SyncMode::Full)).unwrap_err();
    assert_eq!(error.kind, ErrorKind::Cancelled);
    let events = events.lock().unwrap();
    let last_progress = events
        .iter()
        .rposition(|e| matches!(e, Event::IngestProgress(p) if p.op == Some(op("c"))))
        .unwrap();
    let finished = events
        .iter()
        .position(|e| *e == Event::IngestFinished(IngestFinished { op: Some(op("c")) }))
        .unwrap();
    assert!(finished > last_progress);
}

#[test]
fn a_paced_answer_is_cancelled_while_writing_and_stays_in_the_chat() {
    let emit: EventSink = Arc::new(|_| {});
    let config = mock::MockConfig {
        pace: Duration::from_millis(30),
        ..mock::MockConfig::default()
    };
    let MockSetup { backend, .. } = block_on(mock::seeded_backend(emit, &config)).unwrap();
    let canceller = backend.clone();
    let deltas = Arc::new(Mutex::new(0_usize));
    let seen = Arc::clone(&deltas);
    let watcher = std::thread::spawn(move || {
        // Cancel from another thread once a few pieces were written, as the window does.
        while *seen.lock().unwrap() < 3 {
            std::thread::sleep(Duration::from_millis(5));
        }
        canceller.cancel(&OperationId("paced".into()));
    });
    let answer = block_on(backend.ask(
        AskArgs {
            op: op("paced"),
            question: QUESTION.into(),
            chat: Some(ChatId::new("paced")),
            level: AnswerLevel::Fast,
        },
        move |event| {
            if matches!(event, AnswerEvent::Delta { .. }) {
                *deltas.lock().unwrap() += 1;
            }
        },
    ))
    .unwrap();
    watcher.join().unwrap();
    assert_eq!(answer.outcome, AnswerOutcome::Cancelled);
    let chat = block_on(backend.get_chat(&ChatId::new("paced")))
        .unwrap()
        .unwrap();
    assert_eq!(chat.turns[0].answer.outcome, AnswerOutcome::Cancelled);
    assert!(chat.turns[0].answer.text.len() < 100);
}

#[test]
fn ask_again_asks_a_saved_turn_again_and_appends_the_answer() {
    let Setup { backend, .. } = setup();
    let chat = ChatId::new("again");
    for (id, question) in [("q1", QUESTION), ("q2", "Wer unterschreibt das Protokoll?")] {
        block_on(backend.ask(
            AskArgs {
                op: op(id),
                question: question.into(),
                chat: Some(chat.clone()),
                level: AnswerLevel::Fast,
            },
            |_| {},
        ))
        .unwrap();
    }
    let again = |turn: u32, level: AnswerLevel, events: &Mutex<Vec<AnswerEvent>>| {
        block_on(backend.ask_again(
            AskAgainArgs {
                op: op("again"),
                chat: chat.clone(),
                turn,
                level,
            },
            |event| events.lock().unwrap().push(event),
        ))
    };
    let events = Mutex::new(Vec::new());
    let answer = again(0, AnswerLevel::Fast, &events).unwrap();
    // The first turn had no history: asked again, it is a first question again (case A).
    assert!(matches!(
        events.lock().unwrap().first(),
        Some(AnswerEvent::Status {
            phase: AnswerPhase::Searching,
            ..
        })
    ));
    let saved = block_on(backend.get_chat(&chat)).unwrap().unwrap();
    let questions: Vec<&str> = saved.turns.iter().map(|t| t.question.as_str()).collect();
    assert_eq!(
        questions,
        [QUESTION, "Wer unterschreibt das Protokoll?", QUESTION]
    );
    assert_eq!(saved.turns[2].answer, answer);
    // Errors as with `ask`: a missing model, an unknown turn or chat.
    let none = Mutex::new(Vec::new());
    assert_eq!(
        again(0, AnswerLevel::Balanced, &none).unwrap_err().kind,
        ErrorKind::ModelMissing
    );
    assert_eq!(
        again(9, AnswerLevel::Fast, &none).unwrap_err().kind,
        ErrorKind::InvalidInput
    );
    let unknown = block_on(backend.ask_again(
        AskAgainArgs {
            op: op("x"),
            chat: ChatId::new("nope"),
            turn: 0,
            level: AnswerLevel::Fast,
        },
        |_| {},
    ));
    assert_eq!(unknown.unwrap_err().kind, ErrorKind::InvalidInput);
    assert!(none.lock().unwrap().is_empty());
}

#[test]
fn a_level_the_machine_does_not_offer_is_preselected_as_fast() {
    let setup = setup_with(&mock::MockConfig {
        total_ram_bytes: 4 * 1024 * 1024 * 1024,
        ..mock::MockConfig::default()
    });
    let store = setup.backend.knowledge_base().store();
    block_on(store.update_settings(Box::new(|s| {
        s.answer_level = AnswerLevel::Balanced;
        Ok(())
    })))
    .unwrap();
    let state = block_on(setup.backend.setup_state()).unwrap();
    assert_eq!(state.tier, HardwareTier::BelowMinimum);
    assert_eq!(state.answer_level, AnswerLevel::Fast);
    // The setting stays; it applies again on a machine that offers it (ADR 0074).
    assert_eq!(
        block_on(setup.backend.get_settings()).unwrap().answer_level,
        AnswerLevel::Balanced
    );
}

#[test]
fn offline_package_levels_names_the_levels_complete_in_the_folder() {
    // models.toml with small sizes, so the test writes kilobytes, not gigabytes.
    let mut manifest =
        kb_core::ModelManifest::from_toml_str(include_str!("../../models.toml")).unwrap();
    for (i, file) in manifest
        .models
        .iter_mut()
        .flat_map(|m| m.files.iter_mut())
        .enumerate()
    {
        file.size_bytes = file.size_bytes.map(|_| 100 + i as u64);
    }
    let setup = setup_with(&mock::MockConfig {
        manifest: Some(manifest.clone()),
        ..mock::MockConfig::default()
    });
    let backend = &setup.backend;
    let package = std::env::temp_dir().join(format!("ragoxide-package-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&package);
    // Not chosen in the dialog for an offline package: refused.
    let error = block_on(backend.offline_package_levels(package.clone())).unwrap_err();
    assert_eq!(error.kind, ErrorKind::InvalidInput);
    backend.allow_picked(FolderPurpose::OfflinePackage, &package);
    assert!(
        block_on(backend.offline_package_levels(package.clone()))
            .unwrap()
            .is_empty()
    );

    // Write the files of the fast and balanced levels with their sizes.
    let write = |level: AnswerLevel| {
        for entry in manifest.models_for(HardwareTier::Supported, level).unwrap() {
            for file in &entry.files {
                let path = package.join(&entry.id).join(&file.path);
                std::fs::create_dir_all(path.parent().unwrap()).unwrap();
                let handle = std::fs::File::create(&path).unwrap();
                handle.set_len(file.size_bytes.unwrap_or(1)).unwrap();
            }
        }
    };
    write(AnswerLevel::Balanced);
    let levels = block_on(backend.offline_package_levels(package.clone())).unwrap();
    assert_eq!(levels, [AnswerLevel::Fast, AnswerLevel::Balanced]);
    // A file of the wrong size makes its level incomplete.
    let fast = manifest
        .answer_model(HardwareTier::Supported, AnswerLevel::Fast)
        .unwrap();
    let sized = fast.files.iter().find(|f| f.size_bytes.is_some()).unwrap();
    std::fs::File::create(package.join(&fast.id).join(&sized.path))
        .unwrap()
        .set_len(1)
        .unwrap();
    assert!(
        block_on(backend.offline_package_levels(package.clone()))
            .unwrap()
            .is_empty()
    );
    std::fs::remove_dir_all(&package).unwrap();
}
