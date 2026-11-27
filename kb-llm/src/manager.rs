//! The one server of a generator: which level is loaded, who uses it, starting,
//! switching, crash and idle (F-F4 to F-F6, ADR 0055, ADR 0170).
//!
//! Every call that needs the server holds a [`Lease`] while it runs: `count_tokens`
//! from its start to its return, `generate` until its stream ends, is cancelled or is
//! dropped. A call for another level waits until no lease on the loaded server is left,
//! then stops that server (and waits for the process to end) before it starts its own;
//! while it waits, new calls for the loaded level wait behind it, so it is not starved.
//! A start runs in its own task: a caller that is dropped while the model loads leaves
//! the start running (F-F6). A server of `Balanced` or `Thorough` without a lease for
//! the idle time is stopped (ADR 0170).

use std::sync::{Arc, Mutex, MutexGuard};

use kb_core::{AnswerLevel, CancellationToken, Error, Result};
use tokio::sync::{Notify, watch};

use crate::config::Resolved;
use crate::server::{Exit, Launch, QUOTED_STDERR_BYTES, Readiness, ServerProcess, is_bind_failure};

/// Attempts to start a server when the port turns out to be taken: the first and up to
/// three more (F-F4).
const PORT_ATTEMPTS: usize = 4;

/// Result of a start, shared with every caller waiting for it.
type StartResult = Option<std::result::Result<Arc<ServerProcess>, String>>;

/// State shared by the generator, its streams and its tasks.
#[derive(Debug)]
pub(crate) struct Shared {
    pub(crate) config: Resolved,
    pub(crate) client: reqwest::Client,
    state: Mutex<State>,
    changed: Notify,
    activity: watch::Sender<u64>,
    shutdown: CancellationToken,
}

#[derive(Debug)]
struct State {
    slot: Slot,
    /// Identifies the current slot; leases of an older slot no longer count.
    slot_id: u64,
    /// Leases on the current slot (callers waiting for its start included).
    users: usize,
    /// Callers waiting to replace the current slot by another level.
    switch_waiters: usize,
}

#[derive(Debug)]
enum Slot {
    Empty,
    Starting {
        level: AnswerLevel,
        abort: CancellationToken,
        ready: watch::Receiver<StartResult>,
    },
    Running {
        level: AnswerLevel,
        server: Arc<ServerProcess>,
    },
    /// The server of the slot is being stopped; the slot becomes `Empty` once the
    /// process has ended.
    Stopping,
}

/// What a caller does next, decided under the lock.
enum Step {
    Use(Lease),
    Join(Claim, watch::Receiver<StartResult>),
    Wait { loads: bool },
}

impl Shared {
    pub(crate) fn new(config: Resolved) -> Result<Self> {
        let client = reqwest::Client::builder()
            .no_proxy()
            .connect_timeout(std::time::Duration::from_secs(5))
            .pool_max_idle_per_host(0)
            .build()
            .map_err(|e| Error::Generator(format!("cannot build the HTTP client: {e}")))?;
        Ok(Self {
            config,
            client,
            state: Mutex::new(State {
                slot: Slot::Empty,
                slot_id: 0,
                users: 0,
                switch_waiters: 0,
            }),
            changed: Notify::new(),
            activity: watch::channel(0).0,
            shutdown: CancellationToken::new(),
        })
    }

