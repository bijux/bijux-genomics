use anyhow::{anyhow, Result};
use rusqlite::Connection;

pub(crate) fn ensure_sqlite_schema_version(conn: &Connection, target_version: i32) -> Result<()> {
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

fn ensure_column(conn: &Connection, table: &str, column: &str, definition: &str) -> Result<()> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({table})"))?;
    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        let name: String = row.get(1)?;
        if name == column {
            return Ok(());
        }
    }
    let sql = format!("ALTER TABLE {table} ADD COLUMN {column} {definition}");
    conn.execute(&sql, [])?;
    Ok(())
}

pub(crate) fn ensure_inserted_at_column(conn: &Connection, table: &str) -> Result<()> {
    ensure_column(
        conn,
        table,
        "inserted_at",
        "TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
    )
}

pub(crate) fn ensure_record_id_column(conn: &Connection, table: &str) -> Result<()> {
    ensure_column(conn, table, "record_id", "INTEGER NOT NULL DEFAULT 0")
}

pub(crate) fn ensure_params_hash_column(conn: &Connection, table: &str) -> Result<()> {
    ensure_column(conn, table, "params_hash", "TEXT NOT NULL DEFAULT ''")
}

pub(crate) fn ensure_identity_index(conn: &Connection, table: &str) -> Result<()> {
    let index_name = format!("{table}_identity_idx");
    let sql = format!(
        "CREATE UNIQUE INDEX IF NOT EXISTS {index_name} \
         ON {table} (tool, tool_version, image_digest, runner, platform, input_hash, params_hash)"
    );
    conn.execute(&sql, [])?;
    Ok(())
}

pub(crate) fn ensure_image_qa_identity_index(conn: &Connection) -> Result<()> {
    let sql = "CREATE UNIQUE INDEX IF NOT EXISTS image_qa_v1_identity_idx \
               ON image_qa_v1 (tool, stage, tool_version, image_digest, runner, platform, input_hash)";
    conn.execute(sql, [])?;
    Ok(())
}
