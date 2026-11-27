//! With the stand-in server: executor, one server at a time, switching, dropped starts,
//! crashes between calls, processes, proxy, idle and many calls (F03, F05, F11 to F13,
//! F29, F30, F36, F43).

mod common;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use common::*;
use futures::StreamExt;
use kb_core::{
    AnswerLevel, AnswerPhase, CancellationToken, Error, GenerationEvent, Generator, HardwareTier,
};
use kb_llm::{GeneratorConfig, LlamaGenerator};
use serde_json::json;

/// A second task on the same runtime that counts its ticks (a sleep of 1 ms each, about
/// 16 ms with the timer of Windows) and adds up the time of gaps longer than
/// [`STALL_GAP`], in which the executor did not run it.
struct Ticker {
    ticks: Arc<AtomicUsize>,
    stalled_us: Arc<AtomicUsize>,
    task: tokio::task::JoinHandle<()>,
}

/// A gap between two ticks longer than this counts as a stall of the executor.
const STALL_GAP: Duration = Duration::from_millis(40);

impl Ticker {
    fn start() -> Self {
        let ticks = Arc::new(AtomicUsize::new(0));
        let stalled_us = Arc::new(AtomicUsize::new(0));
        let (counter, stalled) = (ticks.clone(), stalled_us.clone());
        let task = tokio::spawn(async move {
            let mut last = Instant::now();
            loop {
                tokio::time::sleep(Duration::from_millis(1)).await;
                let gap = last.elapsed();
                last = Instant::now();
                if gap > STALL_GAP {
                    stalled.fetch_add(gap.as_micros() as usize, Ordering::SeqCst);
                }
                counter.fetch_add(1, Ordering::SeqCst);
            }
        });
        Self {
            ticks,
            stalled_us,
            task,
        }
    }

    fn ticks(&self) -> usize {
        self.ticks.load(Ordering::SeqCst)
    }

    fn stalled(&self) -> Duration {
        Duration::from_micros(self.stalled_us.load(Ordering::SeqCst) as u64)
    }
}

/// How the executor fared during one asynchronous phase.
struct Share {
    /// Ticks made, against the rate of an idle second before.
    ticks: f64,
    /// Share of the phase spent in gaps longer than [`STALL_GAP`].
    stalled: f64,
}

/// Runs `phase` and measures [`Share`].
async fn tick_share<T>(
    ticker: &Ticker,
    per_second: f64,
    phase: impl std::future::Future<Output = T>,
) -> (T, Share, Duration) {
    let (before, stalled_before) = (ticker.ticks(), ticker.stalled());
    let begin = Instant::now();
    let value = phase.await;
    let took = begin.elapsed();
    let ticks = (ticker.ticks() - before) as f64;
    let stalled = (ticker.stalled() - stalled_before).as_secs_f64();
    let share = Share {
        ticks: ticks / (took.as_secs_f64() * per_second),
        stalled: stalled / took.as_secs_f64(),
    };
    (value, share, took)
}

/// Least share of ticks in an asynchronous phase (a free executor reaches about 100 %).
const MIN_TICK_SHARE: f64 = 0.5;

/// Largest share of a phase in stalls. Blocking 50 ms per poll of `/health` (every
/// 50 ms) stalls about half of the start; a free executor stays far below.
const MAX_STALLED_SHARE: f64 = 0.25;

/// Longest time for the synchronous parts (return of `generate`, `drop`).
const MAX_SYNC: Duration = Duration::from_millis(50);