    fn lock(&self) -> MutexGuard<'_, State> {
        match self.state.lock() {
            Ok(state) => state,
            Err(poisoned) => poisoned.into_inner(),
        }
    }

    /// Wakes everyone waiting for a change of the state.
    fn changed(&self) {
        self.changed.notify_waiters();
    }

    /// The level whose server is running, if any.
    pub(crate) fn loaded(&self) -> Option<(AnswerLevel, Option<u32>)> {
        match &self.lock().slot {
            Slot::Running { level, server } if server.exited_now().is_none() => {
                Some((*level, server.pid()))
            }
            _ => None,
        }
    }

    /// Stops the server and every start, without waiting (the generator is dropped).
    pub(crate) fn shut_down(&self) {
        self.shutdown.cancel();
        match &self.lock().slot {
            Slot::Running { server, .. } => server.kill(),
            Slot::Starting { abort, .. } => abort.cancel(),
            Slot::Empty | Slot::Stopping => {}
        }
        self.changed();
    }

    /// A lease on a ready server of `level`, starting or loading it if needed.
    /// `on_load` is called once, before waiting, if the server has to start or load the
    /// model of `level`. Dropping the future gives up the wait; a start keeps running.
    pub(crate) async fn acquire(
        self: &Arc<Self>,
        level: AnswerLevel,
        on_load: &mut (dyn FnMut() + Send),
    ) -> Result<Lease> {
        let mut announced = false;
        let mut announce = |loads: bool| {
            if loads && !announced {
                announced = true;
                on_load();
            }
        };
        let mut mark = SwitchMark {
            shared: self.clone(),
            counted: false,
        };
        loop {
            let notified = self.changed.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            let step = {
                let mut state = self.lock();
                self.step(&mut state, level, &mut mark.counted)?
            };
            match step {
                Step::Use(lease) => return Ok(lease),
                Step::Join(claim, ready) => {
                    announce(true);
                    return self.join(claim, ready).await;
                }
                Step::Wait { loads } => {
                    announce(loads);
                    tokio::select! {
                        () = notified => {}
                        () = self.shutdown.cancelled() => {}
                    }
                }
            }
        }
    }

    /// One decision of [`Shared::acquire`].
    fn step(
        self: &Arc<Self>,
        state: &mut State,
        level: AnswerLevel,
        counted: &mut bool,
    ) -> Result<Step> {
        if *counted {
            state.switch_waiters -= 1;
            *counted = false;
        }
        loop {
            if self.shutdown.is_cancelled() {
                return Err(Error::Generator("the generator has been dropped".into()));
            }
            match &state.slot {
                Slot::Running { server, .. } if server.exited_now().is_some() => {
                    state.slot = Slot::Empty;
                }
                Slot::Running { level: loaded, .. } | Slot::Starting { level: loaded, .. }
                    if *loaded == level && state.switch_waiters > 0 =>
                {
                    return Ok(Step::Wait { loads: false });
                }
                Slot::Running {
                    level: loaded,
                    server,
                } if *loaded == level => {
                    let server = server.clone();
                    let claim = self.claim(state);
                    return Ok(Step::Use(Lease { claim, server }));
                }
                Slot::Starting {
                    level: loaded,
                    ready,
                    ..
                } if *loaded == level => {
                    let ready = ready.clone();
                    let claim = self.claim(state);
                    return Ok(Step::Join(claim, ready));
                }
                Slot::Stopping => return Ok(Step::Wait { loads: true }),
                Slot::Empty => {
                    // The caller that starts a server joins it at once, ahead of
                    // callers waiting to switch.
                    let ready = self.begin_start(state, level);
                    let claim = self.claim(state);
                    return Ok(Step::Join(claim, ready));
                }
                Slot::Running { .. } | Slot::Starting { .. } => {
                    state.switch_waiters += 1;
                    *counted = true;
                    if state.users == 0 {
                        self.begin_stop(state);
                    }
                    return Ok(Step::Wait { loads: true });
                }
            }
        }
    }

    /// Counts a new lease on the current slot.
    fn claim(self: &Arc<Self>, state: &mut State) -> Claim {
        state.users += 1;
        self.activity.send_modify(|epoch| *epoch += 1);
        Claim {
            shared: self.clone(),
            slot_id: state.slot_id,
        }
    }

    /// Waits for the start that `claim` joined.
    async fn join(
        self: &Arc<Self>,
        claim: Claim,
        mut ready: watch::Receiver<StartResult>,
    ) -> Result<Lease> {
        let result = tokio::select! {
            result = ready.wait_for(Option::is_some) => result.ok().and_then(|r| (*r).clone()),
            () = self.shutdown.cancelled() => {
                return Err(Error::Generator("the generator has been dropped".into()));
            }
        };
        match result {
            Some(Ok(server)) => Ok(Lease { claim, server }),
            Some(Err(message)) => Err(Error::Generator(message)),
            None => Err(Error::Generator(
                "the start of the answer model server ended unexpectedly".into(),
            )),
        }
    }

    /// Starts the server of `level` in its own task; returns the receiver of its result.
    fn begin_start(
        self: &Arc<Self>,
        state: &mut State,
        level: AnswerLevel,
    ) -> watch::Receiver<StartResult> {
        state.slot_id += 1;
        state.users = 0;
        // Cancelled by a switch and, as a child, by dropping the generator.
        let abort = self.shutdown.child_token();
        let (done, ready) = watch::channel(None);
        state.slot = Slot::Starting {
            level,
            abort: abort.clone(),
            ready: ready.clone(),
        };
        tokio::spawn(start_task(self.clone(), level, state.slot_id, abort, done));
        ready
    }

    /// Stops the current slot (no lease left) in its own task.
    fn begin_stop(self: &Arc<Self>, state: &mut State) {
        match std::mem::replace(&mut state.slot, Slot::Stopping) {
            Slot::Running { server, .. } => {
                let shared = self.clone();
                let slot_id = state.slot_id;
                tokio::spawn(async move {
                    server.stop().await;
                    shared.stopped(slot_id);
                });
            }
            // The start task sees `Stopping`, stops its process and empties the slot.
            Slot::Starting { abort, .. } => abort.cancel(),
            other => state.slot = other,
        }
    }

    /// The process of slot `slot_id` has ended after a stop.
    fn stopped(&self, slot_id: u64) {
        let mut state = self.lock();
        if state.slot_id == slot_id && matches!(state.slot, Slot::Stopping) {
            state.slot = Slot::Empty;
        }
        drop(state);
        self.changed();
    }

    fn release(&self, slot_id: u64) {
        let mut state = self.lock();
        if state.slot_id == slot_id && state.users > 0 {
            state.users -= 1;
        }
        self.activity.send_modify(|epoch| *epoch += 1);
        drop(state);
        self.changed();
    }
}

