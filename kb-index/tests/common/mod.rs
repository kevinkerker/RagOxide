//! Helpers shared by the integration tests of `kb-index`.

#![allow(dead_code)]

use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

use kb_core::{
    Anchor, Chunk, ChunkId, DocumentId, EmbeddingSpace, IndexEntry, Provenance, SearchMode,
    SearchRequest, SourceFile,
};
use kb_index::LanceIndex;

/// A fresh, empty folder below the target directory of the tests.
pub fn test_dir(name: &str) -> PathBuf {
    static N: AtomicU32 = AtomicU32::new(0);
    let n = N.fetch_add(1, Ordering::SeqCst);
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("kb-index-tests")
        .join(format!("{name}-{}-{n}", std::process::id()));
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("clear test dir");
    }
    dir
}

/// The space of the mock embedder (64 dimensions).
pub fn space(dimension: u32) -> EmbeddingSpace {
    EmbeddingSpace {
        model_id: "test-model".into(),
        dimension,
    }
}

/// A chunk of `document` with ordinal `n`.
pub fn chunk(document: &str, n: u32, text: &str, language: Option<&str>) -> Chunk {
    let doc = DocumentId(document.to_string());
    Chunk {
        id: ChunkId::new(&doc, n),
        text: text.to_string(),
        language: language.map(str::to_string),
        provenance: Provenance {
            source: SourceFile {
                document_id: doc,
                path: PathBuf::from(format!("C:\\Docs\\{document}.md")),
                name: format!("{document}.md"),
                content_hash: "0".repeat(64),
            },
            heading_path: vec!["Abschnitt".into()],
            anchors: vec![Anchor::Paragraph { element: n }],
        },
    }
}

/// A unit vector along axis `i` (modulo the dimension).
pub fn axis(dimension: u32, i: usize) -> Vec<f32> {
    let mut v = vec![0.0; dimension as usize];
    v[i % dimension as usize] = 1.0;
    v
}

/// A unit vector from a seed (deterministic, spread over all axes).
pub fn seeded(dimension: u32, seed: u64) -> Vec<f32> {
    let mut x = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
    let mut v: Vec<f32> = (0..dimension)
        .map(|_| {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            (x >> 11) as f32 / (1u64 << 53) as f32 - 0.5
        })
        .collect();
    let n = v.iter().map(|a| a * a).sum::<f32>().sqrt();
    v.iter_mut().for_each(|a| *a /= n);
    v
}

/// An entry with a seeded vector.
pub fn entry(c: Chunk, dimension: u32, seed: u64) -> IndexEntry {
    IndexEntry {
        chunk: c,
        embedding: seeded(dimension, seed),
    }
}

/// A request in `mode` without document filter.
pub fn request(text: &str, limit: u32, mode: SearchMode) -> SearchRequest {
    SearchRequest {
        text: text.to_string(),
        limit,
        documents: None,
        mode,
    }
}

/// Opens a fresh test index.
pub async fn open(name: &str, dimension: u32) -> (LanceIndex, PathBuf) {
    let dir = test_dir(name);
    let index = LanceIndex::open(&dir, space(dimension), 0.5)
        .await
        .expect("open test index");
    (index, dir)
}

/// Chunk ids of hits, in order.
pub fn ids(hits: &[kb_core::SearchHit]) -> Vec<String> {
    hits.iter().map(|h| h.chunk.id.0.clone()).collect()
}

/// Removes a test folder after the index is dropped.
pub fn remove(dir: PathBuf) {
    let _ = std::fs::remove_dir_all(dir);
}
