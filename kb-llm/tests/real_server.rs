//! With the real `llama-server` and the real models (F-F12, F40). Without the variables
//! `RAGOXIDE_TEST_LLAMA_SERVER_DIR` and `RAGOXIDE_TEST_APP_MODELS_DIR` every test here
//! reports "übersprungen" with the name of the missing variable and passes. With both,
//! the server files and the models are checked against `models.toml` first; a wrong file
//! fails the test. `gemma-4-e2b` with drafter is required, `gemma-4-e4b` and
//! `gemma-4-12b` are optional (their parts are skipped if absent).
//!
//! The tests take a lock so that only one real server runs at a time (12B: about 8 GB).

mod common;
#[path = "common/real.rs"]
mod real;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use common::{
    assert_valid, child_test, collect, count_phase, is_child, run, starts_with_starting_model, text,
};
use futures::StreamExt;
use kb_core::{
    AnswerLevel, AnswerPhase, CancellationToken, Error, GenerationEvent, GenerationRequest,
    Generator, HardwareTier, Message, ModelManifest, Role,
};
use kb_llm::{GeneratorConfig, LlamaGenerator};
use real::*;
use serde_json::{Value, json};

const THREADS: u32 = 8;
const CONTEXT: u32 = 5120;

/// One real server at a time, across the tests of this binary (each has its own runtime).
async fn lock() -> tokio::sync::MutexGuard<'static, ()> {
    static REAL: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());
    REAL.lock().await
}

/// Folders, checked files and the levels present.
struct Ctx {
    env: RealEnv,
    manifest: ModelManifest,
    levels: Vec<AnswerLevel>,
}

fn ctx(test: &str) -> Option<Ctx> {
    let env = RealEnv::get(test)?;
    let manifest = repo_manifest();
    verify_server(&manifest, &env).unwrap_or_else(|e| panic!("{test}: {e}"));
    let mut levels = Vec::new();
    for level in AnswerLevel::ALL {
        let present = verify_level(&manifest, &env, HardwareTier::Supported, level)
            .unwrap_or_else(|e| panic!("{test}: {e}"));
        if present {
            levels.push(level);
        } else if level == AnswerLevel::Fast {
            panic!(
                "{test}: the model of Fast with drafter is required in {}",
                env.models.display()
            );
        } else {
            eprintln!("{test}: Stufe {level:?} übersprungen, Modell nicht vorhanden");
        }
    }
    Some(Ctx {
        env,
        manifest,
        levels,
    })
}

impl Ctx {
    fn generator_with(
        &self,
        manifest: ModelManifest,
        tier: HardwareTier,
        context: u32,
    ) -> LlamaGenerator {
        let config = GeneratorConfig::new(
            manifest,
            &self.env.models,
            tier,
            context,
            self.env.server(),
            THREADS,
        );
        LlamaGenerator::new(config).unwrap()
    }

    fn generator(&self, tier: HardwareTier) -> LlamaGenerator {
        self.generator_with(self.manifest.clone(), tier, CONTEXT)
    }
}

/// Samples the TCP connections of the test process and of every `llama-server` process
/// every 300 ms; any address outside this machine is recorded.
struct NetWatch {
    running: Arc<AtomicBool>,
    foreign: Arc<Mutex<Vec<String>>>,
    samples: Arc<Mutex<usize>>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl NetWatch {
    fn start() -> Self {
        let running = Arc::new(AtomicBool::new(true));
        let foreign = Arc::new(Mutex::new(Vec::new()));
        let samples = Arc::new(Mutex::new(0));
        let thread = {
            let (running, foreign, samples) = (running.clone(), foreign.clone(), samples.clone());
            std::thread::spawn(move || {
                while running.load(Ordering::SeqCst) {
                    let mut pids = vec![std::process::id()];
                    pids.extend(server_pids());
                    for pid in pids {
                        for (local, remote, state) in tcp_of(pid) {
                            let listening = state.contains("ABH") || state.contains("LISTEN");
                            let ok = loopback(&local)
                                && (listening && (remote == "0.0.0.0:0" || remote == "[::]:0")
                                    || loopback(&remote));
                            if !ok {
                                foreign
                                    .lock()
                                    .unwrap()
                                    .push(format!("{pid}: {local} -> {remote} {state}"));
                            }
                        }
                    }
                    *samples.lock().unwrap() += 1;
                    std::thread::sleep(Duration::from_millis(300));
                }
            })
        };
        Self {
            running,
            foreign,
            samples,
            thread: Some(thread),
        }
    }

    fn finish(mut self) -> (usize, Vec<String>) {
        self.running.store(false, Ordering::SeqCst);
        self.thread.take().unwrap().join().unwrap();
        let samples = *self.samples.lock().unwrap();
        let foreign = self.foreign.lock().unwrap().clone();
        (samples, foreign)
    }
}

/// Process ids of all running `llama-server.exe`.
fn server_pids() -> Vec<u32> {
    let output = std::process::Command::new("tasklist")
        .args(["/FI", "IMAGENAME eq llama-server.exe", "/FO", "CSV", "/NH"])
        .output()
        .unwrap();
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|l| l.split("\",\"").nth(1))
        .filter_map(|p| p.trim_matches('"').parse().ok())
        .collect()
}