/// F05: on a runtime with one thread a second task keeps running while the server starts
/// and loads (2 s), while a stream runs (6 pieces of 500 ms), while a switch ends one
/// server and loads another (2 s) and after the generator is dropped; `generate` and
/// `drop` return at once. No retry: a blocked executor must fail the first run.
#[tokio::test(flavor = "current_thread")]
async fn the_executor_is_never_blocked() {
    let setup = Setup::new();
    setup.control(json!({"load_ms": 2000, "piece_delay_ms": 500}));
    let generator = setup.generator(HardwareTier::Supported);
    let ticker = Ticker::start();
    let before = ticker.ticks();
    tokio::time::sleep(Duration::from_secs(1)).await;
    let per_second = (ticker.ticks() - before) as f64;
    assert!(per_second > 20.0, "ticker too slow: {per_second}/s");

    let (count, start_share, start_took) = tick_share(
        &ticker,
        per_second,
        generator.count_tokens(AnswerLevel::Fast, "a"),
    )
    .await;
    count.unwrap();
    assert!(start_took >= Duration::from_millis(1900), "{start_took:?}");

    let begin = Instant::now();
    let stream = generator
        .generate(request(AnswerLevel::Fast, "x"), CancellationToken::new())
        .await
        .unwrap();
    let generate_took = begin.elapsed();
    let ((events, error), stream_share, stream_took) =
        tick_share(&ticker, per_second, collect(stream)).await;
    assert!(error.is_none());
    assert_valid(&events);
    assert!(
        stream_took >= Duration::from_millis(2500),
        "{stream_took:?}"
    );

    let (count, switch_share, _) = tick_share(
        &ticker,
        per_second,
        generator.count_tokens(AnswerLevel::Balanced, "a"),
    )
    .await;
    count.unwrap();

    let begin = Instant::now();
    drop(generator);
    let drop_took = begin.elapsed();
    let ((), after_drop_share, _) = tick_share(
        &ticker,
        per_second,
        tokio::time::sleep(Duration::from_millis(500)),
    )
    .await;
    ticker.task.abort();
    let phases = [
        ("start", start_share),
        ("stream", stream_share),
        ("switch", switch_share),
        ("after drop", after_drop_share),
    ];
    let shares: Vec<String> = phases
        .iter()
        .map(|(what, s)| format!("{what} {:.2}/{:.2}", s.ticks, s.stalled))
        .collect();
    println!(
        "ticks/s {per_second}; ticks/stalled: {}; generate {generate_took:?}, drop {drop_took:?}",
        shares.join(", ")
    );
    for (what, share) in &phases {
        assert!(
            share.ticks >= MIN_TICK_SHARE,
            "{what}: only {:.0} % of the ticks",
            share.ticks * 100.0
        );
        assert!(
            share.stalled <= MAX_STALLED_SHARE,
            "{what}: stalled {:.0} % of the time",
            share.stalled * 100.0
        );
    }
    assert!(generate_took <= MAX_SYNC, "generate took {generate_took:?}");
    assert!(drop_took <= MAX_SYNC, "drop took {drop_took:?}");
}

/// F11: one model at a time over ten switches; counting in another level loads silently.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn one_server_at_a_time() {
    let setup = Setup::new();
    setup.control(json!({"load_ms": 100}));
    let generator = setup.generator(HardwareTier::Supported);
    let watching = Arc::new(AtomicBool::new(true));
    let most = Arc::new(AtomicUsize::new(0));
    let watcher = {
        let watching = watching.clone();
        let most = most.clone();
        let model_dir = setup.model_dir.clone();
        std::thread::spawn(move || {
            let starts = model_dir.join("fake-records").join("starts.jsonl");
            while watching.load(Ordering::SeqCst) {
                let pids: Vec<u32> = std::fs::read_to_string(&starts)
                    .unwrap_or_default()
                    .lines()
                    .filter_map(|l| serde_json::from_str::<serde_json::Value>(l).ok())
                    .filter_map(|v| v["pid"].as_u64())
                    .map(|p| p as u32)
                    .collect();
                let running = running_pids();
                let alive = pids.iter().filter(|p| running.contains(p)).count();
                most.fetch_max(alive, Ordering::SeqCst);
            }
        })
    };
    for i in 0..10 {
        let level = if i % 2 == 0 {
            AnswerLevel::Fast
        } else {
            AnswerLevel::Balanced
        };
        let (events, error) = run(&generator, request(level, "x")).await;
        assert!(error.is_none(), "{error:?}");
        assert!(starts_with_starting_model(&events), "switch {i}");
        assert_eq!(generator.loaded_level(), Some(level));
    }
    watching.store(false, Ordering::SeqCst);
    watcher.join().unwrap();
    assert_eq!(setup.starts().len(), 10);
    assert!(
        setup.overlaps().is_empty(),
        "a server started while another ran"
    );
    assert!(
        most.load(Ordering::SeqCst) <= 1,
        "at most one server process"
    );

    // Counting in another level loads without an event; the next stream of that level
    // has no StartingModel, like the mock.
    assert_eq!(generator.loaded_level(), Some(AnswerLevel::Balanced));
    generator
        .count_tokens(AnswerLevel::Thorough, "a")
        .await
        .unwrap();
    assert_eq!(generator.loaded_level(), Some(AnswerLevel::Thorough));
    let (events, _) = run(&generator, request(AnswerLevel::Thorough, "x")).await;
    assert!(!starts_with_starting_model(&events));
    assert_valid(&events);
    assert!(setup.overlaps().is_empty());
}

