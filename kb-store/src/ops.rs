//! The store operations as synchronous functions on the connection; each write is one
//! transaction. Every value from the caller is a bound parameter (`?1`, `?2`, ...).

use std::path::Path;

use kb_core::{
    Chat, ChatId, ChatSummary, ChatTurn, Document, DocumentId, DocumentRecord, Error, Result,
    Settings, SettingsUpdate, path_key,
};
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};

use crate::codec::{check_chat, check_document, decode_time, encode_time, record_key};
use crate::error::{decode, encode, sqlite};

fn read_json<T: serde::de::DeserializeOwned>(what: &str, text: &str) -> Result<T> {
    serde_json::from_str(text).map_err(|e| decode(what, e))
}

fn write_json<T: serde::Serialize>(what: &str, value: &T) -> Result<String> {
    serde_json::to_string(value).map_err(|e| encode(what, e))
}

pub(crate) fn upsert_document(conn: &mut Connection, record: &DocumentRecord) -> Result<()> {
    let key = record_key(&record.path);
    let json = write_json(&format!("record {}", record.id), record)?;
    let tx = conn
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(sqlite)?;
    let holder: Option<String> = tx
        .query_row(
            "SELECT id FROM documents WHERE path_key = ?1",
            [&key],
            |r| r.get(0),
        )
        .optional()
        .map_err(sqlite)?;
    if let Some(holder) = holder.filter(|h| *h != record.id.0) {
        return Err(Error::InvalidInput(format!(
            "record {} has the path {}, which record {holder} already has (one record per file)",
            record.id,
            record.path.display()
        )));
    }
    tx.execute(
        "INSERT INTO documents (id, path_key, record) VALUES (?1, ?2, ?3)
         ON CONFLICT (id) DO UPDATE SET path_key = excluded.path_key, record = excluded.record",
        params![record.id.0, key, json],
    )
    .map_err(sqlite)?;
    tx.commit().map_err(sqlite)
}

pub(crate) fn document(conn: &Connection, id: &DocumentId) -> Result<Option<DocumentRecord>> {
    conn.query_row("SELECT record FROM documents WHERE id = ?1", [&id.0], |r| {
        r.get::<_, String>(0)
    })
    .optional()
    .map_err(sqlite)?
    .map(|json| read_json(&format!("record {id}"), &json))
    .transpose()
}

pub(crate) fn document_by_path(conn: &Connection, path: &Path) -> Result<Option<DocumentRecord>> {
    let key = path_key(path)?;
    conn.query_row(
        "SELECT id, record FROM documents WHERE path_key = ?1",
        [&key],
        |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
    )
    .optional()
    .map_err(sqlite)?
    .map(|(id, json)| read_json(&format!("record {id}"), &json))
    .transpose()
}

pub(crate) fn documents(conn: &Connection) -> Result<Vec<DocumentRecord>> {
    // `BINARY` compares the UTF-8 bytes, the order of `str` in Rust: sorted by path_key
    // byte-wise, as the contract asks, not by a collation of SQLite.
    let mut statement = conn
        .prepare("SELECT id, record FROM documents ORDER BY path_key COLLATE BINARY")
        .map_err(sqlite)?;
    let rows = statement
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
        .map_err(sqlite)?;
    let mut records = Vec::new();
    for row in rows {
        let (id, json) = row.map_err(sqlite)?;
        records.push(read_json(&format!("record {id}"), &json)?);
    }
    Ok(records)
}

pub(crate) fn delete_document(conn: &mut Connection, id: &DocumentId) -> Result<()> {
    let tx = conn
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(sqlite)?;
    tx.execute("DELETE FROM documents WHERE id = ?1", [&id.0])
        .map_err(sqlite)?;
    tx.execute("DELETE FROM contents WHERE document_id = ?1", [&id.0])
        .map_err(sqlite)?;
    tx.commit().map_err(sqlite)
}

pub(crate) fn save_content(conn: &mut Connection, document: &Document) -> Result<()> {
    check_document(document)?;
    let id = &document.source.document_id;
    let json = write_json(&format!("content of {id}"), document)?;
    let tx = conn
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(sqlite)?;
    tx.execute(
        "INSERT INTO contents (document_id, document) VALUES (?1, ?2)
         ON CONFLICT (document_id) DO UPDATE SET document = excluded.document",
        params![id.0, json],
    )
    .map_err(sqlite)?;
    tx.commit().map_err(sqlite)
}

