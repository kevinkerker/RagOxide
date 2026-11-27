//! One `llama-server` process: arguments, start, pipes, job object, readiness and end.

use std::ffi::OsString;
use std::fmt;
use std::io;
use std::net::{Ipv4Addr, TcpListener};
use std::path::Path;
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use kb_core::CancellationToken;
use tokio::io::{AsyncRead, AsyncReadExt};
use tokio::sync::watch;

use crate::config::ModelFiles;
use crate::tail::Tail;

/// Bytes of the error output quoted in an error message (F14).
pub(crate) const QUOTED_STDERR_BYTES: usize = 4 * 1024;

/// How often `/health` is polled while the server loads.
const HEALTH_INTERVAL: Duration = Duration::from_millis(50);

/// Upper limit for one `/health` request; the server answers it at once, also while
/// loading.
const HEALTH_TIMEOUT: Duration = Duration::from_secs(2);

/// `CREATE_NO_WINDOW`: the server runs without a console window (F-F4, F31).
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// What the server is started with.
#[derive(Debug, Clone)]
pub(crate) struct Launch<'a> {
    pub(crate) server: &'a Path,
    pub(crate) files: &'a ModelFiles,
    pub(crate) context_tokens: u32,
    pub(crate) threads: u32,
}

/// The command line of the server (F-F2). Without `-b` (ADR 0101), without `--metrics`
/// and without any option that downloads; `-ngl 0` always (ADR 0160).
pub(crate) fn arguments(launch: &Launch<'_>, port: u16, key: &str) -> Vec<OsString> {
    let threads = launch.threads.to_string();
    let mut args: Vec<OsString> = vec![
        "-m".into(),
        launch.files.model.clone().into_os_string(),
        "-c".into(),
        launch.context_tokens.to_string().into(),
        "-t".into(),
        threads.clone().into(),
        "-tb".into(),
        threads.into(),
        "-ngl".into(),
        "0".into(),
        "-np".into(),
        "1".into(),
        "--cache-ram".into(),
        "0".into(),
        "--host".into(),
        "127.0.0.1".into(),
        "--port".into(),
        port.to_string().into(),
        "--no-webui".into(),
        "--api-key".into(),
        key.into(),
        "--no-slots".into(),
    ];
    if let Some(draft) = &launch.files.draft {
        args.extend([
            "--model-draft".into(),
            draft.clone().into_os_string(),
            "--spec-type".into(),
            "draft-mtp".into(),
            "--spec-draft-n-max".into(),
            "4".into(),
            "-ngld".into(),
            "0".into(),
        ]);
    }
    args
}

/// A random key for one start, 32 bytes as hex.
pub(crate) fn new_key() -> io::Result<String> {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).map_err(|e| io::Error::other(e.to_string()))?;
    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}

/// A port on 127.0.0.1 that is free right now.
pub(crate) fn free_port() -> io::Result<u16> {
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))?;
    Ok(listener.local_addr()?.port())
}

/// Whether the error output says that the server could not bind its port.
pub(crate) fn is_bind_failure(stderr: &str) -> bool {
    stderr.contains("couldn't bind")
}

/// How the process ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Exit {
    /// Exit code, `None` if unknown.
    pub(crate) code: Option<i32>,
}

impl fmt::Display for Exit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.code {
            Some(code) => write!(f, "exit code {code}"),
            None => f.write_str("exit code unknown"),
        }
    }
}

/// A started server process. Ends when [`ServerProcess::kill`] is called, when the
/// monitoring task is dropped (the child is killed on drop) and, through the job object,
/// when the app process ends.
#[derive(Debug)]
pub(crate) struct ServerProcess {
    port: u16,
    key: String,
    pid: Option<u32>,
    exit: watch::Receiver<Option<Exit>>,
    /// `true` once the pipes are read to their end (or after 500 ms) after the exit.
    drained: watch::Receiver<bool>,
    kill: CancellationToken,
    stderr: Arc<Mutex<Tail>>,
}

