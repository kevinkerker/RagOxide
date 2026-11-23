//! Adding, replacing and deleting (E18 to E25).

mod common;

use common::*;
use kb_core::{
    Anchor, BoundingBox, DocumentId, Error, Index, IndexEntry, SearchMode, SearchRequest,
};

const DIM: u32 = 64;
const MODES: [SearchMode; 3] = [SearchMode::Hybrid, SearchMode::Vector, SearchMode::FullText];

#[tokio::test]
async fn added_entries_are_found_at_once_in_all_modes() {
    // E18
    let (index, dir) = open("e18", DIM).await;
    index
        .add(&[
            entry(
                chunk("doc-a", 0, "Ein Text über Quarzuhren.", Some("de")),
                DIM,
                1,
            ),
            entry(
                chunk("doc-b", 0, "Ein anderer Text über Kaffee.", Some("de")),
                DIM,
                2,
            ),
        ])
        .await
        .unwrap();
    let q = seeded(DIM, 1);
    for mode in MODES {
        let hits = index
            .search(&request("Quarzuhren", 10, mode), &q)
            .await
            .unwrap();
        assert!(
            ids(&hits).contains(&"doc-a#0".to_string()),
            "{mode:?}: {:?}",
            ids(&hits)
        );
    }
    let hits = index
        .search(&request("Quarzuhren", 10, SearchMode::FullText), &[])
        .await
        .unwrap();
    assert_eq!(ids(&hits), vec!["doc-a#0"]);
    drop(index);
    remove(dir);
}

#[tokio::test]
async fn an_existing_chunk_id_is_replaced() {
    // E19
    let (index, dir) = open("e19", DIM).await;
    index
        .add(&[entry(
            chunk("doc-a", 0, "Alter Wortlaut Zitronenfalter", Some("de")),
            DIM,
            1,
        )])
        .await
        .unwrap();
    let mut new = entry(
        chunk("doc-a", 0, "Neuer Wortlaut Schwalbenschwanz", Some("de")),
        DIM,
        7,
    );
    new.chunk.provenance.heading_path = vec!["Neu".into()];
    index.add(std::slice::from_ref(&new)).await.unwrap();
    let old = index
        .search(&request("Zitronenfalter", 10, SearchMode::FullText), &[])
        .await
        .unwrap();
    assert!(old.is_empty(), "{:?}", ids(&old));
    let hits = index
        .search(&request("Schwalbenschwanz", 10, SearchMode::FullText), &[])
        .await
        .unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].chunk, new.chunk);
    let all = index
        .search(
            &request("Wortlaut", u32::MAX, SearchMode::Vector),
            &seeded(DIM, 7),
        )
        .await
        .unwrap();
    assert_eq!(ids(&all), vec!["doc-a#0"], "the chunk exists exactly once");
    assert!(
        (all[0].vector_score.unwrap() - 1.0).abs() < 1e-5,
        "new embedding stored"
    );
    drop(index);
    remove(dir);
}

#[tokio::test]
async fn duplicate_ids_in_one_call_keep_the_last_entry() {
    // E19, README "Doppelte IDs"
    let (index, dir) = open("e19-dup", DIM).await;
    index
        .add(&[
            entry(chunk("doc-a", 0, "erste Fassung Igel", Some("de")), DIM, 1),
            entry(
                chunk("doc-a", 0, "zweite Fassung Dachs", Some("de")),
                DIM,
                2,
            ),
        ])
        .await
        .unwrap();
    let all = index
        .search(&request("Fassung", 10, SearchMode::FullText), &[])
        .await
        .unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].chunk.text, "zweite Fassung Dachs");
    drop(index);
    remove(dir);
}

#[tokio::test]
async fn a_wrong_embedding_length_adds_nothing() {
    // E20
    let (index, dir) = open("e20", DIM).await;
    let bad = IndexEntry {
        chunk: chunk("doc-b", 0, "falsch Murmeltier", Some("de")),
        embedding: vec![0.1; DIM as usize - 1],
    };
    let result = index
        .add(&[
            entry(
                chunk("doc-a", 0, "gültig davor Murmeltier", Some("de")),
                DIM,
                1,
            ),
            bad,
            entry(
                chunk("doc-c", 0, "gültig danach Murmeltier", Some("de")),
                DIM,
                3,
            ),
        ])
        .await;
    assert!(
        matches!(result, Err(Error::EmbeddingMismatch(_))),
        "{result:?}"
    );
    for mode in MODES {
        let hits = index
            .search(&request("Murmeltier", 10, mode), &seeded(DIM, 1))
            .await
            .unwrap();
        assert!(hits.is_empty(), "{mode:?}");
    }
    drop(index);
    remove(dir);
}