/// F12: a switch waits for the running stream of the other level; after a cancel it
/// happens within 1 s, also while the server still computes.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_switch_waits_for_running_streams() {
    let setup = Setup::new();
    setup.control(json!({"piece_delay_ms": 150}));
    let generator = Arc::new(setup.generator(HardwareTier::Supported));
    let mut stream = generator
        .generate(request(AnswerLevel::Fast, "x"), CancellationToken::new())
        .await
        .unwrap();
    // Wait until it writes.
    let mut events = Vec::new();
    while !matches!(events.last(), Some(GenerationEvent::Text(_))) {
        events.push(stream.next().await.unwrap().unwrap());
    }
    let counting = {
        let generator = generator.clone();
        tokio::spawn(async move {
            let count = generator.count_tokens(AnswerLevel::Balanced, "a b").await;
            (count, Instant::now())
        })
    };
    let (rest, error) = collect(stream).await;
    let stream_end = Instant::now();
    assert!(
        error.is_none(),
        "the stream of Fast ends normally: {error:?}"
    );
    events.extend(rest);
    assert_eq!(text(&events), "Hallo Welt, das ist eine Antwort.");
    let (count, counted_at) = counting.await.unwrap();
    assert_eq!(count.unwrap(), 2);
    assert!(counted_at >= stream_end);
    assert_eq!(generator.loaded_level(), Some(AnswerLevel::Balanced));
    assert!(setup.overlaps().is_empty());

    // Cancel while reading; the server keeps its slot for 5 s after the client left.
    setup.control(json!({"stall": "reading", "after_close_ms": 5000}));
    let cancel = CancellationToken::new();
    let mut stream = generator
        .generate(request(AnswerLevel::Balanced, "x"), cancel.clone())
        .await
        .unwrap();
    while !matches!(
        stream.next().await,
        Some(Ok(GenerationEvent::Phase {
            progress: Some(_),
            ..
        }))
    ) {}
    let counting = {
        let generator = generator.clone();
        tokio::spawn(async move { generator.count_tokens(AnswerLevel::Fast, "a").await })
    };
    tokio::time::sleep(Duration::from_millis(200)).await;
    assert!(!counting.is_finished(), "the switch waits for the stream");
    let begin = Instant::now();
    cancel.cancel();
    assert!(matches!(stream.next().await, Some(Err(Error::Cancelled))));
    assert_eq!(counting.await.unwrap().unwrap(), 1);
    assert!(
        begin.elapsed() < Duration::from_secs(1),
        "switch took {:?}",
        begin.elapsed()
    );
    assert_eq!(generator.loaded_level(), Some(AnswerLevel::Fast));
    assert!(setup.overlaps().is_empty());
}

/// F13: a start whose caller was dropped keeps loading; the next call of the level does
/// not start again.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_dropped_start_keeps_loading() {
    let setup = Setup::new();
    setup.control(json!({"load_ms": 1500}));
    let generator = setup.generator(HardwareTier::Supported);
    let dropped = tokio::time::timeout(
        Duration::from_millis(300),
        generator.count_tokens(AnswerLevel::Balanced, "a"),
    )
    .await;
    assert!(dropped.is_err(), "dropped while loading");
    // A generate while it still loads reports the load and joins it.
    let (events, error) = run(&generator, request(AnswerLevel::Balanced, "x")).await;
    assert!(error.is_none());
    assert!(starts_with_starting_model(&events));
    assert_eq!(
        generator
            .count_tokens(AnswerLevel::Balanced, "a b")
            .await
            .unwrap(),
        2
    );
    assert_eq!(setup.starts().len(), 1, "one start");

    // Dropped and then nothing: the server is up when the next call comes.
    let setup = Setup::new();
    setup.control(json!({"load_ms": 600}));
    let generator = setup.generator(HardwareTier::Supported);
    let _ = tokio::time::timeout(
        Duration::from_millis(100),
        generator.count_tokens(AnswerLevel::Fast, "a"),
    )
    .await;
    tokio::time::sleep(Duration::from_millis(1200)).await;
    assert_eq!(generator.loaded_level(), Some(AnswerLevel::Fast));
    let (events, _) = run(&generator, request(AnswerLevel::Fast, "x")).await;
    assert!(!starts_with_starting_model(&events));
    assert_eq!(setup.starts().len(), 1);
}