pub(crate) fn content(conn: &Connection, id: &DocumentId) -> Result<Option<Document>> {
    let what = format!("content of {id}");
    // Parsed straight from the text SQLite holds, without a copy of the string.
    let parsed = conn
        .query_row(
            "SELECT document FROM contents WHERE document_id = ?1",
            [&id.0],
            |r| Ok(serde_json::from_str::<Document>(r.get_ref(0)?.as_str()?)),
        )
        .optional()
        .map_err(sqlite)?;
    parsed.map(|p| p.map_err(|e| decode(&what, e))).transpose()
}

pub(crate) fn save_chat(conn: &mut Connection, chat: &Chat) -> Result<()> {
    check_chat(chat)?;
    let turns = write_json(&format!("chat {}", chat.id), &chat.turns)?;
    let tx = conn
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(sqlite)?;
    tx.execute(
        "INSERT INTO chats (id, title, created_at, updated_at, turns) VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT (id) DO UPDATE SET title = excluded.title, created_at = excluded.created_at,
             updated_at = excluded.updated_at, turns = excluded.turns",
        params![
            chat.id.0,
            chat.title,
            encode_time(chat.created_at),
            encode_time(chat.updated_at),
            turns
        ],
    )
    .map_err(sqlite)?;
    tx.commit().map_err(sqlite)
}

pub(crate) fn chat(conn: &Connection, id: &ChatId) -> Result<Option<Chat>> {
    let row = conn
        .query_row(
            "SELECT title, created_at, updated_at, turns FROM chats WHERE id = ?1",
            [&id.0],
            |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, i64>(1)?,
                    r.get::<_, i64>(2)?,
                    r.get::<_, String>(3)?,
                ))
            },
        )
        .optional()
        .map_err(sqlite)?;
    let Some((title, created_at, updated_at, turns)) = row else {
        return Ok(None);
    };
    let turns: Vec<ChatTurn> = read_json(&format!("chat {id}"), &turns)?;
    Ok(Some(Chat {
        id: id.clone(),
        title,
        created_at: decode_time(created_at),
        updated_at: decode_time(updated_at),
        turns,
    }))
}

pub(crate) fn chats(conn: &Connection) -> Result<Vec<ChatSummary>> {
    // Newest first; equal times by id ascending, byte-wise (`BINARY`).
    let mut statement = conn
        .prepare(
            "SELECT id, title, created_at, updated_at FROM chats
             ORDER BY updated_at DESC, id COLLATE BINARY ASC",
        )
        .map_err(sqlite)?;
    statement
        .query_map([], |r| {
            Ok(ChatSummary {
                id: ChatId(r.get(0)?),
                title: r.get(1)?,
                created_at: decode_time(r.get(2)?),
                updated_at: decode_time(r.get(3)?),
            })
        })
        .and_then(Iterator::collect)
        .map_err(sqlite)
}

pub(crate) fn delete_chat(conn: &mut Connection, id: &ChatId) -> Result<()> {
    conn.execute("DELETE FROM chats WHERE id = ?1", [&id.0])
        .map_err(sqlite)?;
    Ok(())
}

fn read_settings(conn: &Connection) -> Result<Settings> {
    conn.query_row("SELECT settings FROM settings WHERE id = 1", [], |r| {
        r.get::<_, String>(0)
    })
    .optional()
    .map_err(sqlite)?
    // Fields missing in stored JSON take their default (`#[serde(default)]`).
    .map_or_else(
        || Ok(Settings::default()),
        |json| read_json("settings", &json),
    )
}

fn write_settings(conn: &Connection, settings: &Settings) -> Result<()> {
    let json = write_json("settings", settings)?;
    conn.execute(
        "INSERT INTO settings (id, settings) VALUES (1, ?1)
         ON CONFLICT (id) DO UPDATE SET settings = excluded.settings",
        [json],
    )
    .map_err(sqlite)?;
    Ok(())
}

pub(crate) fn settings(conn: &Connection) -> Result<Settings> {
    read_settings(conn)
}

pub(crate) fn save_settings(conn: &mut Connection, settings: &Settings) -> Result<()> {
    write_settings(conn, settings)
}

pub(crate) fn update_settings(conn: &mut Connection, update: SettingsUpdate) -> Result<Settings> {
    // Read, change and write in one transaction on the only connection: no other
    // settings write can come in between.
    let tx = conn
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(sqlite)?;
    let mut settings = read_settings(&tx)?;
    // An error of the update is returned unchanged; the transaction rolls back.
    update(&mut settings)?;
    write_settings(&tx, &settings)?;
    tx.commit().map_err(sqlite)?;
    Ok(settings)
}
