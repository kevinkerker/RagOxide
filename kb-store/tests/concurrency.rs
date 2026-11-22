//! Concurrent calls, dropped futures and the async executor (G25, G35, G36, G37).

mod common;

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;

use common::*;
use futures::FutureExt;
use kb_core::*;
use kb_store::SqliteStore;

fn folder(n: usize) -> PathBuf {
    PathBuf::from(format!(r"C:\Ablage\Ordner{n:03}"))
}

#[test]
fn fifty_concurrent_updates_give_fifty_folders() {
    // G25, first part
    let scratch = Scratch::new();
    run(async {
        let store = Arc::new(open(&scratch.db()).await);
        let tasks: Vec<_> = (0..50)
            .map(|n| {
                let store = Arc::clone(&store);
                tokio::spawn(async move {
                    store
                        .update_settings(Box::new(move |s| {
                            s.watched_folders.push(folder(n));
                            Ok(())
                        }))
                        .await
                })
            })
            .collect();
        for task in tasks {
            task.await.unwrap().unwrap();
        }
        let mut folders = store.settings().await.unwrap().watched_folders;
        folders.sort();
        assert_eq!(folders, (0..50).map(folder).collect::<Vec<_>>());
    });
}

/// Marker folder of the `save_settings` call `n`.
fn marker(n: usize) -> PathBuf {
    PathBuf::from(format!(r"C:\Gespeichert{n:02}"))
}

#[test]
fn updates_and_saves_are_serialisable() {
    // G25, second part: the outcome equals some order in which the calls ran one by one.
    let scratch = Scratch::new();
    run(async {
        let store = Arc::new(open(&scratch.db()).await);
        let mut tasks = Vec::new();
        for n in 0..60 {
            let store = Arc::clone(&store);
            tasks.push(tokio::spawn(async move {
                if n % 10 == 5 {
                    let saved = Settings {
                        watched_folders: vec![marker(n)],
                        ..Settings::default()
                    };
                    store.save_settings(&saved).await.unwrap();
                    None
                } else {
                    let returned = store
                        .update_settings(Box::new(move |s| {
                            s.watched_folders.push(folder(n));
                            Ok(())
                        }))
                        .await
                        .unwrap();
                    Some((n, returned.watched_folders))
                }
            }));
        }
        let mut updates = Vec::new();
        for task in tasks {
            if let Some(update) = task.await.unwrap() {
                updates.push(update);
            }
        }
        let final_folders = store.settings().await.unwrap().watched_folders;
        // Every update saw the state of one serial order: its own folder last, after
        // the folders of earlier updates since the save it followed (or the start).
        for (n, seen) in &updates {
            assert_eq!(seen.last(), Some(&folder(*n)), "update {n} saw {seen:?}");
        }
        // The updates that followed the same save form one chain of prefixes.
        let first = |l: &[PathBuf]| l.first().cloned();
        for (a, seen_a) in &updates {
            for (b, seen_b) in &updates {
                if first(seen_a) == first(seen_b) {
                    let (short, long) = if seen_a.len() <= seen_b.len() {
                        (seen_a, seen_b)
                    } else {
                        (seen_b, seen_a)
                    };
                    assert_eq!(&long[..short.len()], &short[..], "updates {a} and {b}");
                }
            }
        }
        // The final state is the last save plus exactly the updates that came after it.
        let last_save = first(&final_folders).expect("six saves ran");
        assert!((0..60).any(|n| marker(n) == last_save), "{final_folders:?}");
        let mut after: Vec<PathBuf> = updates
            .iter()
            .filter(|(_, seen)| first(seen).as_ref() == Some(&last_save))
            .map(|(n, _)| folder(*n))
            .collect();
        let mut kept = final_folders[1..].to_vec();
        after.sort();
        kept.sort();
        assert_eq!(kept, after, "no update after the last save is lost");

        // No update that ends after the last save is lost.
        for n in 100..105 {
            store
                .update_settings(Box::new(move |s| {
                    s.watched_folders.push(folder(n));
                    Ok(())
                }))
                .await
                .unwrap();
        }
        let later = store.settings().await.unwrap().watched_folders;
        assert_eq!(&later[..final_folders.len()], &final_folders[..]);
        assert_eq!(
            &later[final_folders.len()..],
            &(100..105).map(folder).collect::<Vec<_>>()[..]
        );
    });
}

