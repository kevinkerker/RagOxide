//! With the stand-in server: sequences, text, cancel, drop, crash, pipes, error texts
//! and robustness (F18 to F21, F25 to F28, F32, F35, F43).

mod common;

use std::time::{Duration, Instant};

use common::*;
use futures::StreamExt;
use kb_core::{
    AnswerLevel, AnswerPhase, CancellationToken, Error, GenerationEvent, GenerationRequest,
    Generator, HardwareTier, Message, PhaseProgress, Role,
};
use serde_json::json;

/// F18: every complete generation is a valid sequence, with and without progress, start
/// and reasoning.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn every_complete_generation_is_valid() {
    for progress in [true, false] {
        for reasoning in [false, true] {
            let setup = Setup::new();
            let mut control = json!({"progress": progress});
            if reasoning {
                control["reasoning"] = json!(["Ich ", "denke ", "nach."]);
            }
            setup.control(control);
            let generator = setup.generator(HardwareTier::Supported);
            for start in [true, false] {
                let (events, error) = run(&generator, request(AnswerLevel::Fast, "Frage")).await;
                assert!(error.is_none(), "{error:?}");
                assert_valid(&events);
                assert_eq!(starts_with_starting_model(&events), start);
                assert_eq!(text(&events), "Hallo Welt, das ist eine Antwort.");
            }
        }
    }
}

/// F19: `ReadingSources` at least once, without progress if the server reports none;
/// with progress every report in order, reaching the total.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn reading_sources_with_and_without_progress() {
    let setup = Setup::new();
    setup.control(json!({"progress": false, "prompt_delay_ms": 100}));
    let generator = setup.generator(HardwareTier::Supported);
    let (events, _) = run(&generator, request(AnswerLevel::Fast, "a b c d")).await;
    assert_eq!(progress(&events), vec![None]);

    // total = 2 system + 4 user words + 8 template tokens.
    setup.control(json!({"progress_steps": [3, 3, 9, 14]}));
    let (events, _) = run(&generator, request(AnswerLevel::Fast, "a b c d")).await;
    let reports = progress(&events);
    let p = |done| Some(PhaseProgress { done, total: 14 });
    assert_eq!(reports, vec![None, p(0), p(3), p(3), p(9), p(14)]);
    assert_valid(&events);
}

/// F19, ADR 0171: the first `ReadingSources` comes right after sending, also while the
/// server is still busy with an earlier request and sends nothing.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn reading_sources_comes_while_the_server_is_busy() {
    let setup = Setup::new();
    setup.control(json!({"stall": "writing"}));
    let generator = setup.generator(HardwareTier::Supported);
    let mut first = generator
        .generate(request(AnswerLevel::Fast, "eins"), CancellationToken::new())
        .await
        .unwrap();
    while !matches!(first.next().await, Some(Ok(GenerationEvent::Text(_)))) {}
    // The stand-in reads its control per request: the second one does not stall.
    setup.control(json!({}));
    // The first request holds the only slot; the second gets no headers.
    let mut second = generator
        .generate(request(AnswerLevel::Fast, "zwei"), CancellationToken::new())
        .await
        .unwrap();
    let begin = Instant::now();
    let item = tokio::time::timeout(Duration::from_secs(2), second.next())
        .await
        .expect("ReadingSources while busy");
    assert_eq!(
        item.unwrap().unwrap(),
        GenerationEvent::Phase {
            phase: AnswerPhase::ReadingSources,
            progress: None
        }
    );
    assert!(begin.elapsed() < Duration::from_millis(500));
    // Nothing more until the first one ends.
    assert!(
        tokio::time::timeout(Duration::from_millis(300), second.next())
            .await
            .is_err()
    );
    drop(first);
    let (rest, error) = collect(second).await;
    assert!(error.is_none(), "{error:?}");
    assert!(text(&rest).contains("Antwort"));
}

