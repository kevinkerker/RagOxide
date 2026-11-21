//! Text of a write that was cut off is overwritten at the next opening (G51, ADR 0142).
//!
//! A child process (this test program) starts the same statement as `save_content` or
//! `save_chat` with a page cache of one page, so that SQLite writes the pages into the
//! database file before the commit, and then ends without closing anything. The parent
//! opens the database again and searches the bytes of every file.

use std::process::Command;
use std::time::Instant;

use super::*;
use crate::open::{
    EXIT_DURING_OVERWRITE, FAIL_OVERWRITE, ZERO_CHUNK, fill_free_pages, overwrite_marker, sibling,
};

const ROLE: &str = "KB_STORE_UNIT_CRASH_ROLE";
const DB: &str = "KB_STORE_UNIT_CRASH_DB";
pub(super) const DOC_MARK: &str = "ABGEBROCHEN-Dokument-Ä4R7";
const CHAT_MARK: &str = "ABGEBROCHEN-Antwort-Ö2T9";
/// The chat the child saves after an overwrite that failed (G54 (a)).
const AFTER_FAILURE: &str = "chat-nach-fehlschlag";

fn marked_document(elements: usize) -> Document {
    let mut document = big_document("doc-y", elements);
    for (i, element) in document.elements.iter_mut().enumerate() {
        element.text = format!("{i} {DOC_MARK} {}", element.text);
    }
    document
}

fn marked_chat(parts: usize) -> Chat {
    let mut chat = chat("chat-y", "abgebrochen", AnswerOutcome::Complete);
    chat.turns[0].answer.text = (0..parts).map(|i| format!("{CHAT_MARK} {i}. ")).collect();
    chat
}

/// Entry of the child process. Does nothing when run as a normal test.
#[test]
fn child_process() {
    let (Ok(role), Ok(db)) = (std::env::var(ROLE), std::env::var(DB)) else {
        return;
    };
    let db = PathBuf::from(db);
    if role == "reopen" {
        run(async { drop(SqliteStore::open(&db).await.expect("reopen")) });
        println!("KBSTORE-REOPENED");
        return;
    }
    if role == "fail-overwrite" {
        // The overwrite fails; the opening goes on, and the store reads and writes.
        FAIL_OVERWRITE.store(true, std::sync::atomic::Ordering::SeqCst);
        let used = run(async {
            let store = SqliteStore::open(&db).await?;
            let read = store.documents().await?.len();
            let written = chat(
                AFTER_FAILURE,
                "nach dem Fehlschlag",
                AnswerOutcome::Complete,
            );
            store.save_chat(&written).await?;
            let back = store.chat(&written.id).await? == Some(written);
            Ok::<_, Error>((read, back))
        });
        match used {
            Ok((read, back)) => println!("KBSTORE-OPENED read {read} written {back}"),
            Err(e) => println!("KBSTORE-ERR {e}"),
        }
        return;
    }
    if role == "overwrite" {
        EXIT_DURING_OVERWRITE.store(true, std::sync::atomic::Ordering::SeqCst);
        let _ = run(SqliteStore::open(&db));
        panic!("the opening did not overwrite free pages");
    }
    run(async {
        let store = SqliteStore::open(&db).await.expect("open");
        let _: Result<()> = store
            .with_connection(move |c| {
                let started = || -> rusqlite::Result<()> {
                    c.pragma_update(None, "cache_size", 1)?;
                    c.execute_batch("BEGIN IMMEDIATE;")?;
                    if role == "content" {
                        let document = marked_document(400);
                        c.execute(
                            "INSERT INTO contents (document_id, document) VALUES (?1, ?2)
                             ON CONFLICT (document_id) DO UPDATE SET document = excluded.document",
                            rusqlite::params![
                                document.source.document_id.0,
                                serde_json::to_string(&document).expect("json")
                            ],
                        )?;
                    } else {
                        let chat = marked_chat(6_000);
                        c.execute(
                            "INSERT INTO chats (id, title, created_at, updated_at, turns)
                             VALUES (?1, ?2, 0, 0, ?3)",
                            rusqlite::params![
                                chat.id.0,
                                chat.title,
                                serde_json::to_string(&chat.turns).expect("json")
                            ],
                        )?;
                    }
                    Ok(())
                };
                started().expect("write started");
                // Ends hard: no commit, no rollback, no closing.
                std::process::exit(0);
            })
            .await;
    });
}

