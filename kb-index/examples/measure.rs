//! Measurement protocol of `kb-index` (criterion E50): adds 5,000 and 100,000
//! entries with dimension 768 (texts derived from the passages of the embedding test,
//! random L2-normalised vectors), times `add` per 1,000 entries, `search` per mode
//! and the maintenance, measures the folder before and after maintenance and checks
//! that the vector search equals the exact cosine over all entries (E34).
//!
//! ```text
//! cargo run --release -p kb-index --example measure [-- 5000,100000]
//! ```
//!
//! The indexes are written below `target/kb-index-measure/` of the workspace.

use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use kb_core::{
    Anchor, Chunk, ChunkId, DocumentId, EmbeddingSpace, Index, IndexEntry, Provenance, SearchMode,
    SearchRequest, SourceFile,
};
use kb_index::LanceIndex;

const DIMENSION: u32 = 768;
const BATCH: usize = 1000;
const QUERIES: usize = 20;
const EXACT_CHECKS: u64 = 3;

fn random_unit(seed: u64) -> Vec<f32> {
    let mut x = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
    let mut v: Vec<f32> = (0..DIMENSION)
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

fn cosine(a: &[f32], b: &[f32]) -> f32 {
    let (mut d, mut na, mut nb) = (0.0f64, 0.0f64, 0.0f64);
    for (x, y) in a.iter().zip(b) {
        d += f64::from(*x) * f64::from(*y);
        na += f64::from(*x).powi(2);
        nb += f64::from(*y).powi(2);
    }
    (d / (na.sqrt() * nb.sqrt())) as f32
}

fn dir_size(dir: &Path) -> u64 {
    std::fs::read_dir(dir)
        .map(|it| {
            it.flatten()
                .map(|e| {
                    let p = e.path();
                    if p.is_dir() {
                        dir_size(&p)
                    } else {
                        e.metadata().map(|m| m.len()).unwrap_or(0)
                    }
                })
                .sum()
        })
        .unwrap_or(0)
}

fn today() -> String {
    let days = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() / 86_400)
        .unwrap_or(0) as i64;
    // Civil date from days since 1970-01-01 (H. Hinnant's algorithm).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!("{y:04}-{m:02}-{d:02}")
}

fn jsonl_texts(path: &Path, field: &str) -> Vec<String> {
    std::fs::read_to_string(path)
        .expect("test data")
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            let v: serde_json::Value = serde_json::from_str(l).expect("json line");
            v[field].as_str().unwrap_or_default().to_string()
        })
        .collect()
}

fn entry(i: usize, texts: &[String]) -> IndexEntry {
    let doc = DocumentId(format!("doc-{:06}", i / 10));
    let n = (i % 10) as u32;
    IndexEntry {
        chunk: Chunk {
            id: ChunkId::new(&doc, n),
            text: format!("{} (Fassung {i})", texts[i % texts.len()]),
            language: Some("de".into()),
            provenance: Provenance {
                source: SourceFile {
                    document_id: doc.clone(),
                    path: PathBuf::from(format!("C:\\Messung\\{}.md", doc.0)),
                    name: format!("{}.md", doc.0),
                    content_hash: "0".repeat(64),
                },
                heading_path: vec!["Messung".into()],
                anchors: vec![Anchor::Paragraph { element: n }],
            },
        },
        embedding: random_unit(i as u64 + 1),
    }
}

