//! Creating, opening, persistence, format version, maintenance and local-only
//! paths (E12 to E16, E60, E61).

mod common;

use std::path::{Path, PathBuf};
use std::process::Command;

use common::*;
use kb_core::{DocumentId, Error, Index, IndexEntry, SearchHit, SearchMode};
use kb_index::{LanceIndex, MAINTENANCE_EVERY, META_FILE};

const DIM: u32 = 64;
const MODES: [SearchMode; 3] = [SearchMode::Hybrid, SearchMode::Vector, SearchMode::FullText];
const CHILD_DIR: &str = "KB_INDEX_TEST_CHILD_DIR";
const CHILD_OUT: &str = "KB_INDEX_TEST_CHILD_OUT";
const CHILD_ACTION: &str = "KB_INDEX_TEST_CHILD_ACTION";

fn entries() -> Vec<IndexEntry> {
    let texts = [
        ("doc-a", "Der Bundestag wählt den Bundeskanzler."),
        ("doc-a", "Die Abgeordneten stimmen ab."),
        ("doc-b", "Krankenhäuser melden Betten."),
        ("doc-c", "Kaffee am Nachmittag."),
    ];
    texts
        .iter()
        .enumerate()
        .map(|(i, (d, t))| entry(chunk(d, i as u32, t, Some("de")), DIM, i as u64))
        .collect()
}

/// All hits of fixed queries in all three modes, as JSON (exact scores).
async fn snapshot(index: &LanceIndex) -> String {
    let mut all: Vec<Vec<SearchHit>> = Vec::new();
    for text in ["Bundestag Kanzler", "Betten Kaffee", "Abgeordnete"] {
        for mode in MODES {
            all.push(
                index
                    .search(&request(text, 10, mode), &seeded(DIM, 2))
                    .await
                    .unwrap(),
            );
        }
    }
    serde_json::to_string(&all).unwrap()
}

fn read_meta(dir: &Path) -> serde_json::Value {
    serde_json::from_str(&std::fs::read_to_string(dir.join(META_FILE)).unwrap()).unwrap()
}

#[tokio::test]
async fn a_missing_or_empty_folder_gives_an_empty_index_with_its_metadata() {
    // E12
    let dir = test_dir("e12");
    let index = LanceIndex::open(&dir, space(DIM), 0.25).await.unwrap();
    assert_eq!(index.space(), space(DIM));
    for mode in MODES {
        assert!(
            index
                .search(&request("x", 5, mode), &seeded(DIM, 1))
                .await
                .unwrap()
                .is_empty()
        );
    }
    let meta = read_meta(&dir);
    assert_eq!(meta["model_id"], "test-model");
    assert_eq!(meta["dimension"], DIM);
    assert_eq!(meta["vector_weight"], 0.25);
    assert_eq!(meta["format_version"], kb_index::FORMAT_VERSION);
    drop(index);
    let empty = test_dir("e12-empty");
    std::fs::create_dir_all(&empty).unwrap();
    let index = LanceIndex::open(&empty, space(DIM), 0.5).await.unwrap();
    assert_eq!(index.space(), space(DIM));
    drop(index);
    remove(dir);
    remove(empty);
}

#[tokio::test]
async fn reopening_in_the_same_process_keeps_everything() {
    // E13, same process: after add, delete, replace and maintenance.
    let dir = test_dir("e13-same");
    let index = LanceIndex::open(&dir, space(DIM), 0.5).await.unwrap();
    index.add(&entries()).await.unwrap();
    let check = async |index: LanceIndex| -> LanceIndex {
        let before = snapshot(&index).await;
        drop(index);
        let index = LanceIndex::open(&dir, space(DIM), 0.5).await.unwrap();
        assert_eq!(index.space(), space(DIM));
        assert_eq!(snapshot(&index).await, before);
        index
    };
    let index = check(index).await;
    index
        .delete_document(&DocumentId("doc-c".into()))
        .await
        .unwrap();
    let index = check(index).await;
    index
        .delete_document(&DocumentId("doc-b".into()))
        .await
        .unwrap();
    index
        .add(&[entry(
            chunk("doc-b", 0, "Neue Fassung über Betten.", Some("de")),
            DIM,
            9,
        )])
        .await
        .unwrap();
    let index = check(index).await;
    index.optimize().await.unwrap();
    let index = check(index).await;
    drop(index);
    remove(dir);
}