/// F29: a server that ended between two calls is started again without an error.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_server_that_ended_between_calls_is_restarted() {
    let setup = Setup::new();
    let generator = setup.generator(HardwareTier::Supported);
    generator
        .count_tokens(AnswerLevel::Fast, "a")
        .await
        .unwrap();
    setup.kill_running();
    assert_eq!(
        generator
            .count_tokens(AnswerLevel::Fast, "a b")
            .await
            .unwrap(),
        2
    );
    assert_eq!(setup.starts().len(), 2);
    setup.kill_running();
    let (events, error) = run(&generator, request(AnswerLevel::Fast, "x")).await;
    assert!(error.is_none(), "{error:?}");
    assert!(starts_with_starting_model(&events));
    assert_valid(&events);
    assert_eq!(setup.starts().len(), 3);
}

/// F29: the end of the process is known at once, also while its pipes stay open a
/// while (here a helper process holds them): the next call starts a new server instead
/// of using the dead one.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn the_end_of_the_server_is_known_before_its_pipes_close() {
    let setup = Setup::new();
    setup.control(json!({"hold_pipes_on_exit": true}));
    let generator = setup.generator(HardwareTier::Supported);
    for round in 0..3 {
        generator
            .count_tokens(AnswerLevel::Fast, "a")
            .await
            .unwrap();
        // End the server, then call 100 ms after it ended: inside the 500 ms in which
        // its pipes are still read. The stand-in records the exit right before it ends
        // (a refused connection is no quick sign on Windows: connecting to a closed port
        // retries for about a second).
        let start = setup.starts().pop().unwrap();
        let port: u16 = arg(&start, "--port").unwrap().parse().unwrap();
        {
            use std::io::Write;
            let mut stream = std::net::TcpStream::connect(("127.0.0.1", port)).unwrap();
            let _ = stream
                .write_all(b"POST /fake-exit HTTP/1.1\r\nHost: x\r\nContent-Length: 0\r\n\r\n");
        }
        let begin = Instant::now();
        while setup
            .events()
            .iter()
            .filter(|e| e["event"] == "exit")
            .count()
            <= round
        {
            assert!(
                begin.elapsed() < Duration::from_secs(3),
                "server did not end"
            );
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
        let count = generator.count_tokens(AnswerLevel::Fast, "a b").await;
        assert_eq!(count.unwrap(), 2, "round {round}");
    }
    assert_eq!(setup.starts().len(), 4);
}

/// Creation time and the time its end was signalled (FILETIME) of every stand-in that
/// runs while `body` runs. A PowerShell watcher holds a handle on each and takes the
/// time of its `Exited` event, which fires when the process object is signalled, after
/// the process has been torn down; the `ExitTime` of Windows is taken earlier, when the
/// termination begins.
async fn process_times<F: std::future::Future<Output = ()>>(
    dir: &std::path::Path,
    body: F,
) -> Vec<(u32, u64, u64)> {
    let stop = dir.join("watch-stop");
    let ready = dir.join("watch-ready");
    let script = format!(
        "$seen = @{{}}; New-Item -ItemType File -Path '{ready}' | Out-Null; \
         while (-not (Test-Path '{stop}')) {{ \
           foreach ($p in Get-Process -Name kb-llm-fake-server -ErrorAction SilentlyContinue) {{ \
             if (-not $seen.ContainsKey($p.Id)) {{ try {{ $p.EnableRaisingEvents = $true; \
               $null = Register-ObjectEvent -InputObject $p -EventName Exited -SourceIdentifier (\"p\" + $p.Id); \
               $seen[$p.Id] = $p }} catch {{}} }} }}; \
           Start-Sleep -Milliseconds 5 }}; \
         foreach ($p in $seen.Values) {{ $null = $p.WaitForExit(10000) }}; \
         Start-Sleep -Milliseconds 300; \
         foreach ($p in $seen.Values) {{ \
           $e = Get-Event -SourceIdentifier (\"p\" + $p.Id) -ErrorAction SilentlyContinue | Select-Object -First 1; \
           if ($e) {{ \"$($p.Id) $($p.StartTime.ToFileTimeUtc()) $($e.TimeGenerated.ToFileTimeUtc())\" }} }}",
        ready = ready.display(),
        stop = stop.display()
    );
    let watcher = std::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", &script])
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    while !ready.exists() {
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    body.await;
    std::fs::write(&stop, b"").unwrap();
    let output = tokio::task::spawn_blocking(move || watcher.wait_with_output().unwrap())
        .await
        .unwrap();
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| {
            let mut parts = line.split_whitespace().map(|p| p.parse::<u64>().ok());
            Some((parts.next()?? as u32, parts.next()??, parts.next()??))
        })
        .collect()
}