async fn measure(n: usize, root: &Path, texts: &[String], questions: &[String]) {
    let dir = root.join(format!("{n}"));
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("clear measurement folder");
    }
    let space = EmbeddingSpace {
        model_id: "measure".into(),
        dimension: DIMENSION,
    };
    let index = LanceIndex::open(&dir, space, 0.5).await.expect("open");
    let mut batch_ms = Vec::new();
    let mut vectors = Vec::with_capacity(n);
    for start in (0..n).step_by(BATCH) {
        let entries: Vec<IndexEntry> = (start..(start + BATCH).min(n))
            .map(|i| entry(i, texts))
            .collect();
        vectors.extend(
            entries
                .iter()
                .map(|e| (e.chunk.id.0.clone(), e.embedding.clone())),
        );
        let t = Instant::now();
        index.add(&entries).await.expect("add");
        batch_ms.push(t.elapsed().as_secs_f64() * 1000.0);
    }
    let before = dir_size(&dir);
    let t = Instant::now();
    index.optimize().await.expect("optimize");
    let optimize_ms = t.elapsed().as_secs_f64() * 1000.0;
    let after = dir_size(&dir);

    let mut search = Vec::new();
    for mode in [SearchMode::Vector, SearchMode::FullText, SearchMode::Hybrid] {
        let req = |q: &str| SearchRequest {
            text: q.to_string(),
            limit: 10,
            documents: None,
            mode,
        };
        index
            .search(&req(&questions[0]), &random_unit(7))
            .await
            .expect("warm-up");
        let mut times = Vec::new();
        for (i, q) in questions.iter().take(QUERIES).enumerate() {
            let v = random_unit(1_000_000 + i as u64);
            let t = Instant::now();
            index.search(&req(q), &v).await.expect("search");
            times.push(t.elapsed().as_secs_f64() * 1000.0);
        }
        let mean = times.iter().sum::<f64>() / times.len() as f64;
        let max = times.iter().cloned().fold(0.0, f64::max);
        search.push((mode, mean, max));
    }

    let mut exact = true;
    for s in 0..EXACT_CHECKS {
        let q = random_unit(2_000_000 + s);
        let hits = index
            .search(
                &SearchRequest {
                    text: "x".into(),
                    limit: 10,
                    documents: None,
                    mode: SearchMode::Vector,
                },
                &q,
            )
            .await
            .expect("search");
        let mut all: Vec<(&String, f32)> =
            vectors.iter().map(|(id, v)| (id, cosine(&q, v))).collect();
        all.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(b.0)));
        for (h, (id, cos)) in hits.iter().zip(&all) {
            if &h.chunk.id.0 != *id || (h.retrieval_score - cos).abs() > 1e-6 {
                exact = false;
            }
        }
        exact &= hits.len() == 10;
    }

    let mean_batch = batch_ms.iter().sum::<f64>() / batch_ms.len() as f64;
    let max_batch = batch_ms.iter().cloned().fold(0.0, f64::max);
    println!(
        "| {n} | {mean_batch:.0} | {max_batch:.0} | {optimize_ms:.0} | {:.1} | {:.1} |",
        before as f64 / 1e6,
        after as f64 / 1e6
    );
    for (mode, mean, max) in search {
        println!("|   search {mode:?} | Ø {mean:.1} ms | max {max:.1} ms |");
    }
    println!(
        "|   exact cosine order over all {n} entries ({EXACT_CHECKS} queries): {}",
        if exact { "yes" } else { "NO" }
    );
    drop(index);
}

#[tokio::main]
async fn main() {
    let sizes: Vec<usize> = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "5000,100000".into())
        .split(',')
        .map(|s| s.trim().parse().expect("size"))
        .collect();
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let data = crate_dir.join("testdata/embedding-test");
    let texts = jsonl_texts(&data.join("passages.jsonl"), "text");
    let questions = jsonl_texts(&data.join("questions.jsonl"), "q");
    let root = crate_dir.join("../target/kb-index-measure");
    println!(
        "date {}, processor {}, {} logical CPUs",
        today(),
        std::env::var("PROCESSOR_IDENTIFIER").unwrap_or_else(|_| "unknown".into()),
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(0)
    );
    println!(
        "| entries | add per 1,000 (ms, mean) | slowest 1,000 (ms) | maintenance (ms) | folder before (MB) | folder after (MB) |"
    );
    for n in sizes {
        measure(n, &root, &texts, &questions).await;
    }
}