/// Runs this test binary again with one test and the given environment.
fn run_child(test: &str, envs: &[(&str, &Path)]) {
    let mut cmd = Command::new(std::env::current_exe().unwrap());
    cmd.args([test, "--exact", "--ignored", "--test-threads=1"]);
    for (k, v) in envs {
        cmd.env(k, v);
    }
    let out = cmd.output().unwrap();
    assert!(
        out.status.success(),
        "child failed:\n{}\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

#[tokio::test]
#[ignore = "runs only as the child of another test"]
async fn child_process_snapshot() {
    let (Some(dir), Some(out)) = (std::env::var_os(CHILD_DIR), std::env::var_os(CHILD_OUT)) else {
        return;
    };
    let dir = PathBuf::from(dir);
    let index = LanceIndex::open(&dir, space(DIM), 0.5).await.unwrap();
    if std::env::var(CHILD_ACTION).as_deref() == Ok("build") {
        index.add(&entries()).await.unwrap();
        index
            .delete_document(&DocumentId("doc-c".into()))
            .await
            .unwrap();
        index
            .add(&[entry(
                chunk("doc-b", 2, "Neue Fassung über Betten.", Some("de")),
                DIM,
                9,
            )])
            .await
            .unwrap();
        index.optimize().await.unwrap();
    }
    std::fs::write(out, snapshot(&index).await).unwrap();
}

#[tokio::test]
async fn reopening_in_a_new_process_keeps_everything() {
    // E13, new process after the first has closed the index.
    let dir = test_dir("e13-process");
    let index = LanceIndex::open(&dir, space(DIM), 0.5).await.unwrap();
    index.add(&entries()).await.unwrap();
    index
        .delete_document(&DocumentId("doc-c".into()))
        .await
        .unwrap();
    index.optimize().await.unwrap();
    let before = snapshot(&index).await;
    drop(index);
    let out = dir.with_extension("child.json");
    run_child(
        "child_process_snapshot",
        &[(CHILD_DIR, &dir), (CHILD_OUT, &out)],
    );
    assert_eq!(std::fs::read_to_string(&out).unwrap(), before);
    let _ = std::fs::remove_file(out);
    remove(dir);
}

#[tokio::test]
async fn the_index_writes_nothing_outside_its_folder() {
    // E12: a child process with its own empty temporary folder builds, changes,
    // maintains and searches an index; afterwards that folder is still empty.
    let dir = test_dir("e12-outside");
    let tmp = test_dir("e12-tmp");
    std::fs::create_dir_all(&tmp).unwrap();
    let out = dir.with_extension("child.json");
    run_child(
        "child_process_snapshot",
        &[
            (CHILD_DIR, &dir),
            (CHILD_OUT, &out),
            (CHILD_ACTION, Path::new("build")),
            ("TMP", &tmp),
            ("TEMP", &tmp),
            ("TMPDIR", &tmp),
        ],
    );
    let left: Vec<_> = std::fs::read_dir(&tmp).unwrap().collect();
    assert!(left.is_empty(), "files in the temporary folder: {left:?}");
    let top: Vec<String> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert!(
        top.iter().all(|n| n == META_FILE || n == "chunks.lance"),
        "{top:?}"
    );
    let _ = std::fs::remove_file(out);
    remove(dir);
    remove(tmp);
}

#[tokio::test]
async fn another_space_is_a_mismatch_and_leaves_the_data() {
    // E14
    let dir = test_dir("e14");
    let index = LanceIndex::open(&dir, space(DIM), 0.5).await.unwrap();
    index.add(&entries()).await.unwrap();
    let before = snapshot(&index).await;
    drop(index);
    let other_model = kb_core::EmbeddingSpace {
        model_id: "other".into(),
        dimension: DIM,
    };
    for wrong in [other_model, space(DIM * 2)] {
        let r = LanceIndex::open(&dir, wrong.clone(), 0.5).await;
        assert!(
            matches!(r, Err(Error::EmbeddingMismatch(_))),
            "{wrong:?}: {:?}",
            r.err()
        );
    }
    let index = LanceIndex::open(&dir, space(DIM), 0.5).await.unwrap();
    assert_eq!(snapshot(&index).await, before);
    drop(index);
    remove(dir);
}

#[tokio::test]
async fn another_format_version_is_a_mismatch_and_leaves_the_data() {
    // E15
    let dir = test_dir("e15");
    let index = LanceIndex::open(&dir, space(DIM), 0.5).await.unwrap();
    index.add(&entries()).await.unwrap();
    let before = snapshot(&index).await;
    drop(index);
    let original = std::fs::read_to_string(dir.join(META_FILE)).unwrap();
    let mut meta = read_meta(&dir);
    meta["format_version"] = serde_json::json!(kb_index::FORMAT_VERSION + 1);
    std::fs::write(dir.join(META_FILE), meta.to_string()).unwrap();
    match LanceIndex::open(&dir, space(DIM), 0.5).await {
        Err(Error::EmbeddingMismatch(m)) => assert!(m.contains("format version"), "{m}"),
        other => panic!("expected EmbeddingMismatch, got {:?}", other.err()),
    }
    assert_eq!(read_meta(&dir), meta, "metadata untouched");
    std::fs::write(dir.join(META_FILE), original).unwrap();
    let index = LanceIndex::open(&dir, space(DIM), 0.5).await.unwrap();
    assert_eq!(snapshot(&index).await, before);
    drop(index);
    remove(dir);
}

#[tokio::test]
async fn damaged_or_foreign_files_are_an_index_error() {
    // E16
    let foreign = test_dir("e16-foreign");
    std::fs::create_dir_all(&foreign).unwrap();
    std::fs::write(foreign.join("notes.txt"), "not an index").unwrap();
    match LanceIndex::open(&foreign, space(DIM), 0.5).await {
        Err(Error::Index(m)) => assert!(m.contains(&foreign.display().to_string()), "{m}"),
        other => panic!("expected Index error, got {:?}", other.err()),
    }
    remove(foreign);

    let dir = test_dir("e16-meta");
    drop(LanceIndex::open(&dir, space(DIM), 0.5).await.unwrap());
    std::fs::write(dir.join(META_FILE), "{ broken").unwrap();
    match LanceIndex::open(&dir, space(DIM), 0.5).await {
        Err(Error::Index(m)) => assert!(m.contains(&dir.display().to_string()), "{m}"),
        other => panic!("expected Index error, got {:?}", other.err()),
    }
    remove(dir);

    let dir = test_dir("e16-table");
    let index = LanceIndex::open(&dir, space(DIM), 0.5).await.unwrap();
    index.add(&entries()).await.unwrap();
    drop(index);
    for e in walk(&dir.join("chunks.lance")) {
        std::fs::write(e, b"garbage").unwrap();
    }
    match LanceIndex::open(&dir, space(DIM), 0.5).await {
        Err(Error::Index(m)) => assert!(m.contains(&dir.display().to_string()), "{m}"),
        other => panic!("expected Index error, got {:?}", other.err()),
    }
    remove(dir);
}

fn walk(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for e in std::fs::read_dir(dir).unwrap().flatten() {
        let p = e.path();
        if p.is_dir() {
            out.extend(walk(&p));
        } else {
            out.push(p);
        }
    }
    out
}

#[tokio::test]
async fn urls_and_network_shares_are_rejected_before_any_connection() {
    // E60
    for path in [
        "s3://bucket/index",
        "gs://bucket/index",
        "az://container/index",
        "http://example.invalid/index",
        "https://example.invalid/index",
        "file:///C:/index",
        "\\\\server\\share\\index",
    ] {
        let r = LanceIndex::open(path, space(DIM), 0.5).await;
        assert!(
            matches!(r, Err(Error::InvalidInput(_))),
            "{path}: {:?}",
            r.err()
        );
    }
}

#[tokio::test]
async fn maintenance_keeps_the_number_of_versions_bounded() {
    // E61
    let dir = test_dir("e61");
    let index = LanceIndex::open(&dir, space(DIM), 0.5).await.unwrap();
    index.add(&entries()).await.unwrap();
    let doc = DocumentId("doc-b".into());
    let replace = || {
        vec![entry(
            chunk("doc-b", 2, "Krankenhäuser melden Betten.", Some("de")),
            DIM,
            2,
        )]
    };
    index.optimize().await.unwrap();
    let reference = snapshot(&index).await;
    let mut most = 0;
    for _ in 0..100 {
        index.delete_document(&doc).await.unwrap();
        index.add(&replace()).await.unwrap();
        most = most.max(index.version_count().await.unwrap());
    }
    // 200 changes: the last change ran the maintenance.
    let bound = 2 * MAINTENANCE_EVERY as usize + 2;
    println!("at most {most} versions kept during 200 changes");
    assert!(
        most <= bound,
        "up to {most} versions kept after 200 changes"
    );
    assert_eq!(snapshot(&index).await, reference);
    drop(index);
    let index = LanceIndex::open(&dir, space(DIM), 0.5).await.unwrap();
    assert_eq!(snapshot(&index).await, reference);
    drop(index);
    remove(dir);
}

/// The passages of the embedding test as German chunks with seeded vectors.
fn passages() -> Vec<IndexEntry> {
    let text = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata/embedding-test/passages.jsonl"),
    )
    .unwrap();
    let mut ordinals = std::collections::HashMap::new();
    text.lines()
        .filter(|l| !l.trim().is_empty())
        .enumerate()
        .map(|(i, l)| {
            let v: serde_json::Value = serde_json::from_str(l).unwrap();
            let doc = v["doc"].as_str().unwrap().to_string();
            let n = ordinals.entry(doc.clone()).or_insert(0u32);
            let e = entry(
                chunk(&doc, *n, v["text"].as_str().unwrap(), Some("de")),
                DIM,
                i as u64,
            );
            *n += 1;
            e
        })
        .collect()
}

/// Hits of the 52 questions of the embedding test in all three modes, as JSON.
async fn question_snapshot(index: &LanceIndex) -> String {
    let text = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata/embedding-test/questions.jsonl"),
    )
    .unwrap();
    let mut all: Vec<Vec<SearchHit>> = Vec::new();
    for (i, l) in text.lines().filter(|l| !l.trim().is_empty()).enumerate() {
        let v: serde_json::Value = serde_json::from_str(l).unwrap();
        let q = v["q"].as_str().unwrap();
        for mode in MODES {
            all.push(
                index
                    .search(&request(q, 10, mode), &seeded(DIM, 1000 + i as u64))
                    .await
                    .unwrap(),
            );
        }
    }
    serde_json::to_string(&all).unwrap()
}

