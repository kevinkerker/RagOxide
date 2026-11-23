//! The parser: one worker process per file (ADR 0011), models from the model directory.

use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;

use kb_core::{
    CancellationToken, Document, DocumentFormat, Error, HardwareTier, ModelManifest, ModelRole,
    Parser, Result, SourceFile, async_trait,
};
use tokio::io::AsyncReadExt;
use tokio::process::Command;

use crate::convert::{Converted, Failure, PageOrigin};
use crate::worker::Reply;

/// Page workers and threads per worker of docling.rs (F-A1): set explicitly, measured in
/// the parser test (`docs/analyse/docling-rs.md`).
const PDF_WORKERS: &str = "2";
/// Intra-op threads per page worker (F-A1).
const PDF_THREADS: &str = "2";

/// The four parser roles of `models.toml` (`ModelRole`).
const PARSER_ROLES: [ModelRole; 4] = [
    ModelRole::Layout,
    ModelRole::TableStructure,
    ModelRole::OcrDetection,
    ModelRole::OcrRecognition,
];

/// Most bytes of the worker's error output kept for an error message.
const STDERR_TAIL: usize = 2000;

/// ADR 0134: within a process at most one call for PDF or images works at a time; the
/// others wait in the order they came (the semaphore of tokio is fair).
static PAGED_CALLS: tokio::sync::Semaphore = tokio::sync::Semaphore::const_new(1);

/// Numbers the folders of calls, unique within the process.
static CALLS: AtomicU64 = AtomicU64::new(0);

/// Prefix of the working folder of a call (`kb-parse-<process id>-<n>`).
const CALL_PREFIX: &str = "kb-parse-";

/// Marker file in the working folder of a call, held open without sharing while the call
/// runs. A folder with this file that can be opened exclusively belongs to the package
/// and to no running call.
const CALL_LOCK: &str = "kb-parse.lock";

/// File name of the worker program on this platform (`kb-parse-worker.exe` on Windows).
pub fn worker_file_name() -> String {
    format!("kb-parse-worker{}", std::env::consts::EXE_SUFFIX)
}

/// The worker program next to the running executable, where the installer puts it.
pub fn worker_next_to_executable() -> std::io::Result<PathBuf> {
    let exe = std::env::current_exe()?;
    let dir = exe
        .parent()
        .ok_or_else(|| std::io::Error::other("the executable has no folder"))?;
    Ok(dir.join(worker_file_name()))
}

/// Everything the parser needs, given when it is created.
#[derive(Debug, Clone)]
pub struct ParserConfig {
    /// The shared model manifest (`models.toml`); the parser takes the entries of the
    /// four parser roles ([`ModelRole::Layout`], [`ModelRole::TableStructure`],
    /// [`ModelRole::OcrDetection`], [`ModelRole::OcrRecognition`]) with
    /// [`ModelManifest::select`].
    pub manifest: ModelManifest,
    /// Hardware tier for [`ModelManifest::select`].
    pub tier: HardwareTier,
    /// Model directory: a model file lies at `<model_dir>/<entry id>/<file path>`.
    pub model_dir: PathBuf,
    /// The worker program (see [`worker_next_to_executable`]).
    pub worker: PathBuf,
    /// Folder for the working folders of calls, removed after each call. Defaults to the
    /// system's temporary folder.
    pub work_dir: PathBuf,
}

impl ParserConfig {
    /// Configuration with the system's temporary folder as `work_dir`.
    pub fn new(
        manifest: ModelManifest,
        tier: HardwareTier,
        model_dir: PathBuf,
        worker: PathBuf,
    ) -> Self {
        Self {
            manifest,
            tier,
            model_dir,
            worker,
            work_dir: std::env::temp_dir(),
        }
    }
}

/// A parse result together with what docling.rs produced for it, for checks.
#[derive(Debug, Clone)]
pub struct TracedDocument {
    /// The document, as [`Parser::parse`] returns it.
    pub document: Document,
    /// The docling JSON of the same conversion (`export_to_json`).
    pub docling: serde_json::Value,
    /// For every element, the docling item it comes from (`#/texts/3`); a title row
    /// taken out of a table names that table.
    pub sources: Vec<String>,
    /// For PDF, what the conversion report says about each page's text.
    pub origins: Vec<PageOrigin>,
}

