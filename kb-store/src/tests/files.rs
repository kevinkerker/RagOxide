//! Every file the store writes lies in the folder of the database (G02).
//!
//! A child process (this test program) runs every method, stores a large content twice
//! and migrates, with the temporary folder of the system pointed at a path below which
//! no file can be created (a plain file). Had SQLite tried to put a temporary file
//! there, the call would have failed. Afterwards only the data folder exists, and it
//! holds only the database and its journal.

use std::process::Command;

use super::*;

const CHILD_DB: &str = "KB_STORE_UNIT_CHILD_DB";

/// Entry of the child process. Does nothing when run as a normal test.
#[test]
fn child_process() {
    let Ok(db) = std::env::var(CHILD_DB) else {
        return;
    };
    let db = PathBuf::from(db);
    run(async {
        let store = SqliteStore::open(&db).await.expect("open");
        let temp_store: i64 = store
            .with_connection(|c| {
                c.pragma_query_value(None, "temp_store", |r| r.get(0))
                    .map_err(crate::error::sqlite)
            })
            .await
            .expect("temp_store");
        assert_eq!(temp_store, 2, "PRAGMA temp_store is MEMORY");
        fill(&store).await;
        // Enough chats with long titles that sorting the chat list exceeds the page
        // cache; on disk, SQLite would sort in temporary files.
        let long = "Lange Überschrift ".repeat(140);
        for i in 0..1_500_u64 {
            store
                .save_chat(&Chat {
                    id: ChatId::new(format!("lang-{i:05}")),
                    title: format!("{i} {long}"),
                    created_at: Timestamp(i),
                    updated_at: Timestamp(i * 7 % 1_501),
                    turns: Vec::new(),
                })
                .await
                .expect("chat");
        }
        assert_eq!(store.chats().await.expect("chats").len(), 1_503);
        let records = store.documents().await.expect("documents");
        let first = &records[0];
        store.document(&first.id).await.expect("document");
        store.document_by_path(&first.path).await.expect("by path");
        store.content(&first.id).await.expect("content");
        store.delete_document(&first.id).await.expect("delete");
        store.chat(&ChatId::new("c1")).await.expect("chat");
        store.chats().await.expect("chats");
        store
            .delete_chat(&ChatId::new("c2"))
            .await
            .expect("delete chat");
        store.settings().await.expect("settings");
        store
            .update_settings(Box::new(|s| {
                s.watched_folders.push(PathBuf::from(r"C:\Neu"));
                Ok(())
            }))
            .await
            .expect("update");
        // A large content, stored twice: replacing it needs a large statement journal.
        let big = big_document("doc-big", 100_000);
        store.save_content(&big).await.expect("big");
        store.save_content(&big).await.expect("big again");
        assert_eq!(
            store.content(&big.source.document_id).await.expect("read"),
            Some(big.clone())
        );
        store
            .delete_document(&big.source.document_id)
            .await
            .expect("delete big");
        drop(store);
        // Migration with backup, then the opening that removes the backup.
        drop(
            SqliteStore::open_with(&db, &SCHEMA_V2)
                .await
                .expect("migrate"),
        );
        let store = SqliteStore::open_with(&db, &SCHEMA_V2)
            .await
            .expect("reopen");
        store.chats().await.expect("chats");
    });
    println!("KBSTORE-DONE");
}

#[test]
fn every_file_lies_in_the_folder_of_the_database() {
    let scratch = Scratch::new();
    let blocked = scratch.root().join("system-temp");
    std::fs::write(&blocked, "").unwrap();
    let output = Command::new(std::env::current_exe().unwrap())
        .args([
            "tests::files::child_process",
            "--exact",
            "--nocapture",
            "--test-threads=1",
        ])
        .env(CHILD_DB, scratch.db())
        .env("TMP", &blocked)
        .env("TEMP", &blocked)
        .env("TMPDIR", &blocked)
        .env("SQLITE_TMPDIR", &blocked)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success() && stdout.contains("KBSTORE-DONE"),
        "{stdout}\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(names(scratch.root()), ["Daten", "system-temp"]);
    assert_eq!(std::fs::read(&blocked).unwrap(), b"");
    assert_eq!(
        names(&scratch.folder()),
        ["library.sqlite", "library.sqlite-journal"]
    );
}
