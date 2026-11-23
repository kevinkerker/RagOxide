//! The worker process: crash, executor, errors, cancel and concurrent calls.

mod common;

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use common::*;
use kb_core::{CancellationToken, Error, HardwareTier, Parser};
use kb_parse::{DoclingParser, ParserConfig};

/// Process ids of running workers whose command line names `work_dir`, i.e. the workers
/// of one parser of this test.
fn workers_in(work_dir: &Path) -> Vec<u32> {
    let script = format!(
        "Get-CimInstance Win32_Process -Filter \"Name='kb-parse-worker.exe'\" | \
         Where-Object {{ $_.CommandLine -like '*{}*' }} | ForEach-Object {{ $_.ProcessId }}",
        work_dir.display()
    );
    let out = std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .output()
        .expect("powershell");
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter_map(|l| l.trim().parse().ok())
        .collect()
}

fn wait_for_worker(work_dir: &Path) -> u32 {
    let start = Instant::now();
    loop {
        if let Some(pid) = workers_in(work_dir).first() {
            return *pid;
        }
        assert!(
            start.elapsed() < Duration::from_secs(120),
            "no worker process appeared"
        );
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn is_empty_dir(dir: &Path) -> bool {
    std::fs::read_dir(dir)
        .map(|mut d| d.next().is_none())
        .unwrap_or(true)
}

/// A parser whose worker program does not exist: any call that starts a worker fails
/// with `Error::Io`.
fn parser_without_worker() -> (DoclingParser, PathBuf) {
    let mut config = config(temp_dir("no-models"));
    config.worker = temp_dir("no-worker").join("kb-parse-worker.exe");
    let work = config.work_dir.clone();
    (DoclingParser::new(config), work)
}

#[test]
fn a06_a_crashed_worker_is_a_parse_error_and_the_next_call_works() {
    let Some(dir) = model_dir() else {
        return skipped("a06");
    };
    let config = config(dir);
    let work = config.work_dir.clone();
    let parser = DoclingParser::new(config);
    let killer = std::thread::spawn({
        let work = work.clone();
        move || {
            let pid = wait_for_worker(&work);
            let status = std::process::Command::new("taskkill")
                .args(["/F", "/PID", &pid.to_string()])
                .output()
                .expect("taskkill");
            assert!(status.status.success());
        }
    });
    let result = parse(&parser, &corpus("de-eu-amtsblatt-l-141-2013.pdf"));
    killer.join().unwrap();
    match result {
        Err(Error::Parse(message)) => assert!(message.contains("worker"), "{message}"),
        other => panic!("{other:?}"),
    }
    let next = parse(&parser, &testdata("heading.pdf")).expect("next call");
    assert!(!next.elements.is_empty());
}

#[test]
fn a08_parse_does_not_block_a_single_threaded_executor() {
    let parser = office_parser();
    let path = corpus("de-destatis-grunddaten-krankenhaeuser-2120611257005.xlsx");
    let source = source(&path);
    let ticks = Arc::new(AtomicUsize::new(0));
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let (doc, during) = runtime.block_on(async {
        let ticker = tokio::spawn({
            let ticks = ticks.clone();
            async move {
                loop {
                    tokio::time::sleep(Duration::from_millis(5)).await;
                    ticks.fetch_add(1, Ordering::Relaxed);
                }
            }
        });
        let start = ticks.load(Ordering::Relaxed);
        let doc = parser.parse(&source, CancellationToken::new()).await;
        let during = ticks.load(Ordering::Relaxed) - start;
        ticker.abort();
        (doc, during)
    });
    assert!(doc.is_ok());
    assert!(
        during >= 10,
        "the second task ran only {during} times while parsing"
    );
}

#[test]
fn a55_unsupported_formats_start_no_worker() {
    let (parser, work) = parser_without_worker();
    let dir = temp_dir("unsupported");
    for name in ["notiz.txt", "bild.bmp", "bild.webp", "ohne-endung"] {
        let path = dir.join(name);
        std::fs::write(&path, b"x").unwrap();
        match parse(&parser, &path) {
            Err(Error::UnsupportedFormat(message)) => assert!(message.contains(name), "{message}"),
            other => panic!("{name}: {other:?}"),
        }
    }
    assert!(is_empty_dir(&work));
}

#[test]
fn a57_unreadable_files_are_io_errors_and_missing_models_model_errors() {
    let parser = office_parser();
    let dir = temp_dir("io");
    match parse(&parser, &dir.join("fehlt.docx")) {
        Err(Error::Io(_)) => {}
        other => panic!("{other:?}"),
    }
    let folder = dir.join("ordner.pdf");
    std::fs::create_dir(&folder).unwrap();
    match parse(&parser, &folder) {
        Err(Error::Io(_)) => {}
        other => panic!("{other:?}"),
    }
    // Empty model directory: PDF and images need a model, the other formats do not.
    for name in ["heading.pdf", "portrait-300dpi.png", "pages.tif"] {
        match parse(&parser, &testdata(name)) {
            Err(Error::Model(message)) => assert!(message.contains("Layout"), "{message}"),
            other => panic!("{name}: {other:?}"),
        }
    }
    for name in [
        "structure.docx",
        "slides.pptx",
        "tables.xlsx",
        "structure.html",
        "structure.md",
    ] {
        assert!(parse(&parser, &testdata(name)).is_ok(), "{name}");
    }
    // All model files but one present: the error names that file.
    let models = temp_dir("partial-models");
    let manifest = manifest();
    let mut last = String::new();
    for role in PARSER_ROLES {
        let entry = manifest.select(role, HardwareTier::Supported).unwrap();
        for file in &entry.files {
            let mut path = models.join(&entry.id);
            path.extend(file.path.split('/'));
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(&path, b"").unwrap();
            last = path.display().to_string();
        }
    }
    std::fs::remove_file(&last).unwrap();
    let parser = DoclingParser::new(config(models));
    match parse(&parser, &testdata("heading.pdf")) {
        Err(Error::Model(message)) => {
            assert!(
                message.contains(&last) && message.contains("OcrRecognition"),
                "{message}"
            )
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn a58_a_cancelled_token_stops_before_the_worker() {
    let (parser, work) = parser_without_worker();
    let cancel = CancellationToken::new();
    cancel.cancel();
    for name in ["structure.docx", "heading.pdf"] {
        let source = source(&testdata(name));
        match run(parser.parse(&source, cancel.clone())) {
            Err(Error::Cancelled) => {}
            other => panic!("{other:?}"),
        }
    }
    assert!(is_empty_dir(&work));
}

#[test]
fn a58_cancel_during_a_large_pdf_kills_the_worker_and_leaves_nothing() {
    let Some(dir) = model_dir() else {
        return skipped("a58");
    };
    let config = config(dir);
    let work = config.work_dir.clone();
    let parser = Arc::new(DoclingParser::new(config));
    let cancel = CancellationToken::new();
    let source = source(&corpus("de-eu-amtsblatt-l-141-2013.pdf"));
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    let call = runtime.spawn({
        let (parser, cancel) = (parser.clone(), cancel.clone());
        async move { parser.parse(&source, cancel).await }
    });
    wait_for_worker(&work);
    std::thread::sleep(Duration::from_secs(2));
    let cancelled_at = Instant::now();
    cancel.cancel();
    let result = runtime.block_on(call).unwrap();
    let took = cancelled_at.elapsed();
    notice(&format!(
        "a58: Zeit vom Abbruch bis zur Rückkehr {:.3} s",
        took.as_secs_f64()
    ));
    assert!(matches!(result, Err(Error::Cancelled)), "{result:?}");
    assert!(workers_in(&work).is_empty(), "worker still running");
    assert!(
        is_empty_dir(&work),
        "files of the call left in {}",
        work.display()
    );
    let next = parse(&parser, &testdata("heading.pdf")).expect("next call");
    assert!(!next.elements.is_empty());
}

#[test]
fn a59_concurrent_calls_give_the_same_results() {
    let parser = Arc::new(office_parser());
    let names = [
        "structure.docx",
        "slides.pptx",
        "tables.xlsx",
        "structure.html",
        "structure.md",
        "formats.xlsx",
    ];
    let sequential: Vec<_> = names
        .iter()
        .map(|n| parse(&parser, &testdata(n)).unwrap())
        .collect();
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    let concurrent: Vec<_> = runtime.block_on(async {
        let calls: Vec<_> = names
            .iter()
            .flat_map(|n| [n, n])
            .map(|n| {
                let parser = parser.clone();
                let source = source(&testdata(n));
                tokio::spawn(async move { parser.parse(&source, CancellationToken::new()).await })
            })
            .collect();
        let mut out = Vec::new();
        for call in calls {
            out.push(call.await.unwrap().unwrap());
        }
        out
    });
    for (i, doc) in concurrent.iter().enumerate() {
        assert!(*doc == sequential[i / 2], "{}", names[i / 2]);
    }
    // PDF calls take turns (A67); the results are the same as one after the other.
    let Some(dir) = model_dir() else {
        return skipped("a59 (PDF)");
    };
    let parser = Arc::new(DoclingParser::new(config(dir)));
    let pdfs = ["heading.pdf", "origin.pdf"];
    let sequential: Vec<_> = pdfs
        .iter()
        .map(|n| parse(&parser, &testdata(n)).unwrap())
        .collect();
    let concurrent: Vec<_> = runtime.block_on(async {
        let calls: Vec<_> = pdfs
            .iter()
            .map(|n| {
                let parser = parser.clone();
                let source = source(&testdata(n));
                tokio::spawn(async move { parser.parse(&source, CancellationToken::new()).await })
            })
            .collect();
        let mut out = Vec::new();
        for call in calls {
            out.push(call.await.unwrap().unwrap());
        }
        out
    });
    assert!(concurrent == sequential);
}

#[test]
fn the_parser_is_send_and_sync_behind_an_arc() {
    fn check(_: Arc<dyn Parser>) {}
    check(Arc::new(DoclingParser::new(ParserConfig::new(
        manifest(),
        HardwareTier::Supported,
        temp_dir("no-models"),
        worker(),
    ))));
}

/// Waits up to five seconds for a folder to become empty.
fn becomes_empty(dir: &Path) -> bool {
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(5) {
        if is_empty_dir(dir) {
            return true;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    false
}

#[test]
fn a_dropped_call_leaves_no_folder_behind() {
    let config = config(temp_dir("no-models"));
    let work = config.work_dir.clone();
    let parser = DoclingParser::new(config);
    let source = source(&corpus(
        "de-destatis-grunddaten-krankenhaeuser-2120611257005.xlsx",
    ));
    run(async {
        let call = parser.parse(&source, CancellationToken::new());
        // The caller gives up while the worker runs: the future is dropped.
        let result = tokio::time::timeout(Duration::from_millis(300), call).await;
        assert!(result.is_err(), "the call ended before the timeout");
    });
    assert!(
        becomes_empty(&work),
        "folder of the dropped call left in {}",
        work.display()
    );
}

#[test]
fn folders_left_by_earlier_runs_are_removed_if_they_are_ours() {
    let config = config(temp_dir("no-models"));
    let work = config.work_dir.clone();
    // Left by a process that ended during a call: folder name and marker file of the
    // package, with a worker's files in it.
    let left = work.join("kb-parse-1-0");
    std::fs::create_dir_all(left.join("tmp")).unwrap();
    std::fs::write(left.join("kb-parse.lock"), b"").unwrap();
    std::fs::write(left.join("reply.json"), b"{}").unwrap();
    // A folder of a running call: its marker file is held open.
    let busy = work.join("kb-parse-2-0");
    std::fs::create_dir_all(&busy).unwrap();
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create(true).truncate(false);
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.share_mode(0);
    }
    let held = options.open(busy.join("kb-parse.lock")).unwrap();
    // Files before and after the marker file in the listing: deleting stops at the held
    // marker file, so a file before it shows a folder that was emptied.
    std::fs::write(busy.join("eingabe.html"), b"x").unwrap();
    std::fs::write(busy.join("reply.json"), b"{}").unwrap();
    // Not the package's: no marker file, or another name, even with a marker file.
    let foreign = work.join("kb-parse-3-0");
    std::fs::create_dir_all(&foreign).unwrap();
    std::fs::write(foreign.join("daten.txt"), b"x").unwrap();
    let other = work.join("anderes-programm");
    std::fs::create_dir_all(&other).unwrap();
    std::fs::write(other.join("kb-parse.lock"), b"").unwrap();
    let notes = work.join("kb-parse-notizen");
    std::fs::create_dir_all(&notes).unwrap();
    std::fs::write(notes.join("kb-parse.lock"), b"").unwrap();

    let parser = DoclingParser::new(config);
    parse(&parser, &testdata("structure.md")).unwrap();
    assert!(!left.exists(), "left-over folder not removed");
    assert!(
        ["kb-parse.lock", "eingabe.html", "reply.json"]
            .iter()
            .all(|f| busy.join(f).exists()),
        "folder of a running call emptied"
    );
    assert!(foreign.join("daten.txt").exists() && other.join("kb-parse.lock").exists());
    assert!(
        notes.join("kb-parse.lock").exists(),
        "folder of another name removed"
    );
    drop(held);
}

#[test]
fn a07_the_worker_gets_two_workers_and_two_threads() {
    // A stand-in for the worker program that writes down the settings it was started
    // with; the call then fails, as with a crashed worker.
    let dir = temp_dir("fake-worker");
    let fake = dir.join("fake-worker.cmd");
    std::fs::write(
        &fake,
        "@>\"%~dp0settings.txt\" echo %DOCLING_RS_PDF_WORKERS% %DOCLING_RS_PDF_INTRA%\r\n",
    )
    .unwrap();
    let mut config = config(temp_dir("no-models"));
    config.worker = fake;
    let parser = DoclingParser::new(config);
    match parse(&parser, &testdata("structure.md")) {
        Err(Error::Parse(_)) => {}
        other => panic!("{other:?}"),
    }
    let settings = std::fs::read_to_string(dir.join("settings.txt")).unwrap();
    assert_eq!(settings.trim(), "2 2");
}

/// A model directory in the layout of the contract with an empty file for every parser
/// model file: enough for the checks before the worker starts.
fn dummy_models() -> PathBuf {
    let models = temp_dir("dummy-models");
    let manifest = manifest();
    for role in PARSER_ROLES {
        let entry = manifest.select(role, HardwareTier::Supported).unwrap();
        for file in &entry.files {
            let mut path = models.join(&entry.id);
            path.extend(file.path.split('/'));
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(&path, b"").unwrap();
        }
    }
    models
}

/// A parser whose worker is a stand-in that writes `start <file>` and, after `seconds`,
/// `end <file>` into `log.txt` next to it; the call then fails as with a crashed worker.
/// Returns the parser and the log.
fn logging_parser(seconds: u32) -> (Arc<DoclingParser>, PathBuf) {
    let dir = temp_dir("logging-worker");
    let fake = dir.join("fake-worker.cmd");
    std::fs::write(
        &fake,
        format!(
            "@>>\"%~dp0log.txt\" echo start %~n1\r\n@ping -n {} 127.0.0.1 >nul\r\n@>>\"%~dp0log.txt\" echo end %~n1\r\n",
            seconds + 1
        ),
    )
    .unwrap();
    let mut config = config(dummy_models());
    config.worker = fake;
    (Arc::new(DoclingParser::new(config)), dir.join("log.txt"))
}

/// Copies of a test PDF under the given names in a fresh folder.
fn pdf_copies(names: &[&str]) -> Vec<PathBuf> {
    let dir = temp_dir("pdf-copies");
    names
        .iter()
        .map(|n| {
            let path = dir.join(format!("{n}.pdf"));
            std::fs::copy(testdata("heading.pdf"), &path).unwrap();
            path
        })
        .collect()
}

fn log_lines(log: &Path) -> Vec<String> {
    std::fs::read_to_string(log)
        .unwrap_or_default()
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect()
}

#[test]
fn a67_one_pdf_or_image_call_at_a_time_in_the_order_they_come() {
    let (parser, log) = logging_parser(1);
    let pdfs = pdf_copies(&["a", "b", "c"]);
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(async {
        let mut calls = Vec::new();
        for path in &pdfs {
            let parser = parser.clone();
            let source = source(path);
            calls.push(tokio::spawn(async move {
                parser.parse(&source, CancellationToken::new()).await
            }));
            tokio::time::sleep(Duration::from_millis(150)).await;
        }
        // A Markdown call needs no models and does not wait for the PDF calls.
        let parser_md = parser.clone();
        let md = source(&testdata("structure.md"));
        calls.push(tokio::spawn(async move {
            parser_md.parse(&md, CancellationToken::new()).await
        }));
        for call in calls {
            assert!(matches!(call.await.unwrap(), Err(Error::Parse(_))));
        }
    });
    let lines = log_lines(&log);
    let pdf_lines: Vec<&str> = lines
        .iter()
        .map(String::as_str)
        .filter(|l| !l.ends_with("structure"))
        .collect();
    assert_eq!(
        pdf_lines,
        ["start a", "end a", "start b", "end b", "start c", "end c"],
        "{lines:?}"
    );
    let md_start = lines.iter().position(|l| l == "start structure").unwrap();
    let a_end = lines.iter().position(|l| l == "end a").unwrap();
    assert!(
        md_start < a_end,
        "the Markdown call waited for the PDF call: {lines:?}"
    );
}

#[test]
fn a67_a_waiting_call_is_cancelled_at_once_and_does_not_block_the_executor() {
    let (parser, log) = logging_parser(3);
    let pdfs = pdf_copies(&["erster", "wartender"]);
    let ticks = Arc::new(AtomicUsize::new(0));
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(async {
        let ticker = tokio::spawn({
            let ticks = ticks.clone();
            async move {
                loop {
                    tokio::time::sleep(Duration::from_millis(5)).await;
                    ticks.fetch_add(1, Ordering::Relaxed);
                }
            }
        });
        let first = tokio::spawn({
            let (parser, source) = (parser.clone(), source(&pdfs[0]));
            async move { parser.parse(&source, CancellationToken::new()).await }
        });
        // Wait until the first call's worker runs.
        let start = Instant::now();
        while !log_lines(&log).contains(&"start erster".to_string()) {
            assert!(
                start.elapsed() < Duration::from_secs(20),
                "first worker did not start"
            );
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        let cancel = CancellationToken::new();
        let waiting = tokio::spawn({
            let (parser, source, cancel) = (parser.clone(), source(&pdfs[1]), cancel.clone());
            async move { parser.parse(&source, cancel).await }
        });
        let before = ticks.load(Ordering::Relaxed);
        tokio::time::sleep(Duration::from_millis(500)).await;
        assert!(!waiting.is_finished(), "the second call did not wait");
        let during = ticks.load(Ordering::Relaxed) - before;
        // A blocked executor would tick at most once; Windows timers tick every ~16 ms
        // and a loaded machine fewer times, so a few ticks are the evidence.
        assert!(
            during >= 5,
            "the executor ran only {during} ticks while a call waited"
        );
        let cancelled_at = Instant::now();
        cancel.cancel();
        let result = waiting.await.unwrap();
        assert!(matches!(result, Err(Error::Cancelled)), "{result:?}");
        assert!(cancelled_at.elapsed() < Duration::from_millis(500));
        assert!(
            !first.is_finished(),
            "the first call should still be running"
        );
        assert!(matches!(first.await.unwrap(), Err(Error::Parse(_))));
        ticker.abort();
    });
    assert!(!log_lines(&log).iter().any(|l| l.ends_with("wartender")));
}
