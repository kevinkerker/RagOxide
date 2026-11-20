//! Search: general promises, vector, full text, fusion (E26 to E36, E39 to E41,
//! E43 to E47, E57).

mod common;

use common::*;
use kb_core::{DocumentId, Error, Index, IndexEntry, SearchMode, SearchRequest};
use kb_index::LanceIndex;
use kb_index::fusion::rrf_score;

const DIM: u32 = 64;
const MODES: [SearchMode; 3] = [SearchMode::Hybrid, SearchMode::Vector, SearchMode::FullText];

async fn sample(name: &str) -> (LanceIndex, std::path::PathBuf) {
    let (index, dir) = open(name, DIM).await;
    let texts = [
        ("doc-a", "Der Bundestag wählt den Bundeskanzler."),
        ("doc-a", "Die Abgeordneten des Bundestages stimmen ab."),
        ("doc-b", "Krankenhäuser melden ihre Betten."),
        ("doc-b", "Das Grundgesetz regelt die Wahl."),
        ("doc-c", "Kaffee und Kuchen am Nachmittag."),
    ];
    let entries: Vec<IndexEntry> = texts
        .iter()
        .enumerate()
        .map(|(i, (d, t))| entry(chunk(d, i as u32, t, Some("de")), DIM, i as u64))
        .collect();
    index.add(&entries).await.unwrap();
    (index, dir)
}

#[tokio::test]
async fn empty_text_and_limit_zero_are_invalid_in_all_modes() {
    // E26
    let (index, dir) = sample("e26").await;
    let q = seeded(DIM, 0);
    for mode in MODES {
        for text in ["", "   \t\n"] {
            let r = index.search(&request(text, 10, mode), &q).await;
            assert!(
                matches!(r, Err(Error::InvalidInput(_))),
                "{mode:?} {text:?}: {r:?}"
            );
        }
        let r = index.search(&request("Bundestag", 0, mode), &q).await;
        assert!(matches!(r, Err(Error::InvalidInput(_))), "{mode:?}: {r:?}");
    }
    drop(index);
    remove(dir);
}

#[tokio::test]
async fn wrong_vector_length_fails_except_in_full_text() {
    // E27
    let (index, dir) = sample("e27").await;
    for mode in [SearchMode::Vector, SearchMode::Hybrid] {
        for v in [vec![], vec![0.1; DIM as usize + 1]] {
            let r = index.search(&request("Bundestag", 10, mode), &v).await;
            assert!(
                matches!(r, Err(Error::EmbeddingMismatch(_))),
                "{mode:?}: {r:?}"
            );
        }
    }
    let hits = index
        .search(&request("Bundestag", 10, SearchMode::FullText), &[])
        .await
        .unwrap();
    assert!(!hits.is_empty());
    let same = index
        .search(&request("Bundestag", 10, SearchMode::FullText), &[0.3; 7])
        .await
        .unwrap();
    assert_eq!(ids(&hits), ids(&same));
    drop(index);
    remove(dir);
}

#[tokio::test]
async fn results_are_limited_sorted_unreranked_and_repeatable() {
    // E28, with equal scores: ten identical texts and identical vectors.
    let (index, dir) = open("e28", DIM).await;
    let entries: Vec<IndexEntry> = (0..10)
        .map(|n| IndexEntry {
            chunk: chunk(&format!("doc-{n}"), 0, "gleicher Text Pelikan", Some("de")),
            embedding: axis(DIM, 3),
        })
        .collect();
    index.add(&entries).await.unwrap();
    for mode in MODES {
        let first = index
            .search(&request("Pelikan", 4, mode), &axis(DIM, 3))
            .await
            .unwrap();
        assert_eq!(first.len(), 4, "{mode:?}");
        assert!(
            first
                .windows(2)
                .all(|w| w[0].retrieval_score >= w[1].retrieval_score)
        );
        assert!(first.iter().all(|h| h.rerank_score.is_none()));
        for _ in 0..3 {
            let again = index
                .search(&request("Pelikan", 4, mode), &axis(DIM, 3))
                .await
                .unwrap();
            assert_eq!(ids(&first), ids(&again), "{mode:?}");
        }
    }
    drop(index);
    remove(dir);
}

