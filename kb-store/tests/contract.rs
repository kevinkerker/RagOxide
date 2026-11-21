//! The methods of `Store` behave as the contract and `MockStore` say (G01, G06 to G11,
//! G13 to G15, G17, G18, G23, G24, G26, G41, G42).

mod common;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use common::*;
use kb_core::mock::{MockClock, MockStore, fixtures};
use kb_core::*;
use kb_store::SqliteStore;

#[test]
fn store_is_a_trait_object_for_the_knowledge_base_and_creates_its_folders() {
    // G01
    let scratch = Scratch::new();
    let path = scratch
        .root()
        .join("a")
        .join("b")
        .join("c")
        .join("library.sqlite");
    assert!(!path.parent().unwrap().exists());
    run(async {
        let store: Arc<dyn Store> = Arc::new(SqliteStore::open(&path).await.unwrap());
        let kb = knowledge_base(Arc::clone(&store), Arc::new(MockClock::new(Timestamp(1))));
        assert!(kb.documents().await.unwrap().is_empty());
        assert!(kb.store().chats().await.unwrap().is_empty());
    });
    assert!(path.is_file());
}

#[test]
fn relative_database_path_is_invalid_input() {
    // G01: the caller gives the path, the package has none of its own.
    run(async {
        let error = SqliteStore::open("library.sqlite").await.unwrap_err();
        assert!(matches!(error, Error::InvalidInput(_)), "{error:?}");
    });
    assert!(!Path::new("library.sqlite").exists());
}

fn all_kinds_of_records() -> Vec<DocumentRecord> {
    vec![
        record("q", r"C:\Ablage\offen.pdf"),
        DocumentRecord {
            title: Some("Prüfordnung".into()),
            content_hash: Some("ab".repeat(32)),
            size_bytes: Some(123_456),
            modified_at: Some(Timestamp(1_794_700_000_123)),
            status: DocumentStatus::Indexed { chunks: 17 },
            ..record("i", r"C:\Ablage\pruefordnung.pdf")
        },
        DocumentRecord {
            size_bytes: Some(10),
            modified_at: Some(Timestamp(5)),
            status: DocumentStatus::Failed {
                kind: ErrorKind::Parse,
                message: "PDF is encrypted (password required)".into(),
            },
            ..record("f", r"C:\Ablage\geschützt.pdf")
        },
        // Not normalised: the store keeps the path as given.
        record("raw", r"c:/Ablage/./Unterordner/../roh.PDF"),
    ]
}

#[test]
fn records_come_back_equal_in_every_status() {
    // G06
    let scratch = Scratch::new();
    run(async {
        let store = open(&scratch.db()).await;
        for record in all_kinds_of_records() {
            store.upsert_document(&record).await.unwrap();
            assert_eq!(store.document(&record.id).await.unwrap(), Some(record));
        }
        // Replacing with the same id.
        let mut changed = all_kinds_of_records().remove(0);
        changed.status = DocumentStatus::Indexed { chunks: 3 };
        changed.title = Some("Neu".into());
        store.upsert_document(&changed).await.unwrap();
        assert_eq!(store.document(&changed.id).await.unwrap(), Some(changed));
        assert_eq!(store.documents().await.unwrap().len(), 4);
    });
}