/// F20: `Writing` exactly once, before reasoning; reasoning never becomes text; every
/// request asks for no thinking.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn writing_comes_before_hidden_reasoning() {
    let setup = Setup::new();
    setup.control(json!({"reasoning": ["geheim-1 ", "geheim-2"], "pieces": ["", "sicht", "bar"]}));
    let generator = setup.generator(HardwareTier::Supported);
    let (events, error) = run(&generator, request(AnswerLevel::Fast, "x")).await;
    assert!(error.is_none());
    assert_valid(&events);
    assert_eq!(count_phase(&events, AnswerPhase::Writing), 1);
    assert_eq!(text(&events), "sichtbar");
    assert!(!format!("{events:?}").contains("geheim"));
    // `Writing` is the last phase before any text and follows the last progress.
    let writing = events
        .iter()
        .position(|e| {
            matches!(
                e,
                GenerationEvent::Phase {
                    phase: AnswerPhase::Writing,
                    ..
                }
            )
        })
        .unwrap();
    assert!(
        events[writing + 1..]
            .iter()
            .all(|e| matches!(e, GenerationEvent::Text(_)))
    );
    let body = &setup.requests_to("/v1/chat/completions")[0]["body"];
    assert_eq!(body["chat_template_kwargs"]["enable_thinking"], false);

    // Reasoning only, no content: `Writing` still comes (the model generated).
    setup.control(json!({"reasoning": ["nur gedacht"], "pieces": []}));
    let (events, error) = run(&generator, request(AnswerLevel::Fast, "x")).await;
    assert!(error.is_none());
    assert_eq!(count_phase(&events, AnswerPhase::Writing), 1);
    assert_eq!(text(&events), "");
}

/// F21: text pieces in order, empty ones left out, and the joined text exactly the
/// reply, also when the bytes are split inside umlauts, emoji and lines.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn text_survives_split_bytes() {
    let reply = "Größe: Äpfel, Öl und Übung – Grüße 😀👍🏽 aus Köln. \"Zitat\" [1]\\n";
    for split in [1, 2, 3, 5, 7] {
        let setup = Setup::new();
        setup.control(json!({"reply": reply, "split_bytes": split}));
        let generator = setup.generator(HardwareTier::Supported);
        let (events, error) = run(&generator, request(AnswerLevel::Fast, "x")).await;
        assert!(error.is_none(), "{error:?}");
        assert_valid(&events);
        assert_eq!(text(&events), reply, "split {split}");
        assert!(!text(&events).contains('\u{FFFD}'));
    }
    let setup = Setup::new();
    setup.control(json!({"pieces": ["a", "", "b", "", "", "c"]}));
    let generator = setup.generator(HardwareTier::Supported);
    let (events, _) = run(&generator, request(AnswerLevel::Fast, "x")).await;
    let pieces: Vec<&GenerationEvent> = events
        .iter()
        .filter(|e| matches!(e, GenerationEvent::Text(_)))
        .collect();
    assert_eq!(
        pieces,
        ["a", "b", "c"]
            .iter()
            .map(|t| GenerationEvent::Text((*t).into()))
            .collect::<Vec<_>>()
            .iter()
            .collect::<Vec<_>>()
    );
}

/// Waits for an event matching `want`; returns the events seen.
async fn until(
    stream: &mut kb_core::GenerationStream,
    want: impl Fn(&GenerationEvent) -> bool,
) -> Vec<GenerationEvent> {
    let mut seen = Vec::new();
    loop {
        let event = stream.next().await.expect("item").expect("event");
        let done = want(&event);
        seen.push(event);
        if done {
            return seen;
        }
    }
}

/// Cancels and checks: `Err(Cancelled)` as last item within 100 ms, the stand-in sees the
/// connection closed within 1 s (if `connected`).
async fn cancel_and_check(
    setup: &Setup,
    mut stream: kb_core::GenerationStream,
    cancel: CancellationToken,
    connected: bool,
) {
    let cancelled_at = now_ms();
    let begin = Instant::now();
    cancel.cancel();
    let item = stream.next().await;
    assert!(matches!(item, Some(Err(Error::Cancelled))), "{item:?}");
    assert!(stream.next().await.is_none(), "Cancelled is the last item");
    assert!(
        begin.elapsed() < Duration::from_millis(100),
        "{:?}",
        begin.elapsed()
    );
    if connected {
        let closed = setup
            .wait_closed(cancelled_at, Duration::from_secs(3))
            .expect("connection closed");
        assert!(
            closed <= cancelled_at + 1000,
            "closed after {} ms",
            closed - cancelled_at
        );
    }
}