#[tokio::test]
async fn the_document_filter_restricts_all_modes() {
    // E29
    let (index, dir) = sample("e29").await;
    let q = seeded(DIM, 2);
    for mode in MODES {
        let all = index
            .search(&request("Bundestag Wahl Betten", 100, mode), &q)
            .await
            .unwrap();
        let docs: std::collections::BTreeSet<String> = all
            .iter()
            .map(|h| h.chunk.document_id().0.clone())
            .collect();
        assert!(docs.len() >= 2, "{mode:?}: {docs:?}");
        let none = index
            .search(
                &SearchRequest {
                    documents: Some(vec![]),
                    ..request("Bundestag", 100, mode)
                },
                &q,
            )
            .await
            .unwrap();
        assert!(none.is_empty(), "{mode:?}");
        let only_b = index
            .search(
                &SearchRequest {
                    documents: Some(vec![DocumentId("doc-b".into()), DocumentId("nope".into())]),
                    ..request("Bundestag Wahl Betten", 100, mode)
                },
                &q,
            )
            .await
            .unwrap();
        assert!(!only_b.is_empty(), "{mode:?}");
        assert!(
            only_b.iter().all(|h| h.chunk.document_id().0 == "doc-b"),
            "{mode:?}"
        );
    }
    drop(index);
    remove(dir);
}

#[tokio::test]
async fn an_empty_or_emptied_index_returns_nothing() {
    // E30
    let (index, dir) = open("e30", DIM).await;
    for mode in MODES {
        let hits = index
            .search(&request("Bundestag", 10, mode), &seeded(DIM, 1))
            .await
            .unwrap();
        assert!(hits.is_empty());
    }
    index
        .add(&[entry(chunk("doc-a", 0, "Bundestag", Some("de")), DIM, 1)])
        .await
        .unwrap();
    index
        .delete_document(&DocumentId("doc-a".into()))
        .await
        .unwrap();
    for mode in MODES {
        let hits = index
            .search(&request("Bundestag", 10, mode), &seeded(DIM, 1))
            .await
            .unwrap();
        assert!(hits.is_empty(), "{mode:?}");
    }
    drop(index);
    remove(dir);
}

#[tokio::test]
async fn the_largest_limit_returns_at_most_all_chunks() {
    // E31
    let (index, dir) = sample("e31").await;
    for mode in MODES {
        let hits = index
            .search(&request("Bundestag", u32::MAX, mode), &seeded(DIM, 1))
            .await
            .unwrap();
        assert!(hits.len() <= 5, "{mode:?}");
        let filtered = index
            .search(
                &SearchRequest {
                    documents: Some(vec![DocumentId("doc-c".into())]),
                    ..request("Kaffee", u32::MAX, mode)
                },
                &seeded(DIM, 1),
            )
            .await
            .unwrap();
        assert!(filtered.len() <= 1, "{mode:?}");
    }
    let vector = index
        .search(&request("x", u32::MAX, SearchMode::Vector), &seeded(DIM, 1))
        .await
        .unwrap();
    assert_eq!(vector.len(), 5);
    drop(index);
    remove(dir);
}

#[tokio::test]
async fn query_syntax_characters_are_plain_text() {
    // E32
    let (index, dir) = sample("e32").await;
    let specials = [
        "\"Bundestag\"",
        "\"Bundestag",
        "Bundes*",
        "Bundestag?",
        "titel:Bundestag",
        "(Bundestag)",
        "[Bundestag]",
        "{Bundestag}",
        "+Bundestag",
        "-Bundestag",
        "Bundestag~2",
        "Bundestag^3",
        "Bundestag\\",
        "Bundestag/Wahl",
        "Bundestag AND Wahl",
        "Bundestag OR Wahl",
        "NOT Bundestag",
        "x' OR '1'='1",
    ];
    for text in specials {
        for mode in MODES {
            let r = index
                .search(&request(text, 10, mode), &seeded(DIM, 1))
                .await;
            assert!(r.is_ok(), "{text:?} {mode:?}: {r:?}");
        }
        let hits = index
            .search(&request(text, 10, SearchMode::FullText), &[])
            .await
            .unwrap();
        if text.contains("Bundestag") {
            assert!(!hits.is_empty(), "{text:?}");
        }
    }
    let none = index
        .search(&request("?!", 10, SearchMode::FullText), &[])
        .await
        .unwrap();
    assert!(none.is_empty());
    drop(index);
    remove(dir);
}

