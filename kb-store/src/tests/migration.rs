//! Schema versions, migration, backup and its removal (G05, G20, G21, G28 to G30, G52).

use super::*;
use crate::open::sibling;

pub(super) fn backup(db: &Path, version: u32) -> PathBuf {
    sibling(db, &format!(".v{version}.bak"))
}

/// Creates a filled database with schema 1 and returns what it reads.
pub(super) fn filled_v1(db: &Path) -> Snapshot {
    run(async {
        let store = SqliteStore::open(db).await.expect("open");
        fill(&store).await;
        snapshot(&store).await
    })
}

#[test]
fn opening_schema_one_changes_nothing_also_twice() {
    // G28, first part
    let scratch = Scratch::new();
    let db = scratch.db();
    let before = filled_v1(&db);
    let bytes = std::fs::read(&db).unwrap();
    for _ in 0..2 {
        let after = run(async { snapshot(&SqliteStore::open(&db).await.unwrap()).await });
        assert_eq!(after, before);
        assert_eq!(std::fs::read(&db).unwrap(), bytes, "not one byte changed");
        assert_eq!(user_version(&db), 1);
        assert_eq!(
            names(&scratch.folder()),
            ["library.sqlite", "library.sqlite-journal"]
        );
    }
}

#[test]
fn migration_to_schema_two_keeps_every_table_readable() {
    // G28, second part; G29 (backup of the old version with all data)
    let scratch = Scratch::new();
    let db = scratch.db();
    let before = filled_v1(&db);
    let old_bytes = std::fs::read(&db).unwrap();
    let after = run(async {
        let store = SqliteStore::open_with(&db, &SCHEMA_V2).await.unwrap();
        snapshot(&store).await
    });
    assert_eq!(after, before);
    assert_eq!(user_version(&db), 2);
    {
        let conn = rusqlite::Connection::open(&db).unwrap();
        let tags: i64 = conn
            .query_row(
                "SELECT count(*) FROM chat_tags WHERE tag = 'migriert'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(tags, 3, "the migration saw the data of every chat");
        let meta: i64 = conn
            .query_row("SELECT count(*) FROM store_meta", [], |r| r.get(0))
            .unwrap();
        assert!(meta >= 1);
    }

    // The backup is a complete database of schema 1 with all data.
    let saved = backup(&db, 1);
    assert!(saved.is_file());
    assert_eq!(user_version(&saved), 1);
    let elsewhere = Scratch::new();
    std::fs::create_dir_all(elsewhere.folder()).unwrap();
    std::fs::copy(&saved, elsewhere.db()).unwrap();
    let restored = run(async { snapshot(&SqliteStore::open(elsewhere.db()).await.unwrap()).await });
    assert_eq!(restored, before);
    assert_eq!(std::fs::read(&saved).unwrap().len(), old_bytes.len());

    // Opening schema 2 again works and keeps the data.
    let again =
        run(async { snapshot(&SqliteStore::open_with(&db, &SCHEMA_V2).await.unwrap()).await });
    assert_eq!(again, before);
}

#[test]
fn only_the_newest_backup_is_kept_and_an_older_one_goes_only_after_success() {
    // G29
    let scratch = Scratch::new();
    let db = scratch.db();
    let before = filled_v1(&db);
    run(async { drop(SqliteStore::open_with(&db, &SCHEMA_V2).await.unwrap()) });
    assert!(backup(&db, 1).is_file());

    // The backup before 2 -> 3 cannot be written: no migration, nothing changes, and
    // the older backup stays.
    let blocker = sibling(&db, ".v2.bak.partial");
    std::fs::create_dir_all(blocker.join("im-weg")).unwrap();
    std::fs::write(blocker.join("im-weg").join("datei"), "x").unwrap();
    let bytes = std::fs::read(&db).unwrap();
    let error = run(async { SqliteStore::open_with(&db, &SCHEMA_V3).await.unwrap_err() });
    assert!(matches!(error, Error::Store(_)), "{error:?}");
    assert!(error.to_string().contains("backup"), "{error}");
    assert_eq!(std::fs::read(&db).unwrap(), bytes);
    assert_eq!(user_version(&db), 2);
    assert!(backup(&db, 1).is_file());
    assert!(!backup(&db, 2).exists());
    std::fs::remove_dir_all(&blocker).unwrap();

    // With the way free: the new backup is written, the older one removed.
    let after =
        run(async { snapshot(&SqliteStore::open_with(&db, &SCHEMA_V3).await.unwrap()).await });
    assert_eq!(after, before);
    assert_eq!(user_version(&db), 3);
    assert_eq!(
        names(&scratch.folder()),
        [
            "library.sqlite",
            "library.sqlite-journal",
            "library.sqlite.v2.bak"
        ]
    );
    assert_eq!(user_version(&backup(&db, 2)), 2);
}

#[test]
fn a_failed_migration_leaves_the_database_byte_for_byte() {
    // G05
    let scratch = Scratch::new();
    let db = scratch.db();
    let before = filled_v1(&db);
    let bytes = std::fs::read(&db).unwrap();
    let error = run(async {
        SqliteStore::open_with(&db, &SCHEMA_V2_FAILING)
            .await
            .unwrap_err()
    });
    assert!(matches!(error, Error::Store(_)), "{error:?}");
    assert!(error.to_string().contains("migration"), "{error}");
    assert!(
        error.to_string().contains("table_that_does_not_exist"),
        "{error}"
    );
    assert_eq!(std::fs::read(&db).unwrap(), bytes, "rolled back completely");
    assert_eq!(user_version(&db), 1);
    // The backup written before the failed migration stays.
    assert!(backup(&db, 1).is_file());
    assert_eq!(user_version(&backup(&db, 1)), 1);
    // The next successful opening reads everything, writes once and removes it.
    let after = run(async { snapshot(&SqliteStore::open(&db).await.unwrap()).await });
    assert_eq!(after, before);
    assert!(!backup(&db, 1).exists());
}

#[test]
fn every_failed_opening_leaves_the_file_byte_for_byte() {
    // G05: newer schema, migration on a read-only file, a schema without a table.
    let scratch = Scratch::new();
    let db = scratch.db();
    filled_v1(&db);
    let bytes = std::fs::read(&db).unwrap();
    run(async { drop(SqliteStore::open_with(&db, &SCHEMA_V2).await.unwrap()) });
    let migrated = std::fs::read(&db).unwrap();
    assert_ne!(migrated, bytes);
    // Schema 2 is newer than schema 1 of this build.
    let error = run(async { SqliteStore::open(&db).await.unwrap_err() });
    assert!(error.to_string().contains("newer"), "{error}");
    assert_eq!(std::fs::read(&db).unwrap(), migrated);
    // A database that lacks a table of its version fails the schema check.
    {
        let conn = rusqlite::Connection::open(&db).unwrap();
        conn.execute_batch("DROP TABLE chat_tags;").unwrap();
    }
    let damaged = std::fs::read(&db).unwrap();
    let error = run(async { SqliteStore::open_with(&db, &SCHEMA_V2).await.unwrap_err() });
    assert!(error.to_string().contains("schema check failed"), "{error}");
    assert_eq!(std::fs::read(&db).unwrap(), damaged);

    // A read-only file of an older schema is not migrated, and no backup is written.
    let other = Scratch::new();
    let db = other.db();
    filled_v1(&db);
    let bytes = std::fs::read(&db).unwrap();
    set_read_only(&db, true);
    let error = run(async { SqliteStore::open_with(&db, &SCHEMA_V2).await.unwrap_err() });
    assert!(error.to_string().contains("read-only"), "{error}");
    assert_eq!(std::fs::read(&db).unwrap(), bytes);
    assert!(!backup(&db, 1).exists());
    set_read_only(&db, false);
}

#[test]
fn the_backup_is_deleted_at_the_first_successful_opening() {
    // G30
    let scratch = Scratch::new();
    let db = scratch.db();
    filled_v1(&db);
    run(async { drop(SqliteStore::open_with(&db, &SCHEMA_V2).await.unwrap()) });
    assert!(
        backup(&db, 1).is_file(),
        "the backup outlives the migration itself"
    );
    run(async {
        let store = SqliteStore::open_with(&db, &SCHEMA_V2).await.unwrap();
        // The check wrote once.
        let checked: String = store
            .with_connection(|c| {
                c.query_row(
                    "SELECT value FROM store_meta WHERE key = 'checked_with_schema'",
                    [],
                    |r| r.get(0),
                )
                .map_err(crate::error::sqlite)
            })
            .await
            .unwrap();
        assert_eq!(checked, "2");
    });
    assert!(!backup(&db, 1).exists());
    // No copy of the database other than itself and its journal.
    assert_eq!(
        names(&scratch.folder()),
        ["library.sqlite", "library.sqlite-journal"]
    );
}

#[test]
fn the_backup_stays_if_the_write_fails_on_a_writable_file() {
    // G52: the file is writable, but the write of the check fails (a trigger refuses
    // it). The opening succeeds, the backup stays until a check write succeeds.
    let scratch = Scratch::new();
    let db = scratch.db();
    let before = filled_v1(&db);
    run(async { drop(SqliteStore::open_with(&db, &SCHEMA_V2).await.unwrap()) });
    {
        let conn = rusqlite::Connection::open(&db).unwrap();
        conn.execute_batch(
            "CREATE TRIGGER refuse_check BEFORE INSERT ON store_meta
             BEGIN SELECT RAISE(ABORT, 'check write refused'); END;",
        )
        .unwrap();
    }
    assert!(!std::fs::metadata(&db).unwrap().permissions().readonly());
    let read =
        run(async { snapshot(&SqliteStore::open_with(&db, &SCHEMA_V2).await.unwrap()).await });
    assert_eq!(read, before, "the opening itself succeeds");
    assert!(
        backup(&db, 1).is_file(),
        "the backup stays after the failed write"
    );
    {
        let conn = rusqlite::Connection::open(&db).unwrap();
        conn.execute_batch("DROP TRIGGER refuse_check;").unwrap();
    }
    run(async { drop(SqliteStore::open_with(&db, &SCHEMA_V2).await.unwrap()) });
    assert!(!backup(&db, 1).exists());
}

#[test]
fn deleted_text_is_in_no_file_after_a_migration() {
    // G20 and G21 after a migration: once the backup is gone, no file of the data folder
    // holds the text of a chat or document deleted after the migration.
    const CHAT: &str = "Geheimer-Chat-Ü5K8";
    const DOC: &str = "Geheime-Akte-ß3M1";
    let scratch = Scratch::new();
    let db = scratch.db();
    let folder = scratch.folder();
    let mut document = fixtures::documents().remove("notizen.md").unwrap();
    document.source = SourceFile::new(
        DocumentId::new("doc-geheim"),
        Path::new(&format!(r"C:\Ablage\{DOC}.md")),
        "d".repeat(64),
    )
    .unwrap();
    for element in &mut document.elements {
        element.text = format!("Inhalt {DOC} Inhalt");
    }
    run(async {
        let store = SqliteStore::open(&db).await.unwrap();
        fill(&store).await;
        store
            .save_chat(&chat("geheim", CHAT, AnswerOutcome::Complete))
            .await
            .unwrap();
        store.upsert_document(&record_of(&document)).await.unwrap();
        store.save_content(&document).await.unwrap();
    });
    run(async { drop(SqliteStore::open_with(&db, &SCHEMA_V2).await.unwrap()) });
    // Control: the backup holds both.
    assert!(files_containing(&folder, CHAT).contains(&backup(&db, 1)));
    assert!(files_containing(&folder, DOC).contains(&backup(&db, 1)));
    run(async {
        let store = SqliteStore::open_with(&db, &SCHEMA_V2).await.unwrap();
        store.delete_chat(&ChatId::new("geheim")).await.unwrap();
        store
            .delete_document(&DocumentId::new("doc-geheim"))
            .await
            .unwrap();
        assert_eq!(store.chats().await.unwrap().len(), 3);
    });
    assert_eq!(files_containing(&folder, CHAT), Vec::<PathBuf>::new());
    assert_eq!(files_containing(&folder, DOC), Vec::<PathBuf>::new());
    // The path key of the record is the lower-cased path.
    assert_eq!(
        files_containing(&folder, &DOC.to_lowercase()),
        Vec::<PathBuf>::new()
    );
    assert_eq!(names(&folder), ["library.sqlite", "library.sqlite-journal"]);
}