#[test]
fn a_second_record_for_the_same_file_is_refused() {
    // G07
    let scratch = Scratch::new();
    run(async {
        let store = open(&scratch.db()).await;
        let a = record("a", r"C:\Ablage\Bericht.pdf");
        let b = record("b", r"C:\Ablage\anderer.pdf");
        store.upsert_document(&a).await.unwrap();
        store.upsert_document(&b).await.unwrap();
        let before = store.documents().await.unwrap();

        for path in [r"C:\Ablage\Bericht.pdf", r"c:/ablage/BERICHT.pdf"] {
            let error = store.upsert_document(&record("c", path)).await.unwrap_err();
            assert!(matches!(error, Error::InvalidInput(_)), "{error:?}");
        }
        let error = store
            .upsert_document(&record("b", r"C:\ABLAGE\bericht.PDF"))
            .await
            .unwrap_err();
        assert!(matches!(error, Error::InvalidInput(_)), "{error:?}");
        assert_eq!(store.documents().await.unwrap(), before);
        assert!(
            store
                .document(&DocumentId::new("c"))
                .await
                .unwrap()
                .is_none()
        );

        // Same id and path (also in another spelling) replaces.
        let again = DocumentRecord {
            status: DocumentStatus::Indexed { chunks: 1 },
            ..record("a", r"C:\Ablage\Bericht.pdf")
        };
        store.upsert_document(&again).await.unwrap();
        assert_eq!(store.document(&again.id).await.unwrap(), Some(again));
        // A record may move to a free path.
        store
            .upsert_document(&record("b", r"C:\Ablage\verschoben.pdf"))
            .await
            .unwrap();
        assert_eq!(store.documents().await.unwrap().len(), 2);
    });
}

#[test]
fn unknown_ids_are_none() {
    // G08
    let scratch = Scratch::new();
    run(async {
        let store = open(&scratch.db()).await;
        assert!(
            store
                .document(&DocumentId::new("x"))
                .await
                .unwrap()
                .is_none()
        );
        assert!(
            store
                .content(&DocumentId::new("x"))
                .await
                .unwrap()
                .is_none()
        );
        assert!(store.chat(&ChatId::new("x")).await.unwrap().is_none());
    });
}

#[cfg(windows)]
#[test]
fn records_are_found_by_path_in_every_spelling() {
    // G09
    let scratch = Scratch::new();
    run(async {
        let store = open(&scratch.db()).await;
        let local = record("local", r"C:\Ablage\Prüfplan.PDF");
        let unc = record("unc", r"\\nas\Freigabe\Berichte\Jahr.pdf");
        store.upsert_document(&local).await.unwrap();
        store.upsert_document(&unc).await.unwrap();
        let local_spellings = [
            r"C:\Ablage\Prüfplan.PDF".to_owned(),
            r"C:\ABLAGE\prüfplan.pdf".to_owned(),
            "C:/Ablage/Prüfplan.PDF".to_owned(),
            r"C:\Ablage\.\Berichte\..\Prüfplan.PDF".to_owned(),
            r"\\?\C:\Ablage\Prüfplan.PDF".to_owned(),
            r"c:\Ablage\Prüfplan.PDF".to_owned(),
            "C:\\Ablage\\Pru\u{308}fplan.PDF".to_owned(),
        ];
        for spelling in &local_spellings {
            let found = store.document_by_path(Path::new(spelling)).await.unwrap();
            assert_eq!(found.as_ref(), Some(&local), "{spelling}");
        }
        for spelling in [
            r"\\nas\Freigabe\Berichte\Jahr.pdf",
            r"\\NAS\freigabe\berichte\JAHR.pdf",
            r"\\?\UNC\nas\Freigabe\Berichte\Jahr.pdf",
            "//nas/Freigabe/Berichte/./Jahr.pdf",
        ] {
            let found = store.document_by_path(Path::new(spelling)).await.unwrap();
            assert_eq!(found.as_ref(), Some(&unc), "{spelling}");
        }
        assert!(
            store
                .document_by_path(Path::new(r"C:\Ablage\fehlt.pdf"))
                .await
                .unwrap()
                .is_none()
        );
        let error = store
            .document_by_path(Path::new(r"Ablage\Prüfplan.PDF"))
            .await
            .unwrap_err();
        assert!(matches!(error, Error::InvalidInput(_)), "{error:?}");
    });
}