#[tokio::test]
async fn vector_mode_returns_all_chunks_whatever_their_similarity() {
    // E33, E35
    let (index, dir) = open("e33", DIM).await;
    let q = axis(DIM, 0);
    let mut negative = vec![0.0; DIM as usize];
    negative[0] = -1.0;
    let mut low = vec![0.0; DIM as usize];
    low[0] = 0.05;
    low[1] = (1.0f32 - 0.05 * 0.05).sqrt();
    index
        .add(&[
            IndexEntry {
                chunk: chunk("doc-a", 0, "gleich", Some("de")),
                embedding: q.clone(),
            },
            IndexEntry {
                chunk: chunk("doc-a", 1, "fern", Some("de")),
                embedding: negative,
            },
            IndexEntry {
                chunk: chunk("doc-a", 2, "schwach", Some("de")),
                embedding: low,
            },
        ])
        .await
        .unwrap();
    let hits = index
        .search(&request("egal", 3, SearchMode::Vector), &q)
        .await
        .unwrap();
    assert_eq!(ids(&hits), vec!["doc-a#0", "doc-a#2", "doc-a#1"]);
    assert!((hits[1].vector_score.unwrap() - 0.05).abs() < 1e-5);
    assert!((hits[2].vector_score.unwrap() + 1.0).abs() < 1e-5);
    drop(index);
    remove(dir);
}

fn exact_cosine(a: &[f32], b: &[f32]) -> f32 {
    let dot: f64 = a
        .iter()
        .zip(b)
        .map(|(x, y)| f64::from(*x) * f64::from(*y))
        .sum();
    let na: f64 = a.iter().map(|x| f64::from(*x).powi(2)).sum::<f64>().sqrt();
    let nb: f64 = b.iter().map(|x| f64::from(*x).powi(2)).sum::<f64>().sqrt();
    (dot / (na * nb)) as f32
}

#[tokio::test]
async fn vector_search_is_exact_cosine() {
    // E34 (the index with 100,000 entries is checked by the measurement example)
    let (index, dir) = open("e34", DIM).await;
    let entries: Vec<IndexEntry> = (0..500)
        .map(|n| {
            entry(
                chunk(&format!("doc-{}", n % 7), n, "Text", Some("de")),
                DIM,
                u64::from(n) + 100,
            )
        })
        .collect();
    index.add(&entries).await.unwrap();
    for seed in 0..5 {
        let q = seeded(DIM, seed);
        let hits = index
            .search(&request("Text", 50, SearchMode::Vector), &q)
            .await
            .unwrap();
        let mut expected: Vec<(String, f32)> = entries
            .iter()
            .map(|e| (e.chunk.id.0.clone(), exact_cosine(&q, &e.embedding)))
            .collect();
        expected.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        assert_eq!(
            ids(&hits),
            expected
                .iter()
                .take(50)
                .map(|e| e.0.clone())
                .collect::<Vec<_>>()
        );
        for (h, (_, cos)) in hits.iter().zip(&expected) {
            assert_eq!(h.retrieval_score, h.vector_score.unwrap());
            assert!((h.retrieval_score - cos).abs() < 1e-6);
            assert!((-1.0..=1.0).contains(&h.retrieval_score));
        }
    }
    drop(index);
    remove(dir);
}

