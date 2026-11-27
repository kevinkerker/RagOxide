//! With the stand-in server: levels, construction, the command line, start and its
//! failures, counting tokens and the request body (F07 to F10, F14 to F17, F22 to F24).

mod common;

use std::fs;
use std::time::{Duration, Instant};

use common::*;
use kb_core::mock::MockGenerator;
use kb_core::{
    AnswerLevel, AnswerPhase, CancellationToken, Error, GenerationRequest, Generator, HardwareTier,
    Message, Role,
};
use kb_llm::{DEFAULT_IDLE_TIMEOUT, GeneratorConfig, LlamaGenerator};
use serde_json::json;

const LEVELS: [AnswerLevel; 3] = AnswerLevel::ALL;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn construction_checks_its_parameters() {
    let setup = Setup::new();
    let config = setup.config(HardwareTier::Supported);
    assert_eq!(config.idle_timeout, DEFAULT_IDLE_TIMEOUT);
    assert_eq!(DEFAULT_IDLE_TIMEOUT, Duration::from_secs(300));

    let relative = GeneratorConfig {
        model_dir: "models".into(),
        ..config.clone()
    };
    assert!(matches!(
        LlamaGenerator::new(relative),
        Err(Error::InvalidInput(_))
    ));
    let no_context = GeneratorConfig {
        context_tokens: 0,
        ..config.clone()
    };
    assert!(matches!(
        LlamaGenerator::new(no_context),
        Err(Error::Config(_))
    ));
    let no_threads = GeneratorConfig {
        threads: 0,
        ..config.clone()
    };
    assert!(matches!(
        LlamaGenerator::new(no_threads),
        Err(Error::Config(_))
    ));
    // Any absolute form of the folder works.
    let other_form = GeneratorConfig {
        model_dir: format!("{}\\.\\", setup.model_dir.display()).into(),
        ..config
    };
    let generator = LlamaGenerator::new(other_form).unwrap();
    generator.check_level(AnswerLevel::Fast).unwrap();
    assert_eq!(setup.starts().len(), 0, "construction starts nothing");
}

/// F07, F08: offered levels, missing files, folders in place of files, the drafter per
/// tier, and the same categories as the mock.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn check_level_looks_only_at_the_files() {
    let setup = Setup::new();
    let supported = setup.generator(HardwareTier::Supported);
    let below = setup.generator(HardwareTier::BelowMinimum);
    for level in LEVELS {
        supported.check_level(level).unwrap();
    }
    below.check_level(AnswerLevel::Fast).unwrap();
    for level in [AnswerLevel::Balanced, AnswerLevel::Thorough] {
        let error = below.check_level(level).unwrap_err();
        assert!(matches!(error, Error::InvalidInput(_)), "{error:?}");
        let mock = MockGenerator::new("x").offering(&[AnswerLevel::Fast]);
        assert_eq!(mock.check_level(level).unwrap_err().kind(), error.kind());
    }

    // The drafter of the fast model: needed on Supported, not below the minimum.
    let fast = setup.level_files(HardwareTier::Supported, AnswerLevel::Fast);
    assert_eq!(fast.len(), 2, "model and drafter on Supported");
    assert_eq!(
        setup
            .level_files(HardwareTier::BelowMinimum, AnswerLevel::Fast)
            .len(),
        1
    );
    let drafter = fast
        .iter()
        .find(|p| {
            !setup
                .level_files(HardwareTier::BelowMinimum, AnswerLevel::Fast)
                .contains(p)
        })
        .unwrap()
        .clone();
    fs::remove_file(&drafter).unwrap();
    let error = supported.check_level(AnswerLevel::Fast).unwrap_err();
    assert!(
        matches!(&error, Error::ModelMissing(m) if m.contains("Fast")),
        "{error:?}"
    );
    let mock = MockGenerator::new("x").without_model(AnswerLevel::Fast);
    assert_eq!(
        mock.check_level(AnswerLevel::Fast).unwrap_err().kind(),
        error.kind()
    );
    below.check_level(AnswerLevel::Fast).unwrap();

    // A folder in place of a file counts as missing.
    fs::create_dir(&drafter).unwrap();
    assert!(matches!(
        supported.check_level(AnswerLevel::Fast),
        Err(Error::ModelMissing(_))
    ));

    // Every file of every level is needed on Supported.
    for level in [AnswerLevel::Balanced, AnswerLevel::Thorough] {
        for file in setup.level_files(HardwareTier::Supported, level) {
            let saved = fs::read(&file).unwrap();
            fs::remove_file(&file).unwrap();
            let error = supported.check_level(level).unwrap_err();
            assert!(
                matches!(&error, Error::ModelMissing(m) if m.contains(&format!("{level:?}"))),
                "{error:?}"
            );
            fs::write(&file, saved).unwrap();
            supported.check_level(level).unwrap();
        }
    }

    // Quick, and no process, no request.
    let start = Instant::now();
    for level in LEVELS {
        let _ = supported.check_level(level);
    }
    assert!(
        start.elapsed() < Duration::from_millis(10),
        "{:?}",
        start.elapsed()
    );
    assert!(setup.starts().is_empty());
}