#[tokio::test]
async fn adding_nothing_succeeds_and_changes_nothing() {
    // E21
    let (index, dir) = open("e21", DIM).await;
    index.add(&[]).await.unwrap();
    index
        .add(&[entry(chunk("doc-a", 0, "Text", Some("de")), DIM, 1)])
        .await
        .unwrap();
    let versions = index.version_count().await.unwrap();
    index.add(&[]).await.unwrap();
    assert_eq!(index.version_count().await.unwrap(), versions);
    let hits = index
        .search(&request("Text", 10, SearchMode::Vector), &seeded(DIM, 1))
        .await
        .unwrap();
    assert_eq!(ids(&hits), vec!["doc-a#0"]);
    drop(index);
    remove(dir);
}

#[tokio::test]
async fn deleting_a_document_removes_all_its_chunks_and_nothing_else() {
    // E22
    let (index, dir) = open("e22", DIM).await;
    let mut entries = Vec::new();
    for n in 0..3 {
        entries.push(entry(
            chunk("doc-1", n, "Fuchs im Wald", Some("de")),
            DIM,
            u64::from(n),
        ));
        entries.push(entry(
            chunk("doc-10", n, "Fuchs im Feld", Some("de")),
            DIM,
            10 + u64::from(n),
        ));
    }
    index.add(&entries).await.unwrap();
    index
        .delete_document(&DocumentId("doc-1".into()))
        .await
        .unwrap();
    index
        .delete_document(&DocumentId("doc-unknown".into()))
        .await
        .unwrap();
    for mode in MODES {
        let all = index
            .search(&request("Fuchs", 100, mode), &seeded(DIM, 1))
            .await
            .unwrap();
        assert_eq!(ids(&all), {
            let mut v: Vec<String> = all.iter().map(|h| h.chunk.id.0.clone()).collect();
            v.retain(|i| i.starts_with("doc-10#"));
            v
        });
        assert_eq!(all.len(), 3, "{mode:?}");
        let filtered = index
            .search(
                &SearchRequest {
                    documents: Some(vec![DocumentId("doc-1".into())]),
                    ..request("Fuchs", 100, mode)
                },
                &seeded(DIM, 1),
            )
            .await
            .unwrap();
        assert!(filtered.is_empty(), "{mode:?}");
    }
    drop(index);
    remove(dir);
}

#[tokio::test]
async fn a_shorter_new_version_leaves_no_old_chunk() {
    // E23
    let (index, dir) = open("e23", DIM).await;
    let doc = DocumentId("doc-a".into());
    let old: Vec<IndexEntry> = (0..5)
        .map(|n| {
            entry(
                chunk(
                    "doc-a",
                    n,
                    &format!("alte Fassung Teil{n} Elster"),
                    Some("de"),
                ),
                DIM,
                u64::from(n),
            )
        })
        .collect();
    index.add(&old).await.unwrap();
    index.delete_document(&doc).await.unwrap();
    let new: Vec<IndexEntry> = (0..2)
        .map(|n| {
            entry(
                chunk(
                    "doc-a",
                    n,
                    &format!("neue Fassung Teil{n} Amsel"),
                    Some("de"),
                ),
                DIM,
                50 + u64::from(n),
            )
        })
        .collect();
    index.add(&new).await.unwrap();
    for mode in MODES {
        let all = index
            .search(&request("Fassung Elster Amsel", 100, mode), &seeded(DIM, 3))
            .await
            .unwrap();
        let mut got = ids(&all);
        got.sort();
        assert_eq!(got, vec!["doc-a#0", "doc-a#1"], "{mode:?}");
        assert!(
            all.iter().all(|h| h.chunk.text.contains("neue")),
            "{mode:?}"
        );
    }
    drop(index);
    remove(dir);
}