fn assert_no_server_left() {
    let begin = Instant::now();
    while !server_pids().is_empty() {
        assert!(
            begin.elapsed() < Duration::from_secs(10),
            "a llama-server is still running"
        );
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn short_request(level: AnswerLevel, max_tokens: u32) -> GenerationRequest {
    let articles = articles();
    GenerationRequest {
        level,
        messages: vec![
            Message {
                role: Role::System,
                content: SYSTEM.into(),
            },
            Message {
                role: Role::User,
                content: format!(
                    "Quellen:\n\n[1] {}\n\n[2] {}\n\nFrage: Was schützt Artikel 1? Antworte in zwei Sätzen.",
                    articles[0], articles[1]
                ),
            },
        ],
        max_tokens: Some(max_tokens),
        temperature: Some(0.0),
    }
}

/// F40: a wrong file fails with a clear message (independent of the variables).
#[test]
fn a_wrong_file_fails_the_check() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("llama-server.exe");
    std::fs::write(&path, b"not the server").unwrap();
    let error = verify_file(&path, &"0".repeat(64)).unwrap_err();
    assert!(error.contains("does not match models.toml"), "{error}");
    verify_file(&path, &sha256(&path).unwrap()).unwrap();
}

/// F33, F34, F40: checked files; the server listens on 127.0.0.1 only, rejects requests
/// without the key, has no `/slots` and no web interface.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn real_server_is_closed_to_others() {
    let Some(ctx) = ctx("real_server_is_closed_to_others") else {
        return;
    };
    let _guard = lock().await;
    let watch = NetWatch::start();
    let generator = ctx.generator(HardwareTier::Supported);
    let begin = Instant::now();
    assert_eq!(
        generator.count_tokens(AnswerLevel::Fast, "").await.unwrap(),
        0
    );
    println!("E2B with drafter ready after {:?}", begin.elapsed());
    let pid = generator.server_pid().unwrap();
    let rows = tcp_of(pid);
    let listening: Vec<_> = rows
        .iter()
        .filter(|r| r.2.contains("ABH") || r.2.contains("LISTEN"))
        .collect();
    assert!(!listening.is_empty(), "{rows:?}");
    for (local, _, _) in &listening {
        assert!(local.starts_with("127.0.0.1:"), "listens on {local}");
    }
    let (port, key) = port_and_key(pid);
    let base = format!("http://127.0.0.1:{port}");
    let client = client();
    let body = json!({"messages": [{"role": "user", "content": "Hallo"}], "max_tokens": 2});
    for path in [
        "/v1/chat/completions",
        "/tokenize",
        "/completion",
        "/apply-template",
    ] {
        let status = client
            .post(format!("{base}{path}"))
            .json_body(&body)
            .send()
            .await
            .unwrap()
            .status();
        assert_eq!(status.as_u16(), 401, "{path} without key");
        let status = client
            .post(format!("{base}{path}"))
            .bearer_auth("falsch")
            .json_body(&body)
            .send()
            .await
            .unwrap()
            .status();
        assert_eq!(status.as_u16(), 401, "{path} with a wrong key");
    }
    let slots = client
        .get(format!("{base}/slots"))
        .bearer_auth(&key)
        .send()
        .await
        .unwrap();
    assert_ne!(slots.status().as_u16(), 200, "/slots is off");
    let root = client.get(format!("{base}/")).send().await.unwrap();
    assert_eq!(root.status().as_u16(), 404, "no web interface");
    let health = client.get(format!("{base}/health")).send().await.unwrap();
    assert_eq!(health.status().as_u16(), 200);
    drop(generator);
    let (samples, foreign) = watch.finish();
    println!("network: {samples} samples, connections outside 127.0.0.1: {foreign:?}");
    assert!(foreign.is_empty(), "{foreign:?}");
    assert_no_server_left();

    // LLAMA_ARG_HOST=0.0.0.0 in the environment of the app changes nothing.
    let output = child_test(
        "child_real_host_variable",
        &[("LLAMA_ARG_HOST", "0.0.0.0"), ("LLAMA_ARG_PORT", "1")],
    )
    .output()
    .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "{stdout}\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(stdout.contains("CHILD-OK"), "{stdout}");
    assert_no_server_left();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn child_real_host_variable() {
    if !is_child() {
        return;
    }
    // The parent has checked the files already.
    let Some(env) = RealEnv::get("child_real_host_variable") else {
        return;
    };
    let config = GeneratorConfig::new(
        repo_manifest(),
        &env.models,
        HardwareTier::BelowMinimum,
        CONTEXT,
        env.server(),
        THREADS,
    );
    let generator = LlamaGenerator::new(config).unwrap();
    generator
        .count_tokens(AnswerLevel::Fast, "a")
        .await
        .unwrap();
    let pid = generator.server_pid().unwrap();
    for (local, _, state) in tcp_of(pid) {
        if state.contains("ABH") || state.contains("LISTEN") {
            assert!(local.starts_with("127.0.0.1:"), "listens on {local}");
        }
    }
    println!("CHILD-OK");
}

/// A small extension so the tests read like the requests they send.
trait JsonBody {
    fn json_body(self, value: &Value) -> Self;
}

impl JsonBody for reqwest::RequestBuilder {
    fn json_body(self, value: &Value) -> Self {
        self.header("content-type", "application/json")
            .body(value.to_string())
    }
}

/// F16, F17: counts equal `/tokenize` asked directly, with and without drafter;
/// counting during a running answer is quick.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn real_token_counts() {
    let Some(ctx) = ctx("real_token_counts") else {
        return;
    };
    let _guard = lock().await;
    let texts = [
        "Die Würde des Menschen ist unantastbar.",
        "Jeder hat das Recht, seine Meinung in Wort, Schrift und Bild frei zu äußern und zu verbreiten.",
        "Größenordnung: 1.234,56 € für Übernachtung, Frühstück und Öffentliche Verkehrsmittel (ÖPNV).",
        "Zeile eins\nZeile zwei\n\tEingerückt – mit Gedankenstrich „Anführungszeichen“ und ß.",
        "Grundgesetz für die Bundesrepublik Deutschland, Artikel 5 Absatz 1 Satz 2.",
    ];
    let with_drafter = ctx.generator(HardwareTier::Supported);
    let mut counts = Vec::new();
    for text in texts {
        let count = with_drafter
            .count_tokens(AnswerLevel::Fast, text)
            .await
            .unwrap();
        assert_eq!(
            count,
            with_drafter
                .count_tokens(AnswerLevel::Fast, text)
                .await
                .unwrap()
        );
        counts.push(count);
    }
    let (port, key) = port_and_key(with_drafter.server_pid().unwrap());
    let client = client();
    for (text, count) in texts.iter().zip(&counts) {
        let answer: Value = serde_json::from_str(
            &client
                .post(format!("http://127.0.0.1:{port}/tokenize"))
                .bearer_auth(&key)
                .json_body(&json!({"content": text, "add_special": false}))
                .send()
                .await
                .unwrap()
                .text()
                .await
                .unwrap(),
        )
        .unwrap();
        assert_eq!(answer["tokens"].as_array().unwrap().len(), *count, "{text}");
    }
    println!("counts: {counts:?}");

    // F17: during an answer of the same level.
    let (request, tokens) = sources_prompt(&with_drafter, AnswerLevel::Fast, 3400).await;
    println!("prompt of {tokens} tokens");
    let mut stream = with_drafter
        .generate(request, CancellationToken::new())
        .await
        .unwrap();
    while !matches!(
        stream.next().await,
        Some(Ok(GenerationEvent::Phase {
            progress: Some(_),
            ..
        }))
    ) {}
    let begin = Instant::now();
    let count = with_drafter
        .count_tokens(AnswerLevel::Fast, texts[1])
        .await
        .unwrap();
    let took = begin.elapsed();
    println!("count_tokens during an answer: {took:?}");
    assert_eq!(count, counts[1]);
    assert!(took < Duration::from_millis(200), "{took:?}");
    drop(stream);
    drop(with_drafter);
    assert_no_server_left();

    let without = ctx.generator(HardwareTier::BelowMinimum);
    for (text, count) in texts.iter().zip(&counts) {
        assert_eq!(
            without.count_tokens(AnswerLevel::Fast, text).await.unwrap(),
            *count
        );
    }
    drop(without);
    assert_no_server_left();
}

