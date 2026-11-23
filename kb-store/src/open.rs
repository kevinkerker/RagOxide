//! Opening a database: lock, schema check, creation, backup and migration (ADR 0119,
//! 0122, 0128).
//!
//! Nothing here writes to an existing database before its header has been read and
//! accepted, so an opening that fails leaves the file byte for byte as it was. A
//! migration runs in one transaction; if it fails, SQLite rolls it back from the journal.

use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use kb_core::{Error, Result};
use rusqlite::backup::{Backup, StepResult};
use rusqlite::{Connection, OpenFlags, Transaction, TransactionBehavior};

use crate::error::{in_use, is_full, is_locked, opening, sqlite};
use crate::schema::{APPLICATION_ID, Schema};

/// What the header of the database says, relative to the schema of this build.
enum Found {
    /// No tables, version 0: a new or empty file.
    Empty,
    /// The schema of this build.
    Current,
    /// An older schema of this package.
    Older(u32),
}

/// Opens the database at `path` with `schema`, creating folders, file and tables as
/// needed and migrating an older schema. Runs on the database thread.
pub(crate) fn open(path: &Path, schema: &Schema) -> Result<Connection> {
    if !path.is_absolute() {
        return Err(Error::InvalidInput(format!(
            "database path is not absolute: {}",
            path.display()
        )));
    }
    let (Some(folder), Some(_)) = (path.parent(), path.file_name()) else {
        return Err(Error::InvalidInput(format!(
            "database path has no file name: {}",
            path.display()
        )));
    };
    fs::create_dir_all(folder).map_err(|e| {
        Error::Store(format!(
            "cannot create the folder {}: {e}",
            folder.display()
        ))
    })?;
    let existed = path.symlink_metadata().is_ok();
    let result = open_file(path, schema);
    if result.is_err() && !existed {
        // A file this call created and could not fill is empty; leave nothing behind.
        if path.metadata().is_ok_and(|m| m.len() == 0) {
            let _ = fs::remove_file(path);
            let _ = fs::remove_file(sibling(path, "-journal"));
        }
    }
    result
}

