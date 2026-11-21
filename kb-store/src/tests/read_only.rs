//! A read-only database file is not opened (G55, ADR 0144, ADR 0122).
//!
//! `open` ends with `Error::Store` naming the write protection before it rolls back a
//! journal, sets a marker, writes or removes a backup or migrates; every file in the
//! folder stays byte for byte. Without the write protection the next opening works and
//! does what is pending.

use super::crash::{
    DOC_MARK, abort_in_child, has_helper_table, integrity, journal_len, occurrences, prepare,
    recover_without_the_rule, reopen_and_check,
};
use super::migration::{backup, filled_v1};
use super::*;
use crate::open::overwrite_marker;

/// Name and bytes of every file below `folder`, sorted by name.
fn files(folder: &Path) -> Vec<(String, Vec<u8>)> {
    let mut files: Vec<(String, Vec<u8>)> = walk(folder)
        .into_iter()
        .map(|p| {
            let name = p.file_name().unwrap().to_string_lossy().into_owned();
            (name, std::fs::read(&p).unwrap())
        })
        .collect();
    files.sort();
    files
}

/// Opens the read-only `db` with `schema` and checks the refusal and that no file in
/// its folder changed.
fn refused(scratch: &Scratch, schema: &'static Schema) {
    let db = scratch.db();
    set_read_only(&db, true);
    let before = files(&scratch.folder());
    let error = run(async { SqliteStore::open_with(&db, schema).await.unwrap_err() });
    assert!(matches!(error, Error::Store(_)), "{error:?}");
    assert!(error.to_string().contains("is read-only"), "{error}");
    assert_eq!(files(&scratch.folder()), before, "every file byte for byte");
    set_read_only(&db, false);
}

#[test]
fn a_read_only_new_database_is_not_opened() {
    // An empty file, as a new database starts.
    let scratch = Scratch::new();
    let db = scratch.db();
    std::fs::create_dir_all(scratch.folder()).unwrap();
    std::fs::write(&db, "").unwrap();
    refused(&scratch, &schema::CURRENT);
    assert_eq!(names(&scratch.folder()), ["library.sqlite"]);
    run(async {
        let store = SqliteStore::open(&db).await.unwrap();
        assert_eq!(store.settings().await.unwrap(), Settings::default());
    });

    // A database just created with the current schema, nothing pending.
    let scratch = Scratch::new();
    let db = scratch.db();
    run(async { drop(SqliteStore::open(&db).await.unwrap()) });
    refused(&scratch, &schema::CURRENT);
    run(async {
        let store = SqliteStore::open(&db).await.unwrap();
        store.save_settings(&settings()).await.unwrap();
    });
}

#[test]
fn a_read_only_database_with_a_hot_journal_is_not_opened() {
    let scratch = Scratch::new();
    let db = scratch.db();
    let before = prepare(&db, 800);
    abort_in_child("content", &db);
    assert!(journal_len(&db) > 0);
    // No rollback and no marker: the journal and the database are unchanged.
    refused(&scratch, &schema::CURRENT);
    assert!(!overwrite_marker(&db).exists());
    // Writable again: recovery and overwrite as after any cut-off write (G51).
    reopen_and_check(&scratch, &before, DOC_MARK);
}

#[test]
fn a_read_only_database_with_a_marker_is_not_opened() {
    let scratch = Scratch::new();
    let db = scratch.db();
    let before = prepare(&db, 800);
    abort_in_child("content", &db);
    recover_without_the_rule(&db);
    std::fs::write(overwrite_marker(&db), "").unwrap();
    refused(&scratch, &schema::CURRENT);
    assert!(overwrite_marker(&db).is_file());
    assert!(occurrences(&scratch.folder(), DOC_MARK) > 0);
    // Writable again: the overwrite is done.
    let after = run(async { snapshot(&SqliteStore::open(&db).await.unwrap()).await });
    assert_eq!(after, before);
    assert_eq!(occurrences(&scratch.folder(), DOC_MARK), 0);
    assert!(!overwrite_marker(&db).exists());
    assert!(!has_helper_table(&db));
    assert_eq!(integrity(&db), "ok");
}

#[test]
fn a_read_only_database_with_a_backup_is_not_opened() {
    let scratch = Scratch::new();
    let db = scratch.db();
    let before = filled_v1(&db);
    run(async { drop(SqliteStore::open_with(&db, &SCHEMA_V2).await.unwrap()) });
    assert!(backup(&db, 1).is_file());
    // The backup is neither checked nor removed.
    refused(&scratch, &SCHEMA_V2);
    assert!(backup(&db, 1).is_file());
    // Writable again: the check runs and removes the backup (G30).
    let after =
        run(async { snapshot(&SqliteStore::open_with(&db, &SCHEMA_V2).await.unwrap()).await });
    assert_eq!(after, before);
    assert!(!backup(&db, 1).exists());
}