/// F18, F20, F21, F22: valid sequences for every model present; no reasoning; German
/// text without U+FFFD; E4B at temperature 0 streams the same text as without stream;
/// `max_tokens` 5 gives at most 5 tokens.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn real_sequences_and_text() {
    let Some(ctx) = ctx("real_sequences_and_text") else {
        return;
    };
    let _guard = lock().await;
    let watch = NetWatch::start();
    let mut runs = vec![(HardwareTier::BelowMinimum, AnswerLevel::Fast)];
    runs.extend(ctx.levels.iter().map(|l| (HardwareTier::Supported, *l)));
    for (tier, level) in runs {
        let generator = ctx.generator(tier);
        let begin = Instant::now();
        let (events, error) = run(&generator, short_request(level, 48)).await;
        assert!(error.is_none(), "{tier:?} {level:?}: {error:?}");
        assert_valid(&events);
        assert!(starts_with_starting_model(&events));
        assert_eq!(count_phase(&events, AnswerPhase::Writing), 1);
        let answer = text(&events);
        println!(
            "{tier:?} {level:?}: {:?} in {:?}: {answer:?}",
            events.len(),
            begin.elapsed()
        );
        assert!(!answer.is_empty());
        assert!(!answer.contains('\u{FFFD}'), "{answer}");

        if level == AnswerLevel::Fast && tier == HardwareTier::Supported {
            // F22: max_tokens 5.
            let (events, error) = run(&generator, short_request(level, 5)).await;
            assert!(error.is_none());
            let short = text(&events);
            let tokens = generator.count_tokens(level, &short).await.unwrap();
            println!("max_tokens 5: {short:?} = {tokens} tokens");
            assert!(tokens <= 5, "{tokens}");
            // F20: the server sends no reasoning with enable_thinking false.
            let (port, key) = port_and_key(generator.server_pid().unwrap());
            let request = short_request(level, 32);
            let raw = client()
                .post(format!("http://127.0.0.1:{port}/v1/chat/completions"))
                .bearer_auth(&key)
                .json_body(&json!({
                    "messages": request.messages, "stream": true, "max_tokens": 32,
                    "temperature": 0.0, "return_progress": true,
                    "chat_template_kwargs": {"enable_thinking": false}
                }))
                .send()
                .await
                .unwrap()
                .text()
                .await
                .unwrap();
            assert!(raw.contains("[DONE]"));
            assert!(!raw.contains("reasoning_content\":\""), "no reasoning");
        }
        if level == AnswerLevel::Balanced {
            // F21: streamed text equals the text without stream (E4B, temperature 0).
            let request = short_request(level, 64);
            let (events, _) = run(&generator, request.clone()).await;
            let streamed = text(&events);
            let (port, key) = port_and_key(generator.server_pid().unwrap());
            let whole: Value = serde_json::from_str(
                &client()
                    .post(format!("http://127.0.0.1:{port}/v1/chat/completions"))
                    .bearer_auth(&key)
                    .json_body(&json!({
                        "messages": request.messages, "max_tokens": 64, "temperature": 0.0,
                        "chat_template_kwargs": {"enable_thinking": false}
                    }))
                    .send()
                    .await
                    .unwrap()
                    .text()
                    .await
                    .unwrap(),
            )
            .unwrap();
            let whole = whole["choices"][0]["message"]["content"].as_str().unwrap();
            assert_eq!(streamed, whole, "E4B streamed and whole text");
            println!(
                "E4B streamed text equals the text without stream ({} chars)",
                whole.len()
            );
        }
        drop(generator);
        assert_no_server_left();
    }
    let (samples, foreign) = watch.finish();
    println!("network: {samples} samples, connections outside 127.0.0.1: {foreign:?}");
    assert!(foreign.is_empty(), "{foreign:?}");
}

