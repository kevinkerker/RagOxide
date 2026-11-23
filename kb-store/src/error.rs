//! Mapping of SQLite and serialisation errors onto the contract's [`Error`].

use std::path::Path;

use kb_core::Error;
use rusqlite::ffi::ErrorCode;

/// `true` if SQLite reports that the database or its disk is full (`SQLITE_FULL`).
pub(crate) fn is_full(error: &rusqlite::Error) -> bool {
    error.sqlite_error_code() == Some(ErrorCode::DiskFull)
}

/// An SQLite error with the original text of SQLite: [`Error::StorageFull`] if the
/// database or its disk is full, [`Error::Store`] otherwise.
pub(crate) fn sqlite(error: rusqlite::Error) -> Error {
    if is_full(&error) {
        Error::StorageFull(error.to_string())
    } else {
        Error::Store(error.to_string())
    }
}

/// `true` if SQLite reports that another connection holds a lock on the database.
pub(crate) fn is_locked(error: &rusqlite::Error) -> bool {
    matches!(
        error.sqlite_error_code(),
        Some(ErrorCode::DatabaseBusy | ErrorCode::DatabaseLocked)
    )
}

/// Error of an opening that found the database locked by another connection.
pub(crate) fn in_use(path: &Path, error: &rusqlite::Error) -> Error {
    Error::Store(format!(
        "the database {} is in use by another process (only one process may open it): {error}",
        path.display()
    ))
}

/// Error of an opening while reading the database header. A full database or disk is
/// [`Error::StorageFull`], as in [`sqlite`].
pub(crate) fn opening(path: &Path, error: rusqlite::Error) -> Error {
    if is_locked(&error) {
        return in_use(path, &error);
    }
    if is_full(&error) {
        return Error::StorageFull(format!(
            "cannot open the database {}: {error}",
            path.display()
        ));
    }
    if error.sqlite_error_code() == Some(ErrorCode::NotADatabase) {
        return Error::Store(format!(
            "{} is not an SQLite database: {error}",
            path.display()
        ));
    }
    Error::Store(format!(
        "cannot open the database {}: {error}",
        path.display()
    ))
}

/// A value passed in that cannot be written as JSON (for example a path that is not
/// valid Unicode) is bad input.
pub(crate) fn encode(what: &str, error: serde_json::Error) -> Error {
    Error::InvalidInput(format!("{what} cannot be stored: {error}"))
}

/// A stored value that cannot be read back is a store failure.
pub(crate) fn decode(what: &str, error: serde_json::Error) -> Error {
    Error::Store(format!("stored {what} cannot be read: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn failure(code: std::os::raw::c_int, text: &str) -> rusqlite::Error {
        rusqlite::Error::SqliteFailure(rusqlite::ffi::Error::new(code), Some(text.into()))
    }

    #[test]
    fn a_full_database_is_storage_full_and_every_other_code_a_store_error() {
        let path = Path::new(r"C:\Datenibliothek.db");
        let full = || failure(rusqlite::ffi::SQLITE_FULL, "database or disk is full");
        let error = sqlite(full());
        assert!(matches!(error, Error::StorageFull(_)), "{error:?}");
        assert_eq!(error.to_string(), "storage full: database or disk is full");
        let error = opening(path, full());
        assert!(matches!(error, Error::StorageFull(_)), "{error:?}");
        assert!(
            error.to_string().starts_with("storage full: ")
                && error.to_string().ends_with(": database or disk is full"),
            "{error}"
        );

        let other = || failure(rusqlite::ffi::SQLITE_IOERR, "disk I/O error");
        let error = sqlite(other());
        assert!(matches!(error, Error::Store(_)), "{error:?}");
        assert_eq!(error.to_string(), "store error: disk I/O error");
        let error = opening(path, other());
        assert!(matches!(error, Error::Store(_)), "{error:?}");
        assert!(error.to_string().ends_with(": disk I/O error"), "{error}");

        // The special cases of the opening stay.
        let error = opening(
            path,
            failure(rusqlite::ffi::SQLITE_BUSY, "database is locked"),
        );
        assert!(matches!(error, Error::Store(_)), "{error:?}");
        assert!(
            error.to_string().contains("in use by another process"),
            "{error}"
        );
        let error = opening(
            path,
            failure(rusqlite::ffi::SQLITE_NOTADB, "file is not a database"),
        );
        assert!(matches!(error, Error::Store(_)), "{error:?}");
        assert!(
            error.to_string().contains("is not an SQLite database"),
            "{error}"
        );
    }
}
