//! A full database is `Error::StorageFull`, a write that SQLite refuses otherwise is
//! `Error::Store`, both with the text of SQLite and never a panic (G40, K45).

use super::*;

#[test]
fn a_full_database_is_storage_full_with_the_text_of_sqlite() {
    let scratch = Scratch::new();
    run(async {
        let store = SqliteStore::open(scratch.db()).await.unwrap();
        fill(&store).await;
        let before = snapshot(&store).await;
        // No page may be added: the next larger write fails with SQLITE_FULL.
        store
            .with_connection(|c| {
                let pages: i64 = c
                    .pragma_query_value(None, "page_count", |r| r.get(0))
                    .map_err(crate::error::sqlite)?;
                c.pragma_update(None, "max_page_count", pages)
                    .map_err(crate::error::sqlite)
            })
            .await
            .unwrap();
        let big = big_document("doc-big", 5_000);
        let error = store.save_content(&big).await.unwrap_err();
        assert!(matches!(error, Error::StorageFull(_)), "{error:?}");
        assert_eq!(error.to_string(), "storage full: database or disk is full");
        let mut chat = chat("voll", &"x".repeat(100_000), AnswerOutcome::Complete);
        chat.turns.push(chat.turns[0].clone());
        let error = store.save_chat(&chat).await.unwrap_err();
        assert!(matches!(error, Error::StorageFull(_)), "{error:?}");
        assert_eq!(error.to_string(), "storage full: database or disk is full");
        // Nothing half written, and the store keeps working.
        assert!(
            store
                .content(&big.source.document_id)
                .await
                .unwrap()
                .is_none()
        );
        assert!(store.chat(&chat.id).await.unwrap().is_none());
        assert_eq!(snapshot(&store).await, before);
        store.delete_chat(&ChatId::new("c1")).await.unwrap();
    });
}

#[test]
fn a_write_that_sqlite_refuses_is_a_store_error_with_the_text_of_sqlite() {
    // SQLite refuses every write once `query_only` is set after the opening. (A
    // read-only attribute set after the opening does not stop writes through the handle
    // that is already open under Windows; opening a read-only file is G55.)
    let scratch = Scratch::new();
    run(async {
        let store = SqliteStore::open(scratch.db()).await.unwrap();
        store
            .save_chat(&chat("c1", "eins", AnswerOutcome::Complete))
            .await
            .unwrap();
        store
            .with_connection(|c| {
                c.pragma_update(None, "query_only", true)
                    .map_err(crate::error::sqlite)
            })
            .await
            .unwrap();
        let error = store
            .save_chat(&chat("c2", "zwei", AnswerOutcome::Complete))
            .await
            .unwrap_err();
        assert!(matches!(error, Error::Store(_)), "{error:?}");
        assert_eq!(
            error.to_string(),
            "store error: attempt to write a readonly database"
        );
        let error = store
            .update_settings(Box::new(|s| {
                s.watched_folders.push(PathBuf::from(r"C:\Ablage"));
                Ok(())
            }))
            .await
            .unwrap_err();
        assert!(matches!(error, Error::Store(_)), "{error:?}");
        assert!(error.to_string().contains("readonly"), "{error}");
        // Nothing written, and the store keeps working for reads.
        assert_eq!(store.settings().await.unwrap(), Settings::default());
        assert_eq!(store.chats().await.unwrap().len(), 1);
    });
}
