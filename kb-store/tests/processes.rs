//! Tests with a child process: only one process opens the database (G33), and every
//! write survives a hard abort of the process whole or not at all (G34).
//!
//! The child is this test program itself, started with the test `child_process` and an
//! environment variable naming its role.

mod common;

use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdout, Command, Stdio};
use std::sync::mpsc;
use std::time::Duration;

use common::*;
use futures::executor::block_on;
use kb_core::*;
use kb_store::SqliteStore;

const ROLE: &str = "KB_STORE_TEST_ROLE";
const DB: &str = "KB_STORE_TEST_DB";
const PREFIX: &str = "KBSTORE-";

/// Entry of the child process. Does nothing when run as a normal test.
#[test]
fn child_process() {
    let (Ok(role), Ok(db)) = (std::env::var(ROLE), std::env::var(DB)) else {
        return;
    };
    let db = PathBuf::from(db);
    match role.as_str() {
        "try-open" => match block_on(SqliteStore::open(&db)) {
            Ok(_) => println!("{PREFIX}OPEN-OK"),
            Err(e) => println!("{PREFIX}OPEN-ERR {:?}|{e}", e.kind()),
        },
        "hold" => {
            let store = block_on(SqliteStore::open(&db)).expect("open");
            println!("{PREFIX}READY");
            loop {
                std::thread::sleep(Duration::from_millis(50));
                block_on(store.chats()).expect("read while holding");
            }
        }
        "crash" => crash_loop(&db),
        other => panic!("unknown role {other}"),
    }
}

/// A child process that is killed when the guard is dropped, also when a test fails.
struct ChildGuard(Child);

impl ChildGuard {
    fn kill(&mut self) {
        self.0.kill().expect("kill");
        self.0.wait().expect("wait");
    }

    fn wait_success(&mut self) -> bool {
        self.0.wait().expect("child ends").success()
    }
}

impl Drop for ChildGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn spawn(role: &str, db: &Path) -> (ChildGuard, mpsc::Receiver<String>) {
    let mut child = Command::new(std::env::current_exe().expect("test program"))
        .args([
            "child_process",
            "--exact",
            "--nocapture",
            "--test-threads=1",
        ])
        .env(ROLE, role)
        .env(DB, db)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn child");
    let stdout: ChildStdout = child.stdout.take().expect("stdout");
    let (lines, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        for line in BufReader::new(stdout).lines().map_while(Result::ok) {
            // The test harness may print its own text before ours on the same line.
            if let Some(at) = line.find(PREFIX)
                && lines.send(line[at + PREFIX.len()..].to_owned()).is_err()
            {
                break;
            }
        }
    });
    (ChildGuard(child), receiver)
}

fn next_line(lines: &mpsc::Receiver<String>) -> String {
    lines
        .recv_timeout(Duration::from_secs(60))
        .expect("line from the child")
}

/// Starts a child that tries to open `db` and returns its report.
fn try_open_in_child(db: &Path) -> String {
    let (mut child, lines) = spawn("try-open", db);
    let line = next_line(&lines);
    assert!(child.wait_success());
    line
}