/// The `Exited` event can come late under load (single switches up to 30 ms behind the
/// end in the full suite), so the test judges the switches together: a switch that waits
/// for the end shows a margin of 1 to 10 ms on most switches, a switch that does not
/// wait creates the new process 25 to 50 ms before the old one has ended on every switch
/// (counter-check). At most this many of the nine switches may show a negative margin.
const MAX_LATE_SIGNALS: usize = 3;

/// F11, F-F5: on every switch the old process has ended before the new one is created
/// (exit time of the old against the creation time of the new, from Windows).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn the_old_server_ends_before_the_new_one_starts() {
    let setup = Setup::new();
    // Each stand-in holds 1 GiB, so that ending it takes a moment: a switch that does
    // not wait for the end would create the next process before the old one has ended.
    setup.control(json!({"load_ms": 200, "hold_memory_mb": 1024}));
    let generator = setup.generator(HardwareTier::Supported);
    let times = process_times(setup.dir.path(), async {
        for i in 0..10 {
            let level = if i % 2 == 0 {
                AnswerLevel::Fast
            } else {
                AnswerLevel::Balanced
            };
            generator.count_tokens(level, "a").await.unwrap();
        }
        let pid = generator.server_pid().unwrap();
        drop(generator);
        while pid_alive(pid) {
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    })
    .await;
    let ours = setup.pids();
    assert_eq!(ours.len(), 10);
    let mut mine: Vec<(u32, u64, u64)> =
        times.into_iter().filter(|t| ours.contains(&t.0)).collect();
    assert_eq!(mine.len(), 10, "the watcher saw every server");
    mine.sort_by_key(|t| t.1);
    let margins: Vec<i64> = mine
        .windows(2)
        .map(|p| (p[1].1 as i64 - p[0].2 as i64) / 10)
        .collect();
    println!("creation of the new minus end of the old server, µs: {margins:?}");
    // Most switches, and the typical one (median), with the old process ended before the
    // new one was created.
    let negative = margins.iter().filter(|m| **m < 0).count();
    assert!(
        negative <= MAX_LATE_SIGNALS,
        "{negative} of {} switches created the new server before the old one ended",
        margins.len()
    );
    let mut sorted = margins.clone();
    sorted.sort();
    let median = sorted[sorted.len() / 2];
    assert!(
        median >= 0,
        "median {median} µs: the new server starts before the old one has ended"
    );
}

/// F30: dropping the generator ends the server within 5 s.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn dropping_the_generator_ends_the_server() {
    let setup = Setup::new();
    setup.control(json!({"stall": "writing"}));
    let generator = setup.generator(HardwareTier::Supported);
    let mut stream = generator
        .generate(request(AnswerLevel::Fast, "x"), CancellationToken::new())
        .await
        .unwrap();
    while !matches!(stream.next().await, Some(Ok(GenerationEvent::Text(_)))) {}
    assert_eq!(setup.alive(), 1);
    drop(generator);
    let took = setup
        .wait_none_alive(Duration::from_secs(5))
        .expect("server ended within 5 s");
    println!("server ended {took:?} after the drop");
    let (_, error) = collect(stream).await;
    assert!(matches!(error, Some(Error::Generator(_))), "{error:?}");

    // Also while it is still loading.
    let setup = Setup::new();
    setup.control(json!({"load_ms": 10_000}));
    let generator = setup.generator(HardwareTier::Supported);
    let _ = tokio::time::timeout(
        Duration::from_millis(300),
        generator.count_tokens(AnswerLevel::Fast, "a"),
    )
    .await;
    assert_eq!(setup.alive(), 1);
    drop(generator);
    setup
        .wait_none_alive(Duration::from_secs(5))
        .expect("loading server ended within 5 s");
}

