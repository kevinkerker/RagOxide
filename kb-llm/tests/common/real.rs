//! Helpers for runs with the real `llama-server` and the real models (F-F12): the two
//! variables, checksums against `models.toml`, prompts from the German corpus, and the
//! command line, ports and connections of the running server. Shared by the tests with
//! the real server and the measurement program.

#![allow(dead_code)]

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Mutex, OnceLock};

use kb_core::{
    AnswerLevel, GenerationRequest, Generator, HardwareTier, Message, ModelManifest, Role,
};

/// Folder with the unpacked `llama-b12359-bin-win-cpu-x64.zip`.
pub const SERVER_VAR: &str = "RAGOXIDE_TEST_LLAMA_SERVER_DIR";
/// Model folder in the layout `<id>/<path>`.
pub const MODELS_VAR: &str = "RAGOXIDE_TEST_APP_MODELS_DIR";

/// The two folders of F-F12.
#[derive(Debug, Clone)]
pub struct RealEnv {
    pub server_dir: PathBuf,
    pub models: PathBuf,
}

impl RealEnv {
    /// The folders, or `None` with a visible "übersprungen" naming the missing variable.
    pub fn get(test: &str) -> Option<Self> {
        let server_dir = std::env::var_os(SERVER_VAR);
        let models = std::env::var_os(MODELS_VAR);
        match (server_dir, models) {
            (Some(server_dir), Some(models)) => Some(Self {
                server_dir: server_dir.into(),
                models: models.into(),
            }),
            (server_dir, _) => {
                let missing = if server_dir.is_none() {
                    SERVER_VAR
                } else {
                    MODELS_VAR
                };
                eprintln!("{test}: übersprungen, {missing} ist nicht gesetzt");
                None
            }
        }
    }

    pub fn server(&self) -> PathBuf {
        self.server_dir.join("llama-server.exe")
    }
}

fn hash_cache() -> &'static Mutex<HashMap<PathBuf, String>> {
    static CACHE: OnceLock<Mutex<HashMap<PathBuf, String>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// SHA-256 of a file as lowercase hex, computed by `certutil` of Windows (several
/// gigabytes per minute, unlike a hash in a test build without optimisation); each file
/// is hashed once per process.
pub fn sha256(path: &Path) -> std::io::Result<String> {
    if let Some(hash) = hash_cache().lock().unwrap().get(path) {
        return Ok(hash.clone());
    }
    if !path.is_file() {
        return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "no file"));
    }
    let output = Command::new("certutil")
        .arg("-hashfile")
        .arg(path)
        .arg("SHA256")
        .output()?;
    let hash = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| line.replace(' ', "").to_ascii_lowercase())
        .find(|line| line.len() == 64 && line.bytes().all(|b| b.is_ascii_hexdigit()))
        .ok_or_else(|| std::io::Error::other("certutil printed no SHA-256"))?;
    hash_cache()
        .lock()
        .unwrap()
        .insert(path.to_path_buf(), hash.clone());
    Ok(hash)
}

/// Checks one file against its expected SHA-256.
pub fn verify_file(path: &Path, expected: &str) -> Result<(), String> {
    let actual = sha256(path).map_err(|e| format!("{} cannot be read: {e}", path.display()))?;
    if actual == expected {
        Ok(())
    } else {
        Err(format!(
            "{} does not match models.toml: SHA-256 {actual}, expected {expected}",
            path.display()
        ))
    }
}

/// Checks the files of the server release that `models.toml` lists inside the archive.
pub fn verify_server(manifest: &ModelManifest, env: &RealEnv) -> Result<(), String> {
    let runtime = manifest
        .runtimes
        .iter()
        .find(|r| r.id == "llama-server")
        .ok_or("no runtime llama-server in models.toml")?;
    let mut checked = 0;
    for file in runtime.files.iter().filter(|f| f.archive.is_some()) {
        let expected = file
            .sha256
            .as_deref()
            .ok_or("runtime file without checksum")?;
        verify_file(&env.server_dir.join(&file.path), expected)?;
        checked += 1;
    }
    if checked < 3 {
        return Err(format!("only {checked} server files listed in models.toml"));
    }
    Ok(())
}

/// Whether the files of `level` on `tier` are present (`Ok(false)` if not all are), and
/// if so, whether they match `models.toml`.
pub fn verify_level(
    manifest: &ModelManifest,
    env: &RealEnv,
    tier: HardwareTier,
    level: AnswerLevel,
) -> Result<bool, String> {
    let entry = manifest
        .answer_model(tier, level)
        .ok_or("level not offered")?;
    let paths: Vec<(PathBuf, &str)> = entry
        .files
        .iter()
        .map(|f| {
            let path = f
                .path
                .split('/')
                .fold(env.models.join(&entry.id), |p, part| p.join(part));
            (path, f.sha256.as_deref().unwrap_or(""))
        })
        .collect();
    if !paths.iter().all(|(p, _)| p.is_file()) {
        return Ok(false);
    }
    for (path, expected) in paths {
        verify_file(&path, expected)?;
    }
    Ok(true)
}