fn open_file(path: &Path, schema: &Schema) -> Result<Connection> {
    let flags = OpenFlags::SQLITE_OPEN_READ_WRITE
        | OpenFlags::SQLITE_OPEN_CREATE
        | OpenFlags::SQLITE_OPEN_NO_MUTEX;
    // Opening reads nothing yet: SQLite looks at the header and at a journal only with
    // the first access below.
    let mut conn = Connection::open_with_flags(path, flags).map_err(|e| opening(path, e))?;
    // SQLite opens a file it may not write (read-only attribute or rights) for reading
    // instead. Such a file is not opened at all (ADR 0144, ADR 0122 without exception):
    // checked before a journal is rolled back, a marker is set, a backup is written or
    // removed, or a migration begins, so that every file stays as it is.
    if conn.is_readonly("main").map_err(sqlite)? {
        return Err(Error::Store(format!(
            "the database {} is read-only (read-only attribute or missing write permission) and was not opened; nothing was changed. Remove the write protection and open it again",
            path.display()
        )));
    }

    // A journal with content is a transaction that was cut off (the journal is truncated
    // after every completed one). Looked at before SQLite rolls it back, which empties
    // the journal. So that the overwrite of the free pages (ADR 0142) is not forgotten
    // when it fails or is cut off in turn, a marker file is set first and removed only
    // after the overwrite succeeded. If the marker cannot be set, nothing is read and
    // the journal stays as it is.
    let marker = overwrite_marker(path);
    let journal_has_content = sibling(path, "-journal")
        .metadata()
        .is_ok_and(|m| m.len() > 0);
    if journal_has_content && !marker.is_file() {
        set_marker(&marker).map_err(|e| {
            Error::Store(format!(
                "cannot set the marker {} for overwriting free pages after an interrupted write; the database was not opened: {e}",
                marker.display()
            ))
        })?;
    }
    let overwrite = marker.is_file();
    // The only connection never waits for a lock: a lock held elsewhere is another
    // process (or a second store in this process), reported at once.
    conn.busy_timeout(Duration::ZERO).map_err(sqlite)?;
    // Before the first access, so that every lock taken is kept until the connection
    // closes and the operating system releases it with the process (ADR 0122).
    conn.pragma_update(None, "locking_mode", "EXCLUSIVE")
        .map_err(sqlite)?;
    // Temporary tables, indices and statement journals stay in memory, never in the
    // temporary folder of the system.
    conn.pragma_update(None, "temp_store", "MEMORY")
        .map_err(sqlite)?;

    let found = read_header(&conn, path, schema)?;

    // Take the exclusive lock now, before anything is written (a backup included). It
    // fails if another connection holds any lock on the file.
    match conn.execute_batch("BEGIN EXCLUSIVE; COMMIT;") {
        Ok(()) => {}
        Err(e) if is_locked(&e) => return Err(in_use(path, &e)),
        Err(e) => return Err(opening(path, e)),
    }

    // Applies to writing only and leaves the file unchanged: a rollback journal that is
    // truncated after every transaction, writes that reach the disk before a commit
    // returns, and deleted content overwritten with zeros (ADR 0120).
    conn.pragma_update(None, "journal_mode", "TRUNCATE")
        .map_err(sqlite)?;
    conn.pragma_update(None, "synchronous", "FULL")
        .map_err(sqlite)?;
    conn.pragma_update(None, "secure_delete", "ON")
        .map_err(sqlite)?;

    // After a cut-off write, before anything else (a backup would copy free pages too).
    // If the overwrite fails (a full disk, for example), SQLite has rolled it back and
    // the file is intact: the database is opened all the same, and the marker stays, so
    // that every later opening tries again until it succeeds (ADR 0142). Until then text
    // of the cut-off write can remain in free pages.
    if overwrite && overwrite_free_pages(&mut conn, ZERO_CHUNK).is_ok() {
        // A marker that cannot be removed only repeats the overwrite at the next opening.
        let _ = fs::remove_file(&marker);
    }

    match found {
        Found::Empty => create(&mut conn, schema)?,
        Found::Older(version) => {
            back_up(&conn, path, version)?;
            migrate(&mut conn, schema, version)?;
        }
        Found::Current => {
            check_schema(&conn, schema)?;
            remove_backup_after_check(&mut conn, path, schema)?;
        }
    }
    Ok(conn)
}

/// Reads application id, schema version and number of tables, and decides.
fn read_header(conn: &Connection, path: &Path, schema: &Schema) -> Result<Found> {
    let read = || -> rusqlite::Result<(i32, u32, i64)> {
        let application_id = conn.pragma_query_value(None, "application_id", |r| r.get(0))?;
        let version = conn.pragma_query_value(None, "user_version", |r| r.get(0))?;
        let tables = conn.query_row("SELECT count(*) FROM sqlite_schema", [], |r| r.get(0))?;
        Ok((application_id, version, tables))
    };
    let (application_id, version, tables) = read().map_err(|e| opening(path, e))?;
    if tables == 0 && version == 0 && application_id == 0 {
        return Ok(Found::Empty);
    }
    if application_id != APPLICATION_ID {
        return Err(Error::Store(format!(
            "{} is an SQLite database of another application (application id {application_id:#x}), not a database of this application",
            path.display()
        )));
    }
    if version > schema.version {
        return Err(Error::Store(format!(
            "the database {} has schema version {version}, newer than the supported schema version {}; it was written by a newer version of the application and is left unchanged",
            path.display(),
            schema.version
        )));
    }
    if version == schema.version {
        return Ok(Found::Current);
    }
    if version == 0 {
        return Err(Error::Store(format!(
            "the database {} has no schema version",
            path.display()
        )));
    }
    Ok(Found::Older(version))
}