#[cfg(windows)]
#[test]
fn documents_are_sorted_by_path_key_like_the_mock() {
    // G10
    let paths = [
        r"C:\Ablage\z.pdf",
        r"C:\Ablage\Ä.pdf",
        r"C:\Ablage\a.pdf",
        r"C:\Ablage\B.pdf",
        "C:\\Ablage\\A\u{308}rger.pdf",
        r"C:\Ablage\Ärmel.pdf",
        r"c:\ablage\Zebra.pdf",
        r"C:\Ablage\_unterstrich.pdf",
        r"C:\Ablage\ß.pdf",
        r"C:\Ablage\ss.pdf",
        r"D:\a.pdf",
        r"\\nas\Freigabe\a.pdf",
    ];
    let scratch = Scratch::new();
    run(async {
        let store = open(&scratch.db()).await;
        let mock = MockStore::new();
        for (i, path) in paths.iter().enumerate() {
            let record = record(&format!("d{i:02}"), path);
            store.upsert_document(&record).await.unwrap();
            mock.upsert_document(&record).await.unwrap();
        }
        let ours: Vec<PathBuf> = store
            .documents()
            .await
            .unwrap()
            .into_iter()
            .map(|r| r.path)
            .collect();
        let theirs: Vec<PathBuf> = mock
            .documents()
            .await
            .unwrap()
            .into_iter()
            .map(|r| r.path)
            .collect();
        assert_eq!(ours, theirs);
        let keys: Vec<String> = ours.iter().map(|p| path_key(p).unwrap()).collect();
        let mut sorted = keys.clone();
        sorted.sort();
        assert_eq!(keys, sorted, "byte-wise by path_key");
    });
}

#[test]
fn deleting_a_document_removes_record_and_content_only() {
    // G11
    let scratch = Scratch::new();
    run(async {
        let store = open(&scratch.db()).await;
        let library = fixture_library();
        for (record, document) in &library {
            store.upsert_document(record).await.unwrap();
            store.save_content(document).await.unwrap();
        }
        // The chats cite chunks of the fixture documents.
        for chat in sample_chats() {
            store.save_chat(&chat).await.unwrap();
        }
        store.save_settings(&sample_settings()).await.unwrap();
        let before = snapshot(&store, &[]).await;

        let (gone, _) = &library[0];
        store.delete_document(&gone.id).await.unwrap();
        assert!(store.document(&gone.id).await.unwrap().is_none());
        assert!(store.content(&gone.id).await.unwrap().is_none());
        let after = snapshot(&store, &[]).await;
        assert_eq!(after.documents.len(), before.documents.len() - 1);
        let expected: Vec<_> = before
            .documents
            .iter()
            .filter(|r| r.id != gone.id)
            .cloned()
            .collect();
        assert_eq!(after.documents, expected);
        for (record, document) in &library[1..] {
            assert_eq!(
                store.content(&record.id).await.unwrap().as_ref(),
                Some(document)
            );
        }
        assert_eq!(after.chats, before.chats);
        assert_eq!(after.chat_contents, before.chat_contents);
        assert_eq!(after.settings, before.settings);

        // Unknown id: no error, nothing changes.
        store
            .delete_document(&DocumentId::new("unbekannt"))
            .await
            .unwrap();
        store.delete_document(&gone.id).await.unwrap();
        assert_eq!(snapshot(&store, &[]).await, after);
    });
}

#[test]
fn every_fixture_document_comes_back_equal() {
    // G13
    let scratch = Scratch::new();
    let documents = fixtures::documents();
    // The fixtures cover what the criterion names.
    let all: Vec<&Document> = documents.values().collect();
    assert!(all.iter().any(|d| d.pages.iter().any(|p| p.from_ocr)));
    assert!(all.iter().any(|d| d.elements.iter().any(|e| matches!(
        &e.kind,
        ElementKind::Table(t) if t.cells.iter().any(|c| c.anchor.is_some())
    ))));
    assert!(all.iter().any(|d| d.elements.iter().any(|e| {
        e.anchors
            .iter()
            .any(|a| matches!(a, Anchor::Slide { in_notes: true, .. }))
    })));
    run(async {
        let store = open(&scratch.db()).await;
        for document in documents.values() {
            store.save_content(document).await.unwrap();
        }
        for document in documents.values() {
            let id = &document.source.document_id;
            assert_eq!(
                store.content(id).await.unwrap().as_ref(),
                Some(document),
                "{id}"
            );
        }
    });
}