#[test]
fn a_dropped_write_is_whole_or_absent_and_leaves_no_lock() {
    // G35
    let scratch = Scratch::new();
    run(async {
        let store = open(&scratch.db()).await;
        let big = big_document("doc-big", 20_000, 200);
        let chat = sample_chats().remove(0);
        for round in 0..5 {
            // Polled once (the work is handed over), then dropped.
            assert!(
                store.save_content(&big).now_or_never().is_none(),
                "round {round}"
            );
            let pending = store.save_chat(&chat);
            drop(pending.now_or_never());
            let pending = store.update_settings(Box::new(move |s| {
                s.watched_folders.push(folder(round));
                Ok(())
            }));
            drop(pending.now_or_never());
            let pending = store.delete_document(&big.source.document_id);
            drop(pending.now_or_never());
            // The next call of every kind succeeds, and the state is whole.
            let content = store.content(&big.source.document_id).await.unwrap();
            assert!(content.is_none() || content.as_ref() == Some(&big));
            let saved = store.chat(&chat.id).await.unwrap();
            assert!(saved.is_none() || saved.as_ref() == Some(&chat));
            let folders = store.settings().await.unwrap().watched_folders;
            assert_eq!(folders, (0..=round).map(folder).collect::<Vec<_>>());
        }
        // Dropped before the first poll: nothing happens.
        let unsaved = Chat {
            id: ChatId::new("never"),
            ..sample_chats().remove(1)
        };
        let never = store.save_chat(&unsaved);
        drop(never);
        store.save_content(&big).await.unwrap();
        assert_eq!(
            store.content(&big.source.document_id).await.unwrap(),
            Some(big.clone())
        );
        assert!(store.chat(&ChatId::new("never")).await.unwrap().is_none());
        store
            .delete_document(&big.source.document_id)
            .await
            .unwrap();
        assert!(
            store
                .content(&big.source.document_id)
                .await
                .unwrap()
                .is_none()
        );
    });
}