#[tokio::test]
async fn full_text_returns_only_matching_chunks_without_vector_score() {
    // E36
    let (index, dir) = sample("e36").await;
    let hits = index
        .search(&request("Kaffee", 10, SearchMode::FullText), &[])
        .await
        .unwrap();
    assert_eq!(ids(&hits), vec!["doc-c#4"]);
    assert!(hits[0].vector_score.is_none());
    let none = index
        .search(&request("Zeppelin", 10, SearchMode::FullText), &[])
        .await
        .unwrap();
    assert!(none.is_empty());
    let several = index
        .search(&request("Bundestag Kaffee", 10, SearchMode::FullText), &[])
        .await
        .unwrap();
    assert!(
        several
            .windows(2)
            .all(|w| w[0].retrieval_score >= w[1].retrieval_score)
    );
    drop(index);
    remove(dir);
}

#[tokio::test]
async fn english_forms_are_found_and_one_query_finds_both_languages() {
    // E39
    let (index, dir) = open("e39", DIM).await;
    index
        .add(&[
            entry(
                chunk("en", 0, "Every employee gets a laptop.", Some("en")),
                DIM,
                1,
            ),
            entry(
                chunk("en", 1, "The employees provided feedback.", Some("en")),
                DIM,
                2,
            ),
            entry(chunk("en", 2, "We provide training.", Some("en")), DIM, 3),
            entry(chunk("de", 0, "Der Bundestag tagt.", Some("de")), DIM, 4),
        ])
        .await
        .unwrap();
    let find = |t: &'static str| {
        let index = &index;
        async move {
            ids(&index
                .search(&request(t, 10, SearchMode::FullText), &[])
                .await
                .unwrap())
        }
    };
    let e = find("employees").await;
    assert!(
        e.contains(&"en#0".to_string()) && e.contains(&"en#1".to_string()),
        "{e:?}"
    );
    let e = find("employee").await;
    assert!(e.contains(&"en#1".to_string()), "{e:?}");
    let p = find("provided").await;
    assert!(p.contains(&"en#2".to_string()), "{p:?}");
    let p = find("provide").await;
    assert!(p.contains(&"en#1".to_string()), "{p:?}");
    let both = find("employees Bundestages").await;
    assert!(
        both.contains(&"en#1".to_string()) && both.contains(&"de#0".to_string()),
        "{both:?}"
    );
    // Compound splitting and German stop words only apply to German chunks.
    let kanzler = find("Kanzler").await;
    assert!(kanzler.is_empty());
    drop(index);
    remove(dir);
}

#[tokio::test]
async fn other_languages_use_the_default_analyser() {
    // E40
    let (index, dir) = open("e40", DIM).await;
    index
        .add(&[
            entry(chunk("fr", 0, "Le Parlement vote.", Some("fr")), DIM, 1),
            entry(chunk("none", 0, "Bundeskanzler ohne Sprache", None), DIM, 2),
        ])
        .await
        .unwrap();
    let hits = index
        .search(&request("parlement", 10, SearchMode::FullText), &[])
        .await
        .unwrap();
    assert_eq!(ids(&hits), vec!["fr#0"]);
    let hits = index
        .search(&request("BUNDESKANZLER", 10, SearchMode::FullText), &[])
        .await
        .unwrap();
    assert_eq!(ids(&hits), vec!["none#0"]);
    // No stemming and no compound parts for the default analyser.
    let hits = index
        .search(&request("Kanzler", 10, SearchMode::FullText), &[])
        .await
        .unwrap();
    assert!(hits.is_empty());
    drop(index);
    remove(dir);
}

#[tokio::test]
async fn full_text_ignores_case_also_for_umlauts() {
    // E41
    let (index, dir) = open("e41", DIM).await;
    index
        .add(&[
            entry(chunk("a", 0, "Ärzte und Pflege", Some("de")), DIM, 1),
            entry(chunk("a", 1, "Der Bundestag tagt", Some("de")), DIM, 2),
            entry(chunk("a", 2, "Preise für Gießereikoks", Some("de")), DIM, 3),
        ])
        .await
        .unwrap();
    for (q, want) in [
        ("ÄRZTE", "a#0"),
        ("bundestag", "a#1"),
        ("gießereikoks", "a#2"),
    ] {
        let hits = index
            .search(&request(q, 10, SearchMode::FullText), &[])
            .await
            .unwrap();
        assert_eq!(ids(&hits), vec![want], "{q}");
    }
    drop(index);
    remove(dir);
}

