//! The same sequence through `KnowledgeBase`, once with `MockStore` and once with
//! `SqliteStore`, with the same mocks around: every result and the state afterwards are
//! equal (G43).

#![cfg(windows)]

mod common;

use std::path::Path;
use std::sync::{Arc, Mutex};

use common::*;
use futures::StreamExt;
use kb_core::mock::{MockClock, MockFiles, MockStore};
use kb_core::*;

/// Everything the sequence returns, in order.
#[derive(Debug, PartialEq)]
struct Outcome {
    settings_after_add: Settings,
    first_sync: Vec<FileResult>,
    documents: Vec<DocumentRecord>,
    contents: Vec<Option<Document>>,
    first_answer: Vec<AnswerEvent>,
    second_answer: Vec<AnswerEvent>,
    asked_again: Vec<AnswerEvent>,
    chat_before_delete: Option<Chat>,
    chats_before_delete: Vec<ChatSummary>,
    settings_after_exclude: Settings,
    second_sync: Vec<FileResult>,
    documents_after_exclude: Vec<DocumentRecord>,
    settings_after_include: Settings,
    third_sync: Vec<FileResult>,
    progress: Vec<IngestEvent>,
    scope: Vec<ScopeEvent>,
    final_documents: Vec<DocumentRecord>,
    final_chats: Vec<ChatSummary>,
    final_chat: Option<Chat>,
    final_settings: Settings,
}

async fn answer(stream: Result<AnswerStream>) -> Vec<AnswerEvent> {
    stream
        .expect("ask")
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .map(|e| e.expect("event"))
        .collect()
}

async fn sequence(store: Arc<dyn Store>) -> Outcome {
    let clock = Arc::new(MockClock::new(Timestamp(1_794_700_800_000)));
    let kb = knowledge_base(Arc::clone(&store), Arc::clone(&clock) as Arc<dyn Clock>);
    let progress_events = Arc::new(Mutex::new(Vec::new()));
    let scope_events = Arc::new(Mutex::new(Vec::new()));
    let progress: ProgressCallback = {
        let events = Arc::clone(&progress_events);
        Arc::new(move |e| events.lock().unwrap().push(e))
    };
    let scope: ScopeCallback = {
        let events = Arc::clone(&scope_events);
        Arc::new(move |e| events.lock().unwrap().push(e))
    };
    let sync = || {
        kb.sync(
            Arc::clone(&progress),
            Arc::clone(&scope),
            SyncMode::Normal,
            CancellationToken::new(),
        )
    };
    let chat = ChatId::new("chat-vergleich");
    let excluded = Path::new(MockFiles::FIXTURE_FOLDER).join("budget.xlsx");

    let settings_after_add = kb
        .add_folder(Path::new(MockFiles::FIXTURE_FOLDER))
        .await
        .unwrap();
    let first_sync = sync().await.unwrap();
    let documents = kb.documents().await.unwrap();
    let mut contents = Vec::new();
    for record in &documents {
        contents.push(kb.document_content(&record.id).await.unwrap());
    }
    let first_answer = answer(
        kb.ask(
            "Wie oft wird ein Gerät geprüft?",
            Some(&chat),
            AnswerLevel::Fast,
            CancellationToken::new(),
        )
        .await,
    )
    .await;
    clock.advance(60_000);
    let second_answer = answer(
        kb.ask(
            "Und wer legt die Frist fest?",
            Some(&chat),
            AnswerLevel::Balanced,
            CancellationToken::new(),
        )
        .await,
    )
    .await;
    clock.advance(60_000);
    let asked_again = answer(
        kb.ask_again(&chat, 0, AnswerLevel::Thorough, CancellationToken::new())
            .await,
    )
    .await;
    kb.rename_chat(&chat, "Prüffristen der Geräte")
        .await
        .unwrap();
    let chat_before_delete = store.chat(&chat).await.unwrap();
    let chats_before_delete = store.chats().await.unwrap();
    let settings_after_exclude = kb.exclude(&excluded).await.unwrap();
    let second_sync = sync().await.unwrap();
    let documents_after_exclude = kb.documents().await.unwrap();
    let settings_after_include = kb.include(&excluded).await.unwrap();
    let third_sync = sync().await.unwrap();
    kb.delete_chat(&chat).await.unwrap();

    let progress = progress_events.lock().unwrap().clone();
    let scope = scope_events.lock().unwrap().clone();
    Outcome {
        settings_after_add,
        first_sync,
        documents,
        contents,
        first_answer,
        second_answer,
        asked_again,
        chat_before_delete,
        chats_before_delete,
        settings_after_exclude,
        second_sync,
        documents_after_exclude,
        settings_after_include,
        third_sync,
        progress,
        scope,
        final_documents: store.documents().await.unwrap(),
        final_chats: store.chats().await.unwrap(),
        final_chat: store.chat(&chat).await.unwrap(),
        final_settings: store.settings().await.unwrap(),
    }
}

#[test]
fn knowledge_base_gives_the_same_results_with_mock_and_sqlite_store() {
    // G43
    let scratch = Scratch::new();
    run(async {
        let with_mock = sequence(Arc::new(MockStore::new())).await;
        let with_sqlite = sequence(Arc::new(open(&scratch.db()).await)).await;

        // The sequence did what it should, so that the comparison means something.
        assert_eq!(with_mock.documents.len(), 7);
        assert!(with_mock.contents.iter().all(Option::is_some));
        assert_eq!(
            with_mock.chat_before_delete.as_ref().unwrap().turns.len(),
            3
        );
        assert_eq!(
            with_mock.chat_before_delete.as_ref().unwrap().title,
            "Prüffristen der Geräte"
        );
        assert_eq!(with_mock.documents_after_exclude.len(), 6);
        assert_eq!(with_mock.final_documents.len(), 7);
        assert!(with_mock.final_chats.is_empty() && with_mock.final_chat.is_none());

        assert_eq!(with_sqlite, with_mock);
    });
}
