//! Opening, reopening and what survives a restart (G03, G04, G12, G16, G19, G27, G31,
//! G32).

mod common;

use common::*;
use kb_core::*;
use kb_store::{SCHEMA_VERSION, SqliteStore};
use rusqlite::Connection;

#[test]
fn opening_a_missing_file_creates_schema_one_without_backup() {
    // G03
    let scratch = Scratch::new();
    let db = scratch.db();
    assert!(!scratch.data_folder().exists());
    run(async {
        let store = open(&db).await;
        assert_eq!(store.settings().await.unwrap(), Settings::default());
        assert!(store.documents().await.unwrap().is_empty());
        assert!(store.chats().await.unwrap().is_empty());
    });
    assert_eq!(SCHEMA_VERSION, 1);
    let conn = Connection::open(&db).unwrap();
    let version: u32 = conn
        .pragma_query_value(None, "user_version", |r| r.get(0))
        .unwrap();
    assert_eq!(version, 1);
    drop(conn);
    let names = file_names(&scratch.data_folder());
    assert!(
        names.iter().all(|n| !n.contains(".bak")),
        "no backup: {names:?}"
    );
}

#[test]
fn a_text_file_is_not_opened_and_stays_byte_for_byte() {
    // G04
    let scratch = Scratch::new();
    let db = scratch.db();
    std::fs::create_dir_all(db.parent().unwrap()).unwrap();
    let text = "Das ist keine Datenbank, sondern eine Notiz mit Umlauten: äöü ß.\n".repeat(50);
    std::fs::write(&db, &text).unwrap();
    run(async {
        let error = SqliteStore::open(&db).await.unwrap_err();
        assert!(matches!(error, Error::Store(_)), "{error:?}");
        assert!(
            error.to_string().contains("not an SQLite database"),
            "{error}"
        );
    });
    assert_eq!(std::fs::read(&db).unwrap(), text.as_bytes());
    assert_eq!(file_names(&scratch.data_folder()), ["library.sqlite"]);
}

#[test]
fn a_database_of_another_application_is_not_touched() {
    // G04, "Fremde SQLite-Datei": a valid SQLite file of another application.
    let scratch = Scratch::new();
    let db = scratch.db();
    std::fs::create_dir_all(db.parent().unwrap()).unwrap();
    {
        let conn = Connection::open(&db).unwrap();
        conn.execute_batch(
            "PRAGMA user_version = 1;
             CREATE TABLE things (name TEXT);
             INSERT INTO things VALUES ('fremd');",
        )
        .unwrap();
    }
    let before = std::fs::read(&db).unwrap();
    run(async {
        let error = SqliteStore::open(&db).await.unwrap_err();
        assert!(matches!(error, Error::Store(_)), "{error:?}");
        assert!(error.to_string().contains("another application"), "{error}");
    });
    assert_eq!(std::fs::read(&db).unwrap(), before);
    assert_eq!(file_names(&scratch.data_folder()), ["library.sqlite"]);
}

#[test]
fn a_newer_schema_is_refused_and_left_unchanged() {
    // G31
    let scratch = Scratch::new();
    let db = scratch.db();
    run(async {
        let store = open(&db).await;
        store.save_chat(&sample_chats()[0]).await.unwrap();
    });
    {
        let conn = Connection::open(&db).unwrap();
        conn.pragma_update(None, "user_version", 7).unwrap();
    }
    let before = std::fs::read(&db).unwrap();
    let names = file_names(&scratch.data_folder());
    run(async {
        let error = SqliteStore::open(&db).await.unwrap_err();
        assert!(matches!(error, Error::Store(_)), "{error:?}");
        let text = error.to_string();
        assert!(text.contains("schema version 7"), "{text}");
        assert!(text.contains("supported schema version 1"), "{text}");
    });
    assert_eq!(std::fs::read(&db).unwrap(), before);
    assert_eq!(file_names(&scratch.data_folder()), names, "no backup");
}

#[test]
fn queued_and_failed_records_survive_a_restart() {
    // G12
    let scratch = Scratch::new();
    let records = [
        record("q1", r"C:\Ablage\wartet.pdf"),
        DocumentRecord {
            size_bytes: Some(2048),
            modified_at: Some(Timestamp(1_794_700_000_000)),
            status: DocumentStatus::Failed {
                kind: ErrorKind::Parse,
                message: "corrupt xref table at offset 1234".into(),
            },
            ..record("f1", r"C:\Ablage\kaputt.pdf")
        },
        DocumentRecord {
            status: DocumentStatus::Failed {
                kind: ErrorKind::Io,
                message: "The process cannot access the file because it is being used by another process. (os error 32)".into(),
            },
            ..record("f2", r"C:\Ablage\offen.xlsx")
        },
    ];
    run(async {
        let store = open(&scratch.db()).await;
        for r in &records {
            store.upsert_document(r).await.unwrap();
        }
    });
    run(async {
        let store = open(&scratch.db()).await;
        let mut back = store.documents().await.unwrap();
        back.sort_by(|a, b| a.id.cmp(&b.id));
        let mut expected = records.to_vec();
        expected.sort_by(|a, b| a.id.cmp(&b.id));
        assert_eq!(back, expected);
    });
}