#[tokio::test]
async fn german_stop_words_are_ignored() {
    // E57
    let (index, dir) = sample("e57").await;
    let none = index
        .search(&request("der die und", 10, SearchMode::FullText), &[])
        .await
        .unwrap();
    assert!(none.is_empty(), "{:?}", ids(&none));
    let plain = index
        .search(&request("Bundestag Wahl", 10, SearchMode::FullText), &[])
        .await
        .unwrap();
    let with_stop = index
        .search(
            &request("der Bundestag und die Wahl", 10, SearchMode::FullText),
            &[],
        )
        .await
        .unwrap();
    assert_eq!(ids(&plain), ids(&with_stop));
    drop(index);
    remove(dir);
}

#[tokio::test]
async fn hybrid_is_the_weighted_rank_fusion_of_both_lists() {
    // E43, E44
    let (index, dir) = sample("e43").await;
    let q = seeded(DIM, 3);
    let text = "Bundestag Betten";
    let vector = index
        .search(&request(text, 100, SearchMode::Vector), &q)
        .await
        .unwrap();
    let full = index
        .search(&request(text, 100, SearchMode::FullText), &[])
        .await
        .unwrap();
    let hybrid = index
        .search(&request(text, 100, SearchMode::Hybrid), &q)
        .await
        .unwrap();
    assert!(
        full.len() < vector.len(),
        "some hits come from one list only"
    );
    assert_eq!(hybrid.len(), vector.len());
    for h in &hybrid {
        let id = &h.chunk.id;
        let rv = vector.iter().position(|x| &x.chunk.id == id).map(|p| p + 1);
        let rf = full.iter().position(|x| &x.chunk.id == id).map(|p| p + 1);
        let want = rrf_score(0.5, rv, rf) as f32;
        assert!((h.retrieval_score - want).abs() < 1e-7, "{id}");
        let cos = vector
            .iter()
            .find(|x| &x.chunk.id == id)
            .unwrap()
            .vector_score;
        assert_eq!(
            h.vector_score, cos,
            "{id}: cosine also for full-text-only hits"
        );
    }
    drop(index);
    remove(dir);
}

/// Index for fusion tests: `far` has a rare word and a far vector, `near` shares no
/// word with the query but has the query vector; 30 fillers lie between.
async fn fusion_index(name: &str, weight: f32) -> (LanceIndex, std::path::PathBuf, Vec<f32>) {
    let dir = test_dir(name);
    let index = LanceIndex::open(&dir, space(DIM), weight).await.unwrap();
    let q = axis(DIM, 0);
    let mut entries = vec![
        IndexEntry {
            chunk: chunk(
                "far",
                0,
                "Das seltene Wort Zaunkönig steht hier.",
                Some("de"),
            ),
            embedding: axis(DIM, 1),
        },
        IndexEntry {
            chunk: chunk("near", 0, "Ganz anderer Wortlaut ohne Bezug.", Some("de")),
            embedding: q.clone(),
        },
    ];
    for n in 0..30 {
        let mut v = vec![0.0; DIM as usize];
        v[0] = 0.9 - n as f32 * 0.01;
        v[2] = (1.0 - v[0] * v[0]).sqrt();
        entries.push(IndexEntry {
            chunk: chunk(&format!("fill-{n:02}"), 0, "Füllung Meise", Some("de")),
            embedding: v,
        });
    }
    index.add(&entries).await.unwrap();
    (index, dir, q)
}

