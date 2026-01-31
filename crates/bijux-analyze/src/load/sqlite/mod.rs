//! Owner: bijux-analyze
//! `SQLite` connection helpers for benchmark storage.

use std::path::Path;

use rusqlite::Connection;
use serde::de::DeserializeOwned;
use sha2::{Digest, Sha256};

use anyhow::{anyhow, Result};

use crate::model::JsonBlob;

mod queries;
mod rows;

pub use queries::*;

pub(super) fn json_from_str<T: DeserializeOwned>(
    value: &str,
) -> std::result::Result<T, rusqlite::Error> {
    serde_json::from_str(value).map_err(|err| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(err))
    })
}

/// Open a `SQLite` connection and ensure schema compatibility.
///
/// # Errors
/// Returns an error if the database cannot be opened or the schema is incompatible.
pub fn open_sqlite(path: &Path) -> Result<Connection> {
    let conn = Connection::open(path)?;
    ensure_sqlite_schema_version(&conn, 1)?;
    Ok(conn)
}

fn ensure_sqlite_schema_version(conn: &Connection, target_version: i32) -> Result<()> {
    let current: i32 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    if current == 0 {
        conn.execute(&format!("PRAGMA user_version = {target_version}"), [])?;
        return Ok(());
    }
    if current > target_version {
        return Err(anyhow!("unsupported schema version {current}"));
    }
    Ok(())
}

pub(super) fn ensure_inserted_at_column(conn: &Connection, table: &str) -> Result<()> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({table})"))?;
    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        let name: String = row.get(1)?;
        if name == "inserted_at" {
            return Ok(());
        }
    }
    let sql = format!(
        "ALTER TABLE {table} ADD COLUMN inserted_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))"
    );
    conn.execute(&sql, [])?;
    Ok(())
}

pub(super) fn ensure_record_id_column(conn: &Connection, table: &str) -> Result<()> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({table})"))?;
    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        let name: String = row.get(1)?;
        if name == "record_id" {
            return Ok(());
        }
    }
    let sql = format!("ALTER TABLE {table} ADD COLUMN record_id INTEGER NOT NULL DEFAULT 0");
    conn.execute(&sql, [])?;
    Ok(())
}

pub(super) fn ensure_params_hash_column(conn: &Connection, table: &str) -> Result<()> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({table})"))?;
    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        let name: String = row.get(1)?;
        if name == "params_hash" {
            return Ok(());
        }
    }
    let sql = format!("ALTER TABLE {table} ADD COLUMN params_hash TEXT NOT NULL DEFAULT ''");
    conn.execute(&sql, [])?;
    Ok(())
}

pub(super) fn ensure_identity_index(conn: &Connection, table: &str) -> Result<()> {
    let index_name = format!("{table}_identity_idx");
    let sql = format!(
        "CREATE UNIQUE INDEX IF NOT EXISTS {index_name} \
         ON {table} (tool, tool_version, image_digest, runner, platform, input_hash, params_hash)"
    );
    conn.execute(&sql, [])?;
    Ok(())
}

pub(super) fn ensure_image_qa_identity_index(conn: &Connection) -> Result<()> {
    let sql = "CREATE UNIQUE INDEX IF NOT EXISTS image_qa_v1_identity_idx \
               ON image_qa_v1 (tool, stage, tool_version, image_digest, runner, platform, input_hash)";
    conn.execute(sql, [])?;
    Ok(())
}

pub(super) fn params_hash(parameters: &JsonBlob) -> Result<String> {
    let canonical = bijux_core::parameters_json_canonicalization(parameters.as_value());
    let bytes = serde_json::to_vec(&canonical)?;
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    Ok(format!("{:x}", hasher.finalize()))
}