#[test]
fn many_tasks_call_every_method_concurrently_without_errors() {
    // G36: 8 tasks on a runtime with several threads, 1,000 calls over all methods.
    let scratch = Scratch::new();
    const TASKS: usize = 8;
    const ROUNDS: usize = 10;
    run(async {
        let store = Arc::new(open(&scratch.db()).await);
        let barrier = Arc::new(tokio::sync::Barrier::new(TASKS));
        let calls = Arc::new(AtomicU64::new(0));
        let library = fixture_library();
        let mut tasks = Vec::new();
        for t in 0..TASKS {
            let store = Arc::clone(&store);
            let barrier = Arc::clone(&barrier);
            let calls = Arc::clone(&calls);
            let library = library.clone();
            tasks.push(tokio::spawn(async move {
                let count = || calls.fetch_add(1, Ordering::Relaxed);
                // Phase 1: concurrent replacements of the settings.
                store
                    .save_settings(&Settings {
                        color_scheme: ColorScheme::Dark,
                        ..Settings::default()
                    })
                    .await
                    .unwrap();
                count();
                barrier.wait().await;
                // Phase 2: every other method, mixed.
                for round in 0..ROUNDS {
                    let id = format!("t{t}-r{round}");
                    let (fixture_record, fixture_document) = &library[(t + round) % library.len()];
                    let mut document = fixture_document.clone();
                    document.source.document_id = DocumentId::new(&id);
                    let record = DocumentRecord {
                        id: DocumentId::new(&id),
                        path: PathBuf::from(format!(r"C:\Ablage\{id}.pdf")),
                        ..fixture_record.clone()
                    };
                    store.upsert_document(&record).await.unwrap();
                    store.save_content(&document).await.unwrap();
                    assert_eq!(
                        store.document(&record.id).await.unwrap().as_ref(),
                        Some(&record)
                    );
                    assert_eq!(
                        store.document_by_path(&record.path).await.unwrap().as_ref(),
                        Some(&record)
                    );
                    assert_eq!(
                        store.content(&record.id).await.unwrap().as_ref(),
                        Some(&document)
                    );
                    assert!(!store.documents().await.unwrap().is_empty());
                    let mut chat = sample_chats().remove(round % 3);
                    chat.id = ChatId::new(&id);
                    store.save_chat(&chat).await.unwrap();
                    assert_eq!(store.chat(&chat.id).await.unwrap().as_ref(), Some(&chat));
                    assert!(!store.chats().await.unwrap().is_empty());
                    let n = t * 100 + round;
                    store
                        .update_settings(Box::new(move |s| {
                            s.watched_folders.push(folder(n));
                            Ok(())
                        }))
                        .await
                        .unwrap();
                    assert!(
                        store
                            .settings()
                            .await
                            .unwrap()
                            .watched_folders
                            .contains(&folder(n))
                    );
                    // Every second round removes what it wrote, the others keep it.
                    if round % 2 == 1 {
                        store.delete_document(&record.id).await.unwrap();
                        store.delete_chat(&chat.id).await.unwrap();
                    } else {
                        store
                            .delete_document(&DocumentId::new("unknown"))
                            .await
                            .unwrap();
                        store.delete_chat(&ChatId::new("unknown")).await.unwrap();
                    }
                    for _ in 0..14 {
                        count();
                    }
                }
            }));
        }
        for task in tasks {
            task.await.unwrap();
        }
        let calls = calls.load(Ordering::Relaxed);
        assert!(calls >= 1_000, "{calls} calls");

        // No write is lost.
        let kept: Vec<String> = (0..TASKS)
            .flat_map(|t| {
                (0..ROUNDS)
                    .filter(|r| r % 2 == 0)
                    .map(move |r| format!("t{t}-r{r}"))
            })
            .collect();
        let mut ids: Vec<String> = store
            .documents()
            .await
            .unwrap()
            .into_iter()
            .map(|r| r.id.0)
            .collect();
        ids.sort();
        let mut expected = kept.clone();
        expected.sort();
        assert_eq!(ids, expected);
        for id in &kept {
            assert!(store.content(&DocumentId::new(id)).await.unwrap().is_some());
            assert!(store.chat(&ChatId::new(id)).await.unwrap().is_some());
        }
        assert_eq!(store.chats().await.unwrap().len(), kept.len());
        let settings = store.settings().await.unwrap();
        assert_eq!(settings.color_scheme, ColorScheme::Dark);
        let mut folders = settings.watched_folders;
        folders.sort();
        let mut expected: Vec<PathBuf> = (0..TASKS)
            .flat_map(|t| (0..ROUNDS).map(move |r| folder(t * 100 + r)))
            .collect();
        expected.sort();
        assert_eq!(folders, expected);
    });
}

#[test]
fn a_long_call_does_not_block_a_single_threaded_executor() {
    // G37
    let scratch = Scratch::new();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(async {
        let store = SqliteStore::open(scratch.db()).await.unwrap();
        let big = big_document("doc-big", 100_000, 210);
        let done = Arc::new(AtomicBool::new(false));
        let ticks = Arc::new(AtomicU64::new(0));
        let ticker = {
            let done = Arc::clone(&done);
            let ticks = Arc::clone(&ticks);
            tokio::spawn(async move {
                while !done.load(Ordering::SeqCst) {
                    tokio::time::sleep(Duration::from_millis(1)).await;
                    ticks.fetch_add(1, Ordering::SeqCst);
                }
            })
        };
        // Let the ticker start.
        tokio::time::sleep(Duration::from_millis(5)).await;
        let start = ticks.load(Ordering::SeqCst);
        let begun = std::time::Instant::now();
        store.save_content(&big).await.unwrap();
        let back = store.content(&big.source.document_id).await.unwrap();
        let elapsed = begun.elapsed();
        let during = ticks.load(Ordering::SeqCst) - start;
        done.store(true, Ordering::SeqCst);
        ticker.await.unwrap();
        assert_eq!(back.as_ref(), Some(&big));
        // The ticker ran while the calls lasted. A blocked executor thread gives 0 ticks;
        // one tick per 100 ms (at least 5) leaves room for a coarse timer (about 15.6 ms
        // per sleep under Windows) and a busy machine.
        eprintln!("{during} ticks in {elapsed:?}");
        let minimum = u64::try_from(elapsed.as_millis() / 100).unwrap();
        assert!(
            during >= minimum.max(5),
            "{during} ticks in {elapsed:?} on one executor thread"
        );
    });
}