impl ServerProcess {
    /// Starts the server on a free port with a new key; returns once the process runs,
    /// without waiting for it to load. Choosing the port, creating the key, creating the
    /// process and putting it into the job object are system calls and run on the
    /// blocking pool, never on an executor thread (F05). Must be called inside a tokio
    /// runtime.
    pub(crate) async fn spawn(launch: &Launch<'_>) -> io::Result<Arc<Self>> {
        let server = launch.server.to_path_buf();
        let files = launch.files.clone();
        let (context_tokens, threads) = (launch.context_tokens, launch.threads);
        let started = tokio::task::spawn_blocking(move || {
            let launch = Launch {
                server: &server,
                files: &files,
                context_tokens,
                threads,
            };
            spawn_process(&launch)
        })
        .await
        .map_err(io::Error::other)??;
        let Started {
            mut child,
            #[cfg(windows)]
            job,
            port,
            key,
        } = started;

        let pid = child.id();
        let stderr = Arc::new(Mutex::new(Tail::default()));
        let stdout = Arc::new(Mutex::new(Tail::default()));
        let readers = [
            child
                .stdout
                .take()
                .map(|pipe| tokio::spawn(drain(pipe, stdout))),
            child
                .stderr
                .take()
                .map(|pipe| tokio::spawn(drain(pipe, stderr.clone()))),
        ];
        let (exit_tx, exit) = watch::channel(None);
        let (drained_tx, drained) = watch::channel(false);
        let kill = CancellationToken::new();
        let kill_requested = kill.clone();
        tokio::spawn(async move {
            #[cfg(windows)]
            let _job = job;
            let status = tokio::select! {
                status = child.wait() => status,
                () = kill_requested.cancelled() => {
                    let _ = child.start_kill();
                    child.wait().await
                }
            };
            // The end is known at once, so the next call starts a new server instead of
            // using the dead one (F29); the error output follows below.
            let code = status.ok().and_then(|s| s.code());
            let _ = exit_tx.send(Some(Exit { code }));
            // Let the readers take the last output, so that an error message quotes it.
            let _ = tokio::time::timeout(Duration::from_millis(500), async {
                for reader in readers.into_iter().flatten() {
                    let _ = reader.await;
                }
            })
            .await;
            let _ = drained_tx.send(true);
        });
        Ok(Arc::new(Self {
            port,
            key,
            pid,
            exit,
            drained,
            kill,
            stderr,
        }))
    }

    /// The port the server was started on.
    pub(crate) fn port(&self) -> u16 {
        self.port
    }

    /// The process id, `None` if the process ended before it was known.
    pub(crate) fn pid(&self) -> Option<u32> {
        self.pid
    }

    /// The API key of this start.
    pub(crate) fn key(&self) -> &str {
        &self.key
    }

    /// `http://127.0.0.1:<port><path>`.
    pub(crate) fn url(&self, path: &str) -> String {
        format!("http://127.0.0.1:{}{path}", self.port)
    }

    /// How the process ended, `None` while it runs.
    pub(crate) fn exited_now(&self) -> Option<Exit> {
        *self.exit.borrow()
    }

    /// Waits until the process has ended.
    pub(crate) async fn wait_exit(&self) -> Exit {
        let mut exit = self.exit.clone();
        match exit.wait_for(Option::is_some).await {
            Ok(value) => value.unwrap_or(Exit { code: None }),
            // The monitoring task is gone (runtime shut down); its child was killed.
            Err(_) => Exit { code: None },
        }
    }

    /// Asks for the process to be killed, without waiting.
    pub(crate) fn kill(&self) {
        self.kill.cancel();
    }

    /// Kills the process and waits until it has ended.
    pub(crate) async fn stop(&self) -> Exit {
        self.kill();
        self.wait_exit().await
    }

