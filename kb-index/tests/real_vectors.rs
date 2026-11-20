//! Hybrid beats single searches with real vectors (E48, ADR 0118): the 357 passages
//! and 52 questions of the embedding test with their embeddinggemma-2 vectors.

mod common;

use std::path::PathBuf;

use common::*;
use kb_core::{
    Anchor, Chunk, ChunkId, DocumentId, Index, IndexEntry, Provenance, SearchMode, SourceFile,
};
use serde::Deserialize;

const DIM: u32 = 768;

#[derive(Deserialize)]
struct Passage {
    id: String,
    doc: String,
    heading: String,
    text: String,
}

#[derive(Deserialize)]
struct Question {
    q: String,
    rel: Vec<String>,
}

fn data(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("testdata/embedding-test")
        .join(name)
}

fn jsonl<T: for<'de> Deserialize<'de>>(name: &str) -> Vec<T> {
    std::fs::read_to_string(data(name))
        .unwrap()
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).unwrap())
        .collect()
}

fn vectors(name: &str) -> Vec<Vec<f32>> {
    std::fs::read(data(name))
        .unwrap()
        .chunks_exact(DIM as usize * 4)
        .map(|row| {
            row.chunks_exact(4)
                .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
                .collect()
        })
        .collect()
}

#[tokio::test]
async fn hybrid_beats_single_searches_with_real_vectors() {
    let passages: Vec<Passage> = jsonl("passages.jsonl");
    let questions: Vec<Question> = jsonl("questions.jsonl");
    let pv = vectors("passages.f32");
    let qv = vectors("questions.f32");
    let (index, dir) = open("e48", DIM).await;
    let mut ordinals = std::collections::HashMap::new();
    let entries: Vec<IndexEntry> = passages
        .iter()
        .zip(&pv)
        .map(|(p, v)| {
            let doc = DocumentId(p.doc.clone());
            let n = ordinals.entry(p.doc.clone()).or_insert(0u32);
            let chunk = Chunk {
                id: ChunkId::new(&doc, *n),
                text: p.text.clone(),
                language: Some("de".into()),
                provenance: Provenance {
                    source: SourceFile {
                        document_id: doc,
                        path: PathBuf::from(format!("C:\\Korpus\\{}", p.doc)),
                        name: p.doc.clone(),
                        content_hash: "0".repeat(64),
                    },
                    heading_path: p.heading.split(" / ").map(str::to_string).collect(),
                    anchors: vec![Anchor::Paragraph { element: *n }],
                },
            };
            *n += 1;
            IndexEntry {
                chunk,
                embedding: v.clone(),
            }
        })
        .collect();
    let passage_of: std::collections::HashMap<String, String> = entries
        .iter()
        .zip(&passages)
        .map(|(e, p)| (e.chunk.id.0.clone(), p.id.clone()))
        .collect();
    index.add(&entries).await.unwrap();

    let mut found = [0usize; 3];
    let (mut top1, mut top5) = (0, 0);
    for (q, v) in questions.iter().zip(&qv) {
        for (m, mode) in [SearchMode::Hybrid, SearchMode::Vector, SearchMode::FullText]
            .into_iter()
            .enumerate()
        {
            let hits = index.search(&request(&q.q, 10, mode), v).await.unwrap();
            let ranks: Vec<usize> = hits
                .iter()
                .enumerate()
                .filter(|(_, h)| q.rel.contains(&passage_of[&h.chunk.id.0]))
                .map(|(i, _)| i + 1)
                .collect();
            if !ranks.is_empty() {
                found[m] += 1;
            }
            if mode == SearchMode::Vector {
                top1 += usize::from(ranks.first() == Some(&1));
                top5 += usize::from(ranks.first().is_some_and(|r| *r <= 5));
            }
        }
    }
    let [hybrid, vector, full_text] = found;
    println!(
        "found in the first ten of 52: hybrid {hybrid}, vector {vector}, full text {full_text}; vector top-1 {top1}, top-5 {top5}"
    );
    assert!(
        hybrid >= vector.max(full_text),
        "hybrid {hybrid}, vector {vector}, full text {full_text}"
    );
    assert!(
        hybrid > vector.min(full_text),
        "hybrid {hybrid}, vector {vector}, full text {full_text}"
    );
    drop(index);
    remove(dir);
}
