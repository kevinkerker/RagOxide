//! Criterion E06: the test data taken over from the embedding test is complete and
//! consistent (counts, vector length, L2 norm, `rel` ids point to passages).

use std::collections::HashSet;
use std::path::PathBuf;

const DIMENSION: usize = 768;
const PASSAGES: usize = 357;
const QUESTIONS: usize = 52;
const MAX_TOTAL_BYTES: u64 = 2 * 1024 * 1024;

fn data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata/embedding-test")
}

fn read_jsonl(name: &str) -> Vec<serde_json::Value> {
    let text = std::fs::read_to_string(data_dir().join(name)).expect("read jsonl");
    text.lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).expect("valid json line"))
        .collect()
}

fn read_vectors(name: &str) -> Vec<Vec<f32>> {
    let bytes = std::fs::read(data_dir().join(name)).expect("read vectors");
    assert_eq!(
        bytes.len() % (DIMENSION * 4),
        0,
        "{name}: not a whole number of rows"
    );
    bytes
        .chunks_exact(DIMENSION * 4)
        .map(|row| {
            row.chunks_exact(4)
                .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
                .collect()
        })
        .collect()
}

fn assert_unit_rows(name: &str, rows: &[Vec<f32>]) {
    for (i, row) in rows.iter().enumerate() {
        assert_eq!(row.len(), DIMENSION, "{name} row {i}: length");
        let norm = row.iter().map(|v| v * v).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-3, "{name} row {i}: norm {norm}");
    }
}

#[test]
fn passages_and_questions_have_expected_counts_and_fields() {
    let passages = read_jsonl("passages.jsonl");
    let questions = read_jsonl("questions.jsonl");
    assert_eq!(passages.len(), PASSAGES);
    assert_eq!(questions.len(), QUESTIONS);
    for p in &passages {
        for field in ["id", "doc", "heading", "text"] {
            assert!(p[field].is_string(), "passage field {field}: {p}");
        }
    }
    for q in &questions {
        assert!(q["q"].is_string(), "question text: {q}");
        assert!(q["rel"].is_array(), "question rel: {q}");
    }
}

#[test]
fn every_rel_id_is_a_passage() {
    let passages = read_jsonl("passages.jsonl");
    let ids: HashSet<&str> = passages.iter().filter_map(|p| p["id"].as_str()).collect();
    assert_eq!(ids.len(), PASSAGES, "passage ids are unique");
    for q in read_jsonl("questions.jsonl") {
        let rel = q["rel"].as_array().expect("rel array");
        assert!(!rel.is_empty(), "question without rel: {q}");
        for id in rel {
            let id = id.as_str().expect("rel id is a string");
            assert!(ids.contains(id), "rel id {id} is not a passage");
        }
    }
}

#[test]
fn vectors_have_one_unit_row_per_line() {
    let passages = read_vectors("passages.f32");
    let questions = read_vectors("questions.f32");
    assert_eq!(passages.len(), PASSAGES);
    assert_eq!(questions.len(), QUESTIONS);
    assert_unit_rows("passages.f32", &passages);
    assert_unit_rows("questions.f32", &questions);
}

#[test]
fn passage_sources_cover_every_passage_in_order() {
    let passages = read_jsonl("passages.jsonl");
    let tsv = std::fs::read_to_string(data_dir().join("passages-sources.tsv")).expect("read tsv");
    let rows: Vec<Vec<&str>> = tsv
        .lines()
        .skip(1)
        .map(|l| l.split('\t').collect())
        .collect();
    assert_eq!(rows.len(), PASSAGES);
    for (row, p) in rows.iter().zip(&passages) {
        assert_eq!(row.len(), 3, "row {row:?}");
        assert_eq!(Some(row[0]), p["id"].as_str());
    }
}

#[test]
fn test_data_stays_below_two_megabytes() {
    let total: u64 = std::fs::read_dir(data_dir())
        .expect("list test data")
        .map(|e| e.expect("entry").metadata().expect("metadata").len())
        .sum();
    assert!(total <= MAX_TOTAL_BYTES, "test data has {total} bytes");
}
