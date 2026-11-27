//! Stand-in for `llama-server` in the tests of `kb-llm` (F-F11).
//!
//! Takes the command line of `llama-server`, binds `--host`/`--port` and serves the
//! endpoints the package uses, in the formats of `docs/analyse/spike-llama-server-
//! fortschritt.md` and `docs/analyse/tempomessung.md`:
//!
//! - `GET /health`: `503 Loading model` during the load time, then `200 {"status":"ok"}`;
//!   every other endpoint answers 503 while loading.
//! - `POST /tokenize`: one token per whitespace-separated word (plus one with
//!   `add_special: true`), not waiting for the slot.
//! - `POST /v1/chat/completions`: server-sent events with chunked transfer encoding,
//!   `prompt_progress` while "reading", optional `reasoning_content`, the reply as
//!   `content` pieces, `[DONE]`. One slot: a second request waits without headers.
//!   A prompt over the context (`-c`) is `400 exceed_context_size_error`.
//! - The API key (`--api-key`) is required on both POST endpoints (401 otherwise).
//! - `POST /fake-exit` ends the process at once (exit code 7), for tests of a server
//!   that ends between calls; optionally a helper keeps its pipes open a while longer.
//!
//! Control and records live next to the model folder: the model path is
//! `<model_dir>/<id>/<file>`, the control file `<model_dir>/fake-control.json` (read
//! again on every start and request), the records in `<model_dir>/fake-records/`:
//! `starts.jsonl` (arguments, process id, names of `LLAMA_*` variables in the
//! environment), `requests.jsonl` (path, body, whether the key matched), `events.jsonl`
//! (closed connections, crashes) and `overlaps.jsonl` (a second instance started while
//! another one was alive; checked with a lock file held for the life of the process).

use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::Deserialize;
use serde_json::{Value, json};

/// Behaviour of the stand-in, from `fake-control.json`; every field is optional.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
struct Control {
    /// Milliseconds `/health` answers 503 after the start.
    load_ms: u64,
    /// End with this exit code while loading (after `load_ms / 2`), after about 20 KiB
    /// of error output (more than is quoted in an error message).
    exit_during_load: Option<i32>,
    /// Allocate and touch this many MiB at the start, so that ending the process takes a
    /// moment.
    hold_memory_mb: usize,
    /// On `POST /fake-exit`, first start a helper that keeps the pipes of this process
    /// open for a few seconds (the end of the process comes before the end of its pipes).
    hold_pipes_on_exit: bool,
    /// The first `n` starts (counted over the records folder) fail like a taken port.
    reject_starts: usize,
    /// For the first `n` starts another stand-in with another key takes the port, and
    /// this one never answers.
    foreign_on_port_starts: usize,
    /// The reply, split into pieces after each space unless `pieces` is given.
    reply: String,
    /// Explicit content pieces (may be empty strings).
    pieces: Option<Vec<String>>,
    /// Reasoning pieces sent before the content.
    reasoning: Vec<String>,
    /// Send `prompt_progress`.
    progress: bool,
    /// Values of `processed`; default half and all of the prompt.
    progress_steps: Option<Vec<u64>>,
    /// Milliseconds between progress reports (or before the first token without them).
    prompt_delay_ms: u64,
    /// Milliseconds between content pieces.
    piece_delay_ms: u64,
    /// Stop sending without closing: `reading` (after the first progress report) or
    /// `writing` (after the first content piece).
    stall: Option<String>,
    /// End the process (exit code 3) after this many content pieces.
    crash_after: Option<usize>,
    /// Write this many bytes to the error output before the reply.
    stderr_flood: usize,
    /// Split every write of the event stream into pieces of this many bytes.
    split_bytes: Option<usize>,
    /// Break the stream: `not_json`, `missing_fields` or `truncated`.
    invalid: Option<String>,
    /// Keep the slot this many milliseconds after the client closed the connection
    /// (the server computes to the end of its batch, ADR 0101).
    after_close_ms: u64,
}

impl Default for Control {
    fn default() -> Self {
        Self {
            load_ms: 0,
            exit_during_load: None,
            hold_pipes_on_exit: false,
            hold_memory_mb: 0,
            reject_starts: 0,
            foreign_on_port_starts: 0,
            reply: "Hallo Welt, das ist eine Antwort.".into(),
            pieces: None,
            reasoning: Vec::new(),
            progress: true,
            progress_steps: None,
            prompt_delay_ms: 0,
            piece_delay_ms: 0,
            stall: None,
            crash_after: None,
            stderr_flood: 0,
            split_bytes: None,
            invalid: None,
            after_close_ms: 0,
        }
    }
}

