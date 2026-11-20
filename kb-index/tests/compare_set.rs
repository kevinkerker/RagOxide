//! The comparison set of step 1 with the chosen analysis (E42, E07): at least 12 of
//! the 21 rated German questions with compounds or word forms are found in the first
//! ten full-text hits.

mod common;

use std::collections::BTreeMap;
use std::path::PathBuf;

use common::*;
use kb_core::{Index, IndexEntry, SearchMode};
use serde::Deserialize;
use unicode_normalization::UnicodeNormalization;

const DIM: u32 = 64;
/// Found in step 1 by `lance-split-both` and `tantivy-split-both-ge`.
const FOUND_IN_STEP_ONE: usize = 12;

#[derive(Deserialize)]
struct Catalogue {
    questions: Vec<Question>,
}

#[derive(Deserialize)]
struct Question {
    id: String,
    language: String,
    question: String,
    answerable: bool,
    features: Vec<String>,
    #[serde(default)]
    evidence: Vec<Quote>,
    #[serde(default)]
    near_miss: Vec<Quote>,
}

#[derive(Deserialize)]
struct Quote {
    #[serde(default)]
    part: Option<u32>,
    quote: String,
}

#[derive(Deserialize)]
struct Passage {
    id: String,
    text: String,
}

/// The catalogue's normalisation: NFC, soft hyphens removed, `|` as space, runs of
/// whitespace collapsed.
fn normalise(text: &str) -> String {
    let s: String = text
        .nfc()
        .filter(|c| *c != '\u{00AD}')
        .map(|c| if c == '|' { ' ' } else { c })
        .collect();
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[tokio::test]
async fn the_chosen_analysis_finds_at_least_twelve_of_twenty_one() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let catalogue: Catalogue =
        toml::from_str(&std::fs::read_to_string(root.join("../corpus/questions.toml")).unwrap())
            .unwrap();
    let german: Vec<&Question> = catalogue
        .questions
        .iter()
        .filter(|q| q.language == "de")
        .collect();

    // Quotes of all German questions (evidence and near misses), then the passages.
    let mut texts: Vec<(String, String)> = Vec::new();
    for q in &german {
        for (i, e) in q.evidence.iter().chain(&q.near_miss).enumerate() {
            texts.push((format!("{}-{i}", q.id), e.quote.clone()));
        }
    }
    let passages =
        std::fs::read_to_string(root.join("testdata/embedding-test/passages.jsonl")).unwrap();
    for line in passages.lines().filter(|l| !l.trim().is_empty()) {
        let p: Passage = serde_json::from_str(line).unwrap();
        texts.push((format!("passage {}", p.id), p.text));
    }
    assert_eq!(texts.len(), 400);
    let normalised: Vec<String> = texts.iter().map(|(_, t)| normalise(t)).collect();

    let (index, dir) = open("e42", DIM).await;
    let entries: Vec<IndexEntry> = texts
        .iter()
        .enumerate()
        .map(|(i, (doc, t))| entry(chunk(doc, 0, t, Some("de")), DIM, i as u64))
        .collect();
    index.add(&entries).await.unwrap();

    let set: Vec<&&Question> = german
        .iter()
        .filter(|q| {
            q.features
                .iter()
                .any(|f| f == "compound" || f == "inflection")
        })
        .collect();
    assert_eq!(set.len(), 22, "comparison set of E07");
    let mut found = Vec::new();
    let mut rated = 0;
    for q in set {
        if !q.answerable {
            continue;
        }
        rated += 1;
        let hits = index
            .search(&request(&q.question, 10, SearchMode::FullText), &[])
            .await
            .unwrap();
        let top: Vec<usize> = hits
            .iter()
            .map(|h| {
                texts
                    .iter()
                    .position(|(d, _)| *d == h.chunk.document_id().0)
                    .unwrap()
            })
            .collect();
        let mut parts: BTreeMap<u32, Vec<String>> = BTreeMap::new();
        for e in &q.evidence {
            parts
                .entry(e.part.unwrap_or(1))
                .or_default()
                .push(normalise(&e.quote));
        }
        let ok = parts.values().all(|quotes| {
            top.iter()
                .any(|&i| quotes.iter().any(|qt| normalised[i].contains(qt)))
        });
        if ok {
            found.push(q.id.clone());
        }
    }
    assert_eq!(rated, 21);
    println!("found {} of 21: {found:?}", found.len());
    assert!(
        found.len() >= FOUND_IN_STEP_ONE,
        "found {} of 21: {found:?}",
        found.len()
    );
    drop(index);
    remove(dir);
}