#[tokio::test]
async fn maintenance_counts_changes_across_sessions() {
    // E61 across sessions: twelve sessions with ten changes each.
    let dir = test_dir("e61-sessions");
    let all = passages();
    let wiki: Vec<IndexEntry> = all
        .iter()
        .filter(|e| e.chunk.document_id().0 == "wikipedia")
        .cloned()
        .collect();
    assert_eq!(wiki.len(), 45);
    let index = LanceIndex::open(&dir, space(DIM), 0.5).await.unwrap();
    index.add(&all).await.unwrap();
    index.optimize().await.unwrap();
    let reference = question_snapshot(&index).await;
    drop(index);
    let doc = DocumentId("wikipedia".into());
    let mut most = 0;
    for _ in 0..12 {
        let index = LanceIndex::open(&dir, space(DIM), 0.5).await.unwrap();
        for _ in 0..5 {
            index.delete_document(&doc).await.unwrap();
            index.add(&wiki).await.unwrap();
            most = most.max(index.version_count().await.unwrap());
        }
        drop(index);
    }
    println!("at most {most} versions kept during 120 changes in 12 sessions");
    assert!(most <= MAINTENANCE_EVERY as usize + 1, "{most} versions");
    // 120 changes: the maintenance ran with the last change, so the lists equal the
    // maintained state exactly.
    let index = LanceIndex::open(&dir, space(DIM), 0.5).await.unwrap();
    assert_eq!(index.version_count().await.unwrap(), 1);
    assert_eq!(question_snapshot(&index).await, reference);
    drop(index);
    remove(dir);
}

