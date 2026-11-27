//! Helpers shared by the integration tests: a model folder with stand-in files, the
//! control and records of the stand-in server, requests and stream collection.

#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use futures::StreamExt;
use kb_core::{
    AnswerLevel, AnswerPhase, CancellationToken, Error, GenerationEvent, GenerationRequest,
    GenerationStream, Generator, HardwareTier, Message, ModelManifest, PhaseProgress, Role,
};
use kb_llm::{GeneratorConfig, LlamaGenerator};
use serde_json::Value;

/// Context size used with the stand-in.
pub const CONTEXT: u32 = 5120;

/// The manifest of the repository.
pub fn manifest() -> ModelManifest {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../models.toml");
    let text = fs::read_to_string(path).expect("models.toml");
    ModelManifest::from_toml_str(&text).expect("models.toml parses")
}

/// The stand-in server built with this package.
pub fn fake_server() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_kb-llm-fake-server"))
}

/// Milliseconds since the Unix epoch, the clock of the records.
pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// A temporary model folder with a small file for every answer model file of the
/// manifest (all tiers), the control file and the records of the stand-in.
pub struct Setup {
    pub dir: tempfile::TempDir,
    pub model_dir: PathBuf,
}

impl Setup {
    pub fn new() -> Self {
        let dir = tempfile::Builder::new()
            .prefix("kb-llm-test-")
            .tempdir()
            .expect("temp dir");
        let model_dir = dir.path().join("models");
        for entry in manifest().models.iter().filter(|m| m.level.is_some()) {
            for file in &entry.files {
                let path = file_path(&model_dir, &entry.id, &file.path);
                fs::create_dir_all(path.parent().unwrap()).unwrap();
                fs::write(&path, b"stand-in model").unwrap();
            }
        }
        fs::create_dir_all(model_dir.join("fake-records")).unwrap();
        let setup = Self { dir, model_dir };
        setup.control(serde_json::json!({}));
        setup
    }

    /// Path of a model file of `level` on `tier` (index into the entry's files).
    pub fn level_files(&self, tier: HardwareTier, level: AnswerLevel) -> Vec<PathBuf> {
        let entry = manifest().answer_model(tier, level).expect("offered");
        entry
            .files
            .iter()
            .map(|f| file_path(&self.model_dir, &entry.id, &f.path))
            .collect()
    }

    /// Writes the control file of the stand-in.
    pub fn control(&self, value: Value) {
        fs::write(self.model_dir.join("fake-control.json"), value.to_string()).unwrap();
    }

    pub fn config(&self, tier: HardwareTier) -> GeneratorConfig {
        GeneratorConfig::new(manifest(), &self.model_dir, tier, CONTEXT, fake_server(), 4)
    }

    pub fn generator(&self, tier: HardwareTier) -> LlamaGenerator {
        LlamaGenerator::new(self.config(tier)).expect("generator")
    }

    fn records(&self, name: &str) -> Vec<Value> {
        fs::read_to_string(self.model_dir.join("fake-records").join(name))
            .unwrap_or_default()
            .lines()
            .filter_map(|l| serde_json::from_str(l).ok())
            .collect()
    }

    /// Every start of the stand-in: `pid`, `args`, `llama_env`, `time_ms`.
    pub fn starts(&self) -> Vec<Value> {
        self.records("starts.jsonl")
    }

    /// Every request to `/tokenize` and `/v1/chat/completions`.
    pub fn requests(&self) -> Vec<Value> {
        self.records("requests.jsonl")
    }

    /// Requests to one path.
    pub fn requests_to(&self, path: &str) -> Vec<Value> {
        self.requests()
            .into_iter()
            .filter(|r| r["path"] == path)
            .collect()
    }

    /// Closed connections, crashes and exits seen by the stand-in.
    pub fn events(&self) -> Vec<Value> {
        self.records("events.jsonl")
    }

    /// Starts of an instance while another one was alive (must stay empty).
    pub fn overlaps(&self) -> Vec<Value> {
        self.records("overlaps.jsonl")
    }

