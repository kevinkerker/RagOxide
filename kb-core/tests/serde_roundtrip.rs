//! Every public data type survives a JSON round trip unchanged.

use std::fmt::Debug;
use std::path::PathBuf;

use kb_core::mock::fixtures;
use kb_core::*;
use serde::Serialize;
use serde::de::DeserializeOwned;

fn roundtrip<T: Serialize + DeserializeOwned + PartialEq + Debug>(value: &T) {
    let json = serde_json::to_string(value).expect("serialize");
    let back: T = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(&back, value, "round trip changed {json}");
}

fn sample_answer() -> Answer {
    Answer {
        text: "Jedes Gerät wird jährlich geprüft [1].".into(),
        citations: vec![Citation {
            number: 1,
            chunk: fixtures::chunks()[0].clone(),
        }],
        no_sources: false,
        level: AnswerLevel::Balanced,
        outcome: AnswerOutcome::Complete,
    }
}

#[test]
fn document_types() {
    for document in fixtures::documents().values() {
        roundtrip(document);
        roundtrip(&document.source);
        roundtrip(&document.source.document_id);
        roundtrip(&document.format);
        for page in &document.pages {
            roundtrip(page);
        }
        for element in &document.elements {
            roundtrip(element);
            roundtrip(&element.kind);
            for anchor in &element.anchors {
                roundtrip(anchor);
            }
            if let ElementKind::Table(table) = &element.kind {
                roundtrip(table);
                for cell in &table.cells {
                    roundtrip(cell);
                }
            }
        }
    }
    roundtrip(&BoundingBox {
        left: 72.5,
        top: 10.25,
        right: 100.0,
        bottom: 20.125,
    });
    roundtrip(&CellRange {
        start_row: 0,
        start_col: 1,
        end_row: 2,
        end_col: 3,
    });
}

#[test]
fn chunk_and_search_types() {
    for chunk in fixtures::chunks() {
        roundtrip(&chunk.id);
        roundtrip(&chunk.provenance);
        roundtrip(&IndexEntry {
            embedding: vec![0.6, 0.8],
            chunk,
        });
    }
    for hit in fixtures::hits() {
        roundtrip(&hit);
    }
    roundtrip(&EmbeddingSpace {
        model_id: "embeddinggemma-2".into(),
        dimension: 768,
    });
    for mode in [SearchMode::Hybrid, SearchMode::Vector, SearchMode::FullText] {
        roundtrip(&SearchRequest {
            documents: Some(vec![DocumentId::new("doc-a")]),
            mode,
            ..SearchRequest::new("Prüffristen", 5)
        });
    }
}

#[test]
fn answer_and_chat_types() {
    let answer = sample_answer();
    roundtrip(&answer);
    roundtrip(&answer.citations[0]);
    roundtrip(&AnswerEvent::Sources {
        hits: fixtures::hits(),
    });
    for outcome in [
        AnswerOutcome::Complete,
        AnswerOutcome::Cancelled,
        AnswerOutcome::Failed {
            kind: ErrorKind::Generator,
            message: "backend crashed".into(),
        },
    ] {
        roundtrip(&outcome);
    }
    roundtrip(&AnswerEvent::Status {
        phase: AnswerPhase::ReadingSources,
        progress: Some(PhaseProgress {
            done: 10,
            total: 40,
        }),
    });
    roundtrip(&AnswerEvent::Status {
        phase: AnswerPhase::StartingModel,
        progress: None,
    });
    roundtrip(&AnswerEvent::Delta {
        text: "Jedes ".into(),
    });
    roundtrip(&AnswerEvent::Done {
        answer: answer.clone(),
    });
    roundtrip(&GenerationRequest {
        level: AnswerLevel::Thorough,
        messages: vec![
            Message {
                role: Role::System,
                content: "Answer from sources.".into(),
            },
            Message {
                role: Role::User,
                content: "Wie oft?".into(),
            },
            Message {
                role: Role::Assistant,
                content: "Jährlich [1].".into(),
            },
        ],
        max_tokens: Some(256),
        temperature: Some(0.5),
    });
    let chat = Chat {
        id: ChatId::new("chat-1"),
        title: "Prüffristen".into(),
        created_at: Timestamp(1_794_700_800_000),
        updated_at: Timestamp(1_794_787_200_000),
        turns: vec![ChatTurn {
            question: "Wie oft wird geprüft?".into(),
            answer,
        }],
    };
    roundtrip(&chat);
    roundtrip(&ChatSummary {
        id: chat.id.clone(),
        title: chat.title.clone(),
        created_at: chat.created_at,
        updated_at: chat.updated_at,
    });
    // A timestamp is a plain number of milliseconds in JSON.
    let json = serde_json::to_value(&chat).unwrap();
    assert_eq!(json["created_at"], 1_794_700_800_000_u64);
}