/// F09: the command line of the server.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn command_line_follows_f_f2() {
    for tier in [HardwareTier::Supported, HardwareTier::BelowMinimum] {
        let setup = Setup::new();
        let generator = setup.generator(tier);
        for level in manifest().levels(tier) {
            generator.count_tokens(level, "eins zwei").await.unwrap();
            let start = setup.starts().pop().unwrap();
            let entry = manifest().answer_model(tier, level).unwrap();
            let model_file = entry
                .files
                .iter()
                .find(|f| Some(&f.path) != entry.mtp_draft.as_ref())
                .unwrap();
            let expected_model =
                kb_core::canonical_path(&setup.model_dir.join(&entry.id).join(&model_file.path))
                    .unwrap();
            let model = arg(&start, "-m").unwrap();
            assert_eq!(model, expected_model.to_str().unwrap());
            assert!(!model.starts_with(r"\\?\"));
            assert_eq!(arg(&start, "-c"), Some("5120"));
            assert_eq!(arg(&start, "-t"), Some("4"));
            assert_eq!(arg(&start, "-tb"), Some("4"));
            assert_eq!(arg(&start, "-ngl"), Some("0"));
            assert_eq!(arg(&start, "-np"), Some("1"));
            assert_eq!(arg(&start, "--cache-ram"), Some("0"));
            assert_eq!(arg(&start, "--host"), Some("127.0.0.1"));
            assert!(arg(&start, "--port").unwrap().parse::<u16>().is_ok());
            assert!(has_arg(&start, "--no-webui"));
            assert!(has_arg(&start, "--no-slots"));
            let key = arg(&start, "--api-key").unwrap();
            assert_eq!(key.len(), 64);
            for absent in ["-b", "--batch-size", "--metrics", "-hf", "--hf-repo", "-mu"] {
                assert!(!has_arg(&start, absent), "{absent}");
            }
            let drafter = entry.mtp_draft.is_some();
            assert_eq!(
                drafter,
                tier == HardwareTier::Supported,
                "{level:?} {tier:?}"
            );
            assert_eq!(has_arg(&start, "--model-draft"), drafter);
            if drafter {
                let expected = kb_core::canonical_path(
                    &setup
                        .model_dir
                        .join(&entry.id)
                        .join(entry.mtp_draft.as_ref().unwrap()),
                )
                .unwrap();
                assert_eq!(arg(&start, "--model-draft"), expected.to_str());
                assert_eq!(arg(&start, "--spec-type"), Some("draft-mtp"));
                assert_eq!(arg(&start, "--spec-draft-n-max"), Some("4"));
                assert_eq!(arg(&start, "-ngld"), Some("0"));
            } else {
                for absent in ["--spec-type", "--spec-draft-n-max", "-ngld"] {
                    assert!(!has_arg(&start, absent), "{absent}");
                }
            }
        }
    }
}

/// F09: `LLAMA_*` variables of the app do not reach the server. Runs in a child process
/// with such variables set (setting them here would need `unsafe`).
#[test]
fn llama_variables_do_not_reach_the_server() {
    let output = child_test(
        "child_records_the_environment",
        &[
            ("LLAMA_ARG_HOST", "0.0.0.0"),
            ("LLAMA_API_KEY", "x"),
            ("llama_arg_port", "1"),
            ("LLAMA_SERVER_SLOTS_DEBUG", "1"),
        ],
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
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn child_records_the_environment() {
    if !is_child() {
        return;
    }
    assert!(std::env::var_os("LLAMA_ARG_HOST").is_some());
    let setup = Setup::new();
    let generator = setup.generator(HardwareTier::Supported);
    generator
        .count_tokens(AnswerLevel::Fast, "a")
        .await
        .unwrap();
    let start = setup.starts().pop().unwrap();
    assert_eq!(start["llama_env"], json!([]), "{start}");
    assert_eq!(arg(&start, "--host"), Some("127.0.0.1"));
    println!("CHILD-OK");
}

/// F10: the stream comes back at once, `StartingModel` first, `ReadingSources` only
/// after `/health` answered 200.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn generate_returns_before_the_model_is_loaded() {
    let setup = Setup::new();
    setup.control(json!({"load_ms": 2000}));
    let generator = setup.generator(HardwareTier::Supported);
    let begin = Instant::now();
    let stream = generator
        .generate(
            request(AnswerLevel::Fast, "Frage eins"),
            CancellationToken::new(),
        )
        .await
        .unwrap();
    assert!(
        begin.elapsed() < Duration::from_millis(100),
        "{:?}",
        begin.elapsed()
    );
    let mut stream = stream;
    use futures::StreamExt;
    let first = stream.next().await.unwrap().unwrap();
    assert_eq!(
        first,
        kb_core::GenerationEvent::Phase {
            phase: AnswerPhase::StartingModel,
            progress: None
        }
    );
    let second = stream.next().await.unwrap().unwrap();
    assert!(matches!(
        second,
        kb_core::GenerationEvent::Phase {
            phase: AnswerPhase::ReadingSources,
            ..
        }
    ));
    let loaded_after = begin.elapsed();
    assert!(
        loaded_after >= Duration::from_millis(1900),
        "{loaded_after:?}"
    );
    let (rest, error) = collect(stream).await;
    assert!(error.is_none());
    let mut all = vec![first, second];
    all.extend(rest);
    assert_valid(&all);
    assert_eq!(count_phase(&all, AnswerPhase::StartingModel), 1);

    // Loaded: no StartingModel.
    let (events, error) = run(&generator, request(AnswerLevel::Fast, "zwei")).await;
    assert!(error.is_none());
    assert!(!starts_with_starting_model(&events));
    assert_eq!(count_phase(&events, AnswerPhase::StartingModel), 0);
    assert_eq!(setup.starts().len(), 1);
}

/// F14: a start that fails is `Generator` in `count_tokens` and the last item of the
/// stream, with cause, exit code and error output; nothing runs afterwards and the next
/// call tries again.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn failed_starts_are_reported_and_retried() {
    // Wrong path to the server.
    let setup = Setup::new();
    let config = GeneratorConfig {
        server: setup.dir.path().join("missing").join("llama-server.exe"),
        ..setup.config(HardwareTier::Supported)
    };
    let generator = LlamaGenerator::new(config).unwrap();
    let error = generator
        .count_tokens(AnswerLevel::Fast, "a")
        .await
        .unwrap_err();
    assert!(
        matches!(&error, Error::Generator(m) if m.contains("cannot start")),
        "{error}"
    );
    let (events, error) = run(&generator, request(AnswerLevel::Fast, "a")).await;
    assert!(matches!(error, Some(Error::Generator(_))), "{error:?}");
    assert!(starts_with_starting_model(&events));

    // The server ends while loading with an exit code and error output.
    let setup = Setup::new();
    setup.control(json!({"load_ms": 400, "exit_during_load": 5}));
    let generator = setup.generator(HardwareTier::Supported);
    let error = generator
        .count_tokens(AnswerLevel::Fast, "a")
        .await
        .unwrap_err();
    let message = error.to_string();
    assert!(matches!(error, Error::Generator(_)));
    assert!(message.contains("exit code 5"), "{message}");
    assert!(message.contains("failed to load model"), "{message}");
    let key = arg(&setup.starts()[0], "--api-key").unwrap().to_owned();
    assert!(!message.contains(&key), "the key is not in the message");
    let (_, error) = run(&generator, request(AnswerLevel::Fast, "Inhalt-XYZ")).await;
    let message = error.unwrap().to_string();
    assert!(message.contains("exit code 5") && !message.contains("Inhalt-XYZ"));
    assert_eq!(setup.starts().len(), 2, "the next call tries again");
    assert_eq!(setup.alive(), 0);
    // Then the server works again.
    setup.control(json!({}));
    assert_eq!(
        generator
            .count_tokens(AnswerLevel::Fast, "a b")
            .await
            .unwrap(),
        2
    );

    // No port free: the stand-in refuses every port.
    let setup = Setup::new();
    setup.control(json!({"reject_starts": 100}));
    let generator = setup.generator(HardwareTier::Supported);
    let error = generator
        .count_tokens(AnswerLevel::Fast, "a")
        .await
        .unwrap_err();
    assert!(
        matches!(&error, Error::Generator(m) if m.contains("no free port") && m.contains("couldn't bind")),
        "{error}"
    );
    assert_eq!(setup.starts().len(), 4, "first port and three more");
    assert_eq!(setup.alive(), 0);
}

/// The error output quoted in a message is at most 4 KiB.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn quoted_error_output_is_limited() {
    let setup = Setup::new();
    setup.control(json!({"exit_during_load": 1}));
    let generator = setup.generator(HardwareTier::Supported);
    let message = generator
        .count_tokens(AnswerLevel::Fast, "a")
        .await
        .unwrap_err()
        .to_string();
    // The stand-in wrote about 20 KiB before ending; at most 4 KiB of it are quoted,
    // ending with its last lines.
    let quoted = message
        .split_once("error output:\n")
        .map(|(_, quoted)| quoted)
        .expect("quoted error output");
    assert!(quoted.len() <= 4096, "{} bytes quoted", quoted.len());
    assert!(quoted.len() > 3000, "{} bytes quoted", quoted.len());
    assert!(
        quoted.ends_with(&format!(
            "load_model: failed to load model, '{}'",
            arg(&setup.starts()[0], "-m").unwrap()
        )),
        "{quoted}"
    );
}