/// F25: a cancel in every phase ends the stream at once with `Err(Cancelled)` and closes
/// the connection; a token cancelled before reading gives `Cancelled` first and last.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cancel_ends_the_stream_in_every_phase() {
    // Starting (the port is not even open yet) and loading.
    let setup = Setup::new();
    setup.control(json!({"load_ms": 5000}));
    let generator = setup.generator(HardwareTier::Supported);
    let cancel = CancellationToken::new();
    let mut stream = generator
        .generate(request(AnswerLevel::Fast, "x"), cancel.clone())
        .await
        .unwrap();
    until(&mut stream, |e| {
        matches!(
            e,
            GenerationEvent::Phase {
                phase: AnswerPhase::StartingModel,
                ..
            }
        )
    })
    .await;
    tokio::time::sleep(Duration::from_millis(300)).await;
    cancel_and_check(&setup, stream, cancel, false).await;

    // Reading the prompt, the stand-in sends nothing more.
    let setup = Setup::new();
    setup.control(json!({"stall": "reading"}));
    let generator = setup.generator(HardwareTier::Supported);
    let cancel = CancellationToken::new();
    let mut stream = generator
        .generate(request(AnswerLevel::Fast, "x"), cancel.clone())
        .await
        .unwrap();
    until(&mut stream, |e| {
        matches!(
            e,
            GenerationEvent::Phase {
                progress: Some(_),
                ..
            }
        )
    })
    .await;
    cancel_and_check(&setup, stream, cancel, true).await;

    // Writing, the stand-in sends nothing more.
    setup.control(json!({"stall": "writing"}));
    let cancel = CancellationToken::new();
    let mut stream = generator
        .generate(request(AnswerLevel::Fast, "x"), cancel.clone())
        .await
        .unwrap();
    until(&mut stream, |e| matches!(e, GenerationEvent::Text(_))).await;
    cancel_and_check(&setup, stream, cancel, true).await;

    // Waiting for a busy server (no headers yet).
    let blocker = CancellationToken::new();
    let mut busy = generator
        .generate(request(AnswerLevel::Fast, "x"), blocker.clone())
        .await
        .unwrap();
    until(&mut busy, |e| matches!(e, GenerationEvent::Text(_))).await;
    let cancel = CancellationToken::new();
    let mut stream = generator
        .generate(request(AnswerLevel::Fast, "y"), cancel.clone())
        .await
        .unwrap();
    until(&mut stream, |e| {
        matches!(
            e,
            GenerationEvent::Phase {
                phase: AnswerPhase::ReadingSources,
                ..
            }
        )
    })
    .await;
    tokio::time::sleep(Duration::from_millis(100)).await;
    let cancelled_at = now_ms();
    cancel_and_check(&setup, stream, cancel, false).await;
    let begin = Instant::now();
    let closed = loop {
        let closed = setup
            .events()
            .into_iter()
            .find(|e| e["stage"] == "waiting")
            .and_then(|e| e["time_ms"].as_u64());
        if closed.is_some() || begin.elapsed() > Duration::from_secs(3) {
            break closed;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    };
    let closed = closed.expect("the waiting request is closed");
    assert!(
        closed <= cancelled_at + 1000,
        "closed {} ms after the cancel",
        closed.saturating_sub(cancelled_at)
    );
    blocker.cancel();
    drop(busy);

    // Cancelled before reading: Cancelled is the first and last item.
    setup.control(json!({}));
    let cancel = CancellationToken::new();
    cancel.cancel();
    let stream = generator
        .generate(request(AnswerLevel::Fast, "x"), cancel)
        .await
        .unwrap();
    let (events, error) = collect(stream).await;
    assert!(events.is_empty());
    assert!(matches!(error, Some(Error::Cancelled)));
}

/// F25, F37: a token cancelled before `generate` starts no server (many repetitions,
/// the task must check the token before it asks for the server).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_token_cancelled_before_starts_no_server() {
    let setup = Setup::new();
    let generator = setup.generator(HardwareTier::Supported);
    for _ in 0..50 {
        let cancel = CancellationToken::new();
        cancel.cancel();
        let stream = generator
            .generate(request(AnswerLevel::Fast, "x"), cancel)
            .await
            .unwrap();
        // Give the task time to run before the stream is read (and aborts it).
        tokio::time::sleep(Duration::from_millis(10)).await;
        let (events, error) = collect(stream).await;
        assert!(events.is_empty());
        assert!(matches!(error, Some(Error::Cancelled)));
    }
    tokio::time::sleep(Duration::from_millis(500)).await;
    assert!(setup.starts().is_empty(), "{} starts", setup.starts().len());
    assert_eq!(generator.loaded_level(), None);
}

