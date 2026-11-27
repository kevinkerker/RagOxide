//! Measurement program for the protocol in the README (F38): load time with and without
//! drafter, `count_tokens` of about 3,500 tokens, time to the first text and tokens per
//! second at about 1,000 and 3,500 prompt tokens, the wait after a cancel while reading
//! and while writing, memory of the server after loading and at its peak, and the time
//! of a switch between two levels; with processor, threads, date and foreign load.
//!
//! Needs the folders of F-F12 (`RAGOXIDE_TEST_LLAMA_SERVER_DIR`,
//! `RAGOXIDE_TEST_APP_MODELS_DIR`); the files are checked against `models.toml` first.
//! Run it with a warm file cache and nothing else heavy on the machine:
//!
//! ```text
//! cargo run --release -p kb-llm --example measure-llm -- [threads] [fast,balanced,thorough]
//! ```

#[path = "../tests/common/real.rs"]
mod real;

use std::process::Command;
use std::time::{Duration, Instant};

use futures::StreamExt;
use kb_core::{
    AnswerLevel, CancellationToken, GenerationEvent, GenerationRequest, Generator, HardwareTier,
    Message, ModelManifest, Role,
};
use kb_llm::{GeneratorConfig, LlamaGenerator};
use real::*;

const CONTEXT: u32 = 5120;

fn powershell(script: &str) -> String {
    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", script])
        .output()
        .expect("powershell");
    String::from_utf8_lossy(&output.stdout).trim().to_owned()
}

/// Mean CPU load of the whole machine over `seconds`, in percent.
fn system_load(seconds: u32) -> String {
    powershell(&format!(
        "$s = 0; 1..{seconds} | % {{ $s += (Get-CimInstance Win32_Processor | Measure-Object -Property LoadPercentage -Average).Average; Start-Sleep 1 }}; [math]::Round($s / {seconds}, 1)"
    ))
}

/// Working set, private bytes, peak working set and peak private bytes in GB.
fn memory(pid: u32) -> [f64; 4] {
    let text = powershell(&format!(
        "$p = Get-Process -Id {pid}; \"$($p.WorkingSet64) $($p.PrivateMemorySize64) $($p.PeakWorkingSet64) $($p.PeakPagedMemorySize64)\""
    ));
    let mut values = text
        .split_whitespace()
        .map(|v| v.parse::<f64>().unwrap_or(0.0) / 1e9);
    [(); 4].map(|()| values.next().unwrap_or(0.0))
}

struct Measure {
    env: RealEnv,
    manifest: ModelManifest,
    threads: u32,
}

impl Measure {
    fn generator(&self, manifest: &ModelManifest) -> LlamaGenerator {
        let config = GeneratorConfig::new(
            manifest.clone(),
            &self.env.models,
            HardwareTier::Supported,
            CONTEXT,
            self.env.server(),
            self.threads,
        );
        LlamaGenerator::new(config).unwrap()
    }

    /// Time from a fresh generator to a loaded model (`count_tokens` of the empty text).
    async fn load_time(&self, manifest: &ModelManifest, level: AnswerLevel) -> Duration {
        let generator = self.generator(manifest);
        let begin = Instant::now();
        generator.count_tokens(level, "").await.unwrap();
        let took = begin.elapsed();
        drop(generator);
        wait_no_server().await;
        took
    }
}