#[test]
fn cancelled_and_failed_turns_survive_a_restart() {
    // G16
    let scratch = Scratch::new();
    let chats = sample_chats();
    run(async {
        let store = open(&scratch.db()).await;
        for chat in &chats {
            store.save_chat(chat).await.unwrap();
        }
    });
    run(async {
        let store = open(&scratch.db()).await;
        for chat in &chats[1..] {
            let back = store.chat(&chat.id).await.unwrap().unwrap();
            assert_eq!(&back, chat);
            assert_ne!(back.turns[0].answer.outcome, AnswerOutcome::Complete);
            assert!(!back.turns[0].answer.text.is_empty());
            assert!(!back.turns[0].answer.citations.is_empty());
        }
    });
}

#[test]
fn a_deleted_chat_stays_deleted() {
    // G19
    let scratch = Scratch::new();
    let chats = sample_chats();
    run(async {
        let store = open(&scratch.db()).await;
        for chat in &chats {
            store.save_chat(chat).await.unwrap();
        }
        store.delete_chat(&chats[1].id).await.unwrap();
        store.delete_chat(&chats[1].id).await.unwrap();
        store.delete_chat(&ChatId::new("unbekannt")).await.unwrap();
        assert!(store.chat(&chats[1].id).await.unwrap().is_none());
        assert_eq!(store.chats().await.unwrap().len(), 2);
    });
    run(async {
        let store = open(&scratch.db()).await;
        assert!(store.chat(&chats[1].id).await.unwrap().is_none());
        let ids: Vec<ChatId> = store
            .chats()
            .await
            .unwrap()
            .into_iter()
            .map(|c| c.id)
            .collect();
        assert_eq!(ids, [chats[0].id.clone(), chats[2].id.clone()]);
        assert_eq!(
            store.chat(&chats[0].id).await.unwrap().as_ref(),
            Some(&chats[0])
        );
    });
}

#[test]
fn settings_without_a_field_read_it_as_its_default() {
    // G27: settings stored before a field existed are read with its default value.
    let scratch = Scratch::new();
    let saved = sample_settings();
    run(async {
        let store = open(&scratch.db()).await;
        store.save_settings(&saved).await.unwrap();
    });
    {
        let conn = Connection::open(scratch.db()).unwrap();
        let removed = conn
            .execute(
                "UPDATE settings SET settings = json_remove(settings, '$.color_scheme', '$.answer_level')",
                [],
            )
            .unwrap();
        assert_eq!(removed, 1);
        let json: String = conn
            .query_row("SELECT settings FROM settings", [], |r| r.get(0))
            .unwrap();
        assert!(
            !json.contains("color_scheme") && !json.contains("answer_level"),
            "{json}"
        );
    }
    run(async {
        let store = open(&scratch.db()).await;
        let read = store.settings().await.unwrap();
        assert_eq!(
            read,
            Settings {
                color_scheme: ColorScheme::default(),
                answer_level: AnswerLevel::default(),
                ..saved
            }
        );
    });
}

#[test]
fn every_reading_method_returns_the_same_after_a_restart() {
    // G32
    let scratch = Scratch::new();
    let before = run(async {
        let store = open(&scratch.db()).await;
        for (record, document) in fixture_library() {
            store.upsert_document(&record).await.unwrap();
            store.save_content(&document).await.unwrap();
        }
        store
            .upsert_document(&record("queued", r"C:\Ablage\neu.docx"))
            .await
            .unwrap();
        for chat in sample_chats() {
            store.save_chat(&chat).await.unwrap();
        }
        store.save_settings(&sample_settings()).await.unwrap();
        snapshot(&store, &["unknown"]).await
    });
    assert_eq!(before.documents.len(), 8);
    assert_eq!(before.chats.len(), 3);
    assert_eq!(before.contents.iter().filter(|c| c.is_some()).count(), 7);
    let after = run(async {
        let store = open(&scratch.db()).await;
        snapshot(&store, &["unknown"]).await
    });
    assert_eq!(after, before);
}