/// F24: a prompt over the context is `InvalidInput` with the numbers.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn real_prompt_over_the_context() {
    let Some(ctx) = ctx("real_prompt_over_the_context") else {
        return;
    };
    let _guard = lock().await;
    let generator = ctx.generator_with(ctx.manifest.clone(), HardwareTier::BelowMinimum, 512);
    let long = format!("EINDEUTIG-5c2d {}", "Das ist ein Satz. ".repeat(200));
    let (_, error) = run(
        &generator,
        GenerationRequest {
            level: AnswerLevel::Fast,
            messages: vec![common::user(&long)],
            max_tokens: Some(16),
            temperature: None,
        },
    )
    .await;
    let Some(Error::InvalidInput(message)) = error else {
        panic!("{error:?}");
    };
    println!("{message}");
    assert!(message.contains("512") && !message.contains("EINDEUTIG"));
    drop(generator);
    assert_no_server_left();
}

/// F27, F28: after a cancel while reading the next generation works (wait until its
/// first progress reported per level); a killed server ends the stream with
/// `Err(Generator)` within 2 s, and the next call starts it again.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn real_cancel_and_crash() {
    let Some(ctx) = ctx("real_cancel_and_crash") else {
        return;
    };
    let _guard = lock().await;
    let watch = NetWatch::start();
    for level in ctx.levels.clone() {
        let generator = ctx.generator(HardwareTier::Supported);
        let (request, tokens) = sources_prompt(&generator, level, 3400).await;
        let cancel = CancellationToken::new();
        let mut stream = generator.generate(request, cancel.clone()).await.unwrap();
        while !matches!(
            stream.next().await,
            Some(Ok(GenerationEvent::Phase {
                progress: Some(_),
                ..
            }))
        ) {}
        tokio::time::sleep(Duration::from_millis(1500)).await;
        let cancelled = Instant::now();
        cancel.cancel();
        assert!(matches!(stream.next().await, Some(Err(Error::Cancelled))));
        assert!(cancelled.elapsed() < Duration::from_millis(100));
        drop(stream);
        let begin = Instant::now();
        let mut next = generator
            .generate(short_request(level, 8), CancellationToken::new())
            .await
            .unwrap();
        let first = next.next().await.unwrap().unwrap();
        assert_eq!(
            first,
            GenerationEvent::Phase {
                phase: AnswerPhase::ReadingSources,
                progress: None
            },
            "the wait runs under ReadingSources"
        );
        let first_at = begin.elapsed();
        while !matches!(
            next.next().await,
            Some(Ok(GenerationEvent::Phase {
                progress: Some(_),
                ..
            }))
        ) {}
        let progress_at = begin.elapsed();
        let (rest, error) = collect(next).await;
        assert!(error.is_none(), "{error:?}");
        assert!(!text(&rest).is_empty());
        println!(
            "F27 {level:?}: prompt {tokens} tokens cancelled while reading; next request: ReadingSources after {first_at:?}, first progress after {progress_at:?}"
        );
        drop(generator);
        assert_no_server_left();
    }

    // F28 with E2B: kill the server while it writes.
    let generator = ctx.generator(HardwareTier::Supported);
    let mut stream = generator
        .generate(
            short_request(AnswerLevel::Fast, 200),
            CancellationToken::new(),
        )
        .await
        .unwrap();
    let mut events = Vec::new();
    while events
        .iter()
        .filter(|e| matches!(e, GenerationEvent::Text(_)))
        .count()
        < 3
    {
        events.push(stream.next().await.unwrap().unwrap());
    }
    let pid = generator.server_pid().unwrap();
    let killed = Instant::now();
    let status = std::process::Command::new("taskkill")
        .args(["/F", "/PID", &pid.to_string()])
        .output()
        .unwrap();
    assert!(status.status.success());
    let (rest, error) = collect(stream).await;
    let took = killed.elapsed();
    events.extend(rest);
    println!("F28: stream ended {took:?} after taskkill with {error:?}");
    assert!(matches!(error, Some(Error::Generator(_))));
    assert!(took < Duration::from_secs(2));
    assert!(!text(&events).is_empty());
    let (events, error) = run(&generator, short_request(AnswerLevel::Fast, 8)).await;
    assert!(error.is_none(), "{error:?}");
    assert!(starts_with_starting_model(&events));
    assert_valid(&events);
    drop(generator);
    assert_no_server_left();
    let (samples, foreign) = watch.finish();
    println!("network: {samples} samples, connections outside 127.0.0.1: {foreign:?}");
    assert!(foreign.is_empty(), "{foreign:?}");
}