#[tokio::test]
async fn fusion_reads_both_lists_deeper_than_the_limit() {
    // E45, E47
    let (index, dir, q) = fusion_index("e45", 0.5).await;
    let limit = 5;
    let vector = index
        .search(&request("Zaunkönig", 32, SearchMode::Vector), &q)
        .await
        .unwrap();
    let far_rank = ids(&vector).iter().position(|i| i == "far#0").unwrap() + 1;
    assert!(
        far_rank > limit as usize,
        "far is behind the limit in the vector list"
    );
    let full = index
        .search(&request("Zaunkönig", limit, SearchMode::FullText), &[])
        .await
        .unwrap();
    assert!(
        !ids(&full).contains(&"near#0".to_string()),
        "near shares no word"
    );
    let hybrid = index
        .search(&request("Zaunkönig", limit, SearchMode::Hybrid), &q)
        .await
        .unwrap();
    let got = ids(&hybrid);
    assert!(
        got.contains(&"far#0".to_string()),
        "full-text rank 1: {got:?}"
    );
    assert!(
        got.contains(&"near#0".to_string()),
        "vector rank 1: {got:?}"
    );
    drop(index);
    remove(dir);
}

#[tokio::test]
async fn weight_one_follows_vector_and_weight_zero_full_text() {
    // E46
    let (index, dir, q) = fusion_index("e46-one", 1.0).await;
    let v = index
        .search(&request("Meise Zaunkönig", 10, SearchMode::Vector), &q)
        .await
        .unwrap();
    let h = index
        .search(&request("Meise Zaunkönig", 10, SearchMode::Hybrid), &q)
        .await
        .unwrap();
    assert_eq!(ids(&v), ids(&h));
    drop(index);
    remove(dir);
    let (index, dir, q) = fusion_index("e46-zero", 0.0).await;
    let f = index
        .search(&request("Zaunkönig Meise", 5, SearchMode::FullText), &[])
        .await
        .unwrap();
    let h = index
        .search(&request("Zaunkönig Meise", 5, SearchMode::Hybrid), &q)
        .await
        .unwrap();
    assert_eq!(ids(&f), ids(&h));
    drop(index);
    remove(dir);
    for bad in [-0.1, 1.1, f32::NAN, f32::INFINITY] {
        let r = LanceIndex::open(test_dir("e46-bad"), space(DIM), bad).await;
        assert!(
            matches!(r, Err(Error::InvalidInput(_))),
            "{bad}: {:?}",
            r.err()
        );
    }
}

#[tokio::test]
async fn chunks_without_language_do_not_match_stop_words() {
    // V1: the default analysis drops German and English stop words.
    let (index, dir) = sample("v1").await;
    index
        .add(&[
            entry(
                chunk("none", 0, "Der Text und die Sache ist das hier.", None),
                DIM,
                20,
            ),
            entry(
                chunk("none", 1, "We did it and the rest is history.", None),
                DIM,
                21,
            ),
            entry(
                chunk("fr", 0, "Le texte und der Rest.", Some("fr")),
                DIM,
                22,
            ),
        ])
        .await
        .unwrap();
    let only_stop = index
        .search(
            &request("der die und das ist the and", 10, SearchMode::FullText),
            &[],
        )
        .await
        .unwrap();
    assert!(only_stop.is_empty(), "{:?}", ids(&only_stop));
    let hits = index
        .search(
            &request(
                "Wie wählt der Bundestag und die Wahl?",
                10,
                SearchMode::FullText,
            ),
            &[],
        )
        .await
        .unwrap();
    assert!(!hits.is_empty());
    assert!(
        hits.iter()
            .all(|h| h.chunk.language.as_deref() == Some("de")),
        "{:?}",
        ids(&hits)
    );
    drop(index);
    remove(dir);
}