#[test]
fn ingest_types() {
    let id = DocumentId::new("doc-a");
    let statuses = vec![
        FileStatus::Queued,
        FileStatus::Parsing,
        FileStatus::Chunking,
        FileStatus::Embedding,
        FileStatus::Indexing,
        FileStatus::Indexed {
            document_id: id.clone(),
            chunks: 3,
        },
        FileStatus::Unchanged {
            document_id: id.clone(),
        },
        FileStatus::Removed {
            document_id: id.clone(),
        },
        FileStatus::Failed {
            kind: ErrorKind::Parse,
            message: "Datei beschädigt".into(),
        },
    ];
    for status in statuses {
        roundtrip(&IngestEvent {
            path: PathBuf::from(r"C:\Dokumente\Prüfplan.pdf"),
            status: status.clone(),
            completed: 1,
            total: 2,
        });
        roundtrip(&FileResult {
            path: PathBuf::from(r"C:\Dokumente\Prüfplan.pdf"),
            status,
        });
    }
    for status in [
        DocumentStatus::Queued,
        DocumentStatus::Indexed { chunks: 4 },
        DocumentStatus::Failed {
            kind: ErrorKind::Parse,
            message: "verschlüsselt".into(),
        },
    ] {
        roundtrip(&DocumentRecord {
            id: id.clone(),
            path: PathBuf::from(r"C:\Dokumente\Prüfplan.pdf"),
            format: DocumentFormat::Pdf,
            title: Some("Prüfplan".into()),
            content_hash: Some("ab".repeat(32)),
            size_bytes: Some(48_213),
            modified_at: Some(Timestamp(1_794_700_800_000)),
            status,
        });
    }
}

#[test]
fn config_and_hub_types() {
    roundtrip(&Settings::default());
    roundtrip(&Settings {
        watched_folders: vec![PathBuf::from(r"C:\Dokumente")],
        excluded: vec![PathBuf::from(r"C:\Dokumente\Privat")],
        answer_level: AnswerLevel::Thorough,
        ui_language: UiLanguage::English,
        color_scheme: ColorScheme::Dark,
    });
    for scheme in [ColorScheme::System, ColorScheme::Light, ColorScheme::Dark] {
        roundtrip(&scheme);
    }
    roundtrip(&ModelStorage {
        dir: PathBuf::from(r"C:\Modelle"),
        free_bytes: 50_000_000_000,
    });
    roundtrip(&ModelFileStatus::Copying {
        done_bytes: 1,
        total_bytes: Some(2),
    });
    roundtrip(&MissingModelFile {
        model_id: "gemma-4-e2b".into(),
        role: ModelRole::Generator,
        file: "model.gguf".into(),
    });
    roundtrip(&PipelineConfig::default());
    let manifest =
        ModelManifest::from_toml_str(include_str!("../../models.toml")).expect("models.toml");
    roundtrip(&manifest);
    for role in [
        ModelRole::Embedding,
        ModelRole::Reranker,
        ModelRole::Generator,
        ModelRole::Layout,
        ModelRole::TableStructure,
        ModelRole::OcrDetection,
        ModelRole::OcrRecognition,
    ] {
        roundtrip(&role);
    }
    roundtrip(&HardwareInfo {
        total_ram_bytes: 16 << 30,
        logical_cpus: 8,
    });
    roundtrip(&ResolvedModel {
        entry: manifest.models[0].clone(),
        dir: PathBuf::from(r"C:\Modelle\embeddinggemma-2"),
    });
    for status in [
        ModelFileStatus::Verifying,
        ModelFileStatus::Downloading {
            done_bytes: 10,
            total_bytes: Some(100),
        },
        ModelFileStatus::Ready,
    ] {
        roundtrip(&ModelEvent {
            model_id: "gemma-4-e2b".into(),
            role: ModelRole::Generator,
            file: "model.gguf".into(),
            status,
        });
    }
}

