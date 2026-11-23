//! The executor is not blocked (E05) and concurrent calls are safe (E49).

mod common;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, Instant};

use common::*;
use kb_core::{DocumentId, Index, IndexEntry, SearchMode};
use kb_index::LanceIndex;

const DIM: u32 = 64;

/// Runs `work` on a single-threaded runtime next to a task that ticks every
/// millisecond; returns the number of ticks and the longest gap between two ticks.
fn ticks_during<F>(
    work: impl FnOnce(Arc<LanceIndex>) -> F,
    index: Arc<LanceIndex>,
) -> (u64, Duration)
where
    F: std::future::Future<Output = ()>,
{
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async {
        let ticks = Arc::new(AtomicU64::new(0));
        let done = Arc::new(AtomicBool::new(false));
        let (t, d) = (ticks.clone(), done.clone());
        let ticker = tokio::spawn(async move {
            let mut last = Instant::now();
            let mut longest = Duration::ZERO;
            while !d.load(Ordering::SeqCst) {
                tokio::time::sleep(Duration::from_millis(1)).await;
                let now = Instant::now();
                longest = longest.max(now - last);
                last = now;
                t.fetch_add(1, Ordering::SeqCst);
            }
            longest
        });
        tokio::task::yield_now().await;
        work(index).await;
        done.store(true, Ordering::SeqCst);
        let longest = ticker.await.unwrap();
        (ticks.load(Ordering::SeqCst), longest)
    })
}

#[test]
fn add_and_search_do_not_block_a_single_threaded_executor() {
    // E05
    let dir = test_dir("e05");
    let setup = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let index = Arc::new(
        setup
            .block_on(LanceIndex::open(&dir, space(DIM), 0.5))
            .unwrap(),
    );
    let entries: Vec<IndexEntry> = (0..5000)
        .map(|n| {
            entry(
                chunk(
                    &format!("doc-{}", n % 50),
                    n,
                    "Die Abgeordneten des Bundestages beraten über Krankenhausbetten und \
                     Mehrwertsteuer, der Bundeskanzler antwortet.",
                    Some("de"),
                ),
                DIM,
                u64::from(n),
            )
        })
        .collect();
    let (ticks, longest) = ticks_during(
        |index| async move {
            let start = Instant::now();
            index.add(&entries).await.unwrap();
            println!("add of 5000 entries: {:?}", start.elapsed());
        },
        index.clone(),
    );
    println!("during add: {ticks} ticks, longest gap {longest:?}");
    assert!(ticks >= 10, "the second task ran {ticks} times");
    assert!(
        longest < Duration::from_millis(500),
        "longest gap {longest:?}"
    );

    let (ticks, longest) = ticks_during(
        |index| async move {
            for mode in [SearchMode::Hybrid, SearchMode::Vector, SearchMode::FullText] {
                for _ in 0..5 {
                    index
                        .search(
                            &request("Bundestagsabgeordnete Kanzler", 10, mode),
                            &seeded(DIM, 1),
                        )
                        .await
                        .unwrap();
                }
            }
            index
                .delete_document(&DocumentId("doc-1".into()))
                .await
                .unwrap();
        },
        index.clone(),
    );
    println!("during search and delete: {ticks} ticks, longest gap {longest:?}");
    assert!(ticks >= 3, "the second task ran {ticks} times");
    assert!(
        longest < Duration::from_millis(500),
        "longest gap {longest:?}"
    );
    drop(index);
    drop(setup);
    remove(dir);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_calls_on_one_index_are_safe() {
    // E49
    let dir = test_dir("e49");
    let index: Arc<dyn Index> = Arc::new(LanceIndex::open(&dir, space(DIM), 0.5).await.unwrap());
    let mut tasks = Vec::new();
    for w in 0..4u32 {
        let index = index.clone();
        tasks.push(tokio::spawn(async move {
            for b in 0..10u32 {
                let doc = format!("doc-{w}-{b}");
                let entries: Vec<IndexEntry> = (0..20)
                    .map(|n| {
                        entry(
                            chunk(
                                &doc,
                                n,
                                &format!("Schreiber {w} Stapel {b} Eintrag {n} Fischotter"),
                                Some("de"),
                            ),
                            DIM,
                            u64::from(w * 1000 + b * 20 + n),
                        )
                    })
                    .collect();
                index.add(&entries).await.unwrap();
                if b % 3 == 0 {
                    index.delete_document(&DocumentId(doc)).await.unwrap();
                }
            }
        }));
    }
    for s in 0..4u64 {
        let index = index.clone();
        tasks.push(tokio::spawn(async move {
            for i in 0..30u64 {
                for mode in [SearchMode::Hybrid, SearchMode::Vector, SearchMode::FullText] {
                    index
                        .search(
                            &request("Fischotter Stapel", 20, mode),
                            &seeded(DIM, s * 100 + i),
                        )
                        .await
                        .unwrap();
                }
            }
        }));
    }
    for t in tasks {
        t.await.unwrap();
    }
    let all = index
        .search(&request("Fischotter", u32::MAX, SearchMode::FullText), &[])
        .await
        .unwrap();
    // 4 writers x 10 batches x 20 entries, minus the deleted batches 0, 3, 6 and 9.
    assert_eq!(all.len(), 4 * 6 * 20);
    let vector = index
        .search(&request("x", u32::MAX, SearchMode::Vector), &seeded(DIM, 5))
        .await
        .unwrap();
    assert_eq!(vector.len(), 4 * 6 * 20);
    drop(index);
    remove(dir);
}
