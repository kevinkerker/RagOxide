//! Persistence of documents, chats and settings in SQLite (package G, implements
//! [`kb_core::Store`]).
//!
//! [`SqliteStore`] keeps the library (document records, which are also the ingestion
//! jobs), the parsed contents, the chats and the settings in one SQLite database file,
//! so that they survive a restart and a crash.
//!
//! - **Location:** the caller passes the path of the database file; missing folders are
//!   created. Every file the store writes (database, rollback journal, the backup before
//!   a migration) lies in the folder of the database; SQLite keeps its temporary data in
//!   memory (`PRAGMA temp_store = MEMORY`).
//! - **One process:** the store takes an exclusive lock of the operating system on the
//!   database file for as long as it is open (`PRAGMA locking_mode = EXCLUSIVE`). A
//!   second opening, from another process or from the same one, fails with
//!   [`Error::Store`](kb_core::Error::Store) saying that the database is in use. The
//!   lock ends with the process, also after a crash (ADR 0122).
//! - **Executor:** every SQLite call runs on one database thread owned by the store;
//!   the async methods only hand work to it and wait for the result, so they never
//!   block the async executor. Each write is one SQLite transaction that the thread
//!   finishes even if the caller drops the future.
//! - **Deleting overwrites:** `PRAGMA secure_delete = ON` and a rollback journal that
//!   is truncated after each transaction, so the text of a deleted chat or document is
//!   not left in the files (ADR 0120). After a write that was cut off by a crash, the
//!   next opening overwrites every free page with zeros, so that the text of the
//!   unfinished write is not left either (ADR 0142).
//! - **Dropping waits:** dropping the store blocks until the calls already handed over
//!   are done and the file is closed; drop it outside the threads of an async executor.
//! - **Schema versions:** the database carries its schema version (`user_version`) and
//!   the application id of this package. Older versions are migrated after a backup
//!   copy `<file name>.v<old version>.bak`, which is removed at the first successful
//!   opening afterwards; newer versions are refused (ADR 0119, ADR 0128).
//!
//! The layout of the tables and the known limits are described in the package README.

mod codec;
mod error;
mod open;
mod ops;
mod schema;
mod store;
mod worker;

pub use store::SqliteStore;

/// Schema version of the databases this package writes and reads.
pub const SCHEMA_VERSION: u32 = schema::CURRENT.version;

#[cfg(test)]
mod tests;