    /// The last lines of the error output after the process has ended and its pipes
    /// are read, at most `max` bytes.
    pub(crate) async fn final_stderr_tail(&self, max: usize) -> String {
        let mut drained = self.drained.clone();
        let _ = drained.wait_for(|d| *d).await;
        self.stderr_tail(max)
    }

    /// The last lines of the error output, at most `max` bytes.
    pub(crate) fn stderr_tail(&self, max: usize) -> String {
        match self.stderr.lock() {
            Ok(tail) => tail.last_lines(max),
            Err(poisoned) => poisoned.into_inner().last_lines(max),
        }
    }

    /// Whether the server on the port accepts the key of this start: `POST /tokenize`
    /// with an empty text, which answers 401 without the key. On Windows a second
    /// `llama-server` can bind a port that another one listens on, so `/health` 200 alone
    /// does not prove that the own process answers.
    async fn answers_with_key(&self, client: &reqwest::Client) -> bool {
        let body = r#"{"content":"","add_special":false}"#;
        client
            .post(self.url("/tokenize"))
            .bearer_auth(&self.key)
            .header("content-type", "application/json")
            .body(body)
            .timeout(HEALTH_TIMEOUT)
            .send()
            .await
            .is_ok_and(|r| r.status() == reqwest::StatusCode::OK)
    }

    /// Polls `/health` until it answers 200 and the server confirms the key of this
    /// start (ready), the process ends or `stop` fires (F-F3: `503 Loading model` means
    /// loading; a refused connection means the port is not open yet).
    pub(crate) async fn wait_ready(
        &self,
        client: &reqwest::Client,
        stop: &CancellationToken,
    ) -> Readiness {
        loop {
            if let Some(exit) = self.exited_now() {
                return Readiness::Exited(exit);
            }
            let health = client
                .get(self.url("/health"))
                .timeout(HEALTH_TIMEOUT)
                .send();
            tokio::select! {
                biased;
                () = stop.cancelled() => return Readiness::Stopped,
                exit = self.wait_exit() => return Readiness::Exited(exit),
                response = health => {
                    if response.is_ok_and(|r| r.status() == reqwest::StatusCode::OK) {
                        return tokio::select! {
                            biased;
                            () = stop.cancelled() => Readiness::Stopped,
                            exit = self.wait_exit() => Readiness::Exited(exit),
                            own = self.answers_with_key(client) => {
                                if own { Readiness::Ready } else { Readiness::Foreign }
                            }
                        };
                    }
                }
            }
            tokio::select! {
                biased;
                () = stop.cancelled() => return Readiness::Stopped,
                exit = self.wait_exit() => return Readiness::Exited(exit),
                () = tokio::time::sleep(HEALTH_INTERVAL) => {}
            }
        }
    }
}

/// A process started on the blocking pool.
struct Started {
    child: tokio::process::Child,
    #[cfg(windows)]
    job: win32job::Job,
    port: u16,
    key: String,
}

/// Port, key, process and job object; blocking, runs on the blocking pool.
fn spawn_process(launch: &Launch<'_>) -> io::Result<Started> {
    let port = free_port()?;
    let key = new_key()?;
    let mut command = std::process::Command::new(launch.server);
    command
        .args(arguments(launch, port, &key))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    // Settings from LLAMA_* variables (LLAMA_ARG_HOST, LLAMA_API_KEY, ...) would apply
    // to every option the command line does not name (F-F2).
    for (name, _) in std::env::vars_os() {
        if name
            .to_string_lossy()
            .to_ascii_uppercase()
            .starts_with("LLAMA_")
        {
            command.env_remove(&name);
        }
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    let mut command = tokio::process::Command::from(command);
    command.kill_on_drop(true);
    let mut child = command.spawn()?;
    #[cfg(windows)]
    let job = match assign_job(&child) {
        Ok(job) => job,
        Err(error) => {
            let _ = child.start_kill();
            return Err(error);
        }
    };
    Ok(Started {
        child,
        #[cfg(windows)]
        job,
        port,
        key,
    })
}

/// Result of [`ServerProcess::wait_ready`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Readiness {
    /// `/health` answered 200 and the server confirmed the key of this start.
    Ready,
    /// `/health` answered 200, but the server on the port did not accept the key of
    /// this start: another process answers there, the port counts as taken.
    Foreign,
    /// The process ended before it was ready.
    Exited(Exit),
    /// The start was called off.
    Stopped,
}

