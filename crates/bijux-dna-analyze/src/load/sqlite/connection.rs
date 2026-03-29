use std::path::Path;

use anyhow::Result;
use rusqlite::Connection;
use serde::de::DeserializeOwned;

use super::schema::ensure_sqlite_schema_version;

pub(crate) fn json_from_str<T: DeserializeOwned>(
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
