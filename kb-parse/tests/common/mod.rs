//! Helpers shared by the integration tests: paths, parsers with and without models, a
//! cache of corpus results and text normalisation.

#![allow(dead_code)]

use std::collections::HashMap;
use std::future::Future;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex, OnceLock};

use kb_core::{
    Anchor, BoundingBox, CancellationToken, Document, DocumentId, Element, ElementKind,
    HardwareTier, ModelManifest, ModelRole, Parser, Result, SourceFile, content_hash,
};
use kb_parse::{DoclingParser, ParserConfig, TracedDocument};
use unicode_normalization::UnicodeNormalization;

/// The environment variable naming the copy of the docling.rs model release (ADR 0111).
pub const MODELS_VAR: &str = "RAGOXIDE_TEST_MODELS_DIR";

const MANIFEST: &str = include_str!("../../../models.toml");

/// The four parser roles of `models.toml`.
pub const PARSER_ROLES: [ModelRole; 4] = [
    ModelRole::Layout,
    ModelRole::TableStructure,
    ModelRole::OcrDetection,
    ModelRole::OcrRecognition,
];

pub fn manifest() -> ModelManifest {
    ModelManifest::from_toml_str(MANIFEST).expect("models.toml parses")
}

pub fn package_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn testdata(name: &str) -> PathBuf {
    package_dir().join("testdata").join(name)
}

pub fn corpus(name: &str) -> PathBuf {
    package_dir()
        .join("..")
        .join("corpus")
        .join("documents")
        .join(name)
}

pub fn worker() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_kb-parse-worker"))
}

/// A fresh, empty folder below `<temp>/kb-parse-tests/<process id>`. The folders of
/// earlier test processes are removed when a test process first asks for one (cargo runs
/// the test binaries one after another); `remove_dir_all` removes the junctions of a
/// model folder without touching their targets.
pub fn temp_dir(label: &str) -> PathBuf {
    static ROOT: OnceLock<PathBuf> = OnceLock::new();
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let root = ROOT.get_or_init(|| {
        let base = std::env::temp_dir().join("kb-parse-tests");
        let own = std::process::id().to_string();
        if let Ok(entries) = std::fs::read_dir(&base) {
            for entry in entries.flatten() {
                if entry.file_name().to_string_lossy() != own {
                    let _ = std::fs::remove_dir_all(entry.path());
                }
            }
        }
        base.join(own)
    });
    let dir = root.join(format!("{label}-{}", NEXT.fetch_add(1, Ordering::Relaxed)));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    dir
}

pub fn source(path: &Path) -> SourceFile {
    let path = std::path::absolute(path).expect("absolute path");
    let hash = std::fs::File::open(&path)
        .and_then(content_hash)
        .unwrap_or_else(|_| "0".repeat(64));
    SourceFile::new(DocumentId::new("test"), &path, hash).expect("source file")
}

pub fn config(model_dir: PathBuf) -> ParserConfig {
    let mut config = ParserConfig::new(manifest(), HardwareTier::Supported, model_dir, worker());
    config.work_dir = temp_dir("work");
    config
}

/// A parser whose model directory is empty: enough for DOCX, PPTX, XLSX, HTML, Markdown.
pub fn office_parser() -> DoclingParser {
    DoclingParser::new(config(temp_dir("no-models")))
}

/// Writes past the test harness's output capture, so the line shows in every run.
pub fn notice(text: &str) {
    let _ = writeln!(std::io::stderr(), "{text}");
}

/// The model directory in the layout of the contract (`<dir>/<id>/<path>`), built from
/// the release copy named by `RAGOXIDE_TEST_MODELS_DIR` with one directory junction per
/// parser entry; `None` if the variable is not set (the caller reports the skip).
pub fn model_dir() -> Option<PathBuf> {
    static DIR: OnceLock<Option<PathBuf>> = OnceLock::new();
    let dir = DIR.get_or_init(|| {
        // In the form `cmd` takes in any spelling: `\` as separator, no trailing separator
        // (`cmd` reads an unquoted `/name` as a switch).
        let release = std::path::absolute(PathBuf::from(std::env::var_os(MODELS_VAR)?))
            .expect("absolute path");
        let release = release.to_string_lossy().replace('/', "\\");
        let release = PathBuf::from(release.trim_end_matches('\\'));
        let dir = temp_dir("models");
        let manifest = manifest();
        for role in PARSER_ROLES {
            let entry = manifest
                .select(role, HardwareTier::Supported)
                .expect("parser entry in models.toml");
            let link = dir.join(&entry.id);
            let status = std::process::Command::new("cmd")
                .arg("/C")
                .arg("mklink")
                .arg("/J")
                .arg(&link)
                .arg(&release)
                .stdout(std::process::Stdio::null())
                .status()
                .expect("mklink");
            assert!(status.success(), "mklink /J {}", link.display());
        }
        Some(dir)
    });
    dir.clone()
}