/// Creates the tables of `schema` in an empty database, in one transaction.
fn create(conn: &mut Connection, schema: &Schema) -> Result<()> {
    let tx = conn
        .transaction_with_behavior(TransactionBehavior::Exclusive)
        .map_err(sqlite)?;
    (schema.create)(&tx).map_err(sqlite)?;
    tx.pragma_update(None, "application_id", APPLICATION_ID)
        .map_err(sqlite)?;
    tx.pragma_update(None, "user_version", schema.version)
        .map_err(sqlite)?;
    check_schema(&tx, schema)?;
    tx.commit().map_err(sqlite)
}

/// Migrates from `version` to `schema.version` in one transaction, step by step.
fn migrate(conn: &mut Connection, schema: &Schema, version: u32) -> Result<()> {
    let failed = |e: rusqlite::Error| {
        let message = format!(
            "migration from schema version {version} to {} failed: {e}",
            schema.version
        );
        if is_full(&e) {
            Error::StorageFull(message)
        } else {
            Error::Store(message)
        }
    };
    let tx = conn
        .transaction_with_behavior(TransactionBehavior::Exclusive)
        .map_err(failed)?;
    for from in version..schema.version {
        let step = schema
            .migrations
            .iter()
            .find(|m| m.from == from)
            .ok_or_else(|| {
                Error::Store(format!(
                    "no migration from schema version {from} to {}",
                    from + 1
                ))
            })?;
        (step.apply)(&tx).map_err(failed)?;
    }
    tx.pragma_update(None, "user_version", schema.version)
        .map_err(failed)?;
    check_schema(&tx, schema)?;
    tx.commit().map_err(failed)
}

/// Largest zero-filled value written in one row by [`overwrite_free_pages`].
pub(crate) const ZERO_CHUNK: i64 = 64 << 20;

/// The marker file next to the database that says the free pages are still to be
/// overwritten after an interrupted write: `<file name>-overwrite`.
pub(crate) fn overwrite_marker(path: &Path) -> PathBuf {
    sibling(path, "-overwrite")
}

/// Creates the marker and writes it to the disk before SQLite touches the journal.
fn set_marker(marker: &Path) -> std::io::Result<()> {
    let file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(marker)?;
    file.sync_all()?;
    // Also the entry in the folder, where the platform allows opening a folder.
    if let Some(folder) = marker.parent()
        && let Ok(folder) = fs::File::open(folder)
    {
        let _ = folder.sync_all();
    }
    Ok(())
}

/// Test hook: the backup of the database at this path fails like on a full disk (the
/// copy may not grow beyond one page). Keyed by path, so that other tests running at
/// the same time are not affected.
#[cfg(test)]
pub(crate) static FULL_BACKUP_OF: std::sync::Mutex<Option<PathBuf>> = std::sync::Mutex::new(None);

/// Test hook: makes [`overwrite_free_pages`] fail like on a full disk.
#[cfg(test)]
pub(crate) static FAIL_OVERWRITE: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// Test hook: ends the process in the middle of [`overwrite_free_pages`], after the
/// zeros were written and before the transaction ends.
#[cfg(test)]
pub(crate) static EXIT_DURING_OVERWRITE: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// Overwrites every page on the free list with zeros (ADR 0142).
///
/// When a write is cut off, SQLite rolls it back from the journal at the next opening.
/// Pages the write had taken from the free list are not in the journal (they held
/// nothing), so they go back on the free list with the text of the write still in them;
/// `secure_delete` does not see them, because nothing is deleted. This fills a helper
/// table with zero bytes until every free page is used, then drops it, which with
/// `secure_delete` zeroes the pages once more. Everything stays in the database file and
/// its journal; no temporary file and no copy of the database is made (unlike
/// `VACUUM`). The work is one transaction; if it fails or is cut off, SQLite rolls it
/// back and the helper table is gone again, and the marker file (see `open_file`) makes
/// the next opening overwrite the pages again; the opening itself goes on.
fn overwrite_free_pages(conn: &mut Connection, chunk: i64) -> rusqlite::Result<()> {
    let free: i64 = conn.pragma_query_value(None, "freelist_count", |r| r.get(0))?;
    if free == 0 {
        return Ok(());
    }
    #[cfg(test)]
    if FAIL_OVERWRITE.load(std::sync::atomic::Ordering::SeqCst) {
        // No page may be added during the overwrite; the limit is lifted afterwards, as
        // space freed on a full disk would be.
        let limit: i64 = conn.pragma_query_value(None, "max_page_count", |r| r.get(0))?;
        let pages: i64 = conn.pragma_query_value(None, "page_count", |r| r.get(0))?;
        conn.pragma_update(None, "max_page_count", pages)?;
        let result = zero_free_pages(conn, chunk);
        conn.pragma_update(None, "max_page_count", limit)?;
        return result;
    }
    zero_free_pages(conn, chunk)
}