/// F15: the first port is refused, the next one works without an error.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_taken_port_is_replaced() {
    let setup = Setup::new();
    setup.control(json!({"reject_starts": 1}));
    let generator = setup.generator(HardwareTier::Supported);
    assert_eq!(
        generator
            .count_tokens(AnswerLevel::Fast, "a b c")
            .await
            .unwrap(),
        3
    );
    let starts = setup.starts();
    assert_eq!(starts.len(), 2);
    assert_ne!(arg(&starts[0], "--port"), arg(&starts[1], "--port"));
    let (events, error) = run(&generator, request(AnswerLevel::Fast, "x")).await;
    assert!(error.is_none());
    assert_valid(&events);
}

/// F15, F-F4: another server answering on the chosen port with another key (as a second
/// `llama-server` can bind a port in use on Windows): the start does not take it for its
/// own and moves on to the next port.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_foreign_server_on_the_port_is_not_used() {
    let setup = Setup::new();
    setup.control(json!({"foreign_on_port_starts": 1}));
    let generator = setup.generator(HardwareTier::Supported);
    assert_eq!(
        generator
            .count_tokens(AnswerLevel::Fast, "a b c")
            .await
            .unwrap(),
        3
    );
    let starts = setup.starts();
    assert_eq!(starts.len(), 2);
    assert_ne!(arg(&starts[0], "--port"), arg(&starts[1], "--port"));
    // The foreign server answered /health and rejected the key of the first start.
    let foreign_records = setup.model_dir.join("foreign").join("fake-records");
    let foreign: Vec<serde_json::Value> =
        fs::read_to_string(foreign_records.join("requests.jsonl"))
            .unwrap_or_default()
            .lines()
            .filter_map(|l| serde_json::from_str(l).ok())
            .collect();
    assert!(
        foreign
            .iter()
            .any(|r| r["path"] == "/tokenize" && r["authorised"] == false),
        "{foreign:?}"
    );
    // The own server confirmed the key, and generation works.
    assert!(
        setup
            .requests_to("/tokenize")
            .iter()
            .all(|r| r["authorised"] == true)
    );
    let (events, error) = run(&generator, request(AnswerLevel::Fast, "x")).await;
    assert!(error.is_none(), "{error:?}");
    assert_valid(&events);
    // The foreign server ended with the stopped first instance (same job object).
    let foreign_pid = fs::read_to_string(foreign_records.join("starts.jsonl"))
        .unwrap()
        .lines()
        .filter_map(|l| serde_json::from_str::<serde_json::Value>(l).ok())
        .find_map(|v| v["pid"].as_u64())
        .unwrap() as u32;
    let begin = Instant::now();
    while pid_alive(foreign_pid) {
        assert!(
            begin.elapsed() < Duration::from_secs(5),
            "foreign server still runs"
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

/// F16: counting over `/tokenize` without special tokens; empty text is 0 and loads the
/// model; equal text gives equal counts, with and without drafter.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn count_tokens_uses_tokenize_without_special_tokens() {
    let setup = Setup::new();
    let generator = setup.generator(HardwareTier::Supported);
    assert_eq!(
        generator.count_tokens(AnswerLevel::Fast, "").await.unwrap(),
        0
    );
    assert_eq!(
        generator.loaded_level(),
        Some(AnswerLevel::Fast),
        "empty text loads"
    );
    assert_eq!(setup.starts().len(), 1);
    let text = "Die Würde des Menschen ist unantastbar.";
    let a = generator
        .count_tokens(AnswerLevel::Fast, text)
        .await
        .unwrap();
    let b = generator
        .count_tokens(AnswerLevel::Fast, text)
        .await
        .unwrap();
    assert_eq!(a, 6);
    assert_eq!(a, b);
    let tokenize = setup.requests_to("/tokenize");
    let last = tokenize.last().unwrap();
    assert_eq!(last["body"], json!({"content": text, "add_special": false}));
    assert_eq!(last["authorised"], true);

    let below = Setup::new();
    let without_drafter = below.generator(HardwareTier::BelowMinimum);
    assert_eq!(
        without_drafter
            .count_tokens(AnswerLevel::Fast, text)
            .await
            .unwrap(),
        a
    );
    assert!(!has_arg(&below.starts()[0], "--model-draft"));
    assert!(has_arg(&setup.starts()[0], "--model-draft"));
}

/// F17: counting does not wait for a running answer; an unusable level fails like
/// `check_level` and starts nothing.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn count_tokens_does_not_wait_for_an_answer() {
    let setup = Setup::new();
    setup.control(json!({"stall": "writing"}));
    let generator = setup.generator(HardwareTier::Supported);
    let stream = generator
        .generate(request(AnswerLevel::Fast, "x"), CancellationToken::new())
        .await
        .unwrap();
    let mut stream = stream;
    use futures::StreamExt;
    // Wait until the answer writes (and then stalls).
    loop {
        if let kb_core::GenerationEvent::Text(_) = stream.next().await.unwrap().unwrap() {
            break;
        }
    }
    let begin = Instant::now();
    assert_eq!(
        generator
            .count_tokens(AnswerLevel::Fast, "a b")
            .await
            .unwrap(),
        2
    );
    assert!(
        begin.elapsed() < Duration::from_millis(500),
        "{:?}",
        begin.elapsed()
    );
    drop(stream);

    let below = Setup::new();
    let generator = below.generator(HardwareTier::BelowMinimum);
    let error = generator
        .count_tokens(AnswerLevel::Thorough, "a")
        .await
        .unwrap_err();
    assert_eq!(
        error.kind(),
        generator
            .check_level(AnswerLevel::Thorough)
            .unwrap_err()
            .kind()
    );
    fs::remove_file(&below.level_files(HardwareTier::BelowMinimum, AnswerLevel::Fast)[0]).unwrap();
    let error = generator
        .count_tokens(AnswerLevel::Fast, "a")
        .await
        .unwrap_err();
    assert!(matches!(error, Error::ModelMissing(_)));
    assert!(below.starts().is_empty());
}