#[tokio::test]
async fn ids_with_special_characters_work_everywhere() {
    // E24
    let names = [
        "a'b",
        "a\"b",
        "a#b",
        "a%b",
        "a\\b",
        "a;b",
        "a--b",
        "a b",
        "Ärger-Öl",
        "x' OR '1'='1",
    ];
    let (index, dir) = open("e24", DIM).await;
    let mut entries = vec![entry(chunk("plain", 0, "Kranich", Some("de")), DIM, 999)];
    for (i, n) in names.iter().enumerate() {
        entries.push(entry(chunk(n, 0, "Kranich", Some("de")), DIM, i as u64));
    }
    index.add(&entries).await.unwrap();
    for (i, n) in names.iter().enumerate() {
        let filtered = index
            .search(
                &SearchRequest {
                    documents: Some(vec![DocumentId(n.to_string())]),
                    ..request("Kranich", 100, SearchMode::Hybrid)
                },
                &seeded(DIM, i as u64),
            )
            .await
            .unwrap();
        assert_eq!(filtered.len(), 1, "{n}");
        assert_eq!(filtered[0].chunk.document_id().0, *n);
    }
    for n in names {
        index
            .delete_document(&DocumentId(n.to_string()))
            .await
            .unwrap();
        let left = index
            .search(&request("Kranich", 100, SearchMode::FullText), &[])
            .await
            .unwrap();
        assert!(left.iter().all(|h| h.chunk.document_id().0 != n), "{n}");
        assert!(
            ids(&left).contains(&"plain#0".to_string()),
            "{n} touched another document"
        );
    }
    let left = index
        .search(&request("Kranich", 100, SearchMode::FullText), &[])
        .await
        .unwrap();
    assert_eq!(ids(&left), vec!["plain#0"]);
    drop(index);
    remove(dir);
}

#[tokio::test]
async fn chunks_come_back_unchanged() {
    // E25
    let mut chunks = kb_core::mock::fixtures::chunks();
    assert!(chunks.iter().any(|c| {
        c.provenance
            .anchors
            .iter()
            .any(|a| matches!(a, kb_core::Anchor::Slide { in_notes: true, .. }))
    }));
    // The fixtures have no chunk without language; add one.
    let mut unknown = chunk("doc-unknown-language", 0, "Text ohne Sprache", None);
    unknown.provenance.heading_path = vec!["Kapitel".into(), "Abschnitt".into()];
    chunks.push(unknown);
    let (index, dir) = open("e25", DIM).await;
    let entries: Vec<IndexEntry> = chunks
        .iter()
        .enumerate()
        .map(|(i, c)| entry(c.clone(), DIM, i as u64))
        .collect();
    index.add(&entries).await.unwrap();
    let all = index
        .search(&request("x", u32::MAX, SearchMode::Vector), &seeded(DIM, 0))
        .await
        .unwrap();
    assert_eq!(all.len(), chunks.len());
    for c in &chunks {
        let hit = all
            .iter()
            .find(|h| h.chunk.id == c.id)
            .expect("chunk returned");
        assert_eq!(&hit.chunk, c);
    }
    drop(index);
    remove(dir);
}

#[tokio::test]
async fn unusable_embeddings_are_rejected_and_add_nothing() {
    // V7
    let (index, dir) = open("v7-add", DIM).await;
    let mut nan = seeded(DIM, 1);
    nan[5] = f32::NAN;
    let mut inf = seeded(DIM, 1);
    inf[1] = f32::NEG_INFINITY;
    for bad in [nan, inf, vec![0.0; DIM as usize]] {
        let r = index
            .add(&[
                entry(chunk("ok", 0, "gültig Biber", Some("de")), DIM, 1),
                IndexEntry {
                    chunk: chunk("bad", 0, "ungültig Biber", Some("de")),
                    embedding: bad,
                },
            ])
            .await;
        assert!(matches!(r, Err(Error::InvalidInput(_))), "{r:?}");
    }
    let hits = index
        .search(&request("Biber", 10, SearchMode::FullText), &[])
        .await
        .unwrap();
    assert!(hits.is_empty());
    drop(index);
    remove(dir);
}

#[tokio::test]
async fn a_box_that_is_not_finite_is_rejected_and_adds_nothing() {
    // E64
    let (index, dir) = open("e64", DIM).await;
    for (n, value) in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY]
        .into_iter()
        .enumerate()
    {
        let mut bad = chunk("bad", n as u32, "ungültig Fischotter", Some("de"));
        bad.provenance.anchors = vec![Anchor::Page {
            page: 1,
            bbox: BoundingBox {
                left: 10.0,
                top: value,
                right: 20.0,
                bottom: 30.0,
            },
        }];
        let r = index
            .add(&[
                entry(
                    chunk("ok", n as u32, "gültig Fischotter", Some("de")),
                    DIM,
                    1,
                ),
                entry(bad, DIM, 2),
                entry(
                    chunk("ok", 10 + n as u32, "gültig Fischotter", Some("de")),
                    DIM,
                    3,
                ),
            ])
            .await;
        assert!(matches!(r, Err(Error::InvalidInput(_))), "{value}: {r:?}");
    }
    // Neither the bad entry nor the valid ones around it are there, in any mode.
    let q = seeded(DIM, 2);
    for mode in MODES {
        let hits = index
            .search(&request("Fischotter", 10, mode), &q)
            .await
            .unwrap();
        assert!(hits.is_empty(), "{mode:?}: {:?}", ids(&hits));
    }
    drop(index);
    remove(dir);
}