/// Reads a pipe to its end and keeps its last bytes; a pipe that is not read would stop
/// the server (F-F4).
async fn drain(mut pipe: impl AsyncRead + Unpin, tail: Arc<Mutex<Tail>>) {
    let mut buffer = vec![0u8; 16 * 1024];
    loop {
        match pipe.read(&mut buffer).await {
            Ok(0) | Err(_) => return,
            Ok(n) => match tail.lock() {
                Ok(mut tail) => tail.push(&buffer[..n]),
                Err(poisoned) => poisoned.into_inner().push(&buffer[..n]),
            },
        }
    }
}

/// Puts the child into a new job object that kills it when the last handle to the job
/// closes, that is when the monitoring task ends or the app process ends (F-F4).
#[cfg(windows)]
fn assign_job(child: &tokio::process::Child) -> io::Result<win32job::Job> {
    let mut limits = win32job::ExtendedLimitInfo::new();
    limits.limit_kill_on_job_close();
    let job = win32job::Job::create_with_limit_info(&limits).map_err(io::Error::other)?;
    let handle = child
        .raw_handle()
        .ok_or_else(|| io::Error::other("the server process ended at once"))?;
    job.assign_process(handle as isize)
        .map_err(io::Error::other)?;
    Ok(job)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn files(draft: bool) -> ModelFiles {
        ModelFiles {
            model: PathBuf::from(r"C:\Models\m\model.bin"),
            draft: draft.then(|| PathBuf::from(r"C:\Models\m\draft.bin")),
        }
    }

    fn args_of(draft: bool) -> Vec<String> {
        let files = files(draft);
        let launch = Launch {
            server: Path::new(r"C:\Server\llama-server.exe"),
            files: &files,
            context_tokens: 5120,
            threads: 6,
        };
        arguments(&launch, 4711, "secret")
            .into_iter()
            .map(|a| a.to_string_lossy().into_owned())
            .collect()
    }

    #[test]
    fn arguments_follow_f_f2() {
        let args = args_of(false);
        let expected = [
            "-m",
            r"C:\Models\m\model.bin",
            "-c",
            "5120",
            "-t",
            "6",
            "-tb",
            "6",
            "-ngl",
            "0",
            "-np",
            "1",
            "--cache-ram",
            "0",
            "--host",
            "127.0.0.1",
            "--port",
            "4711",
            "--no-webui",
            "--api-key",
            "secret",
            "--no-slots",
        ];
        assert_eq!(args, expected);
        let with_draft = args_of(true);
        assert_eq!(&with_draft[..expected.len()], expected);
        assert_eq!(
            &with_draft[expected.len()..],
            [
                "--model-draft",
                r"C:\Models\m\draft.bin",
                "--spec-type",
                "draft-mtp",
                "--spec-draft-n-max",
                "4",
                "-ngld",
                "0"
            ]
        );
    }

    #[test]
    fn keys_are_random_hex() {
        let a = new_key().unwrap();
        let b = new_key().unwrap();
        assert_eq!(a.len(), 64);
        assert!(a.bytes().all(|c| c.is_ascii_hexdigit()));
        assert_ne!(a, b);
    }

    #[test]
    fn bind_failure_is_recognised() {
        assert!(is_bind_failure(
            "E srv start: couldn't bind HTTP server socket, hostname: 127.0.0.1, port: 1"
        ));
        assert!(!is_bind_failure("failed to load model"));
    }
}