fn child(role: &str, db: &Path) -> std::process::Output {
    Command::new(std::env::current_exe().unwrap())
        .args([
            "tests::crash::child_process",
            "--exact",
            "--nocapture",
            "--test-threads=1",
        ])
        .env(ROLE, role)
        .env(DB, db)
        .output()
        .unwrap()
}

pub(super) fn abort_in_child(role: &str, db: &Path) {
    let output = Command::new(std::env::current_exe().unwrap())
        .args([
            "tests::crash::child_process",
            "--exact",
            "--nocapture",
            "--test-threads=1",
        ])
        .env(ROLE, role)
        .env(DB, db)
        .output()
        .unwrap();
    // The child ends inside the write (code 0) or inside the overwrite (code 3).
    let expected = if role == "overwrite" { 3 } else { 0 };
    assert_eq!(
        output.status.code(),
        Some(expected),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(!String::from_utf8_lossy(&output.stdout).contains("FAILED"));
}

pub(super) fn journal_len(db: &Path) -> u64 {
    sibling(db, "-journal").metadata().map_or(0, |m| m.len())
}

/// `true` if the helper table of the overwrite is in the schema.
pub(super) fn has_helper_table(db: &Path) -> bool {
    let conn = rusqlite::Connection::open(db).unwrap();
    conn.query_row(
        "SELECT count(*) FROM sqlite_schema WHERE name = 'store_overwrite'",
        [],
        |r| r.get::<_, i64>(0),
    )
    .unwrap()
        > 0
}

pub(super) fn integrity(db: &Path) -> String {
    let conn = rusqlite::Connection::open(db).unwrap();
    conn.query_row("PRAGMA integrity_check", [], |r| r.get(0))
        .unwrap()
}

/// Confirmed data plus a large document that was saved and removed again, so that the
/// free list is long and already zeroed. Returns what the store reads.
pub(super) fn prepare(db: &Path, freed_elements: usize) -> Snapshot {
    run(async {
        let store = SqliteStore::open(db).await.unwrap();
        fill(&store).await;
        let freed = big_document("doc-weg", freed_elements);
        store.save_content(&freed).await.unwrap();
        store
            .delete_document(&freed.source.document_id)
            .await
            .unwrap();
        snapshot(&store).await
    })
}

/// Rolls a hot journal back the way any SQLite connection does, without the rule.
pub(super) fn recover_without_the_rule(db: &Path) {
    let conn = rusqlite::Connection::open(db).unwrap();
    let _: i64 = conn
        .query_row("SELECT count(*) FROM sqlite_schema", [], |r| r.get(0))
        .unwrap();
}

pub(super) fn occurrences(folder: &Path, needle: &str) -> usize {
    walk(folder)
        .iter()
        .map(|p| {
            let bytes = std::fs::read(p).unwrap();
            bytes
                .windows(needle.len())
                .filter(|w| *w == needle.as_bytes())
                .count()
        })
        .sum()
}

/// Opens with the store after a cut-off write and checks every promise of G51.
pub(super) fn reopen_and_check(scratch: &Scratch, before: &Snapshot, mark: &str) {
    let db = scratch.db();
    assert!(journal_len(&db) > 0, "the child left a hot journal");
    let after = run(async { snapshot(&SqliteStore::open(&db).await.unwrap()).await });
    assert_eq!(&after, before, "confirmed data unchanged");
    assert_eq!(occurrences(&scratch.folder(), mark), 0);
    assert_eq!(integrity(&db), "ok");
    assert!(!has_helper_table(&db), "no helper table after success");
    assert_eq!(journal_len(&db), 0);
    // The marker is gone after success: only the database and its journal are left.
    assert!(!overwrite_marker(&db).exists());
    assert_eq!(
        names(&scratch.folder()),
        ["library.sqlite", "library.sqlite-journal"]
    );
    // Without a hot journal, opening does not change the file (G28).
    let bytes = std::fs::read(&db).unwrap();
    run(async { drop(SqliteStore::open(&db).await.unwrap()) });
    assert_eq!(std::fs::read(&db).unwrap(), bytes);
}

#[test]
fn text_of_a_cut_off_content_is_overwritten_at_the_next_opening() {
    // Control: without the rule the text stays in the file.
    let control = Scratch::new();
    prepare(&control.db(), 800);
    abort_in_child("content", &control.db());
    assert!(journal_len(&control.db()) > 0);
    recover_without_the_rule(&control.db());
    let left = occurrences(&control.folder(), DOC_MARK);
    eprintln!("control: {left} occurrences of the cut-off document text");
    assert!(left > 0, "control: the cut-off text is in the file");

    let scratch = Scratch::new();
    let db = scratch.db();
    let before = prepare(&db, 800);
    abort_in_child("content", &db);
    reopen_and_check(&scratch, &before, DOC_MARK);

    // Follow-up from the review: then saved shorter and removed.
    run(async {
        let store = SqliteStore::open(&db).await.unwrap();
        let shorter = marked_document(100);
        store.save_content(&shorter).await.unwrap();
        store
            .delete_document(&shorter.source.document_id)
            .await
            .unwrap();
    });
    assert_eq!(occurrences(&scratch.folder(), DOC_MARK), 0);
    assert_eq!(integrity(&db), "ok");
}

#[test]
fn text_of_a_cut_off_chat_is_overwritten_at_the_next_opening() {
    let control = Scratch::new();
    prepare(&control.db(), 800);
    abort_in_child("chat", &control.db());
    recover_without_the_rule(&control.db());
    let left = occurrences(&control.folder(), CHAT_MARK);
    eprintln!("control: {left} occurrences of the cut-off answer text");
    assert!(left > 0, "control: the cut-off text is in the file");

    let scratch = Scratch::new();
    let db = scratch.db();
    let before = prepare(&db, 800);
    abort_in_child("chat", &db);
    reopen_and_check(&scratch, &before, CHAT_MARK);

    // Follow-up from the review: saved shorter (200 parts), then deleted.
    run(async {
        let store = SqliteStore::open(&db).await.unwrap();
        let shorter = marked_chat(200);
        store.save_chat(&shorter).await.unwrap();
        assert_eq!(
            store.chat(&shorter.id).await.unwrap(),
            Some(shorter.clone())
        );
        store.delete_chat(&shorter.id).await.unwrap();
    });
    assert_eq!(occurrences(&scratch.folder(), CHAT_MARK), 0);
    assert_eq!(integrity(&db), "ok");
}

#[test]
fn an_overwrite_that_is_cut_off_is_done_again_at_the_next_opening() {
    let scratch = Scratch::new();
    let db = scratch.db();
    // A free list larger than the page cache, so that zeros reach the file before the
    // overwrite is cut off.
    let before = prepare(&db, 20_000);
    abort_in_child("content", &db);
    assert!(occurrences(&scratch.folder(), DOC_MARK) > 0);
    abort_in_child("overwrite", &db);
    // Either the next opening sees a hot journal again, or the pages are clean already.
    assert!(
        journal_len(&db) > 0 || occurrences(&scratch.folder(), DOC_MARK) == 0,
        "cut-off overwrite left text without a hot journal"
    );
    assert!(
        journal_len(&db) > 0,
        "the overwrite was cut off in its transaction"
    );
    assert!(
        overwrite_marker(&db).is_file(),
        "the marker outlives the cut-off overwrite"
    );
    reopen_and_check(&scratch, &before, DOC_MARK);
}

#[test]
fn the_overwrite_writes_no_file_outside_the_folder() {
    // G51 with G02: the opening after a cut-off write runs in a child whose temporary
    // folder of the system is a plain file, below which no file can be created.
    let scratch = Scratch::new();
    let db = scratch.db();
    prepare(&db, 20_000);
    abort_in_child("content", &db);
    assert!(journal_len(&db) > 0);
    let blocked = scratch.root().join("system-temp");
    std::fs::write(&blocked, "").unwrap();
    let output = Command::new(std::env::current_exe().unwrap())
        .args([
            "tests::crash::child_process",
            "--exact",
            "--nocapture",
            "--test-threads=1",
        ])
        .env(ROLE, "reopen")
        .env(DB, &db)
        .env("TMP", &blocked)
        .env("TEMP", &blocked)
        .env("TMPDIR", &blocked)
        .env("SQLITE_TMPDIR", &blocked)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success() && stdout.contains("KBSTORE-REOPENED"),
        "{stdout}"
    );
    assert_eq!(names(scratch.root()), ["Daten", "system-temp"]);
    assert_eq!(std::fs::read(&blocked).unwrap(), b"");
    assert_eq!(
        names(&scratch.folder()),
        ["library.sqlite", "library.sqlite-journal"]
    );
    assert_eq!(journal_len(&db), 0);
    assert_eq!(occurrences(&scratch.folder(), DOC_MARK), 0);
}