/// F22: the body carries the messages unchanged and the fields of F-F3.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn the_request_carries_the_messages_unchanged() {
    let setup = Setup::new();
    let generator = setup.generator(HardwareTier::Supported);
    let messages = vec![
        Message {
            role: Role::System,
            content: "Regeln:\n- \"zitiere\" mit [n]\n- Pfad C:\\Ablage".into(),
        },
        Message {
            role: Role::User,
            content: "Größe? Äpfel, Öl, Übung, ß 😀".into(),
        },
        Message {
            role: Role::Assistant,
            content: "Frühere Antwort.\r\nZeile 2".into(),
        },
        Message {
            role: Role::User,
            content: "Und jetzt?".into(),
        },
    ];
    for (max_tokens, temperature) in [
        (None, None),
        (Some(17), Some(0.25)),
        (Some(1), None),
        (None, Some(1.5)),
    ] {
        let request = GenerationRequest {
            level: AnswerLevel::Fast,
            messages: messages.clone(),
            max_tokens,
            temperature,
        };
        let (_, error) = run(&generator, request).await;
        assert!(error.is_none());
        let sent = setup.requests_to("/v1/chat/completions").pop().unwrap();
        let body = &sent["body"];
        assert_eq!(body["messages"], serde_json::to_value(&messages).unwrap());
        assert_eq!(body["stream"], true);
        assert_eq!(body["return_progress"], true);
        assert_eq!(
            body["chat_template_kwargs"],
            json!({"enable_thinking": false})
        );
        assert_eq!(
            body.get("max_tokens").and_then(|v| v.as_u64()),
            max_tokens.map(u64::from)
        );
        assert_eq!(
            body.get("temperature").and_then(|v| v.as_f64()),
            temperature.map(f64::from)
        );
        // Sorted: with `serde_json/preserve_order` (enabled in the workspace through
        // feature unification) the keys keep their insertion order.
        let mut keys: Vec<&String> = body.as_object().unwrap().keys().collect();
        keys.sort();
        let mut expected = vec![
            "chat_template_kwargs",
            "messages",
            "return_progress",
            "stream",
        ];
        if max_tokens.is_some() {
            expected.push("max_tokens");
        }
        if temperature.is_some() {
            expected.push("temperature");
        }
        expected.sort();
        assert_eq!(keys, expected);
        assert_eq!(sent["authorised"], true);
    }
}