/// [`Parser`] over docling.rs, running each conversion in its own worker process.
#[derive(Debug)]
pub struct DoclingParser {
    config: ParserConfig,
    /// Set once the folders left behind by earlier runs have been removed.
    swept: AtomicBool,
}

impl DoclingParser {
    /// Creates the parser. Nothing is checked yet: missing models show at the first PDF
    /// or image, a missing worker program at the first call.
    pub fn new(config: ParserConfig) -> Self {
        Self {
            config,
            swept: AtomicBool::new(false),
        }
    }

    /// Parses like [`Parser::parse`] and also returns the docling JSON of the conversion
    /// and the source item of every element.
    pub async fn parse_traced(
        &self,
        source: &SourceFile,
        cancel: CancellationToken,
    ) -> Result<TracedDocument> {
        let (format, converted) = self.run(source, cancel, true).await?;
        let docling = converted.docling.clone().unwrap_or_default();
        let sources = converted.sources.clone();
        let origins = converted.origins.clone();
        Ok(TracedDocument {
            document: document(source, format, converted),
            docling,
            sources,
            origins,
        })
    }

    /// The model files of the four parser roles: role, file in the model directory and
    /// its path relative to the entry. `Error::Model` names a role without an entry or
    /// a file that is missing.
    fn model_files(&self) -> Result<Vec<(ModelRole, PathBuf, String)>> {
        let mut files = Vec::new();
        for role in PARSER_ROLES {
            let entry = self
                .config
                .manifest
                .select(role, self.config.tier)
                .ok_or_else(|| Error::Model(format!("no parser model for role {role:?}")))?;
            for file in &entry.files {
                let mut path = self.config.model_dir.join(&entry.id);
                path.extend(file.path.split('/'));
                if !path.is_file() {
                    return Err(Error::Model(format!(
                        "parser model file missing for role {role:?}: {}",
                        path.display()
                    )));
                }
                files.push((role, path, file.path.clone()));
            }
        }
        Ok(files)
    }

    async fn run(
        &self,
        source: &SourceFile,
        cancel: CancellationToken,
        trace: bool,
    ) -> Result<(DocumentFormat, Converted)> {
        if cancel.is_cancelled() {
            return Err(Error::Cancelled);
        }
        let format = DocumentFormat::from_path(&source.path)
            .ok_or_else(|| Error::UnsupportedFormat(source.path.display().to_string()))?;
        // Unreadable (missing, locked) is an I/O error, so the ingestor tries again.
        tokio::fs::File::open(&source.path).await?;
        let paged = matches!(format, DocumentFormat::Pdf | DocumentFormat::Image);
        let models = if paged {
            self.model_files()?
        } else {
            Vec::new()
        };
        // One PDF or image call at a time; a cancel ends the wait at once, and waiting
        // does not block the executor.
        let _turn = if paged {
            tokio::select! {
                permit = PAGED_CALLS.acquire() => {
                    Some(permit.map_err(|e| Error::Io(std::io::Error::other(e)))?)
                }
                () = cancel.cancelled() => return Err(Error::Cancelled),
            }
        } else {
            None
        };
        if !self.swept.swap(true, Ordering::Relaxed) {
            let work_dir = self.config.work_dir.clone();
            let _ = tokio::task::spawn_blocking(move || sweep(&work_dir)).await;
        }
        let call = CallDir::create(self.config.work_dir.join(format!(
            "{CALL_PREFIX}{}-{}",
            std::process::id(),
            CALLS.fetch_add(1, Ordering::Relaxed)
        )))?;
        // If the caller drops this future, `call` is dropped with it and removes the
        // folder; otherwise it is removed here, waiting for a killed worker's files.
        let result = self.run_in(&call.path, source, models, cancel, trace).await;
        call.remove().await;
        result.map(|converted| (format, converted))
    }