#[tokio::test]
async fn an_interrupted_first_opening_does_not_block_the_folder() {
    // V6: table created, metadata missing (the first opening stopped in between).
    let dir = test_dir("v6");
    let index = LanceIndex::open(&dir, space(DIM), 0.5).await.unwrap();
    drop(index);
    std::fs::remove_file(dir.join(META_FILE)).unwrap();
    std::fs::write(dir.join("kb-index.json.tmp"), "{").unwrap();
    let index = LanceIndex::open(&dir, space(DIM), 0.5).await.unwrap();
    assert!(
        index
            .search(&request("x", 5, SearchMode::Vector), &seeded(DIM, 1))
            .await
            .unwrap()
            .is_empty()
    );
    index.add(&entries()).await.unwrap();
    drop(index);
    let index = LanceIndex::open(&dir, space(DIM), 0.5).await.unwrap();
    assert_eq!(
        index
            .search(&request("x", 10, SearchMode::Vector), &seeded(DIM, 1))
            .await
            .unwrap()
            .len(),
        4
    );
    drop(index);
    remove(dir);
}

/// Opening must fail with `Error::Index` naming the folder, and leave every file.
async fn assert_refused_untouched(dir: &Path) {
    let before = tree(dir);
    match LanceIndex::open(dir, space(DIM), 0.5).await {
        Err(Error::Index(m)) => assert!(m.contains(&dir.display().to_string()), "{m}"),
        other => panic!("expected Index error, got {:?}", other.err()),
    }
    assert_eq!(tree(dir), before, "files changed");
}

