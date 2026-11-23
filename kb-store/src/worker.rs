//! The database thread: owns the only connection and runs every SQLite call.
//!
//! The async methods of the store send a job over a channel and await its result on a
//! one-shot channel, so no SQLite call runs on a thread of the async executor. Jobs run
//! one after another; each write is one transaction. A job that has been sent runs to
//! its end even if the caller drops the future (its result is then discarded), so a
//! write is applied completely or not at all and leaves no lock behind.

use std::panic::{self, AssertUnwindSafe};
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread::{self, JoinHandle};

use futures::channel::oneshot;
use kb_core::{Error, Result};
use rusqlite::Connection;

use crate::open;
use crate::schema::Schema;

/// Work for the database thread.
type Job = Box<dyn FnOnce(&mut Connection) + Send>;

/// Handle of the database thread. Dropping it lets the thread finish the jobs already
/// sent, close the connection (which releases the lock of the file) and end; the drop
/// waits for that.
pub(crate) struct Worker {
    jobs: Option<mpsc::Sender<Job>>,
    thread: Option<JoinHandle<()>>,
}

impl Worker {
    /// Starts the thread, opens the database on it and waits for the outcome.
    pub(crate) async fn start(path: PathBuf, schema: &'static Schema) -> Result<Self> {
        let (jobs, receiver) = mpsc::channel::<Job>();
        let (ready, opened) = oneshot::channel::<Result<()>>();
        let thread = thread::Builder::new()
            .name("kb-store".into())
            .spawn(move || run(&path, schema, &receiver, ready))
            .map_err(|e| Error::Store(format!("cannot start the database thread: {e}")))?;
        let worker = Self {
            jobs: Some(jobs),
            thread: Some(thread),
        };
        match opened.await {
            Ok(Ok(())) => Ok(worker),
            Ok(Err(error)) => Err(error),
            Err(_) => Err(Error::Store(
                "the database thread stopped while opening the database".into(),
            )),
        }
    }

    /// Runs `work` on the database thread and returns its result.
    pub(crate) async fn call<T, F>(&self, work: F) -> Result<T>
    where
        T: Send + 'static,
        F: FnOnce(&mut Connection) -> Result<T> + Send + 'static,
    {
        let (sender, result) = oneshot::channel();
        let job: Job = Box::new(move |conn| {
            // The caller may have dropped its future; the work is done all the same.
            let _ = sender.send(work(conn));
        });
        self.jobs
            .as_ref()
            .ok_or_else(|| Error::Store("the store is closed".into()))?
            .send(job)
            .map_err(|_| Error::Store("the database thread is not running".into()))?;
        result.await.map_err(|_| {
            Error::Store("the database thread failed while handling the call".into())
        })?
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        self.jobs.take();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

/// Body of the database thread.
fn run(
    path: &std::path::Path,
    schema: &'static Schema,
    jobs: &mpsc::Receiver<Job>,
    ready: oneshot::Sender<Result<()>>,
) {
    let opened = panic::catch_unwind(|| open::open(path, schema)).unwrap_or_else(|_| {
        Err(Error::Store(format!(
            "opening the database {} failed unexpectedly",
            path.display()
        )))
    });
    let mut conn = match opened {
        Ok(conn) => conn,
        Err(error) => {
            let _ = ready.send(Err(error));
            return;
        }
    };
    if ready.send(Ok(())).is_err() {
        // The caller gave up waiting; close again.
        return;
    }
    while let Ok(job) = jobs.recv() {
        // A panic in a job (for example in a settings update of the caller) must not
        // end the thread; its open transaction is rolled back.
        if panic::catch_unwind(AssertUnwindSafe(|| job(&mut conn))).is_err()
            && !conn.is_autocommit()
        {
            let _ = conn.execute_batch("ROLLBACK");
        }
    }
}