/// The transaction of [`overwrite_free_pages`].
fn zero_free_pages(conn: &mut Connection, chunk: i64) -> rusqlite::Result<()> {
    let tx = conn.transaction_with_behavior(TransactionBehavior::Exclusive)?;
    tx.execute_batch("CREATE TABLE store_overwrite (zeros BLOB NOT NULL) STRICT;")?;
    fill_free_pages(&tx, chunk)?;
    #[cfg(test)]
    if EXIT_DURING_OVERWRITE.load(std::sync::atomic::Ordering::SeqCst) {
        std::process::exit(3);
    }
    tx.execute_batch("DROP TABLE store_overwrite;")?;
    tx.commit()
}

/// Fills the table `store_overwrite` with zero bytes, in values of at most `chunk`
/// bytes, until the free list is used up.
pub(crate) fn fill_free_pages(tx: &Transaction<'_>, chunk: i64) -> rusqlite::Result<()> {
    let free: i64 = tx.pragma_query_value(None, "freelist_count", |r| r.get(0))?;
    let page: i64 = tx.pragma_query_value(None, "page_size", |r| r.get(0))?;
    // An overflow page carries its page size minus a 4-byte pointer, so `free` pages of
    // bytes need about `free` pages. Part of each value stays in its cell on a leaf page
    // instead of an overflow page; how much depends on the size of the value and on
    // SQLite's rules for cells. The one page more is a margin against that, not a
    // measured need: the test `the_zeros_use_up_the_whole_free_list` finds the free list
    // empty with and without it (one value, and many small ones). It costs one page at
    // the end of the file, which the drop below frees again.
    let mut remaining = (free + 1) * (page - 4);
    while remaining > 0 {
        let size = remaining.min(chunk);
        tx.execute(
            "INSERT INTO store_overwrite (zeros) VALUES (zeroblob(?1))",
            [size],
        )?;
        remaining -= size;
    }
    Ok(())
}

/// Checks that every table of `schema` exists with its columns.
fn check_schema(conn: &Connection, schema: &Schema) -> Result<()> {
    for (table, columns) in schema.tables {
        let mut statement = conn
            .prepare("SELECT name FROM pragma_table_info(?1)")
            .map_err(sqlite)?;
        let found: Vec<String> = statement
            .query_map([table], |r| r.get(0))
            .and_then(Iterator::collect)
            .map_err(sqlite)?;
        if let Some(missing) = columns.iter().find(|c| !found.iter().any(|f| f == *c)) {
            return Err(Error::Store(format!(
                "schema check failed: table {table} lacks column {missing} (schema version {})",
                schema.version
            )));
        }
    }
    Ok(())
}