    /// Process ids of all starts.
    pub fn pids(&self) -> Vec<u32> {
        self.starts()
            .iter()
            .filter_map(|s| s["pid"].as_u64())
            .map(|p| p as u32)
            .collect()
    }

    /// Number of started stand-ins that still run.
    pub fn alive(&self) -> usize {
        let running = running_pids();
        self.pids().iter().filter(|p| running.contains(p)).count()
    }

    /// Waits up to `limit` until no stand-in of this setup runs; the time it took.
    pub fn wait_none_alive(&self, limit: Duration) -> Option<Duration> {
        let start = Instant::now();
        while start.elapsed() < limit {
            if self.alive() == 0 {
                return Some(start.elapsed());
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        None
    }

    /// Waits until a closed connection is recorded; its time in ms since the epoch.
    pub fn wait_closed(&self, after_ms: u64, limit: Duration) -> Option<u64> {
        let start = Instant::now();
        while start.elapsed() < limit {
            let closed = self
                .events()
                .into_iter()
                .filter(|e| e["event"] == "closed")
                .filter_map(|e| e["time_ms"].as_u64())
                .find(|t| *t + 5 >= after_ms);
            if closed.is_some() {
                return closed;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        None
    }

    /// Ends the running stand-in at once (`POST /fake-exit`), like a crash between calls.
    pub fn kill_running(&self) {
        let start = self.starts().pop().expect("a start");
        let args: Vec<String> = serde_json::from_value(start["args"].clone()).unwrap();
        let port = args[args.iter().position(|a| a == "--port").unwrap() + 1].clone();
        use std::io::Write;
        if let Ok(mut stream) =
            std::net::TcpStream::connect(("127.0.0.1", port.parse::<u16>().unwrap()))
        {
            let _ = stream
                .write_all(b"POST /fake-exit HTTP/1.1\r\nHost: x\r\nContent-Length: 0\r\n\r\n");
        }
        let pid = start["pid"].as_u64().unwrap() as u32;
        let begin = Instant::now();
        while running_pids().contains(&pid) && begin.elapsed() < Duration::from_secs(5) {
            std::thread::sleep(Duration::from_millis(20));
        }
    }
}

impl Setup {
    /// Process ids of every stand-in started in this folder, including a foreign one
    /// started on the same port (records in `foreign/fake-records`).
    fn all_pids(&self) -> Vec<u32> {
        let mut pids = Vec::new();
        for records in [
            self.model_dir.join("fake-records"),
            self.model_dir.join("foreign").join("fake-records"),
        ] {
            let text = fs::read_to_string(records.join("starts.jsonl")).unwrap_or_default();
            pids.extend(
                text.lines()
                    .filter_map(|l| serde_json::from_str::<Value>(l).ok())
                    .filter_map(|v| v["pid"].as_u64())
                    .map(|p| p as u32),
            );
        }
        pids
    }
}

/// The folder is removed only after every stand-in of this setup has ended: a running
/// stand-in holds its lock file open, and the folder could not be removed (F46).
impl Drop for Setup {
    fn drop(&mut self) {
        let pids = self.all_pids();
        let begin = Instant::now();
        while !pids.is_empty() && begin.elapsed() < Duration::from_secs(20) {
            let running = running_pids();
            if !pids.iter().any(|p| running.contains(p)) {
                break;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        // Windows releases the files of an ended process shortly after; try for a while.
        for _ in 0..50 {
            if fs::remove_dir_all(self.dir.path()).is_ok() || !self.dir.path().exists() {
                break;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
    }
}

fn file_path(model_dir: &Path, id: &str, relative: &str) -> PathBuf {
    relative
        .split('/')
        .fold(model_dir.join(id), |p, part| p.join(part))
}

/// Process ids of all running processes (from `tasklist`).
pub fn running_pids() -> Vec<u32> {
    let output = Command::new("tasklist")
        .args(["/FO", "CSV", "/NH"])
        .output()
        .expect("tasklist");
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| line.split("\",\"").nth(1))
        .filter_map(|pid| pid.trim_matches('"').parse().ok())
        .collect()
}

/// Whether the process with `pid` runs.
pub fn pid_alive(pid: u32) -> bool {
    running_pids().contains(&pid)
}

pub fn user(text: &str) -> Message {
    Message {
        role: Role::User,
        content: text.into(),
    }
}

pub fn request(level: AnswerLevel, text: &str) -> GenerationRequest {
    GenerationRequest {
        level,
        messages: vec![
            Message {
                role: Role::System,
                content: "Antworte kurz.".into(),
            },
            user(text),
        ],
        max_tokens: Some(64),
        temperature: Some(0.0),
    }
}

/// All items of a stream: the events and the error that ended it, if any.
pub async fn collect(mut stream: GenerationStream) -> (Vec<GenerationEvent>, Option<Error>) {
    let mut events = Vec::new();
    while let Some(item) = stream.next().await {
        match item {
            Ok(event) => events.push(event),
            Err(error) => {
                assert!(stream.next().await.is_none(), "an error is the last item");
                return (events, Some(error));
            }
        }
    }
    (events, None)
}

/// Runs a generation to its end.
pub async fn run(
    generator: &dyn Generator,
    request: GenerationRequest,
) -> (Vec<GenerationEvent>, Option<Error>) {
    let stream = generator
        .generate(request, CancellationToken::new())
        .await
        .expect("stream");
    collect(stream).await
}

/// The text of the `Text` events, joined.
pub fn text(events: &[GenerationEvent]) -> String {
    events
        .iter()
        .filter_map(|e| match e {
            GenerationEvent::Text(t) => Some(t.as_str()),
            _ => None,
        })
        .collect()
}

pub fn phases(events: &[GenerationEvent]) -> Vec<AnswerPhase> {
    events
        .iter()
        .filter_map(|e| match e {
            GenerationEvent::Phase { phase, .. } => Some(*phase),
            _ => None,
        })
        .collect()
}

pub fn progress(events: &[GenerationEvent]) -> Vec<Option<PhaseProgress>> {
    events
        .iter()
        .filter_map(|e| match e {
            GenerationEvent::Phase {
                phase: AnswerPhase::ReadingSources,
                progress,
            } => Some(*progress),
            _ => None,
        })
        .collect()
}

pub fn starts_with_starting_model(events: &[GenerationEvent]) -> bool {
    matches!(
        events.first(),
        Some(GenerationEvent::Phase {
            phase: AnswerPhase::StartingModel,
            ..
        })
    )
}

pub fn count_phase(events: &[GenerationEvent], phase: AnswerPhase) -> usize {
    phases(events).iter().filter(|p| **p == phase).count()
}

/// Asserts a valid sequence (F18) with the checker of `kb-core`.
pub fn assert_valid(events: &[GenerationEvent]) {
    if let Some(violation) = kb_core::mock::generation_order_violation(events) {
        panic!("invalid sequence: {violation}: {events:?}");
    }
}

/// Value of a switch in the recorded arguments of a start.
pub fn arg<'a>(start: &'a Value, name: &str) -> Option<&'a str> {
    let args = start["args"].as_array()?;
    let index = args.iter().position(|a| a == name)?;
    args.get(index + 1)?.as_str()
}

pub fn has_arg(start: &Value, name: &str) -> bool {
    start["args"]
        .as_array()
        .is_some_and(|args| args.iter().any(|a| a == name))
}

/// Runs the test `name` of the current test binary in a child process with `env`; the
/// child test does its work only if `KB_LLM_TEST_CHILD` is set.
pub fn child_test(name: &str, env: &[(&str, &str)]) -> Command {
    let mut command = Command::new(std::env::current_exe().unwrap());
    command
        .args(["--exact", name, "--nocapture", "--test-threads=1"])
        .env("KB_LLM_TEST_CHILD", "1");
    for (key, value) in env {
        command.env(key, value);
    }
    command
}

/// Whether this process runs as the child of [`child_test`].
pub fn is_child() -> bool {
    std::env::var_os("KB_LLM_TEST_CHILD").is_some()
}