#[test]
fn boxes_keep_their_exact_f32_values() {
    // G13: values that have no short decimal form.
    let scratch = Scratch::new();
    let mut document = fixtures::documents().remove("pruefordnung.pdf").unwrap();
    let odd = [
        f32::MIN_POSITIVE,
        f32::EPSILON,
        0.1,
        1.0 / 3.0,
        16_777_217.0,
        f32::MAX,
        -0.0,
        f32::from_bits(5),
    ];
    document.pages[0].width = odd[3];
    document.pages[0].height = odd[5];
    document.elements[0].anchors = odd
        .chunks(4)
        .map(|v| Anchor::Page {
            page: 1,
            bbox: BoundingBox {
                left: v[0],
                top: v[1],
                right: v[2],
                bottom: v[3],
            },
        })
        .collect();
    run(async {
        let store = open(&scratch.db()).await;
        store.save_content(&document).await.unwrap();
        let back = store
            .content(&document.source.document_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(back, document);
        let bits = |d: &Document| -> Vec<u32> {
            d.elements[0]
                .anchors
                .iter()
                .flat_map(|a| match a {
                    Anchor::Page { bbox, .. } => {
                        vec![bbox.left, bbox.top, bbox.right, bbox.bottom]
                    }
                    _ => Vec::new(),
                })
                .map(f32::to_bits)
                .collect()
        };
        assert_eq!(bits(&back), bits(&document), "bit for bit, also -0.0");
    });
}

#[test]
fn non_finite_numbers_are_invalid_input() {
    // Deviation from MockStore: JSON has no NaN, the content could not be read back.
    let scratch = Scratch::new();
    let mut document = fixtures::documents().remove("pruefordnung.pdf").unwrap();
    document.pages[0].width = f32::NAN;
    let mut chat = sample_chats().remove(0);
    if let Anchor::Page { bbox, .. } =
        &mut chat.turns[0].answer.citations[0].chunk.provenance.anchors[0]
    {
        bbox.left = f32::INFINITY;
    } else {
        panic!("the first hit is from a PDF");
    }
    run(async {
        let store = open(&scratch.db()).await;
        let error = store.save_content(&document).await.unwrap_err();
        assert!(matches!(error, Error::InvalidInput(_)), "{error:?}");
        assert!(
            store
                .content(&document.source.document_id)
                .await
                .unwrap()
                .is_none()
        );
        let error = store.save_chat(&chat).await.unwrap_err();
        assert!(matches!(error, Error::InvalidInput(_)), "{error:?}");
        assert!(store.chat(&chat.id).await.unwrap().is_none());
    });
}

#[test]
fn a_second_content_replaces_the_first_completely() {
    // G14
    let scratch = Scratch::new();
    let documents = fixtures::documents();
    let first = documents["onboarding.docx"].clone();
    let mut second = documents["faq.html"].clone();
    second.source = first.source.clone();
    run(async {
        let store = open(&scratch.db()).await;
        store.save_content(&first).await.unwrap();
        store.save_content(&second).await.unwrap();
        let back = store
            .content(&first.source.document_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(back, second);
        for element in &first.elements {
            assert!(
                !back.elements.iter().any(|e| e.text == element.text),
                "old element still readable: {}",
                element.text
            );
        }
    });
}

#[test]
fn chats_come_back_equal_with_every_outcome() {
    // G15
    let scratch = Scratch::new();
    let chats = sample_chats();
    let outcomes: Vec<&AnswerOutcome> = chats
        .iter()
        .flat_map(|c| c.turns.iter().map(|t| &t.answer.outcome))
        .collect();
    assert!(outcomes.contains(&&AnswerOutcome::Complete));
    assert!(outcomes.contains(&&AnswerOutcome::Cancelled));
    assert!(
        outcomes
            .iter()
            .any(|o| matches!(o, AnswerOutcome::Failed { .. }))
    );
    run(async {
        let store = open(&scratch.db()).await;
        for chat in &chats {
            store.save_chat(chat).await.unwrap();
        }
        for chat in &chats {
            assert_eq!(store.chat(&chat.id).await.unwrap().as_ref(), Some(chat));
        }
        // Replacing with the same id, timestamps as given.
        let mut changed = chats[0].clone();
        changed.turns.push(changed.turns[0].clone());
        changed.created_at = Timestamp(7);
        changed.updated_at = Timestamp(3);
        store.save_chat(&changed).await.unwrap();
        assert_eq!(store.chat(&changed.id).await.unwrap(), Some(changed));
        assert_eq!(store.chats().await.unwrap().len(), 3);
    });
}

#[test]
fn chat_list_is_ordered_like_the_mock() {
    // G17
    let scratch = Scratch::new();
    let chat = |id: &str, created: u64, updated: u64| Chat {
        id: ChatId::new(id),
        title: format!("Chat {id}"),
        created_at: Timestamp(created),
        updated_at: Timestamp(updated),
        turns: Vec::new(),
    };
    let chats = [
        chat("a", 100, 900),
        chat("c", 300, 500),
        chat("b", 200, 500),
        chat("B", 200, 500),
        chat("ä", 1, 500),
        chat("z", 0, 0),
        chat("max", 0, u64::MAX),
        chat("big", 0, u64::MAX / 2 + 7),
        chat("10", 5, 500),
        chat("9", 5, 500),
    ];
    run(async {
        let store = open(&scratch.db()).await;
        let mock = MockStore::new();
        for c in &chats {
            store.save_chat(c).await.unwrap();
            mock.save_chat(c).await.unwrap();
        }
        let ours = store.chats().await.unwrap();
        assert_eq!(ours, mock.chats().await.unwrap());
        let order: Vec<&str> = ours.iter().map(|c| c.id.0.as_str()).collect();
        assert_eq!(
            order,
            ["max", "big", "a", "10", "9", "B", "b", "c", "ä", "z"]
        );
        assert_eq!(
            (ours[2].created_at, ours[2].updated_at),
            (Timestamp(100), Timestamp(900))
        );
    });
}

#[test]
fn renaming_changes_the_title_but_not_the_order() {
    // G18
    let scratch = Scratch::new();
    run(async {
        let store: Arc<dyn Store> = Arc::new(open(&scratch.db()).await);
        for chat in sample_chats() {
            store.save_chat(&chat).await.unwrap();
        }
        let kb = knowledge_base(Arc::clone(&store), Arc::new(MockClock::new(Timestamp(1))));
        let before = store.chats().await.unwrap();
        let last = before.last().unwrap().id.clone();
        kb.rename_chat(&last, "  Neuer Titel für die Prüfung  ")
            .await
            .unwrap();
        let after = store.chats().await.unwrap();
        let ids = |l: &[ChatSummary]| l.iter().map(|c| c.id.clone()).collect::<Vec<_>>();
        assert_eq!(ids(&after), ids(&before));
        let renamed = after.iter().find(|c| c.id == last).unwrap();
        assert_eq!(renamed.title, "Neuer Titel für die Prüfung");
        let old = before.iter().find(|c| c.id == last).unwrap();
        assert_eq!(renamed.updated_at, old.updated_at);
        let chat = store.chat(&last).await.unwrap().unwrap();
        assert_eq!(chat.title, "Neuer Titel für die Prüfung");
        assert_eq!(chat.updated_at, old.updated_at);
    });
}

#[test]
fn settings_default_and_round_trip() {
    // G23
    let scratch = Scratch::new();
    run(async {
        let store = open(&scratch.db()).await;
        assert_eq!(store.settings().await.unwrap(), Settings::default());
        let settings = sample_settings();
        store.save_settings(&settings).await.unwrap();
        assert_eq!(store.settings().await.unwrap(), settings);
        let mut reordered = settings.clone();
        reordered.watched_folders.reverse();
        reordered.excluded.reverse();
        reordered.answer_level = AnswerLevel::Balanced;
        reordered.ui_language = UiLanguage::German;
        reordered.color_scheme = ColorScheme::Light;
        store.save_settings(&reordered).await.unwrap();
        assert_eq!(store.settings().await.unwrap(), reordered);
    });
}

#[test]
fn update_settings_saves_and_returns_or_keeps_everything_on_error() {
    // G24
    let scratch = Scratch::new();
    run(async {
        let store = open(&scratch.db()).await;
        let saved = store
            .update_settings(Box::new(|s| {
                s.color_scheme = ColorScheme::Dark;
                s.watched_folders.push(PathBuf::from(r"C:\Ablage"));
                Ok(())
            }))
            .await
            .unwrap();
        assert_eq!(saved.color_scheme, ColorScheme::Dark);
        assert_eq!(store.settings().await.unwrap(), saved);

        for error in [
            Error::InvalidInput("rejected by the update".into()),
            Error::NotFound("folder X".into()),
            Error::Store("inner".into()),
        ] {
            let (kind, text) = (error.kind(), error.to_string());
            let mut slot = Some(error);
            let returned = store
                .update_settings(Box::new(move |s| {
                    s.color_scheme = ColorScheme::Light;
                    s.watched_folders.clear();
                    Err(slot.take().unwrap())
                }))
                .await
                .unwrap_err();
            assert_eq!((returned.kind(), returned.to_string()), (kind, text));
            assert_eq!(store.settings().await.unwrap(), saved);
        }
    });
}

#[cfg(windows)]
#[test]
fn level_and_exclusions_survive_a_restart_unchanged() {
    // G26
    let scratch = Scratch::new();
    let odd_file = PathBuf::from(r"C:\Ablage\Notizen\liste.xyz");
    run(async {
        {
            let store: Arc<dyn Store> = Arc::new(open(&scratch.db()).await);
            let kb = knowledge_base(Arc::clone(&store), Arc::new(MockClock::new(Timestamp(1))));
            kb.add_folder(Path::new(r"C:\Ablage")).await.unwrap();
            kb.exclude(&odd_file).await.unwrap();
            kb.exclude(Path::new(r"C:\Ablage\Entwürfe")).await.unwrap();
            // Thorough is stored as chosen, even where the tier would not offer it.
            store
                .update_settings(Box::new(|s| {
                    s.answer_level = AnswerLevel::Thorough;
                    Ok(())
                }))
                .await
                .unwrap();
        }
        let store: Arc<dyn Store> = Arc::new(open(&scratch.db()).await);
        let settings = store.settings().await.unwrap();
        assert_eq!(settings.answer_level, AnswerLevel::Thorough);
        assert_eq!(
            settings.excluded,
            [odd_file.clone(), PathBuf::from(r"C:\Ablage\Entwürfe")]
        );
        let kb = knowledge_base(Arc::clone(&store), Arc::new(MockClock::new(Timestamp(1))));
        let after = kb.include(&odd_file).await.unwrap();
        assert_eq!(after.excluded, [PathBuf::from(r"C:\Ablage\Entwürfe")]);
        assert_eq!(store.settings().await.unwrap(), after);
    });
}

#[test]
fn limits_survive_storing_exactly() {
    // G41
    let scratch = Scratch::new();
    let long_title: String = "Ä".repeat(10_000);
    let text = "Prüfung 👩‍🔬🇩🇪 e\u{301}\u{302} a\u{308} ﷽ 𝔘𝔫𝔦𝔠𝔬𝔡𝔢";
    let records = [
        DocumentRecord {
            size_bytes: Some(0),
            modified_at: Some(Timestamp(0)),
            title: Some(String::new()),
            ..record("min", r"C:\Ablage\min.pdf")
        },
        DocumentRecord {
            size_bytes: Some(u64::MAX),
            modified_at: Some(Timestamp(u64::MAX)),
            title: Some(long_title.clone()),
            status: DocumentStatus::Indexed { chunks: u32::MAX },
            ..record("max", r"C:\Ablage\max.pdf")
        },
        DocumentRecord {
            title: Some(text.into()),
            status: DocumentStatus::Failed {
                kind: ErrorKind::Io,
                message: text.into(),
            },
            ..record("emoji", &format!(r"C:\Ablage\{text}.pdf"))
        },
    ];
    let chats = [
        Chat {
            id: ChatId::new("min"),
            title: text.into(),
            created_at: Timestamp(0),
            updated_at: Timestamp(0),
            turns: Vec::new(),
        },
        Chat {
            id: ChatId::new(text),
            title: long_title.clone(),
            created_at: Timestamp(u64::MAX),
            updated_at: Timestamp(u64::MAX),
            turns: vec![ChatTurn {
                question: text.into(),
                answer: Answer {
                    text: String::new(),
                    citations: Vec::new(),
                    no_sources: true,
                    level: AnswerLevel::Fast,
                    outcome: AnswerOutcome::Complete,
                },
            }],
        },
    ];
    let mut empty_document = fixtures::documents().remove("notizen.md").unwrap();
    empty_document.elements.clear();
    empty_document.title = Some(long_title.clone());
    run(async {
        let store = open(&scratch.db()).await;
        for r in &records {
            store.upsert_document(r).await.unwrap();
        }
        for c in &chats {
            store.save_chat(c).await.unwrap();
        }
        store.save_content(&empty_document).await.unwrap();
        store.save_settings(&Settings::default()).await.unwrap();
        drop(store);
        let store = open(&scratch.db()).await;
        for r in &records {
            assert_eq!(store.document(&r.id).await.unwrap().as_ref(), Some(r));
        }
        for c in &chats {
            assert_eq!(store.chat(&c.id).await.unwrap().as_ref(), Some(c));
        }
        let summaries = store.chats().await.unwrap();
        assert_eq!(summaries[0].updated_at, Timestamp(u64::MAX));
        assert_eq!(summaries[1].updated_at, Timestamp(0));
        assert_eq!(
            store
                .content(&empty_document.source.document_id)
                .await
                .unwrap(),
            Some(empty_document.clone())
        );
        assert_eq!(store.settings().await.unwrap(), Settings::default());
    });
}

#[test]
fn values_that_look_like_sql_are_stored_as_values() {
    // G42
    let scratch = Scratch::new();
    let evil = "'); DROP TABLE x; --";
    run(async {
        let store = open(&scratch.db()).await;
        let chat = Chat {
            id: ChatId::new(evil),
            title: evil.into(),
            created_at: Timestamp(1),
            updated_at: Timestamp(2),
            turns: vec![ChatTurn {
                question: format!("{evil} DROP TABLE chats; --"),
                answer: Answer {
                    text: "\"; DELETE FROM documents; --".into(),
                    citations: Vec::new(),
                    no_sources: true,
                    level: AnswerLevel::Fast,
                    outcome: AnswerOutcome::Complete,
                },
            }],
        };
        store.save_chat(&chat).await.unwrap();
        assert_eq!(store.chat(&chat.id).await.unwrap(), Some(chat.clone()));
        let record = record(
            "x'); DROP TABLE documents; --",
            r"C:\Ablage\'); DROP TABLE documents; --.pdf",
        );
        store.upsert_document(&record).await.unwrap();
        assert_eq!(
            store.document_by_path(&record.path).await.unwrap().as_ref(),
            Some(&record)
        );
        store
            .delete_chat(&ChatId::new("' OR '1'='1"))
            .await
            .unwrap();
        store
            .delete_document(&DocumentId::new("' OR '1'='1"))
            .await
            .unwrap();
        assert_eq!(store.chats().await.unwrap().len(), 1);
        assert_eq!(store.documents().await.unwrap(), [record]);
    });
}