    async fn run_in(
        &self,
        call: &Path,
        source: &SourceFile,
        models: Vec<(ModelRole, PathBuf, String)>,
        cancel: CancellationToken,
        trace: bool,
    ) -> Result<Converted> {
        let model_dir = call.join("models");
        let temp = call.join("tmp");
        let reply_path = call.join("reply.json");
        {
            let (model_dir, temp) = (model_dir.clone(), temp.clone());
            tokio::task::spawn_blocking(move || stage(&model_dir, &temp, &models))
                .await
                .map_err(|e| Error::Io(std::io::Error::other(e)))??;
        }
        let mut command = Command::new(&self.config.worker);
        command
            .arg(&source.path)
            .arg(&reply_path)
            .current_dir(call)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        if trace {
            command.arg("--trace");
        }
        // Settings of docling.rs from the caller's environment must not change the
        // result; the worker gets exactly these.
        for (key, _) in std::env::vars_os() {
            if key
                .to_string_lossy()
                .to_ascii_uppercase()
                .starts_with("DOCLING")
            {
                command.env_remove(key);
            }
        }
        command
            .env("DOCLING_RS_PDF_WORKERS", PDF_WORKERS)
            .env("DOCLING_RS_PDF_INTRA", PDF_THREADS)
            .env("DOCLING_RS_MODELS_DIR", &model_dir)
            .env("TEMP", &temp)
            .env("TMP", &temp);
        #[cfg(windows)]
        {
            // CREATE_NO_WINDOW: no console window when the app has none.
            command.creation_flags(0x0800_0000);
        }
        let mut child = command.spawn().map_err(|e| {
            Error::Io(std::io::Error::new(
                e.kind(),
                format!("cannot start worker {}: {e}", self.config.worker.display()),
            ))
        })?;
        let mut stdout = child.stdout.take();
        let mut stderr = child.stderr.take();
        let outcome = {
            let io = async {
                let mut out = Vec::new();
                let mut err = Vec::new();
                let (_, _, status) = tokio::join!(
                    async {
                        if let Some(s) = stdout.as_mut() {
                            let _ = s.read_to_end(&mut out).await;
                        }
                    },
                    async {
                        if let Some(s) = stderr.as_mut() {
                            let _ = s.read_to_end(&mut err).await;
                        }
                    },
                    child.wait()
                );
                (status, err)
            };
            tokio::select! {
                result = io => Some(result),
                () = cancel.cancelled() => None,
            }
        };
        let Some((status, stderr)) = outcome else {
            let _ = child.kill().await;
            return Err(Error::Cancelled);
        };
        let status = status?;
        let tail = {
            let text = String::from_utf8_lossy(&stderr);
            let start = text.len().saturating_sub(STDERR_TAIL);
            let start = (start..text.len())
                .find(|&i| text.is_char_boundary(i))
                .unwrap_or(text.len());
            text[start..].trim().to_string()
        };
        let reply = tokio::task::spawn_blocking(move || read_reply(&reply_path))
            .await
            .map_err(|e| Error::Io(std::io::Error::other(e)))?;
        match (status.success(), reply) {
            (true, Some(Reply::Ok(converted))) => Ok(*converted),
            (true, Some(Reply::Err(Failure::Io(message)))) => {
                Err(Error::Io(std::io::Error::other(message)))
            }
            (true, Some(Reply::Err(Failure::Parse(message)))) => Err(Error::Parse(message)),
            _ => Err(Error::Parse(format!(
                "worker process for {} ended with {status} without a result: {tail}",
                source.name
            ))),
        }
    }
}

#[async_trait]
impl Parser for DoclingParser {
    async fn parse(&self, source: &SourceFile, cancel: CancellationToken) -> Result<Document> {
        let (format, converted) = self.run(source, cancel, false).await?;
        Ok(document(source, format, converted))
    }
}

fn document(source: &SourceFile, format: DocumentFormat, converted: Converted) -> Document {
    Document {
        source: source.clone(),
        format,
        title: converted.title,
        language: converted.language,
        pages: converted.pages,
        elements: converted.elements,
    }
}