/// A counted use of a slot; released on drop.
#[derive(Debug)]
pub(crate) struct Claim {
    shared: Arc<Shared>,
    slot_id: u64,
}

impl Drop for Claim {
    fn drop(&mut self) {
        self.shared.release(self.slot_id);
    }
}

/// A ready server and the right to use it until the lease is dropped.
#[derive(Debug)]
pub(crate) struct Lease {
    claim: Claim,
    pub(crate) server: Arc<ServerProcess>,
}

impl Lease {
    /// The shared state of the generator.
    pub(crate) fn shared(&self) -> &Arc<Shared> {
        &self.claim.shared
    }
}

/// Undoes the count of a caller waiting to switch levels when its wait is given up.
struct SwitchMark {
    shared: Arc<Shared>,
    counted: bool,
}

impl Drop for SwitchMark {
    fn drop(&mut self) {
        if self.counted {
            let mut state = self.shared.lock();
            state.switch_waiters -= 1;
            drop(state);
            self.shared.changed();
        }
    }
}

/// Starts the server of `level` for slot `slot_id` and publishes the result.
async fn start_task(
    shared: Arc<Shared>,
    level: AnswerLevel,
    slot_id: u64,
    abort: CancellationToken,
    done: watch::Sender<StartResult>,
) {
    let result = launch(&shared, level, &abort).await;
    let mut stop = None;
    {
        let mut state = shared.lock();
        let current = state.slot_id == slot_id;
        match (&result, current, &state.slot) {
            // `shut_down` takes the lock as well: either it has run (cancelled) or it
            // will see the running server and kill it.
            (Ok(server), true, Slot::Starting { .. }) if !shared.shutdown.is_cancelled() => {
                state.slot = Slot::Running {
                    level,
                    server: server.clone(),
                };
            }
            (Ok(server), true, Slot::Starting { .. }) => {
                state.slot = Slot::Stopping;
                stop = Some(server.clone());
            }
            (Ok(server), _, _) => stop = Some(server.clone()),
            (Err(_), true, Slot::Starting { .. } | Slot::Stopping) => state.slot = Slot::Empty,
            (Err(_), _, _) => {}
        }
    }
    if let Some(server) = stop {
        server.stop().await;
        shared.stopped(slot_id);
    }
    let running = match &result {
        Ok(server) if server.exited_now().is_none() => Some(server.clone()),
        _ => None,
    };
    let _ = done.send(Some(result));
    shared.changed();
    if let Some(server) = running {
        supervise(shared, level, slot_id, server).await;
    }
}