#[test]
fn a_second_process_is_refused_and_the_lock_ends_with_the_first() {
    // G33
    let scratch = Scratch::new();
    let db = scratch.db();
    let chats = sample_chats();
    run(async {
        let first = open(&db).await;
        first.save_chat(&chats[0]).await.unwrap();

        let report = try_open_in_child(&db);
        assert!(report.starts_with("OPEN-ERR Store|"), "{report}");
        assert!(report.contains("in use by another process"), "{report}");

        // The first process works on undisturbed.
        first.save_chat(&chats[1]).await.unwrap();
        assert_eq!(first.chats().await.unwrap().len(), 2);

        // A second store in the same process is refused the same way.
        let error = SqliteStore::open(&db).await.unwrap_err();
        assert!(matches!(error, Error::Store(_)), "{error:?}");
        assert!(
            error.to_string().contains("in use by another process"),
            "{error}"
        );
        first.save_chat(&chats[2]).await.unwrap();
    });
    // After closing, another process opens it.
    assert_eq!(try_open_in_child(&db), "OPEN-OK");

    // A process that holds the database is killed hard; then it opens again without
    // any manual step.
    let (mut holder, lines) = spawn("hold", &db);
    assert_eq!(next_line(&lines), "READY");
    run(async {
        let error = SqliteStore::open(&db).await.unwrap_err();
        assert!(
            error.to_string().contains("in use by another process"),
            "{error}"
        );
    });
    holder.kill();
    run(async {
        let store = open(&db).await;
        assert_eq!(store.chats().await.unwrap().len(), 3);
        store.save_chat(&chats[0]).await.unwrap();
    });
}

// ---- G34 ---------------------------------------------------------------------------

const CRASH_CHAT: &str = "crash-chat";

fn crash_id(i: usize) -> DocumentId {
    DocumentId::new(format!("crash-{i}"))
}

fn crash_record(i: usize) -> DocumentRecord {
    DocumentRecord {
        title: Some(format!("Durchlauf {i}")),
        content_hash: Some(format!("{i:064}")),
        size_bytes: Some(u64::try_from(i).unwrap()),
        modified_at: Some(Timestamp(u64::try_from(i).unwrap())),
        status: DocumentStatus::Indexed { chunks: 2_000 },
        ..record(&crash_id(i).0, &format!(r"C:\Ablage\Absturz\crash-{i}.pdf"))
    }
}

fn crash_content(i: usize) -> Document {
    let mut document = big_document(&crash_id(i).0, 2_000, 150);
    document.title = Some(format!("Durchlauf {i}"));
    document
}

fn crash_turn(k: usize) -> ChatTurn {
    ChatTurn {
        question: format!("Frage {k}"),
        answer: Answer {
            text: format!("Antwort {k}: ").repeat(12),
            citations: Vec::new(),
            no_sources: true,
            level: AnswerLevel::Fast,
            outcome: AnswerOutcome::Complete,
        },
    }
}

fn crash_chat(turns: usize) -> Chat {
    Chat {
        id: ChatId::new(CRASH_CHAT),
        title: "Absturz".into(),
        created_at: Timestamp(1),
        updated_at: Timestamp(u64::try_from(turns).unwrap()),
        turns: (0..turns).map(crash_turn).collect(),
    }
}

fn crash_folder(i: usize) -> PathBuf {
    PathBuf::from(format!(r"C:\Absturz\{i:06}"))
}

/// The child's endless loop of writes. It continues from the state it finds, redoing
/// the last round. Once its first new round is written (the chat has one turn more
/// than at the start), it reports `WROTE`; the parent kills it only after that, so that
/// every round of the parent's kills makes progress, however fast the disk is.
fn crash_loop(db: &Path) {
    block_on(async {
        let store = SqliteStore::open(db).await.expect("open");
        let done = store
            .chat(&ChatId::new(CRASH_CHAT))
            .await
            .expect("chat")
            .map_or(0, |c| c.turns.len());
        println!("{PREFIX}READY");
        for i in done.saturating_sub(1).. {
            store
                .save_content(&crash_content(i))
                .await
                .expect("content");
            store
                .upsert_document(&crash_record(i))
                .await
                .expect("record");
            if i > 0 {
                store
                    .delete_document(&crash_id(i - 1))
                    .await
                    .expect("delete");
            }
            store.save_chat(&crash_chat(i + 1)).await.expect("chat");
            if i == done {
                println!("{PREFIX}WROTE");
            }
            store
                .update_settings(Box::new(move |s| {
                    if !s.watched_folders.contains(&crash_folder(i)) {
                        s.watched_folders.push(crash_folder(i));
                    }
                    Ok(())
                }))
                .await
                .expect("settings");
        }
    });
}

