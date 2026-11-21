//! Mapping of SQLite and serialisation errors onto the contract's [`Error`].

use std::path::Path;

use kb_core::Error;
use rusqlite::ffi::ErrorCode;

/// Any SQLite error as [`Error::Store`] with the original text of SQLite.
pub(crate) fn sqlite(error: rusqlite::Error) -> Error {
    Error::Store(error.to_string())
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

/// Error of an opening while reading the database header.
pub(crate) fn opening(path: &Path, error: rusqlite::Error) -> Error {
    if is_locked(&error) {
        return in_use(path, &error);
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