/// Measurement for the report: opening after a cut-off write with about 10,000 free
/// pages. Run with `cargo test -p kb-store --lib -- --ignored --nocapture`.
#[test]
#[ignore = "measurement for the report, not a check"]
fn measure_opening_after_a_cut_off_write() {
    let scratch = Scratch::new();
    let db = scratch.db();
    prepare(&db, 122_000);
    abort_in_child("content", &db);
    let size = std::fs::metadata(&db).unwrap().len();
    let start = Instant::now();
    run(async { drop(SqliteStore::open(&db).await.unwrap()) });
    let after_cut = start.elapsed();
    // Free pages after the opening: those it overwrote (plus its own two).
    let free: i64 = {
        let conn =
            rusqlite::Connection::open_with_flags(&db, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
                .unwrap();
        conn.pragma_query_value(None, "freelist_count", |r| r.get(0))
            .unwrap()
    };
    let start = Instant::now();
    run(async { drop(SqliteStore::open(&db).await.unwrap()) });
    let normal = start.elapsed();
    println!(
        "opening after a cut-off write: {after_cut:?} with about {free} free pages (file {size} bytes); normal opening {normal:?}; file afterwards {} bytes",
        std::fs::metadata(&db).unwrap().len()
    );
    assert_eq!(occurrences(&scratch.folder(), DOC_MARK), 0);
}

// ---- G54 ---------------------------------------------------------------------------

#[test]
fn a_failed_overwrite_still_opens_and_is_done_again_later() {
    // G54 (a): the overwrite fails like on a full disk. Its rollback empties the journal,
    // the opening succeeds all the same, reading and writing work, and the marker keeps
    // the cause.
    let scratch = Scratch::new();
    let db = scratch.db();
    let before = prepare(&db, 800);
    abort_in_child("content", &db);
    let output = child("fail-overwrite", &db);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains(&format!(
            "KBSTORE-OPENED read {} written true",
            before.documents.len()
        )),
        "{stdout}"
    );
    assert_eq!(journal_len(&db), 0, "the rollback emptied the journal");
    assert!(overwrite_marker(&db).is_file(), "the marker stays");
    assert!(
        occurrences(&scratch.folder(), DOC_MARK) > 0,
        "not overwritten yet: the text can remain in free pages until then"
    );
    assert!(!has_helper_table(&db));
    assert_eq!(integrity(&db), "ok", "the file is intact");
    // The next opening with enough space overwrites although the journal is empty.
    run(async {
        let store = SqliteStore::open(&db).await.unwrap();
        assert!(
            store
                .chat(&ChatId::new(AFTER_FAILURE))
                .await
                .unwrap()
                .is_some(),
            "the write after the failure was kept"
        );
    });
    assert_eq!(occurrences(&scratch.folder(), DOC_MARK), 0);
    assert!(!has_helper_table(&db));
    assert!(!overwrite_marker(&db).exists());
    assert_eq!(integrity(&db), "ok");
    assert_eq!(
        names(&scratch.folder()),
        ["library.sqlite", "library.sqlite-journal"]
    );
    // Apart from that chat, the confirmed data is unchanged.
    let after = run(async {
        let store = SqliteStore::open(&db).await.unwrap();
        store
            .delete_chat(&ChatId::new(AFTER_FAILURE))
            .await
            .unwrap();
        snapshot(&store).await
    });
    assert_eq!(after, before);
}