/// Starts a process and waits until it is ready; retries with another port if the port
/// is taken (the server cannot bind it, or another process answers there with another
/// key). Error texts name cause, exit code and the last lines of the error output,
/// never a prompt (no prompt has reached a server that is not ready).
async fn launch(
    shared: &Arc<Shared>,
    level: AnswerLevel,
    abort: &CancellationToken,
) -> std::result::Result<Arc<ServerProcess>, String> {
    let config = &shared.config;
    // File metadata is a system call: on the blocking pool (F05).
    let files = {
        let shared = shared.clone();
        tokio::task::spawn_blocking(move || shared.config.model_files(level))
            .await
            .map_err(|e| e.to_string())?
            .map_err(|e| e.to_string())?
    };
    let launch = Launch {
        server: &config.server,
        files: &files,
        context_tokens: config.context_tokens,
        threads: config.threads,
    };
    let mut last_bind_error = String::new();
    for _ in 0..PORT_ATTEMPTS {
        let server = ServerProcess::spawn(&launch).await.map_err(|e| {
            format!(
                "cannot start the answer model server {}: {e}",
                config.server.display()
            )
        })?;
        let port = server.port();
        match server.wait_ready(&shared.client, abort).await {
            Readiness::Ready => return Ok(server),
            Readiness::Foreign => {
                server.stop().await;
                last_bind_error = format!(
                    "port {port}: another process answers there and rejects the key of this start"
                );
                continue;
            }
            Readiness::Stopped => {
                server.stop().await;
                return Err("the start of the answer model server was called off".into());
            }
            Readiness::Exited(exit) => {
                let stderr = server.final_stderr_tail(QUOTED_STDERR_BYTES).await;
                if is_bind_failure(&stderr) {
                    last_bind_error = stderr;
                    continue;
                }
                return Err(exited_while_starting(exit, &stderr));
            }
        }
    }
    Err(format!(
        "the answer model server found no free port in {PORT_ATTEMPTS} attempts; last lines of its error output:\n{last_bind_error}"
    ))
}

fn exited_while_starting(exit: Exit, stderr: &str) -> String {
    if stderr.is_empty() {
        format!("the answer model server ended while starting ({exit}) without error output")
    } else {
        format!(
            "the answer model server ended while starting ({exit}); last lines of its error output:\n{stderr}"
        )
    }
}

/// Watches a running server: empties the slot when the process ends (the next call
/// starts it again) and unloads `Balanced` and `Thorough` after the idle time.
async fn supervise(
    shared: Arc<Shared>,
    level: AnswerLevel,
    slot_id: u64,
    server: Arc<ServerProcess>,
) {
    let unloads = level != AnswerLevel::Fast;
    let mut activity = shared.activity.subscribe();
    loop {
        let (idle, epoch) = {
            let state = shared.lock();
            if state.slot_id != slot_id || !matches!(state.slot, Slot::Running { .. }) {
                return;
            }
            (state.users == 0, *activity.borrow_and_update())
        };
        let idle_timer = tokio::time::sleep(shared.config.idle_timeout);
        tokio::select! {
            _ = server.wait_exit() => {
                let mut state = shared.lock();
                if state.slot_id == slot_id && matches!(state.slot, Slot::Running { .. }) {
                    state.slot = Slot::Empty;
                }
                drop(state);
                shared.changed();
                return;
            }
            () = shared.shutdown.cancelled() => return,
            changed = activity.changed() => {
                if changed.is_err() {
                    return;
                }
            }
            () = idle_timer, if unloads && idle => {
                let unload = {
                    let mut state = shared.lock();
                    let unchanged = *activity.borrow() == epoch;
                    let ours = state.slot_id == slot_id
                        && matches!(state.slot, Slot::Running { .. });
                    if ours && unchanged && state.users == 0 {
                        state.slot = Slot::Stopping;
                        true
                    } else {
                        false
                    }
                };
                if unload {
                    server.stop().await;
                    shared.stopped(slot_id);
                    return;
                }
            }
        }
    }
}
