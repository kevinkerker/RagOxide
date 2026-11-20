//! Check program of the parser (A16): parses one file with [`kb_parse::DoclingParser`]
//! and prints the `Document` as JSON.
//!
//! ```text
//! kb-parse-dump <file> [--trace] [--model-dir <dir>] [--cancel-after <ms>]
//! ```
//!
//! - `--trace` prints `{document, docling, sources, origins}`: also the docling JSON of
//!   the same conversion, the docling item of every element and, for PDF, what the
//!   conversion report says about the text of each page.
//! - `--model-dir` is a model directory in the layout of the contract
//!   (`<dir>/<id>/<path>`). Without it, the release copy named by
//!   `RAGOXIDE_TEST_MODELS_DIR` is linked into that layout in a temporary folder, as the
//!   tests do; without either, PDF and images fail with `Error::Model`.
//! - `--cancel-after` cancels the call after that many milliseconds and prints the time
//!   from the cancel to the return.
//!
//! The duration of the call goes to standard error. The manifest is the workspace's
//! `models.toml` at build time; the worker program is expected next to this program.

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{Duration, Instant};

use kb_core::{
    CancellationToken, DocumentId, HardwareTier, ModelManifest, ModelRole, Parser, SourceFile,
    content_hash,
};
use kb_parse::{DoclingParser, ParserConfig, worker_next_to_executable};

const MANIFEST: &str = include_str!("../../../models.toml");

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("kb-parse-dump: {e}");
            ExitCode::FAILURE
        }
    }
}

fn value_of(args: &[String], flag: &str) -> Option<String> {
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1).cloned())
}

fn run(args: &[String]) -> Result<(), String> {
    let trace = args.iter().any(|a| a == "--trace");
    let model_dir = value_of(args, "--model-dir").map(PathBuf::from);
    let cancel_after = value_of(args, "--cancel-after")
        .map(|v| v.parse::<u64>().map_err(|e| format!("--cancel-after: {e}")))
        .transpose()?;
    let mut skip = false;
    let mut file = None;
    for a in args {
        if skip {
            skip = false;
        } else if a == "--model-dir" || a == "--cancel-after" {
            skip = true;
        } else if a != "--trace" {
            file = Some(PathBuf::from(a));
        }
    }
    let file = file
        .ok_or("usage: kb-parse-dump <file> [--trace] [--model-dir <dir>] [--cancel-after <ms>]")?;
    let file = std::path::absolute(&file).map_err(|e| e.to_string())?;
    let manifest = ModelManifest::from_toml_str(MANIFEST).map_err(|e| e.to_string())?;
    let linked = std::env::temp_dir().join(format!("kb-parse-dump-models-{}", std::process::id()));
    let model_dir = match (model_dir, std::env::var_os("RAGOXIDE_TEST_MODELS_DIR")) {
        (Some(dir), _) => dir,
        (None, Some(release)) => {
            link_release(&manifest, Path::new(&release), &linked)?;
            linked.clone()
        }
        (None, None) => linked.clone(),
    };
    let worker = worker_next_to_executable().map_err(|e| e.to_string())?;
    let parser = DoclingParser::new(ParserConfig::new(
        manifest,
        HardwareTier::Supported,
        model_dir,
        worker,
    ));
    let hash = std::fs::File::open(&file)
        .and_then(content_hash)
        .unwrap_or_default();
    let source =
        SourceFile::new(DocumentId::new("dump"), &file, hash).map_err(|e| e.to_string())?;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| e.to_string())?;
    let cancel = CancellationToken::new();
    if let Some(ms) = cancel_after {
        let token = cancel.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(ms));
            token.cancel();
        });
    }
    let start = Instant::now();
    let output = runtime.block_on(async {
        if trace {
            parser.parse_traced(&source, cancel.clone()).await.map(|t| {
                serde_json::json!({"document": t.document, "docling": t.docling, "sources": t.sources, "origins": t.origins})
            })
        } else {
            parser
                .parse(&source, cancel.clone())
                .await
                .map(|d| serde_json::to_value(d).unwrap_or_default())
        }
    });
    let elapsed = start.elapsed();
    let _ = remove_links(&linked);
    eprintln!("duration: {:.3} s", elapsed.as_secs_f64());
    if let Some(ms) = cancel_after {
        let after_cancel = elapsed.saturating_sub(Duration::from_millis(ms));
        eprintln!("cancel to return: {:.3} s", after_cancel.as_secs_f64());
    }
    let value = output.map_err(|e| format!("{e} ({:?})", e.kind()))?;
    println!(
        "{}",
        serde_json::to_string_pretty(&value).map_err(|e| e.to_string())?
    );
    Ok(())
}

/// Builds the model directory of the contract (`<dir>/<id>/<path>`) from a flat copy of
/// the docling.rs release: one directory junction per parser entry, pointing at the copy.
fn link_release(manifest: &ModelManifest, release: &Path, dir: &Path) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let release = system_path(release)?;
    for role in [
        ModelRole::Layout,
        ModelRole::TableStructure,
        ModelRole::OcrDetection,
        ModelRole::OcrRecognition,
    ] {
        let Some(entry) = manifest.select(role, HardwareTier::Supported) else {
            continue;
        };
        let link = dir.join(&entry.id);
        if link.exists() {
            continue;
        }
        let status = std::process::Command::new("cmd")
            .arg("/C")
            .arg("mklink")
            .arg("/J")
            .arg(system_path(&link)?)
            .arg(&release)
            .stdout(std::process::Stdio::null())
            .status()
            .map_err(|e| e.to_string())?;
        if !status.success() {
            return Err(format!("mklink /J {} failed", link.display()));
        }
    }
    Ok(())
}

/// `path` in the form `cmd` takes in any spelling: absolute, `\` as separator, no trailing
/// separator. `cmd` reads an unquoted `/name` (a path without spaces is not quoted) as a
/// switch.
fn system_path(path: &Path) -> Result<PathBuf, String> {
    let absolute = std::path::absolute(path).map_err(|e| e.to_string())?;
    let text = absolute.to_string_lossy().replace('/', "\\");
    let trimmed = text.trim_end_matches('\\');
    // A drive root keeps its separator (`C:\`).
    Ok(PathBuf::from(if trimmed.ends_with(':') {
        format!("{trimmed}\\")
    } else {
        trimmed.to_string()
    }))
}

/// Removes the junctions (not their targets) and the folder.
fn remove_links(dir: &Path) -> std::io::Result<()> {
    if !dir.exists() {
        return Ok(());
    }
    for entry in std::fs::read_dir(dir)? {
        // A junction is removed like an empty directory; its target stays.
        std::fs::remove_dir(entry?.path())?;
    }
    std::fs::remove_dir(dir)
}