#[test]
fn a_marker_that_cannot_be_set_stops_the_opening_before_the_recovery() {
    // G54 (a): something that is no file lies where the marker goes.
    let scratch = Scratch::new();
    let db = scratch.db();
    let before = prepare(&db, 800);
    abort_in_child("content", &db);
    let blocker = overwrite_marker(&db);
    std::fs::create_dir_all(blocker.join("im-weg")).unwrap();
    std::fs::write(blocker.join("im-weg").join("datei"), "x").unwrap();
    let database = std::fs::read(&db).unwrap();
    let journal = std::fs::read(sibling(&db, "-journal")).unwrap();
    assert!(!journal.is_empty());
    let error = run(async { SqliteStore::open(&db).await.unwrap_err() });
    assert!(matches!(error, Error::Store(_)), "{error:?}");
    assert!(
        error.to_string().contains("cannot set the marker"),
        "{error}"
    );
    // SQLite has not touched the files: the journal is still there to recover from.
    assert_eq!(std::fs::read(&db).unwrap(), database);
    assert_eq!(std::fs::read(sibling(&db, "-journal")).unwrap(), journal);
    std::fs::remove_dir_all(&blocker).unwrap();
    reopen_and_check(&scratch, &before, DOC_MARK);
}

#[test]
fn a_marker_without_a_journal_still_overwrites() {
    // A cut after the marker was set and SQLite recovered (or after the overwrite and
    // before the marker was removed): the journal is empty, the marker decides.
    let scratch = Scratch::new();
    let db = scratch.db();
    let before = prepare(&db, 800);
    abort_in_child("content", &db);
    recover_without_the_rule(&db);
    std::fs::write(overwrite_marker(&db), "").unwrap();
    assert_eq!(journal_len(&db), 0);
    assert!(occurrences(&scratch.folder(), DOC_MARK) > 0);
    let after = run(async { snapshot(&SqliteStore::open(&db).await.unwrap()).await });
    assert_eq!(after, before);
    assert_eq!(occurrences(&scratch.folder(), DOC_MARK), 0);
    assert!(!overwrite_marker(&db).exists());
    assert!(!has_helper_table(&db));
    assert_eq!(integrity(&db), "ok");
}