struct Fake {
    started: Instant,
    load: Duration,
    key: Option<String>,
    context: u64,
    control_path: PathBuf,
    records: PathBuf,
    slot: Mutex<()>,
    write_lock: Mutex<()>,
}

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

fn read_control(path: &Path) -> Control {
    fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

fn append(path: &Path, value: &Value) {
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(file, "{value}");
    }
}

fn record(fake: &Fake, file: &str, value: Value) {
    let _guard = fake.write_lock.lock();
    append(&fake.records.join(file), &value);
}

/// Opens the lock file without sharing, so a second live instance cannot.
fn open_exclusive(path: &Path) -> io::Result<File> {
    let mut options = OpenOptions::new();
    options.create(true).truncate(false).write(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.share_mode(0);
    }
    options.open(path)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().is_some_and(|a| a == "--hold-pipes") {
        thread::sleep(Duration::from_secs(3));
        return;
    }
    let value = |name: &str| {
        args.iter()
            .position(|a| a == name)
            .and_then(|i| args.get(i + 1))
            .cloned()
    };
    let model = PathBuf::from(value("-m").unwrap_or_default());
    let Some(base) = model.parent().and_then(Path::parent) else {
        eprintln!("fake server: no model path");
        std::process::exit(2);
    };
    let control_path = base.join("fake-control.json");
    let records = base.join("fake-records");
    let _ = fs::create_dir_all(&records);

    // One instance at a time per model folder: a second live instance is recorded.
    let lock_path = records.join("instance.lock");
    let _lock = match open_exclusive(&lock_path) {
        Ok(file) => file,
        Err(_) => {
            append(
                &records.join("overlaps.jsonl"),
                &json!({"pid": std::process::id(), "time_ms": now_ms() as u64}),
            );
            loop {
                thread::sleep(Duration::from_millis(5));
                if let Ok(file) = open_exclusive(&lock_path) {
                    break file;
                }
            }
        }
    };

    let control = read_control(&control_path);
    let starts_path = records.join("starts.jsonl");
    let index = fs::read_to_string(&starts_path)
        .map(|t| t.lines().count())
        .unwrap_or(0);
    let llama_env: Vec<String> = std::env::vars_os()
        .map(|(name, _)| name.to_string_lossy().into_owned())
        .filter(|name| name.to_ascii_uppercase().starts_with("LLAMA_"))
        .collect();
    append(
        &starts_path,
        &json!({
            "pid": std::process::id(),
            "args": args,
            "llama_env": llama_env,
            "time_ms": now_ms() as u64,
        }),
    );
    // Kept until the process ends.
    let _memory: Vec<u8> = vec![1u8; control.hold_memory_mb * 1024 * 1024];
    let host = value("--host").unwrap_or_else(|| "127.0.0.1".into());
    let port: u16 = value("--port").and_then(|p| p.parse().ok()).unwrap_or(0);

    eprintln!("I srv  llama_server: initializing (stand-in)");
    eprintln!("I srv    load_model: loading model '{}'", model.display());
    if index < control.reject_starts {
        eprintln!(
            "E srv         start: couldn't bind HTTP server socket, hostname: {host}, port: {port}"
        );
        eprintln!("E srv  server_run_i: exiting due to HTTP server error");
        std::process::exit(1);
    }
    if index < control.foreign_on_port_starts {
        // Another server takes the port (with its own key and records), like a second
        // `llama-server` that binds a port already in use; this instance stays
        // unreachable until it is stopped.
        let foreign = base.join("foreign").join("other").join("model.bin");
        let _ = std::process::Command::new(std::env::current_exe().unwrap_or_default())
            .args([
                "-m",
                &foreign.to_string_lossy(),
                "--host",
                &host,
                "--port",
                &port.to_string(),
            ])
            .args(["--api-key", "foreign-key", "-c", "4096"])
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn();
        loop {
            thread::sleep(Duration::from_secs(1));
        }
    }
    let listener = match TcpListener::bind((host.as_str(), port)) {
        Ok(listener) => listener,
        Err(_) => {
            eprintln!(
                "E srv         start: couldn't bind HTTP server socket, hostname: {host}, port: {port}"
            );
            std::process::exit(1);
        }
    };
    if let Some(code) = control.exit_during_load {
        thread::sleep(Duration::from_millis(control.load_ms / 2));
        for line in 0..200 {
            eprintln!("W load: earlier output line {line:03} {}", "-".repeat(80));
        }
        eprintln!("E llama_model_load_from_file_impl: failed to load model");
        eprintln!(
            "E srv    load_model: failed to load model, '{}'",
            model.display()
        );
        std::process::exit(code);
    }

    let fake = Arc::new(Fake {
        started: Instant::now(),
        load: Duration::from_millis(control.load_ms),
        key: value("--api-key"),
        context: value("-c").and_then(|c| c.parse().ok()).unwrap_or(4096),
        control_path,
        records,
        slot: Mutex::new(()),
        write_lock: Mutex::new(()),
    });
    for connection in listener.incoming() {
        let Ok(connection) = connection else { continue };
        let fake = fake.clone();
        thread::spawn(move || {
            let _ = handle(&fake, connection);
        });
    }
}