#[test]
fn scope_events_and_error_kinds() {
    let folder = PathBuf::from(r"C:\Netz\Projekte");
    for event in [
        ScopeEvent::FolderAvailable {
            folder: folder.clone(),
        },
        ScopeEvent::FolderUnavailable {
            folder: folder.clone(),
            kind: ErrorKind::Io,
            message: "network path not found".into(),
        },
        ScopeEvent::ExclusionRemoved {
            path: folder.join("alt"),
        },
    ] {
        roundtrip(&event);
    }
    let json = serde_json::to_value(ScopeEvent::FolderUnavailable {
        folder,
        kind: ErrorKind::Io,
        message: "m".into(),
    })
    .unwrap();
    assert_eq!(json["type"], "folder_unavailable");
    assert_eq!(json["kind"], "io");
    assert_eq!(
        serde_json::to_value(ErrorKind::EmbeddingMismatch).unwrap(),
        "embedding_mismatch"
    );
}

#[test]
fn json_shape_for_the_user_interface() {
    // A slide anchor always writes `in_notes`; without it, it reads as on the slide.
    let json = serde_json::to_value(Anchor::Slide {
        slide: 2,
        in_notes: true,
    })
    .unwrap();
    assert_eq!(
        json,
        serde_json::json!({"type": "slide", "slide": 2, "in_notes": true})
    );
    let json = serde_json::to_value(Anchor::Slide {
        slide: 2,
        in_notes: false,
    })
    .unwrap();
    assert_eq!(
        json,
        serde_json::json!({"type": "slide", "slide": 2, "in_notes": false})
    );
    let old: Anchor = serde_json::from_str(r#"{"type": "slide", "slide": 2}"#).unwrap();
    assert_eq!(
        old,
        Anchor::Slide {
            slide: 2,
            in_notes: false
        }
    );
    let json = serde_json::to_value(Anchor::Paragraph { element: 3 }).unwrap();
    assert_eq!(json, serde_json::json!({"type": "paragraph", "element": 3}));
    let json = serde_json::to_value(ElementKind::ListItem).unwrap();
    assert_eq!(json, serde_json::json!({"type": "list_item"}));
    let json = serde_json::to_value(AnswerEvent::Delta { text: "a".into() }).unwrap();
    assert_eq!(json, serde_json::json!({"type": "delta", "text": "a"}));
    assert_eq!(
        serde_json::to_value(HardwareTier::BelowMinimum).unwrap(),
        "below_minimum"
    );
    assert_eq!(
        serde_json::to_value(AnswerLevel::Thorough).unwrap(),
        "thorough"
    );
    assert_eq!(
        serde_json::to_value(AnswerPhase::Waiting).unwrap(),
        "waiting"
    );
    assert_eq!(
        serde_json::to_value(ErrorKind::ModelMissing).unwrap(),
        "model_missing"
    );
    assert_eq!(serde_json::to_value(UiLanguage::German).unwrap(), "de");
    assert_eq!(serde_json::to_value(ColorScheme::System).unwrap(), "system");
    assert_eq!(
        serde_json::to_value(SyncMode::RetryFailed).unwrap(),
        "retry_failed"
    );
    assert_eq!(serde_json::to_value(SyncMode::Full).unwrap(), "full");
    assert_eq!(
        serde_json::to_value(DocumentId::new("doc-a")).unwrap(),
        "doc-a"
    );
    let provenance = serde_json::to_value(&fixtures::chunks()[0].provenance).unwrap();
    for field in ["source", "heading_path", "anchors"] {
        assert!(provenance.get(field).is_some(), "provenance.{field}");
    }
    for field in ["document_id", "path", "name", "content_hash"] {
        assert!(provenance["source"].get(field).is_some(), "source.{field}");
    }
}

#[test]
fn missing_fields_get_defaults() {
    let settings: Settings = serde_json::from_str("{}").unwrap();
    assert_eq!(settings, Settings::default());
    assert_eq!(settings.ui_language, UiLanguage::German);
    assert_eq!(settings.color_scheme, ColorScheme::System);
    assert_eq!(settings.answer_level, AnswerLevel::Fast, "ADR 0060");
    assert!(settings.excluded.is_empty());
    let config: PipelineConfig = serde_json::from_str(r#"{"max_sources": 4}"#).unwrap();
    assert_eq!(config.max_sources, 4);
    assert_eq!(
        config.chunk_max_tokens,
        PipelineConfig::default().chunk_max_tokens
    );
    let request: SearchRequest =
        serde_json::from_str(r#"{"text": "Fluchtwege", "limit": 3}"#).unwrap();
    assert_eq!(request, SearchRequest::new("Fluchtwege", 3));
}