#[test]
fn a_marker_next_to_a_new_database_is_removed() {
    let scratch = Scratch::new();
    let db = scratch.db();
    std::fs::create_dir_all(scratch.folder()).unwrap();
    std::fs::write(overwrite_marker(&db), "").unwrap();
    run(async {
        let store = SqliteStore::open(&db).await.unwrap();
        assert_eq!(store.settings().await.unwrap(), Settings::default());
    });
    assert_eq!(
        names(&scratch.folder()),
        ["library.sqlite", "library.sqlite-journal"]
    );
}

#[test]
fn the_backup_before_a_migration_holds_no_cut_off_text() {
    // G54 (b): the free pages are overwritten before the backup copies them.
    let scratch = Scratch::new();
    let db = scratch.db();
    let before = prepare(&db, 800);
    abort_in_child("content", &db);
    let after =
        run(async { snapshot(&SqliteStore::open_with(&db, &SCHEMA_V2).await.unwrap()).await });
    assert_eq!(after, before);
    let saved = sibling(&db, ".v1.bak");
    assert!(saved.is_file());
    let bytes = std::fs::read(&saved).unwrap();
    let in_backup = bytes
        .windows(DOC_MARK.len())
        .filter(|w| *w == DOC_MARK.as_bytes())
        .count();
    assert_eq!(in_backup, 0, "no cut-off text in the backup");
    assert_eq!(occurrences(&scratch.folder(), DOC_MARK), 0);
    assert!(!has_helper_table(&db));
}

#[test]
fn the_zeros_use_up_the_whole_free_list() {
    // K6, what the reserve of one page protects: with one value and with many small
    // ones, no free page is left once the helper table is filled.
    let scratch = Scratch::new();
    let db = scratch.db();
    prepare(&db, 3_000);
    let rounds = run(async {
        let store = SqliteStore::open(&db).await.unwrap();
        store
            .with_connection(|c| {
                let check =
                    |c: &mut rusqlite::Connection| -> rusqlite::Result<Vec<(i64, i64, i64)>> {
                        let page: i64 = c.pragma_query_value(None, "page_size", |r| r.get(0))?;
                        let mut rounds = Vec::new();
                        for chunk in [ZERO_CHUNK, 3 * page, 7 * page, 50 * page] {
                            // Rolled back at the end of each round, so the free list stays.
                            let tx = c.transaction()?;
                            let free: i64 =
                                tx.pragma_query_value(None, "freelist_count", |r| r.get(0))?;
                            tx.execute_batch(
                                "CREATE TABLE store_overwrite (zeros BLOB NOT NULL) STRICT;",
                            )?;
                            fill_free_pages(&tx, chunk)?;
                            let left: i64 =
                                tx.pragma_query_value(None, "freelist_count", |r| r.get(0))?;
                            rounds.push((chunk, free, left));
                        }
                        Ok(rounds)
                    };
                check(c).map_err(crate::error::sqlite)
            })
            .await
            .unwrap()
    });
    for (chunk, free, left) in rounds {
        assert!(free > 100, "free list of {free} pages");
        assert_eq!(left, 0, "chunk {chunk}: {left} of {free} pages left");
    }
}