/// F26: dropping the stream closes the connection like a cancel.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn dropping_the_stream_closes_the_connection() {
    let setup = Setup::new();
    for stage in ["reading", "writing"] {
        setup.control(json!({"stall": stage}));
        let generator = setup.generator(HardwareTier::Supported);
        let mut stream = generator
            .generate(request(AnswerLevel::Fast, "x"), CancellationToken::new())
            .await
            .unwrap();
        if stage == "reading" {
            until(&mut stream, |e| {
                matches!(
                    e,
                    GenerationEvent::Phase {
                        progress: Some(_),
                        ..
                    }
                )
            })
            .await;
        } else {
            until(&mut stream, |e| matches!(e, GenerationEvent::Text(_))).await;
        }
        let dropped_at = now_ms();
        drop(stream);
        let closed = setup
            .wait_closed(dropped_at, Duration::from_secs(3))
            .expect("connection closed");
        assert!(closed <= dropped_at + 1000);
        // The server is free again at once for the next request of the same level.
        setup.control(json!({}));
        let (events, error) = run(&generator, request(AnswerLevel::Fast, "y")).await;
        assert!(error.is_none(), "{error:?}");
        assert_valid(&events);
        drop(generator);
        setup
            .wait_none_alive(Duration::from_secs(5))
            .expect("server ended");
    }
}

/// F27: after a cancel while reading, the next generation of the same level works.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn the_next_generation_works_after_a_cancel_while_reading() {
    let setup = Setup::new();
    setup.control(json!({"stall": "reading"}));
    let generator = setup.generator(HardwareTier::Supported);
    let cancel = CancellationToken::new();
    let mut stream = generator
        .generate(request(AnswerLevel::Fast, "x"), cancel.clone())
        .await
        .unwrap();
    until(&mut stream, |e| {
        matches!(
            e,
            GenerationEvent::Phase {
                progress: Some(_),
                ..
            }
        )
    })
    .await;
    cancel.cancel();
    assert!(matches!(stream.next().await, Some(Err(Error::Cancelled))));
    setup.control(json!({}));
    let (events, error) = run(&generator, request(AnswerLevel::Fast, "y")).await;
    assert!(error.is_none());
    assert!(!starts_with_starting_model(&events));
    assert_valid(&events);
    assert_eq!(setup.starts().len(), 1);
}

/// F28: a crash during the stream ends it with `Err(Generator)` within 2 s after the
/// pieces sent before; the next call starts the server again with `StartingModel`.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_crash_ends_the_stream_and_the_next_call_restarts() {
    let setup = Setup::new();
    setup.control(json!({"crash_after": 3, "piece_delay_ms": 50}));
    let generator = setup.generator(HardwareTier::Supported);
    let mut stream = generator
        .generate(request(AnswerLevel::Fast, "x"), CancellationToken::new())
        .await
        .unwrap();
    let mut events = Vec::new();
    let mut texts = 0;
    while texts < 3 {
        let event = stream.next().await.unwrap().unwrap();
        texts += usize::from(matches!(event, GenerationEvent::Text(_)));
        events.push(event);
    }
    let begin = Instant::now();
    let (rest, error) = collect(stream).await;
    assert!(
        begin.elapsed() < Duration::from_secs(2),
        "{:?}",
        begin.elapsed()
    );
    events.extend(rest);
    let Some(Error::Generator(message)) = error else {
        panic!("{error:?}");
    };
    assert!(message.contains("exit code 3"), "{message}");
    assert_eq!(text(&events), "Hallo Welt, das ");
    assert_eq!(
        setup
            .events()
            .iter()
            .filter(|e| e["event"] == "crash")
            .count(),
        1
    );

    setup.control(json!({}));
    let (events, error) = run(&generator, request(AnswerLevel::Fast, "y")).await;
    assert!(error.is_none(), "{error:?}");
    assert!(starts_with_starting_model(&events));
    assert_valid(&events);
    assert_eq!(setup.starts().len(), 2);
}

