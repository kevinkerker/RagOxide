//! The database schema, its version and the migrations between versions (ADR 0119).

use rusqlite::Transaction;

/// Application id in the database header (`PRAGMA application_id`), "ROXD". A database
/// with tables and another id belongs to another application and is never migrated.
pub(crate) const APPLICATION_ID: i32 = 0x524F_5844;

/// One step from schema `from` to schema `from + 1`.
pub(crate) struct Migration {
    /// Version the step starts from.
    pub(crate) from: u32,
    /// Changes the tables; runs inside the transaction of the migration.
    pub(crate) apply: fn(&Transaction<'_>) -> rusqlite::Result<()>,
}

/// A schema version: how to create it from nothing, how to reach it from older
/// versions, and what it must contain.
pub(crate) struct Schema {
    /// Version written to `PRAGMA user_version`.
    pub(crate) version: u32,
    /// Creates all tables of this version in an empty database.
    pub(crate) create: fn(&Transaction<'_>) -> rusqlite::Result<()>,
    /// Steps from every older version, one per version.
    pub(crate) migrations: &'static [Migration],
    /// Tables with their columns, checked at every opening.
    pub(crate) tables: &'static [(&'static str, &'static [&'static str])],
}

/// Schema 1, the schema of this package.
pub(crate) const CURRENT: Schema = Schema {
    version: 1,
    create: create_v1,
    migrations: &[],
    tables: TABLES_V1,
};

/// Tables of schema 1.
pub(crate) const TABLES_V1: &[(&str, &[&str])] = &[
    ("store_meta", &["key", "value"]),
    ("documents", &["id", "path_key", "record"]),
    ("contents", &["document_id", "document"]),
    (
        "chats",
        &["id", "title", "created_at", "updated_at", "turns"],
    ),
    ("settings", &["id", "settings"]),
];

/// Schema 1. All values written by the user are JSON text or plain columns, bound as
/// parameters. `path_key` is computed in Rust ([`kb_core::path_key`]) and compared
/// byte-wise (`BINARY`); timestamps are stored order-preserving (see
/// [`crate::codec::encode_time`]).
const SQL_V1: &str = "
CREATE TABLE store_meta (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL
) STRICT;
CREATE TABLE documents (
    id TEXT PRIMARY KEY NOT NULL,
    path_key TEXT NOT NULL UNIQUE,
    record TEXT NOT NULL
) STRICT;
CREATE TABLE contents (
    document_id TEXT PRIMARY KEY NOT NULL,
    document TEXT NOT NULL
) STRICT;
CREATE TABLE chats (
    id TEXT PRIMARY KEY NOT NULL,
    title TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    turns TEXT NOT NULL
) STRICT;
CREATE TABLE settings (
    id INTEGER PRIMARY KEY NOT NULL CHECK (id = 1),
    settings TEXT NOT NULL
) STRICT;
INSERT INTO store_meta (key, value) VALUES ('created_with_schema', '1');
";

fn create_v1(tx: &Transaction<'_>) -> rusqlite::Result<()> {
    tx.execute_batch(SQL_V1)
}