async fn wait_no_server() {
    loop {
        let output = Command::new("tasklist")
            .args(["/FI", "IMAGENAME eq llama-server.exe", "/NH"])
            .output()
            .unwrap();
        if !String::from_utf8_lossy(&output.stdout).contains("llama-server") {
            return;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

/// Time to the first text and tokens per second of the generation.
async fn answer_speed(
    generator: &LlamaGenerator,
    request: GenerationRequest,
) -> (Duration, f64, usize) {
    let level = request.level;
    let begin = Instant::now();
    let mut stream = generator
        .generate(request, CancellationToken::new())
        .await
        .unwrap();
    let mut first = None;
    let mut last = begin;
    let mut text = String::new();
    while let Some(item) = stream.next().await {
        if let GenerationEvent::Text(piece) = item.unwrap() {
            first.get_or_insert_with(|| begin.elapsed());
            last = Instant::now();
            text.push_str(&piece);
        }
    }
    let first = first.unwrap_or_default();
    let tokens = generator.count_tokens(level, &text).await.unwrap();
    let writing = (last - begin).saturating_sub(first).as_secs_f64();
    let speed = if writing > 0.0 {
        (tokens.saturating_sub(1)) as f64 / writing
    } else {
        0.0
    };
    (first, speed, tokens)
}

fn tiny(level: AnswerLevel) -> GenerationRequest {
    GenerationRequest {
        level,
        messages: vec![Message {
            role: Role::User,
            content: "Sag Hallo.".into(),
        }],
        max_tokens: Some(4),
        temperature: Some(0.0),
    }
}

/// Time from sending a tiny request to its first progress report.
async fn first_progress(generator: &LlamaGenerator, level: AnswerLevel) -> Duration {
    let begin = Instant::now();
    let mut stream = generator
        .generate(tiny(level), CancellationToken::new())
        .await
        .unwrap();
    let mut at = None;
    while let Some(item) = stream.next().await {
        if let GenerationEvent::Phase {
            progress: Some(_), ..
        } = item.unwrap()
        {
            at.get_or_insert_with(|| begin.elapsed());
        }
    }
    at.unwrap_or_default()
}

/// Cancels `request` while reading (1.5 s after its second progress report, inside
/// the first full batch of 2,048 tokens) or while writing (after 5 text pieces); returns
/// the wait of the next tiny request for its first progress report.
async fn wait_after_cancel(
    generator: &LlamaGenerator,
    request: GenerationRequest,
    reading: bool,
) -> Duration {
    let level = request.level;
    let cancel = CancellationToken::new();
    let mut stream = generator.generate(request, cancel.clone()).await.unwrap();
    let mut texts = 0;
    let mut reports = 0;
    while let Some(item) = stream.next().await {
        match item.unwrap() {
            GenerationEvent::Phase {
                progress: Some(_), ..
            } if reading => {
                reports += 1;
                if reports == 2 {
                    tokio::time::sleep(Duration::from_millis(1500)).await;
                    break;
                }
            }
            GenerationEvent::Text(_) if !reading => {
                texts += 1;
                if texts == 5 {
                    break;
                }
            }
            _ => {}
        }
    }
    cancel.cancel();
    drop(stream);
    first_progress(generator, level).await
}

fn secs(d: Duration) -> String {
    format!("{:.2} s", d.as_secs_f64())
}

#[tokio::main]
async fn main() {
    let Some(env) = RealEnv::get("measure") else {
        std::process::exit(2);
    };
    let args: Vec<String> = std::env::args().skip(1).collect();
    let threads: u32 = args.first().and_then(|t| t.parse().ok()).unwrap_or(8);
    let wanted: Vec<AnswerLevel> = match args.get(1) {
        Some(list) => list
            .split(',')
            .filter_map(|l| match l {
                "fast" => Some(AnswerLevel::Fast),
                "balanced" => Some(AnswerLevel::Balanced),
                "thorough" => Some(AnswerLevel::Thorough),
                _ => None,
            })
            .collect(),
        None => AnswerLevel::ALL.to_vec(),
    };
    let manifest = repo_manifest();
    verify_server(&manifest, &env).expect("server files");
    let mut levels = Vec::new();
    for level in wanted {
        if verify_level(&manifest, &env, HardwareTier::Supported, level).expect("model files") {
            levels.push(level);
        } else {
            println!("{level:?}: Modell nicht vorhanden, übersprungen");
        }
    }
    let measure = Measure {
        env,
        manifest: manifest.clone(),
        threads,
    };
    let plain = without_drafters(&manifest);

    println!(
        "Datum: {}",
        powershell("Get-Date -Format 'yyyy-MM-dd HH:mm'")
    );
    println!(
        "Prozessor: {}",
        powershell("(Get-CimInstance Win32_Processor).Name")
    );
    println!("Threads: {threads}, Kontext {CONTEXT}");
    println!(
        "Freier Speicher vorher: {} GB",
        powershell(
            "[math]::Round((Get-CimInstance Win32_OperatingSystem).FreePhysicalMemory / 1MB, 1)"
        )
    );

    for &level in &levels {
        println!("\n## {level:?}");
        println!(
            "Systemlast vorher (10 s, ohne Server): {} %",
            system_load(10)
        );
        // Warm the file cache, then alternate without and with drafter.
        measure.load_time(&measure.manifest, level).await;
        let mut plain_times = Vec::new();
        let mut draft_times = Vec::new();
        for _ in 0..2 {
            plain_times.push(secs(measure.load_time(&plain, level).await));
            draft_times.push(secs(measure.load_time(&measure.manifest, level).await));
        }
        println!(
            "Ladezeit bis /health 200 ohne Drafter: {}",
            plain_times.join(" / ")
        );
        println!(
            "Ladezeit bis /health 200 mit Drafter: {}",
            draft_times.join(" / ")
        );

        let generator = measure.generator(&measure.manifest);
        generator.count_tokens(level, "").await.unwrap();
        let pid = generator.server_pid().unwrap();
        let loaded = memory(pid);
        println!(
            "Speicher nach dem Laden: Working Set {:.2} GB, privat {:.2} GB",
            loaded[0], loaded[1]
        );

        let (long, long_tokens) = sources_prompt(&generator, level, 3400).await;
        let (short, short_tokens) = sources_prompt(&generator, level, 900).await;
        let user_text = long.messages[1].content.clone();
        let begin = Instant::now();
        let counted = generator.count_tokens(level, &user_text).await.unwrap();
        println!(
            "count_tokens für {counted} Token: {}",
            secs(begin.elapsed())
        );

        let load_before = system_load(3);
        for (name, request, tokens) in [
            ("~1.000", short.clone(), short_tokens),
            ("~3.500", long.clone(), long_tokens),
        ] {
            let (first, speed, generated) = answer_speed(&generator, request).await;
            println!(
                "Prompt {name} ({tokens} Token Nutzertext): erster Text nach {}, {speed:.1} Token/s ({generated} Token)",
                secs(first)
            );
        }
        let idle_wait = first_progress(&generator, level).await;
        let after_reading = wait_after_cancel(&generator, long.clone(), true).await;
        let after_writing = wait_after_cancel(&generator, short.clone(), false).await;
        println!(
            "Erster Fortschritt einer kleinen Anfrage: frei {}, nach Abbruch beim Lesen {}, nach Abbruch beim Schreiben {}",
            secs(idle_wait),
            secs(after_reading),
            secs(after_writing)
        );
        let peak = memory(pid);
        println!(
            "Speicher Spitze: Working Set {:.2} GB, privat {:.2} GB (jetzt {:.2} / {:.2} GB)",
            peak[2], peak[3], peak[0], peak[1]
        );
        println!("Systemlast während der Läufe (Stichprobe 3 s vor den Anfragen): {load_before} %");
        drop(generator);
        wait_no_server().await;
    }

    if levels.len() > 1 {
        println!("\n## Wechsel");
        let generator = measure.generator(&measure.manifest);
        let mut order = levels.clone();
        order.push(levels[0]);
        generator.count_tokens(order[0], "").await.unwrap();
        for pair in order.windows(2) {
            let begin = Instant::now();
            generator.count_tokens(pair[1], "").await.unwrap();
            println!(
                "Wechsel {:?} -> {:?}: {}",
                pair[0],
                pair[1],
                secs(begin.elapsed())
            );
        }
        drop(generator);
        wait_no_server().await;
    }
}