/// F32: 10 MB on the error output during an answer do not stop the server; nothing is
/// written to disk by the package.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_flood_on_the_error_output_does_not_block() {
    let setup = Setup::new();
    setup.control(json!({"stderr_flood": 10 * 1024 * 1024}));
    let generator = setup.generator(HardwareTier::Supported);
    let result = tokio::time::timeout(
        Duration::from_secs(60),
        run(&generator, request(AnswerLevel::Fast, "x")),
    )
    .await
    .expect("the answer is not blocked");
    assert!(result.1.is_none());
    assert_eq!(text(&result.0), "Hallo Welt, das ist eine Antwort.");
    // Only the stand-in's own files are in the test folder.
    let mut files: Vec<String> = std::fs::read_dir(setup.dir.path())
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    files.sort();
    assert_eq!(files, ["models"]);
}

/// F35: a unique string in every message appears in no error message.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn error_messages_never_quote_the_messages() {
    const MARK: &str = "GEHEIM-9b1e";
    let request = |extra: &str| GenerationRequest {
        level: AnswerLevel::Fast,
        messages: vec![
            Message {
                role: Role::System,
                content: format!("{MARK} System"),
            },
            Message {
                role: Role::User,
                content: format!("{MARK} Frage {extra}"),
            },
        ],
        max_tokens: Some(8),
        temperature: None,
    };
    let mut messages = Vec::new();
    let setup = Setup::new();
    let generator = setup.generator(HardwareTier::Supported);
    // Crash.
    setup.control(json!({"crash_after": 1}));
    messages.push(run(&generator, request("")).await.1.unwrap().to_string());
    // Prompt over the context.
    setup.control(json!({}));
    messages.push(
        run(&generator, request(&"x ".repeat(6000)))
            .await
            .1
            .unwrap()
            .to_string(),
    );
    // Invalid answers.
    for invalid in ["not_json", "missing_fields", "truncated"] {
        setup.control(json!({"invalid": invalid}));
        messages.push(run(&generator, request("")).await.1.unwrap().to_string());
    }
    // Start fails.
    let failing = Setup::new();
    failing.control(json!({"exit_during_load": 2}));
    let generator = failing.generator(HardwareTier::Supported);
    messages.push(run(&generator, request("")).await.1.unwrap().to_string());
    messages.push(
        generator
            .count_tokens(AnswerLevel::Fast, &format!("{MARK} zählen"))
            .await
            .unwrap_err()
            .to_string(),
    );
    assert_eq!(messages.len(), 7);
    for message in &messages {
        assert!(!message.contains(MARK), "{message}");
        assert!(
            !message.contains("Frage") && !message.contains("System"),
            "{message}"
        );
    }
}

/// F43: invalid answers are `Err(Generator)`; large and odd messages cause no panic.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn robust_against_invalid_answers_and_odd_input() {
    let setup = Setup::new();
    let generator = setup.generator(HardwareTier::Supported);
    for invalid in ["not_json", "missing_fields", "truncated"] {
        setup.control(json!({"invalid": invalid}));
        let (_, error) = run(&generator, request(AnswerLevel::Fast, "x")).await;
        assert!(
            matches!(error, Some(Error::Generator(_))),
            "{invalid}: {error:?}"
        );
    }
    setup.control(json!({}));
    drop(generator);
    setup
        .wait_none_alive(Duration::from_secs(5))
        .expect("server ended");
    let generator = {
        let config = kb_llm::GeneratorConfig {
            context_tokens: 400_000,
            ..setup.config(HardwareTier::Supported)
        };
        kb_llm::LlamaGenerator::new(config).unwrap()
    };
    let big = "Wort ".repeat(200_000);
    assert!(big.len() >= 1_000_000);
    for content in [
        big.as_str(),
        "NUL \u{0} mitten",
        "Emoji 😀👍🏽🇩🇪 und ZWJ 👨‍👩‍👧",
        "Richtung \u{202E}umgekehrt\u{202C} \u{2066}isoliert\u{2069} \u{200F}",
    ] {
        let (events, error) = run(&generator, request(AnswerLevel::Fast, content)).await;
        assert!(error.is_none(), "{error:?}");
        assert_valid(&events);
        let count = generator
            .count_tokens(AnswerLevel::Fast, content)
            .await
            .unwrap();
        assert_eq!(count, content.split_whitespace().count());
        let sent = setup.requests_to("/v1/chat/completions").pop().unwrap();
        assert_eq!(sent["body"]["messages"][1]["content"], content);
    }
}