/// The manifest of the repository.
pub fn repo_manifest() -> ModelManifest {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../models.toml");
    ModelManifest::from_toml_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

/// The manifest with the drafter of every answer model switched off (for load times
/// without drafter on `Supported`).
pub fn without_drafters(manifest: &ModelManifest) -> ModelManifest {
    let mut manifest = manifest.clone();
    for model in manifest.models.iter_mut().filter(|m| m.level.is_some()) {
        if let Some(draft) = model.mtp_draft.take() {
            model.files.retain(|f| f.path != draft);
            model.mtp_tiers.clear();
        }
    }
    manifest
}

/// The articles of the Grundgesetz from the corpus, as in the measurement of
/// `docs/analyse/tempomessung.md` ("Prompts").
pub fn articles() -> Vec<String> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../corpus/documents/de-grundgesetz-bundestag-gesetze-77abca2.md");
    let text = std::fs::read_to_string(path).unwrap();
    text.split("\n### ")
        .skip(1)
        .filter(|part| part.starts_with("Art"))
        .map(|part| {
            let mut lines = part.lines();
            let title = lines.next().unwrap_or_default().trim().to_owned();
            let body: Vec<&str> = lines.map(str::trim).filter(|l| !l.is_empty()).collect();
            format!("Grundgesetz, {title}\n{}", body.join("\n"))
        })
        .collect()
}

pub const SYSTEM: &str = "Du bist ein Assistent für eine interne Wissensdatenbank. Beantworte die Frage ausschließlich anhand der unten aufgeführten, nummerierten Quellen. Antworte auf Deutsch, sachlich und vollständig. Belege jede Aussage mit der Nummer der Quelle in eckigen Klammern, zum Beispiel [3]. Wenn die Quellen die Frage nicht beantworten, sage das ausdrücklich und erfinde nichts.";

pub const QUESTION: &str = "Welche Grundrechte schützen die Meinungs- und Pressefreiheit, wo finden sie ihre Schranken, und in welchen Fällen kann man sie verwirken? Antworte ausführlich in mehreren Absätzen.";

/// A user message with as many articles as fit under `target` tokens of the user text
/// (counted with the generator; the chat template adds about 96 tokens).
pub async fn sources_prompt(
    generator: &dyn Generator,
    level: AnswerLevel,
    target: usize,
) -> (GenerationRequest, usize) {
    let articles = articles();
    let build = |n: usize| {
        let sources: Vec<String> = articles[..n]
            .iter()
            .enumerate()
            .map(|(i, a)| format!("[{}] {a}", i + 1))
            .collect();
        format!("Quellen:\n\n{}\n\nFrage: {QUESTION}", sources.join("\n\n"))
    };
    let mut n = 1;
    let mut tokens = generator.count_tokens(level, &build(1)).await.unwrap();
    while n < articles.len() {
        let next = generator.count_tokens(level, &build(n + 1)).await.unwrap();
        if next > target {
            break;
        }
        n += 1;
        tokens = next;
    }
    let request = GenerationRequest {
        level,
        messages: vec![
            Message {
                role: Role::System,
                content: SYSTEM.into(),
            },
            Message {
                role: Role::User,
                content: build(n),
            },
        ],
        max_tokens: Some(256),
        temperature: Some(0.0),
    };
    (request, tokens)
}

/// Port and API key of a running server, from its command line.
pub fn port_and_key(pid: u32) -> (u16, String) {
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            &format!("(Get-CimInstance Win32_Process -Filter \"ProcessId={pid}\").CommandLine"),
        ])
        .output()
        .unwrap();
    let line = String::from_utf8_lossy(&output.stdout).to_string();
    let words: Vec<&str> = line.split_whitespace().collect();
    let after = |name: &str| {
        words
            .iter()
            .position(|w| *w == name)
            .and_then(|i| words.get(i + 1))
            .map(|w| w.trim_matches('"').to_owned())
            .unwrap_or_else(|| panic!("{name} not in the command line"))
    };
    (after("--port").parse().unwrap(), after("--api-key"))
}

/// TCP rows of `netstat -ano` for `pid`: (local, remote, state).
pub fn tcp_of(pid: u32) -> Vec<(String, String, String)> {
    let output = Command::new("netstat")
        .args(["-ano", "-p", "TCP"])
        .output()
        .unwrap();
    let mut rows = Vec::new();
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() == 5 && parts[0] == "TCP" && parts[4] == pid.to_string() {
            rows.push((parts[1].into(), parts[2].into(), parts[3].into()));
        }
    }
    let output = Command::new("netstat")
        .args(["-ano", "-p", "TCPv6"])
        .output()
        .unwrap();
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() == 5 && parts[0] == "TCP" && parts[4] == pid.to_string() {
            rows.push((parts[1].into(), parts[2].into(), parts[3].into()));
        }
    }
    rows
}

/// Whether an address of `netstat` is on this machine only (127.0.0.1 or ::1), or the
/// unspecified remote of a listening socket.
pub fn loopback(address: &str) -> bool {
    address.starts_with("127.0.0.1:") || address.starts_with("[::1]:")
}

/// A plain HTTP client without proxy for direct requests of the tests.
pub fn client() -> reqwest::Client {
    reqwest::Client::builder().no_proxy().build().unwrap()
}
