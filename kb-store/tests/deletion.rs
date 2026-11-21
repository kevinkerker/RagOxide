//! Deleting overwrites: after `delete_chat` and `delete_document` no text of the deleted
//! object is left in the database file or its journal files (G20, G21; ADR 0120). The
//! same after a migration is tested in the crate's own tests (`src/tests`).

mod common;

use std::path::PathBuf;

use common::*;
use kb_core::mock::fixtures;
use kb_core::*;

/// A marker that occurs nowhere else, in UTF-8 with umlauts.
const CHAT_MARKER: &str = "Geheimnis-Prüfmarke-Ä7Q2";
const DOC_MARKER: &str = "Vertraulich-Größe-Ö9X4";

fn secret_chat() -> Chat {
    let mut chat = sample_chats().remove(0);
    chat.id = ChatId::new("secret-chat");
    chat.title = format!("Titel {CHAT_MARKER} Titel");
    chat.turns[0].question = format!("Frage {CHAT_MARKER} Frage");
    chat.turns[0].answer.text = format!("Antwort {CHAT_MARKER} Antwort [1][2]");
    chat
}

fn secret_document() -> (DocumentRecord, Document) {
    let mut document = fixtures::documents().remove("notizen.md").unwrap();
    let path = PathBuf::from(format!(r"C:\Ablage\Personal\{DOC_MARKER}-akte.md"));
    document.source =
        SourceFile::new(DocumentId::new("doc-secret"), &path, "c".repeat(64)).unwrap();
    for (i, element) in document.elements.iter_mut().enumerate() {
        element.text = format!("Absatz {i} {DOC_MARKER} mit Inhalt");
    }
    let record = DocumentRecord {
        title: Some(format!("Akte {DOC_MARKER}")),
        status: DocumentStatus::Indexed { chunks: 3 },
        ..record("doc-secret", path.to_str().unwrap())
    };
    (record, document)
}

/// Every form in which the path of the document could be written.
fn path_forms(record: &DocumentRecord) -> Vec<String> {
    let path = record.path.to_string_lossy().into_owned();
    vec![
        path.clone(),
        path.replace('\\', "\\\\"),
        path_key(&record.path).unwrap(),
        path_key(&record.path).unwrap().replace('\\', "\\\\"),
        format!("{DOC_MARKER}-akte.md"),
    ]
}

#[test]
fn a_deleted_chat_leaves_no_text_in_any_file() {
    // G20
    let scratch = Scratch::new();
    let folder = scratch.data_folder();
    let chat = secret_chat();
    run(async {
        let store = open(&scratch.db()).await;
        for other in sample_chats() {
            store.save_chat(&other).await.unwrap();
        }
        store.save_chat(&chat).await.unwrap();
        // Saved several times, so that older versions of its pages existed.
        let mut longer = chat.clone();
        longer.turns.push(chat.turns[0].clone());
        store.save_chat(&longer).await.unwrap();
        store.save_chat(&chat).await.unwrap();
    });
    // Control: before deleting, the text is in the database file.
    assert!(!files_containing(&folder, CHAT_MARKER).is_empty());
    run(async {
        let store = open(&scratch.db()).await;
        store.delete_chat(&chat.id).await.unwrap();
        assert_eq!(store.chats().await.unwrap().len(), 3);
    });
    let names = file_names(&folder);
    assert!(
        names.iter().all(|n| n.starts_with("library.sqlite")),
        "{names:?}"
    );
    assert_eq!(
        files_containing(&folder, CHAT_MARKER),
        Vec::<PathBuf>::new()
    );
    // The other chats are still there in the bytes.
    assert!(!files_containing(&folder, "Wie oft wird ein Gerät geprüft?").is_empty());
}

#[test]
fn a_deleted_chat_leaves_no_text_while_the_store_is_open() {
    // G20, and the journal files while the store is still open.
    let scratch = Scratch::new();
    let folder = scratch.data_folder();
    let chat = secret_chat();
    run(async {
        let store = open(&scratch.db()).await;
        store.save_chat(&chat).await.unwrap();
        store.delete_chat(&chat.id).await.unwrap();
        assert_eq!(
            files_containing(&folder, CHAT_MARKER),
            Vec::<PathBuf>::new()
        );
    });
    assert_eq!(
        files_containing(&folder, CHAT_MARKER),
        Vec::<PathBuf>::new()
    );
}

#[test]
fn a_deleted_document_leaves_no_content_or_path_in_any_file() {
    // G21
    let scratch = Scratch::new();
    let folder = scratch.data_folder();
    let (record, document) = secret_document();
    run(async {
        let store = open(&scratch.db()).await;
        for (r, d) in fixture_library() {
            store.upsert_document(&r).await.unwrap();
            store.save_content(&d).await.unwrap();
        }
        store.upsert_document(&record).await.unwrap();
        store.save_content(&document).await.unwrap();
        // The chats cite other documents, not this one.
        for chat in sample_chats() {
            store.save_chat(&chat).await.unwrap();
        }
    });
    // Control: content, path (JSON-escaped in the record) and path key are in the file.
    let forms = path_forms(&record);
    for form in [&forms[1], &forms[2], &forms[4]] {
        assert!(!files_containing(&folder, form).is_empty(), "{form}");
    }
    assert!(!files_containing(&folder, &format!("Absatz 1 {DOC_MARKER}")).is_empty());
    run(async {
        let store = open(&scratch.db()).await;
        store.delete_document(&record.id).await.unwrap();
        assert_eq!(store.documents().await.unwrap().len(), 7);
    });
    assert_eq!(files_containing(&folder, DOC_MARKER), Vec::<PathBuf>::new());
    for form in path_forms(&record) {
        assert_eq!(
            files_containing(&folder, &form),
            Vec::<PathBuf>::new(),
            "{form}"
        );
    }
}