/// A parser with the parser models, or `None` without `RAGOXIDE_TEST_MODELS_DIR`.
pub fn model_parser() -> Option<DoclingParser> {
    model_dir().map(|dir| DoclingParser::new(config(dir)))
}

/// Reports a test that needs the parser models as skipped.
pub fn skipped(test: &str) {
    notice(&format!(
        "{test}: übersprungen, {MODELS_VAR} ist nicht gesetzt"
    ));
}

/// Runs a future on a runtime of its own.
pub fn run<F: Future>(future: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime")
        .block_on(future)
}

/// At most three conversions with models at a time: each worker takes 1 to 3 GB.
fn throttle<T>(work: impl FnOnce() -> T) -> T {
    static SLOTS: OnceLock<(Mutex<usize>, Condvar)> = OnceLock::new();
    let (lock, cvar) = SLOTS.get_or_init(|| (Mutex::new(3), Condvar::new()));
    {
        let mut free = lock.lock().unwrap();
        while *free == 0 {
            free = cvar.wait(free).unwrap();
        }
        *free -= 1;
    }
    let result = work();
    *lock.lock().unwrap() += 1;
    cvar.notify_one();
    result
}

pub fn parse(parser: &DoclingParser, path: &Path) -> Result<Document> {
    let source = source(path);
    throttle(|| run(parser.parse(&source, CancellationToken::new())))
}

pub fn parse_traced(parser: &DoclingParser, path: &Path) -> Result<TracedDocument> {
    let source = source(path);
    throttle(|| run(parser.parse_traced(&source, CancellationToken::new())))
}

/// Two conversions of a corpus or test file: with `parse` and with `parse_traced`.
pub struct Twice {
    pub first: Document,
    pub traced: TracedDocument,
}

/// The two conversions of `path`, made once per test binary.
pub fn twice(parser: &DoclingParser, path: &Path) -> Arc<Twice> {
    type Cache = Mutex<HashMap<PathBuf, Arc<OnceLock<Arc<Twice>>>>>;
    static CACHE: OnceLock<Cache> = OnceLock::new();
    let cell = CACHE
        .get_or_init(Default::default)
        .lock()
        .unwrap()
        .entry(path.to_path_buf())
        .or_default()
        .clone();
    cell.get_or_init(|| {
        let first = parse(parser, path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        let traced =
            parse_traced(parser, path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        Arc::new(Twice { first, traced })
    })
    .clone()
}

/// Normalisation of `corpus/questions.toml`: NFC, soft hyphens removed, `|` as a space,
/// whitespace collapsed.
pub fn norm(text: &str) -> String {
    let text: String = text.nfc().filter(|&c| c != '\u{00AD}').collect();
    text.replace('|', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn is_furniture(e: &Element) -> bool {
    e.kind == ElementKind::Furniture
}

/// The boxes of an element on `page`.
pub fn boxes_on(e: &Element, page: u32) -> Vec<BoundingBox> {
    e.anchors
        .iter()
        .filter_map(|a| match a {
            Anchor::Page { page: p, bbox } if *p == page => Some(*bbox),
            _ => None,
        })
        .collect()
}

pub fn contains(b: &BoundingBox, x: f32, y: f32) -> bool {
    b.left <= x && x <= b.right && b.top <= y && y <= b.bottom
}

/// The expected values of the own test files (`testdata/expected.json`).
pub fn expected() -> serde_json::Value {
    let text = std::fs::read_to_string(testdata("expected.json")).expect("expected.json");
    serde_json::from_str(&text).expect("expected.json parses")
}

/// The corpus files of one format by extension.
pub fn corpus_files(extensions: &[&str]) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> =
        std::fs::read_dir(package_dir().join("..").join("corpus").join("documents"))
            .expect("corpus")
            .map(|e| e.expect("entry").path())
            .filter(|p| {
                p.extension()
                    .and_then(|e| e.to_str())
                    .is_some_and(|e| extensions.contains(&e))
            })
            .collect();
    files.sort();
    files
}

/// `language` and `pages` of a corpus file from `corpus/manifest.toml`.
pub fn manifest_entry(file: &str) -> (String, Option<u32>) {
    let text = std::fs::read_to_string(
        package_dir()
            .join("..")
            .join("corpus")
            .join("manifest.toml"),
    )
    .expect("corpus manifest");
    let value: toml::Table = toml::from_str(&text).expect("corpus manifest parses");
    let doc = value["documents"]
        .as_array()
        .expect("documents")
        .iter()
        .find(|d| d["file"].as_str() == Some(file))
        .unwrap_or_else(|| panic!("{file} not in corpus manifest"));
    (
        doc["language"].as_str().expect("language").to_string(),
        doc.get("pages")
            .and_then(|p| p.as_integer())
            .map(|p| p as u32),
    )
}