/// Prepares the call's folder: the temporary folder of the worker and the model files at
/// `<models>/<file path>`, the layout docling.rs reads through `DOCLING_RS_MODELS_DIR`.
/// Files are hard links into the model directory; where that fails (another volume) they
/// are copied.
fn stage(models: &Path, temp: &Path, files: &[(ModelRole, PathBuf, String)]) -> Result<()> {
    std::fs::create_dir_all(temp)?;
    std::fs::create_dir_all(models)?;
    for (_, from, relative) in files {
        let mut to = models.to_path_buf();
        to.extend(relative.split('/'));
        if let Some(parent) = to.parent() {
            std::fs::create_dir_all(parent)?;
        }
        if std::fs::hard_link(from, &to).is_err() {
            std::fs::copy(from, &to)?;
        }
    }
    Ok(())
}

fn read_reply(path: &Path) -> Option<Reply> {
    let file = std::fs::File::open(path).ok()?;
    serde_json::from_reader(std::io::BufReader::new(file)).ok()
}

/// Attempts to remove a call's folder: right after a killed worker Windows may still hold
/// its files for a moment, so this tries for up to two seconds.
const REMOVE_ATTEMPTS: u32 = 40;
/// Pause between two attempts.
const REMOVE_PAUSE: Duration = Duration::from_millis(50);

/// The working folder of one call with its marker file held open.
struct CallDir {
    path: PathBuf,
    lock: Option<std::fs::File>,
}

impl CallDir {
    /// Creates the folder and its marker file.
    fn create(path: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&path)?;
        let lock = open_exclusive(&path.join(CALL_LOCK), true)?;
        Ok(Self {
            path,
            lock: Some(lock),
        })
    }

    /// Removes the folder after the call.
    async fn remove(mut self) {
        self.lock = None;
        for _ in 0..REMOVE_ATTEMPTS {
            match tokio::fs::remove_dir_all(&self.path).await {
                Ok(()) => return,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => return,
                Err(_) => tokio::time::sleep(REMOVE_PAUSE).await,
            }
        }
    }
}

impl Drop for CallDir {
    /// The call ended without [`CallDir::remove`] (its future was dropped): remove the
    /// folder on a thread of its own, since the killed worker may still hold files.
    fn drop(&mut self) {
        if self.lock.take().is_none() || !self.path.exists() {
            return;
        }
        let path = self.path.clone();
        std::thread::spawn(move || {
            for _ in 0..REMOVE_ATTEMPTS {
                match std::fs::remove_dir_all(&path) {
                    Ok(()) => return,
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => return,
                    Err(_) => std::thread::sleep(REMOVE_PAUSE),
                }
            }
        });
    }
}

/// Opens (and with `create` creates) a file for writing without sharing it: on Windows no
/// other handle may open it while this one is open.
fn open_exclusive(path: &Path, create: bool) -> std::io::Result<std::fs::File> {
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create(create).truncate(false);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.share_mode(0);
    }
    options.open(path)
}

/// Removes the working folders that earlier runs left behind (a process that ended
/// during a call): folders in `work_dir` named `kb-parse-…` with the marker file of the
/// package that no running call holds open. Only on Windows, where an open marker file
/// cannot be opened a second time; folders without the marker are never touched.
/// Whether `name` is the name of a call's working folder: `kb-parse-<digits>-<digits>`.
fn is_call_name(name: &str) -> bool {
    let digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
    name.strip_prefix(CALL_PREFIX)
        .and_then(|rest| rest.split_once('-'))
        .is_some_and(|(pid, n)| digits(pid) && digits(n))
}

fn sweep(work_dir: &Path) {
    if !cfg!(windows) {
        return;
    }
    let Ok(entries) = std::fs::read_dir(work_dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let ours =
            is_call_name(&entry.file_name().to_string_lossy()) && path.join(CALL_LOCK).is_file();
        if ours && open_exclusive(&path.join(CALL_LOCK), false).is_ok() {
            let _ = std::fs::remove_dir_all(&path);
        }
    }
}
