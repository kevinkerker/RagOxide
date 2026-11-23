//! A large content (G38) and the measurements for the report (G39).

mod common;

use std::time::Instant;

use common::*;
use kb_core::*;

#[test]
fn a_large_content_is_stored_and_read() {
    // G38: 100,000 elements and more than 20 MB of text.
    let scratch = Scratch::new();
    let big = big_document("doc-big", 100_000, 210);
    let text: usize = big.elements.iter().map(|e| e.text.len()).sum();
    assert!(text >= 20_000_000, "{text} bytes of text");
    run(async {
        let store = open(&scratch.db()).await;
        store.save_content(&big).await.unwrap();
        assert_eq!(
            store
                .content(&big.source.document_id)
                .await
                .unwrap()
                .as_ref(),
            Some(&big)
        );
    });
    run(async {
        let store = open(&scratch.db()).await;
        assert_eq!(
            store.content(&big.source.document_id).await.unwrap(),
            Some(big)
        );
    });
}

/// G39: prints the measurements of the report. Run with
/// `cargo test -p kb-store --test large -- --ignored --nocapture`.
#[test]
#[ignore = "measurement for the report, not a check"]
fn measure() {
    let scratch = Scratch::new();
    let db = scratch.db();
    let big = big_document("doc-big", 100_000, 210);
    let text: usize = big.elements.iter().map(|e| e.text.len()).sum();
    run(async {
        let store = open(&db).await;
        let start = Instant::now();
        store.save_content(&big).await.unwrap();
        let save = start.elapsed();
        let start = Instant::now();
        let back = store.content(&big.source.document_id).await.unwrap();
        let read = start.elapsed();
        assert!(back.is_some());
        drop(store);
        let size = std::fs::metadata(&db).unwrap().len();
        println!(
            "large content: {} elements, {text} bytes of text; save {save:?}, read {read:?}, file {size} bytes",
            big.elements.len()
        );

        let store = open(&db).await;
        let start = Instant::now();
        for i in 0..10_000 {
            let record = DocumentRecord {
                title: Some(format!("Dokument {i}")),
                content_hash: Some("ab".repeat(32)),
                size_bytes: Some(123_456),
                modified_at: Some(Timestamp(1_794_700_000_000)),
                status: DocumentStatus::Indexed { chunks: 12 },
                ..record(
                    &format!("doc-{i:05}"),
                    &format!(r"C:\Ablage\Ordner{}\Datei {i}.pdf", i % 50),
                )
            };
            store.upsert_document(&record).await.unwrap();
        }
        let upserts = start.elapsed();
        let mut times = Vec::new();
        for _ in 0..5 {
            let start = Instant::now();
            let all = store.documents().await.unwrap();
            times.push(start.elapsed());
            assert_eq!(all.len(), 10_000);
        }
        drop(store);
        let size = std::fs::metadata(&db).unwrap().len();
        println!(
            "10,000 records: upserts {upserts:?} ({:?} each); documents() {times:?}; file {size} bytes",
            upserts / 10_000
        );
    });
}