struct Request {
    method: String,
    path: String,
    headers: BTreeMap<String, String>,
    body: Vec<u8>,
}

fn read_request(stream: &TcpStream) -> io::Result<Request> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut line = String::new();
    reader.read_line(&mut line)?;
    let mut parts = line.split_whitespace();
    let method = parts.next().unwrap_or_default().to_owned();
    let path = parts.next().unwrap_or_default().to_owned();
    let mut headers = BTreeMap::new();
    loop {
        let mut header = String::new();
        if reader.read_line(&mut header)? == 0 {
            break;
        }
        let header = header.trim_end();
        if header.is_empty() {
            break;
        }
        if let Some((name, value)) = header.split_once(':') {
            headers.insert(name.trim().to_ascii_lowercase(), value.trim().to_owned());
        }
    }
    let length: usize = headers
        .get("content-length")
        .and_then(|l| l.parse().ok())
        .unwrap_or(0);
    let mut body = vec![0; length];
    reader.read_exact(&mut body)?;
    Ok(Request {
        method,
        path,
        headers,
        body,
    })
}

fn respond(mut stream: &TcpStream, status: &str, body: &Value) -> io::Result<()> {
    let body = body.to_string();
    write!(
        stream,
        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )?;
    stream.flush()
}

fn handle(fake: &Fake, stream: TcpStream) -> io::Result<()> {
    let _ = stream.set_nodelay(true);
    let request = read_request(&stream)?;
    let loading = fake.started.elapsed() < fake.load;
    let unavailable =
        json!({"error": {"message": "Loading model", "type": "unavailable_error", "code": 503}});
    match (request.method.as_str(), request.path.as_str()) {
        ("GET", "/health") if loading => respond(&stream, "503 Service Unavailable", &unavailable),
        ("GET", "/health") => respond(&stream, "200 OK", &json!({"status": "ok"})),
        ("POST", "/fake-exit") => {
            if read_control(&fake.control_path).hold_pipes_on_exit {
                // Inherits stdout and stderr and keeps them open after this process ends.
                let _ = std::process::Command::new(std::env::current_exe().unwrap_or_default())
                    .arg("--hold-pipes")
                    .stdin(std::process::Stdio::null())
                    .spawn();
            }
            record(
                fake,
                "events.jsonl",
                json!({"event": "exit", "time_ms": now_ms() as u64}),
            );
            std::process::exit(7);
        }
        _ if loading => respond(&stream, "503 Service Unavailable", &unavailable),
        ("POST", path @ ("/tokenize" | "/v1/chat/completions")) => {
            let expected = fake.key.as_ref().map(|k| format!("Bearer {k}"));
            let authorised =
                expected.is_none() || request.headers.get("authorization") == expected.as_ref();
            let body: Value = serde_json::from_slice(&request.body).unwrap_or(Value::Null);
            record(
                fake,
                "requests.jsonl",
                json!({"path": path, "body": body, "authorised": authorised, "time_ms": now_ms() as u64}),
            );
            if !authorised {
                return respond(
                    &stream,
                    "401 Unauthorized",
                    &json!({"error": {"message": "Invalid API Key", "type": "authentication_error", "code": 401}}),
                );
            }
            if path == "/tokenize" {
                let content = body.get("content").and_then(Value::as_str).unwrap_or("");
                let special = body
                    .get("add_special")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
                let count = content.split_whitespace().count() + usize::from(special);
                let tokens: Vec<usize> = (0..count).collect();
                respond(&stream, "200 OK", &json!({"tokens": tokens}))
            } else {
                chat(fake, &stream, &body)
            }
        }
        ("GET", "/slots") => respond(
            &stream,
            "501 Not Implemented",
            &json!({"error": {"code": 501, "message": "This server does not support slots endpoint.", "type": "not_supported_error"}}),
        ),
        _ => respond(
            &stream,
            "404 Not Found",
            &json!({"error": {"message": "File Not Found", "type": "not_found_error", "code": 404}}),
        ),
    }
}