/// F23: invalid requests and unusable levels are errors before the stream; nothing
/// starts.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn invalid_requests_fail_before_the_stream() {
    let setup = Setup::new();
    let generator = setup.generator(HardwareTier::BelowMinimum);
    let base = request(AnswerLevel::Fast, "x");
    let cases = [
        GenerationRequest {
            messages: Vec::new(),
            ..base.clone()
        },
        GenerationRequest {
            max_tokens: Some(0),
            ..base.clone()
        },
        GenerationRequest {
            temperature: Some(f32::NAN),
            ..base.clone()
        },
        GenerationRequest {
            temperature: Some(f32::INFINITY),
            ..base.clone()
        },
        GenerationRequest {
            temperature: Some(f32::NEG_INFINITY),
            ..base.clone()
        },
        GenerationRequest {
            temperature: Some(-0.5),
            ..base.clone()
        },
    ];
    for case in cases {
        let result = generator
            .generate(case.clone(), CancellationToken::new())
            .await;
        assert!(matches!(result, Err(Error::InvalidInput(_))), "{case:?}");
    }
    let result = generator
        .generate(
            GenerationRequest {
                level: AnswerLevel::Balanced,
                ..base.clone()
            },
            CancellationToken::new(),
        )
        .await;
    assert!(matches!(result, Err(Error::InvalidInput(_))));
    fs::remove_file(&setup.level_files(HardwareTier::BelowMinimum, AnswerLevel::Fast)[0]).unwrap();
    let result = generator.generate(base, CancellationToken::new()).await;
    assert!(matches!(result, Err(Error::ModelMissing(_))));
    tokio::time::sleep(Duration::from_millis(200)).await;
    assert!(setup.starts().is_empty(), "no process started");
}

/// F24: a prompt over the context ends the stream with `InvalidInput` and the token
/// numbers, without the prompt.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_prompt_over_the_context_is_invalid_input() {
    let setup = Setup::new();
    let generator = setup.generator(HardwareTier::Supported);
    let long = format!("EINDEUTIG-7F3A {}", "Wort ".repeat(6000));
    let (events, error) = run(&generator, request(AnswerLevel::Fast, &long)).await;
    let Some(Error::InvalidInput(message)) = error else {
        panic!("{error:?}");
    };
    assert!(
        message.contains("6011") && message.contains("5120"),
        "{message}"
    );
    assert!(!message.contains("EINDEUTIG"));
    assert!(!message.contains("Wort"));
    assert_eq!(
        phases(&events),
        vec![AnswerPhase::StartingModel, AnswerPhase::ReadingSources]
    );
    // The server stays usable.
    let (events, error) = run(&generator, request(AnswerLevel::Fast, "kurz")).await;
    assert!(error.is_none());
    assert_valid(&events);
}