/// F30: a process holding a generator with a running server is killed hard; the job
/// object ends the server within 5 s.
#[test]
fn a_hard_kill_of_the_app_ends_the_server() {
    let setup = Setup::new();
    let mut child = child_test(
        "child_holds_a_server",
        &[("KB_LLM_CHILD_MODELS", setup.model_dir.to_str().unwrap())],
    )
    .stdout(std::process::Stdio::piped())
    .spawn()
    .unwrap();
    let begin = Instant::now();
    while setup.alive() == 0 {
        assert!(
            begin.elapsed() < Duration::from_secs(30),
            "server did not start"
        );
        std::thread::sleep(Duration::from_millis(50));
    }
    // Wait until it is ready, then kill the child process hard (TerminateProcess).
    std::thread::sleep(Duration::from_millis(500));
    let pid = setup.pids()[0];
    assert!(pid_alive(pid));
    child.kill().unwrap();
    let _ = child.wait();
    let took = setup
        .wait_none_alive(Duration::from_secs(5))
        .expect("server ended within 5 s after the hard kill");
    println!("server ended {took:?} after the hard kill");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn child_holds_a_server() {
    if !is_child() {
        return;
    }
    let model_dir = std::env::var("KB_LLM_CHILD_MODELS").unwrap();
    let config = GeneratorConfig::new(
        manifest(),
        model_dir,
        HardwareTier::Supported,
        CONTEXT,
        fake_server(),
        2,
    );
    let generator = LlamaGenerator::new(config).unwrap();
    generator
        .count_tokens(AnswerLevel::Fast, "a")
        .await
        .unwrap();
    println!("READY");
    tokio::time::sleep(Duration::from_secs(120)).await;
}

/// F03: proxy variables pointing at a port nobody listens on do not matter: requests
/// reach the stand-in. Runs in a child process with the variables set.
#[test]
fn proxy_variables_are_ignored() {
    let output = child_test(
        "child_uses_the_server_despite_a_proxy",
        &[
            ("HTTP_PROXY", "http://127.0.0.1:9"),
            ("HTTPS_PROXY", "http://127.0.0.1:9"),
            ("ALL_PROXY", "http://127.0.0.1:9"),
            ("http_proxy", "http://127.0.0.1:9"),
            ("all_proxy", "http://127.0.0.1:9"),
            ("NO_PROXY", ""),
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
async fn child_uses_the_server_despite_a_proxy() {
    if !is_child() {
        return;
    }
    assert!(std::env::var_os("HTTP_PROXY").is_some());
    let setup = Setup::new();
    let generator = setup.generator(HardwareTier::Supported);
    assert_eq!(
        generator
            .count_tokens(AnswerLevel::Fast, "a b")
            .await
            .unwrap(),
        2
    );
    let (events, error) = run(&generator, request(AnswerLevel::Fast, "x")).await;
    assert!(error.is_none(), "{error:?}");
    assert_valid(&events);
    assert_eq!(setup.requests_to("/v1/chat/completions").len(), 1);
    // The confirmation of the key at the start plus the count.
    assert_eq!(setup.requests_to("/tokenize").len(), 2);
    println!("CHILD-OK");
}

/// F36: `Balanced` and `Thorough` are unloaded after the idle time, `Fast` stays; a
/// running stream and a waiting `count_tokens` are not idle.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn idle_models_of_the_large_levels_are_unloaded() {
    let setup = Setup::new();
    let config = GeneratorConfig {
        idle_timeout: Duration::from_secs(1),
        ..setup.config(HardwareTier::Supported)
    };
    let generator = LlamaGenerator::new(config).unwrap();

    for level in [AnswerLevel::Balanced, AnswerLevel::Thorough] {
        let starts = setup.starts().len();
        let (events, _) = run(&generator, request(level, "x")).await;
        assert!(starts_with_starting_model(&events));
        tokio::time::sleep(Duration::from_millis(600)).await;
        assert_eq!(generator.loaded_level(), Some(level), "not yet idle");
        tokio::time::sleep(Duration::from_millis(1000)).await;
        assert_eq!(generator.loaded_level(), None, "{level:?} unloaded");
        assert_eq!(setup.alive(), 0, "server ended");
        let (events, error) = run(&generator, request(level, "x")).await;
        assert!(error.is_none());
        assert!(starts_with_starting_model(&events), "loads again");
        assert_eq!(setup.starts().len(), starts + 2);
    }

    // Fast stays loaded.
    generator
        .count_tokens(AnswerLevel::Fast, "a")
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_millis(2000)).await;
    assert_eq!(generator.loaded_level(), Some(AnswerLevel::Fast));
    assert_eq!(setup.alive(), 1);

    // A stream of 2.4 s and a count waiting 1.5 s for the load are no idle time.
    setup.control(json!({"piece_delay_ms": 400}));
    let starts = setup.starts().len();
    let (events, error) = run(&generator, request(AnswerLevel::Balanced, "x")).await;
    assert!(error.is_none());
    assert_eq!(text(&events), "Hallo Welt, das ist eine Antwort.");
    assert_eq!(
        setup.starts().len(),
        starts + 1,
        "no unload during the stream"
    );
    setup.control(json!({"load_ms": 1500}));
    generator
        .count_tokens(AnswerLevel::Thorough, "a")
        .await
        .unwrap();
    assert_eq!(generator.loaded_level(), Some(AnswerLevel::Thorough));
    // Counted from the end of the last call.
    tokio::time::sleep(Duration::from_millis(700)).await;
    generator
        .count_tokens(AnswerLevel::Thorough, "a")
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_millis(700)).await;
    assert_eq!(generator.loaded_level(), Some(AnswerLevel::Thorough));
    tokio::time::sleep(Duration::from_millis(800)).await;
    assert_eq!(generator.loaded_level(), None);
}

/// Handles and threads of this process (PowerShell `Get-Process`).
fn handles_and_threads() -> (u64, u64) {
    let output = std::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            &format!(
                "$p = Get-Process -Id {}; \"$($p.HandleCount) $($p.Threads.Count)\"",
                std::process::id()
            ),
        ])
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&output.stdout);
    let mut numbers = text.split_whitespace().map(|n| n.parse::<u64>().unwrap());
    (numbers.next().unwrap(), numbers.next().unwrap())
}