/// Checks after a kill that every object is whole, in the state before or after the
/// interrupted call. Returns the number of turns of the chat.
async fn check_after_kill(store: &SqliteStore, round: usize) -> usize {
    let turns = match store.chat(&ChatId::new(CRASH_CHAT)).await.unwrap() {
        // No chat with half turns: every turn is complete and in order.
        Some(chat) => {
            assert_eq!(chat, crash_chat(chat.turns.len()), "round {round}");
            chat.turns.len()
        }
        None => 0,
    };
    let records = store.documents().await.unwrap();
    let index = |id: &DocumentId| -> usize { id.0["crash-".len()..].parse().unwrap() };
    let mut with_content = Vec::new();
    for i in turns.saturating_sub(2)..turns + 2 {
        if let Some(content) = store.content(&crash_id(i)).await.unwrap() {
            assert_eq!(
                content,
                crash_content(i),
                "round {round}: content {i} whole"
            );
            with_content.push(i);
        }
    }
    for record in &records {
        let i = index(&record.id);
        assert_eq!(record, &crash_record(i), "round {round}");
        // No record without its content (delete_document removes both at once).
        assert!(
            with_content.contains(&i),
            "round {round}: record {i} without content"
        );
    }
    // At most the newest two documents exist; older ones were deleted whole.
    if let Some(&newest) = with_content.iter().max() {
        assert!(
            with_content.iter().all(|i| *i + 1 >= newest),
            "round {round}: {with_content:?}"
        );
    }
    // Folders: a gapless prefix, one per finished round of the child.
    let folders = store.settings().await.unwrap().watched_folders;
    assert_eq!(
        folders,
        (0..folders.len()).map(crash_folder).collect::<Vec<_>>()
    );
    assert!(
        folders.len() + 1 >= turns && folders.len() <= turns,
        "round {round}: {} folders, {turns} turns",
        folders.len()
    );
    turns
}

#[test]
fn every_write_survives_a_hard_abort_whole_or_not_at_all() {
    // G34: at least twenty hard aborts at different times.
    let scratch = Scratch::new();
    let db = scratch.db();
    let mut seed: u64 = 0x2545_F491_4F6C_DD1D;
    let journal = db.with_file_name("library.sqlite-journal");
    let mut last_turns = 0;
    let mut inside_a_transaction = 0;
    for round in 0..24 {
        let (mut child, lines) = spawn("crash", &db);
        assert_eq!(next_line(&lines), "READY", "round {round}");
        // The kill comes only after the child wrote a new round (its chat is
        // committed), so the progress below does not depend on the speed of the disk.
        assert_eq!(next_line(&lines), "WROTE", "round {round}");
        // A different moment in every round (xorshift), 10 to 400 ms after that.
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        std::thread::sleep(Duration::from_millis(10 + seed % 390));
        child.kill();
        // A journal with content is a transaction cut off in the middle.
        if journal.metadata().is_ok_and(|m| m.len() > 0) {
            inside_a_transaction += 1;
        }

        let turns = run(async {
            let store = SqliteStore::open(&db).await.expect("opens after the abort");
            check_after_kill(&store, round).await
        });
        let conn = rusqlite::Connection::open(&db).unwrap();
        let integrity: String = conn
            .query_row("PRAGMA integrity_check", [], |r| r.get(0))
            .unwrap();
        assert_eq!(integrity, "ok", "round {round}");
        drop(conn);
        assert!(
            turns > last_turns,
            "round {round}: the child made progress ({last_turns} -> {turns} turns)"
        );
        last_turns = turns;
    }
    eprintln!(
        "{last_turns} rounds written, {inside_a_transaction} of 24 aborts inside a transaction"
    );
    assert!(
        last_turns >= 24,
        "the child made progress: {last_turns} rounds"
    );
    assert!(inside_a_transaction >= 1, "no abort hit a transaction");
}