/// Relative path, size and content hash of every file below `dir`.
fn tree(dir: &Path) -> Vec<(String, Vec<u8>)> {
    let mut out: Vec<(String, Vec<u8>)> = walk(dir)
        .into_iter()
        .map(|p| {
            let rel = p.strip_prefix(dir).unwrap().display().to_string();
            (rel, std::fs::read(&p).unwrap())
        })
        .collect();
    out.sort();
    out
}

#[tokio::test]
async fn a_folder_without_metadata_is_only_reused_when_it_is_an_empty_own_table() {
    // N1 (E16): five cases, none may delete anything.
    // 1. Own filled index whose metadata file is gone.
    let dir = test_dir("n1-filled");
    let index = LanceIndex::open(&dir, space(DIM), 0.5).await.unwrap();
    index.add(&entries()).await.unwrap();
    drop(index);
    std::fs::remove_file(dir.join(META_FILE)).unwrap();
    assert_refused_untouched(&dir).await;
    remove(dir);

    // 2. A foreign LanceDB table called `chunks` with other columns and three rows.
    let dir = test_dir("n1-foreign-table");
    std::fs::create_dir_all(&dir).unwrap();
    {
        use lancedb::arrow::arrow_array::{Int32Array, RecordBatch};
        use lancedb::arrow::arrow_schema::{DataType, Field, Schema};
        let schema =
            std::sync::Arc::new(Schema::new(vec![Field::new("n", DataType::Int32, false)]));
        let batch = RecordBatch::try_new(
            schema,
            vec![std::sync::Arc::new(Int32Array::from(vec![1, 2, 3]))],
        )
        .unwrap();
        let db = lancedb::connect(&dir.to_string_lossy())
            .execute()
            .await
            .unwrap();
        db.create_table("chunks", batch).execute().await.unwrap();
    }
    assert_refused_untouched(&dir).await;
    remove(dir);

    // 3. A folder `chunks.lance` with a foreign file.
    let dir = test_dir("n1-foreign-folder");
    std::fs::create_dir_all(dir.join("chunks.lance")).unwrap();
    std::fs::write(dir.join("chunks.lance").join("notes.txt"), "foreign").unwrap();
    assert_refused_untouched(&dir).await;
    remove(dir);

    // 4. A file called `chunks.lance`.
    let dir = test_dir("n1-file");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("chunks.lance"), "foreign").unwrap();
    assert_refused_untouched(&dir).await;

    // 5. The same with another foreign file next to it.
    std::fs::write(dir.join("other.txt"), "foreign").unwrap();
    assert_refused_untouched(&dir).await;
    remove(dir);

    // A damaged own table without metadata is refused as well.
    let dir = test_dir("n1-damaged");
    drop(LanceIndex::open(&dir, space(DIM), 0.5).await.unwrap());
    std::fs::remove_file(dir.join(META_FILE)).unwrap();
    for f in walk(&dir.join("chunks.lance")) {
        std::fs::write(f, b"garbage").unwrap();
    }
    assert_refused_untouched(&dir).await;
    remove(dir);
}