/// After an opening with the current schema: if a backup from a migration lies next to
/// the database, read every table and write once; if both succeed, the database is
/// usable and the backup is deleted (ADR 0128). A failed write keeps the backup for the
/// next opening.
fn remove_backup_after_check(conn: &mut Connection, path: &Path, schema: &Schema) -> Result<()> {
    let backups = backup_files(path);
    if backups.is_empty() {
        return Ok(());
    }
    for (table, _) in schema.tables {
        // Table names come from the schema of this package, not from a user.
        conn.query_row(&format!("SELECT count(*) FROM {table}"), [], |r| {
            r.get::<_, i64>(0)
        })
        .map_err(sqlite)?;
    }
    let write = |conn: &mut Connection| -> rusqlite::Result<()> {
        let tx: Transaction<'_> = conn.transaction()?;
        tx.execute(
            "INSERT INTO store_meta (key, value) VALUES ('checked_with_schema', ?1)
             ON CONFLICT (key) DO UPDATE SET value = excluded.value",
            [schema.version.to_string()],
        )?;
        tx.commit()
    };
    if write(conn).is_err() {
        return Ok(());
    }
    for backup in backups {
        // Deleted through the file system, not moved to a recycle bin.
        let _ = fs::remove_file(backup);
    }
    Ok(())
}

/// Writes a complete copy of the database as `<file name>.v<version>.bak` with the
/// online backup of SQLite, first under a temporary name in the same folder, then
/// renamed. Only after that are older backups of this database removed. If anything
/// fails, the migration does not run.
fn back_up(conn: &Connection, path: &Path, version: u32) -> Result<()> {
    let target = sibling(path, &format!(".v{version}.bak"));
    let partial = sibling(path, &format!(".v{version}.bak.partial"));
    let message = |e: String| {
        format!(
            "backup of {} before the migration from schema version {version} failed, nothing was migrated: {e}",
            path.display()
        )
    };
    let failed = |e: String| Error::Store(message(e));
    if partial.symlink_metadata().is_ok() {
        fs::remove_file(&partial).map_err(|e| failed(e.to_string()))?;
    }
    let copy = || -> rusqlite::Result<()> {
        let mut destination = Connection::open_with_flags(
            &partial,
            OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_CREATE
                | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;
        destination.pragma_update(None, "temp_store", "MEMORY")?;
        #[cfg(test)]
        if FULL_BACKUP_OF
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_deref()
            == Some(path)
        {
            destination.pragma_update(None, "max_page_count", 1)?;
        }
        {
            let backup = Backup::new(conn, &mut destination)?;
            while backup.step(-1)? != StepResult::Done {}
        }
        destination.close().map_err(|(_, e)| e)
    };
    if let Err(e) = copy() {
        let _ = fs::remove_file(&partial);
        let _ = fs::remove_file(sibling(&partial, "-journal"));
        return Err(if is_full(&e) {
            Error::StorageFull(message(e.to_string()))
        } else {
            failed(e.to_string())
        });
    }
    if let Err(e) = fs::rename(&partial, &target) {
        let _ = fs::remove_file(&partial);
        return Err(failed(e.to_string()));
    }
    for older in backup_files(path) {
        if older != target {
            let _ = fs::remove_file(older);
        }
    }
    Ok(())
}

/// The path of the database with `suffix` appended to its file name.
pub(crate) fn sibling(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.file_name().map(OsString::from).unwrap_or_default();
    name.push(suffix);
    path.with_file_name(name)
}

/// Backups of this database in its folder: `<file name>.v<digits>.bak`, and leftovers
/// of an interrupted backup (`.bak.partial`).
pub(crate) fn backup_files(path: &Path) -> Vec<PathBuf> {
    let (Some(folder), Some(name)) = (path.parent(), path.file_name().and_then(|n| n.to_str()))
    else {
        return Vec::new();
    };
    let Ok(entries) = fs::read_dir(folder) else {
        return Vec::new();
    };
    let prefix = format!("{name}.v");
    entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|candidate| {
            let Some(file) = candidate.file_name().and_then(|n| n.to_str()) else {
                return false;
            };
            let Some(rest) = file.strip_prefix(&prefix) else {
                return false;
            };
            let digits = rest
                .strip_suffix(".bak")
                .or_else(|| rest.strip_suffix(".bak.partial"));
            digits.is_some_and(|d| !d.is_empty() && d.bytes().all(|b| b.is_ascii_digit()))
        })
        .collect()
}