/// Whether the client has closed the connection (it sends nothing after the request).
fn closed(stream: &TcpStream) -> bool {
    if stream.set_nonblocking(true).is_err() {
        return true;
    }
    let mut byte = [0u8; 1];
    let result = match stream.peek(&mut byte) {
        Ok(0) => true,
        Ok(_) => false,
        Err(e) if e.kind() == io::ErrorKind::WouldBlock => false,
        Err(_) => true,
    };
    let _ = stream.set_nonblocking(false);
    result
}

fn record_closed(fake: &Fake, stage: &str) {
    record(
        fake,
        "events.jsonl",
        json!({"event": "closed", "stage": stage, "time_ms": now_ms() as u64}),
    );
}

/// Sleeps `ms`, checking for a closed connection every 10 ms; `false` if it closed.
fn wait_open(fake: &Fake, stream: &TcpStream, ms: u64, stage: &str) -> bool {
    let end = Instant::now() + Duration::from_millis(ms);
    loop {
        if closed(stream) {
            record_closed(fake, stage);
            return false;
        }
        let now = Instant::now();
        if now >= end {
            return true;
        }
        thread::sleep((end - now).min(Duration::from_millis(10)));
    }
}

/// Sends nothing until the client closes the connection.
fn stall_until_closed(fake: &Fake, stream: &TcpStream, stage: &str) {
    while !closed(stream) {
        thread::sleep(Duration::from_millis(10));
    }
    record_closed(fake, stage);
}

/// Writes the event stream as HTTP chunks, optionally split into small pieces.
struct Events<'a> {
    stream: &'a TcpStream,
    split: Option<usize>,
}

impl Events<'_> {
    fn raw(&mut self, bytes: &[u8]) -> io::Result<()> {
        let size = self.split.unwrap_or(bytes.len()).max(1);
        for piece in bytes.chunks(size) {
            let mut out = self.stream;
            write!(out, "{:x}\r\n", piece.len())?;
            out.write_all(piece)?;
            out.write_all(b"\r\n")?;
            out.flush()?;
            if self.split.is_some() {
                thread::sleep(Duration::from_millis(1));
            }
        }
        Ok(())
    }

    fn data(&mut self, value: &Value) -> io::Result<()> {
        self.raw(format!("data: {value}\n\n").as_bytes())
    }

    fn end(&mut self) -> io::Result<()> {
        let mut out = self.stream;
        out.write_all(b"0\r\n\r\n")?;
        out.flush()
    }
}

fn chunk(delta: Value) -> Value {
    json!({
        "choices": [{"finish_reason": null, "index": 0, "delta": delta}],
        "object": "chat.completion.chunk",
    })
}

fn chat(fake: &Fake, stream: &TcpStream, body: &Value) -> io::Result<()> {
    let control = read_control(&fake.control_path);
    let words: u64 = body
        .get("messages")
        .and_then(Value::as_array)
        .map(|messages| {
            messages
                .iter()
                .filter_map(|m| m.get("content").and_then(Value::as_str))
                .map(|c| c.split_whitespace().count() as u64)
                .sum()
        })
        .unwrap_or(0);
    // The chat template adds a few tokens, like the real server.
    let total = words + 8;
    if total > fake.context {
        return respond(
            stream,
            "400 Bad Request",
            &json!({"error": {
                "code": 400,
                "message": format!("request ({total} tokens) exceeds the available context size ({} tokens), try increasing it", fake.context),
                "type": "exceed_context_size_error",
                "n_prompt_tokens": total,
                "n_ctx": fake.context,
            }}),
        );
    }

    // One slot: wait silently (no headers) while another request runs.
    let _slot = loop {
        if let Ok(slot) = fake.slot.try_lock() {
            break slot;
        }
        if closed(stream) {
            record_closed(fake, "waiting");
            return Ok(());
        }
        thread::sleep(Duration::from_millis(10));
    };
    let result = answer(fake, stream, &control, total);
    // Like the real server, keep computing for a while after the client left.
    if control.after_close_ms > 0 && closed(stream) {
        thread::sleep(Duration::from_millis(control.after_close_ms));
    }
    result
}