/// Adds `n` table versions without going through the index (deletes that match
/// nothing), as a crashed or interrupted earlier session could leave them.
async fn add_versions(dir: &Path, n: usize) {
    let db = lancedb::connect(&dir.to_string_lossy())
        .execute()
        .await
        .unwrap();
    let table = db.open_table("chunks").execute().await.unwrap();
    for _ in 0..n {
        table.delete("id = 'does-not-exist'").await.unwrap();
    }
}

#[tokio::test]
async fn opening_runs_the_maintenance_that_is_due() {
    // N2: more than MAINTENANCE_EVERY versions are cleaned up when the index opens.
    let dir = test_dir("n2");
    let index = LanceIndex::open(&dir, space(DIM), 0.5).await.unwrap();
    index.add(&entries()).await.unwrap();
    index.optimize().await.unwrap();
    let reference = snapshot(&index).await;
    drop(index);
    add_versions(&dir, MAINTENANCE_EVERY as usize + 5).await;
    let index = LanceIndex::open(&dir, space(DIM), 0.5).await.unwrap();
    assert_eq!(index.version_count().await.unwrap(), 1);
    assert_eq!(snapshot(&index).await, reference);
    drop(index);
    remove(dir);
}

#[tokio::test]
async fn deleting_an_unknown_document_counts_as_a_change() {
    // N3: such a delete adds a version in LanceDB, so it may trigger the maintenance.
    let dir = test_dir("n3");
    let index = LanceIndex::open(&dir, space(DIM), 0.5).await.unwrap();
    index.add(&entries()).await.unwrap();
    let mut most = 0;
    for _ in 0..50 {
        index
            .delete_document(&DocumentId("unknown".into()))
            .await
            .unwrap();
        most = most.max(index.version_count().await.unwrap());
    }
    assert!(most <= MAINTENANCE_EVERY as usize, "{most} versions");
    drop(index);
    remove(dir);
}

#[tokio::test]
async fn verbatim_unc_paths_and_files_are_rejected() {
    // K1, K3
    for path in [
        "\\\\?\\UNC\\server\\share\\index",
        "\\\\?\\unc\\server\\share\\index",
    ] {
        let r = LanceIndex::open(path, space(DIM), 0.5).await;
        assert!(
            matches!(r, Err(Error::InvalidInput(_))),
            "{path}: {:?}",
            r.err()
        );
    }
    let dir = test_dir("k3");
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("a-file");
    std::fs::write(&file, "x").unwrap();
    match LanceIndex::open(&file, space(DIM), 0.5).await {
        Err(Error::Index(m)) => assert!(m.contains(&file.display().to_string()), "{m}"),
        other => panic!("expected Index error, got {:?}", other.err()),
    }
    remove(dir);
}