/// F43: 100 alternating calls in two levels leave no process, handles or threads
/// behind. Runs in its own process so other tests do not disturb the counts.
#[test]
fn many_alternating_calls_leave_nothing_behind() {
    let output = child_test("child_alternates_100_calls", &[])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "{stdout}\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(stdout.contains("CHILD-OK"), "{stdout}");
    for line in stdout.lines().filter(|l| l.contains("COUNTS")) {
        println!("{line}");
    }
}

#[test]
fn child_alternates_100_calls() {
    if !is_child() {
        return;
    }
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .max_blocking_threads(4)
        .thread_keep_alive(Duration::from_millis(200))
        .enable_all()
        .build()
        .unwrap();
    let setup = Setup::new();
    // Warm up the runtime and the client once, so their threads exist before counting.
    runtime.block_on(async {
        let generator = setup.generator(HardwareTier::Supported);
        generator
            .count_tokens(AnswerLevel::Fast, "a")
            .await
            .unwrap();
    });
    assert!(setup.wait_none_alive(Duration::from_secs(5)).is_some());
    std::thread::sleep(Duration::from_millis(1000));
    let before = handles_and_threads();
    runtime.block_on(async {
        let generator = setup.generator(HardwareTier::Supported);
        for i in 0..100 {
            let level = if i % 2 == 0 {
                AnswerLevel::Fast
            } else {
                AnswerLevel::Balanced
            };
            if i % 4 < 2 {
                generator.count_tokens(level, "a b").await.unwrap();
            } else {
                let (events, error) = run(&generator, request(level, "x")).await;
                assert!(error.is_none(), "{error:?}");
                assert_eq!(count_phase(&events, AnswerPhase::StartingModel), 1);
            }
        }
    });
    assert_eq!(setup.starts().len(), 101);
    assert!(
        setup.wait_none_alive(Duration::from_secs(5)).is_some(),
        "no process left"
    );
    assert!(setup.overlaps().is_empty());
    std::thread::sleep(Duration::from_millis(1500));
    let after = handles_and_threads();
    println!(
        "COUNTS handles {} -> {}, threads {} -> {}",
        before.0, after.0, before.1, after.1
    );
    assert!(after.0 <= before.0 + 20, "handles {before:?} -> {after:?}");
    assert!(after.1 <= before.1 + 2, "threads {before:?} -> {after:?}");
    println!("CHILD-OK");
}