#[tokio::test]
async fn fusion_uses_ranks_beyond_the_limit() {
    // V3, E45: `both` is on rank 8 of both lists; with `limit` 5 it only enters the
    // result if the fusion reads deeper than the limit (k = 5: 2 x 0.5 / 13 beats the
    // second rank of a single list, 0.5 / 7).
    let dir = test_dir("v3");
    let index = LanceIndex::open(&dir, space(DIM), 0.5).await.unwrap();
    let q = axis(DIM, 0);
    let along = |c: f32| {
        let mut v = vec![0.0; DIM as usize];
        v[0] = c;
        v[1] = (1.0 - c * c).sqrt();
        v
    };
    let mut entries = Vec::new();
    for i in 0..7u32 {
        entries.push(IndexEntry {
            chunk: chunk(
                &format!("vec-{i}"),
                0,
                "Nur ein Vektor ohne das Wort.",
                Some("de"),
            ),
            embedding: along(0.99 - i as f32 * 0.01),
        });
        entries.push(IndexEntry {
            chunk: chunk(
                &format!("txt-{i}"),
                0,
                "Zaunkönig Zaunkönig Zaunkönig",
                Some("de"),
            ),
            embedding: along(-0.5),
        });
    }
    entries.push(IndexEntry {
        chunk: chunk(
            "both",
            0,
            "Ein Zaunkönig sitzt hier zwischen vielen anderen Wörtern im langen Satz.",
            Some("de"),
        ),
        embedding: along(0.91),
    });
    // Fillers push the `txt` chunks behind the fusion depth of the vector list.
    for i in 0..150u32 {
        entries.push(IndexEntry {
            chunk: chunk(
                &format!("fill-{i:03}"),
                0,
                "Füllung ohne Bezug.",
                Some("de"),
            ),
            embedding: along(0.0),
        });
    }
    index.add(&entries).await.unwrap();
    let vector = index
        .search(&request("Zaunkönig", 20, SearchMode::Vector), &q)
        .await
        .unwrap();
    let full = index
        .search(&request("Zaunkönig", 20, SearchMode::FullText), &[])
        .await
        .unwrap();
    assert_eq!(ids(&vector).iter().position(|i| i == "both#0"), Some(7));
    assert_eq!(ids(&full).iter().position(|i| i == "both#0"), Some(7));
    let hybrid = index
        .search(&request("Zaunkönig", 5, SearchMode::Hybrid), &q)
        .await
        .unwrap();
    assert!(
        ids(&hybrid).contains(&"both#0".to_string()),
        "{:?}",
        ids(&hybrid)
    );
    drop(index);
    remove(dir);
}

#[tokio::test]
async fn ties_at_the_cut_are_ordered_by_chunk_id() {
    // V4: 60 equal rows, `limit` 10: the ten smallest chunk ids, also after maintenance.
    let (index, dir) = open("v4", DIM).await;
    // Inserted in reverse order, so the smallest ids are stored last.
    let entries: Vec<IndexEntry> = (0..60)
        .rev()
        .map(|n| IndexEntry {
            chunk: chunk(
                &format!("tie-{n:02}"),
                0,
                "gleicher Text Pelikan",
                Some("de"),
            ),
            embedding: axis(DIM, 3),
        })
        .collect();
    index.add(&entries).await.unwrap();
    let want: Vec<String> = (0..10).map(|n| format!("tie-{n:02}#0")).collect();
    for round in 0..2 {
        for mode in MODES {
            let hits = index
                .search(&request("Pelikan", 10, mode), &axis(DIM, 3))
                .await
                .unwrap();
            assert_eq!(ids(&hits), want, "{mode:?}, round {round}");
        }
        index.optimize().await.unwrap();
    }
    drop(index);
    remove(dir);
}

#[tokio::test]
async fn unusable_query_vectors_are_invalid_input() {
    // V7
    let (index, dir) = sample("v7-search").await;
    let mut nan = seeded(DIM, 1);
    nan[3] = f32::NAN;
    let mut inf = seeded(DIM, 1);
    inf[0] = f32::INFINITY;
    let zero = vec![0.0; DIM as usize];
    for mode in [SearchMode::Vector, SearchMode::Hybrid] {
        for v in [&nan, &inf, &zero] {
            let r = index.search(&request("Bundestag", 10, mode), v).await;
            assert!(matches!(r, Err(Error::InvalidInput(_))), "{mode:?}: {r:?}");
        }
    }
    assert!(
        !index
            .search(&request("Bundestag", 10, SearchMode::FullText), &nan)
            .await
            .unwrap()
            .is_empty()
    );
    drop(index);
    remove(dir);
}