/// The event stream of one answer, while holding the slot.
fn answer(fake: &Fake, stream: &TcpStream, control: &Control, total: u64) -> io::Result<()> {
    let mut out = stream;
    out.write_all(
        b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n",
    )?;
    out.flush()?;
    let mut events = Events {
        stream,
        split: control.split_bytes,
    };

    if control.stderr_flood > 0 {
        let block = vec![b'#'; 64 * 1024];
        let mut left = control.stderr_flood;
        let mut stderr = io::stderr().lock();
        while left > 0 {
            let n = left.min(block.len());
            stderr.write_all(&block[..n])?;
            left -= n;
        }
        stderr.flush()?;
    }

    let stall = control.stall.as_deref();
    if control.progress {
        let steps = control
            .progress_steps
            .clone()
            .unwrap_or_else(|| vec![total / 2, total]);
        let progress = |processed: u64| {
            let mut value = chunk(json!({"role": "assistant", "content": null}));
            value["prompt_progress"] =
                json!({"total": total, "cache": 0, "processed": processed, "time_ms": 1});
            value
        };
        events.data(&progress(0))?;
        if stall == Some("reading") {
            stall_until_closed(fake, stream, "reading");
            return Ok(());
        }
        let delay = control.prompt_delay_ms / steps.len().max(1) as u64;
        for processed in steps {
            if !wait_open(fake, stream, delay, "reading") {
                return Ok(());
            }
            events.data(&progress(processed))?;
        }
    } else if !wait_open(fake, stream, control.prompt_delay_ms, "reading") {
        return Ok(());
    } else if stall == Some("reading") {
        stall_until_closed(fake, stream, "reading");
        return Ok(());
    }
    events.data(&chunk(json!({"role": "assistant", "content": null})))?;

    for piece in &control.reasoning {
        events.data(&chunk(json!({"reasoning_content": piece})))?;
    }
    match control.invalid.as_deref() {
        Some("not_json") => {
            events.raw(b"data: {this is not json\n\n")?;
            return events.end();
        }
        Some("missing_fields") => {
            events.data(&json!({"object": "chat.completion.chunk"}))?;
            return events.end();
        }
        Some("truncated") => {
            events.raw(b"data: {\"choices\":[{\"delta\":{\"content\":\"abc")?;
            return events.end();
        }
        _ => {}
    }
    let pieces = control.pieces.clone().unwrap_or_else(|| {
        control
            .reply
            .split_inclusive(' ')
            .map(str::to_owned)
            .collect()
    });
    for (i, piece) in pieces.iter().enumerate() {
        if control.crash_after == Some(i) {
            record(
                fake,
                "events.jsonl",
                json!({"event": "crash", "time_ms": now_ms() as u64}),
            );
            std::process::exit(3);
        }
        if events.data(&chunk(json!({"content": piece}))).is_err() {
            record_closed(fake, "writing");
            return Ok(());
        }
        if i == 0 && stall == Some("writing") {
            stall_until_closed(fake, stream, "writing");
            return Ok(());
        }
        if !wait_open(fake, stream, control.piece_delay_ms, "writing") {
            return Ok(());
        }
    }
    if control.crash_after == Some(pieces.len()) {
        record(
            fake,
            "events.jsonl",
            json!({"event": "crash", "time_ms": now_ms() as u64}),
        );
        std::process::exit(3);
    }
    let mut last = json!({
        "choices": [{"finish_reason": "stop", "index": 0, "delta": {}}],
        "object": "chat.completion.chunk",
    });
    last["timings"] = json!({"predicted_n": pieces.len()});
    events.data(&last)?;
    events.raw(b"data: [DONE]\n\n")?;
    events.end()
}
