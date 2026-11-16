//! With the `ts` feature every serialised type derives `ts_rs::TS`, and the generated
//! TypeScript matches the JSON form.

use kb_core::*;
use ts_rs::{Config, TS};

fn cfg() -> Config {
    Config::new().with_large_int("number")
}

fn decl<T: TS>() -> String {
    T::decl(&cfg())
}

/// Fails to compile if a serialised type lacks the derive.
#[test]
fn every_serialised_type_has_a_typescript_declaration() {
    macro_rules! all {
        ($($ty:ty),* $(,)?) => { vec![$(decl::<$ty>()),*] };
    }
    let declarations = all![
        DocumentId,
        DocumentFormat,
        SourceFile,
        Document,
        Page,
        Element,
        ElementKind,
        Table,
        TableCell,
        Provenance,
        Anchor,
        BoundingBox,
        CellRange,
        ChunkId,
        Chunk,
        EmbeddingSpace,
        SearchMode,
        SearchRequest,
        SearchHit,
        IndexEntry,
        Role,
        Message,
        GenerationRequest,
        Citation,
        Answer,
        AnswerEvent,
        ChatId,
        ChatTurn,
        Chat,
        ChatSummary,
        DocumentRecord,
        DocumentStatus,
        FileStatus,
        IngestEvent,
        FileResult,
        ScopeEvent,
        ErrorKind,
        HardwareTier,
        UiLanguage,
        ColorScheme,
        Settings,
        PipelineConfig,
        ModelManifest,
        RuntimeEntry,
        ModelRole,
        ModelEntry,
        ModelFile,
        HardwareInfo,
        ResolvedModel,
        MissingModelFile,
        ModelFileStatus,
        ModelEvent,
        Timestamp,
        AnswerPhase,
        PhaseProgress,
        AnswerOutcome,
        ModelStorage,
        SyncMode,
        AnswerLevel,
        RuntimeFile,
        FileSource,
        HuggingFaceSource,
        ReleaseSource,
    ];
    assert!(declarations.iter().all(|d| d.starts_with("type ")));
}

#[test]
fn declarations_follow_the_json_form() {
    assert_eq!(decl::<DocumentId>(), "type DocumentId = string;");
    let anchor = decl::<Anchor>();
    for tag in ["\"page\"", "\"slide\"", "\"sheet\"", "\"paragraph\""] {
        assert!(anchor.contains(&format!("\"type\": {tag}")), "{anchor}");
    }
    let settings = decl::<Settings>();
    for field in ["watched_folders", "excluded", "color_scheme", "ui_language"] {
        assert!(settings.contains(field), "{settings}");
    }
    assert!(decl::<ColorScheme>().contains("\"system\""));
    assert!(decl::<Answer>().contains("no_sources: boolean"));
    assert!(decl::<HardwareInfo>().contains("total_ram_bytes: number"));
    assert_eq!(decl::<Timestamp>(), "type Timestamp = number;");
    assert!(decl::<ChatSummary>().contains("updated_at: Timestamp"));
    assert!(decl::<DocumentRecord>().contains("modified_at: Timestamp | null"));
    let failed = decl::<FileStatus>();
    assert!(failed.contains("kind: ErrorKind"), "{failed}");
}
