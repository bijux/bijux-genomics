// imports provided by queries_core.rs

/// Insert an image QA input hash into the v1 table.
///
/// # Errors
/// Returns an error if the table cannot be created or the record cannot be inserted.
pub fn insert_image_qa_input_v1(
    conn: &Connection,
    stage: &str,
    input_hash: &str,
    platform: &str,
    runner: &str,
) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS image_qa_inputs_v1 (\
         record_id INTEGER PRIMARY KEY AUTOINCREMENT,\
         stage TEXT NOT NULL,\
         input_hash TEXT NOT NULL,\
         platform TEXT NOT NULL,\
         runner TEXT NOT NULL,\
         schema_version INTEGER NOT NULL,\
         UNIQUE(stage, input_hash, platform, runner)\
         )",
        [],
    )?;
    ensure_record_id_column(conn, "image_qa_inputs_v1")?;
    conn.execute(
        "INSERT OR IGNORE INTO image_qa_inputs_v1 (\
         stage, input_hash, platform, runner, schema_version\
         ) VALUES (?1, ?2, ?3, ?4, ?5)",
        (
            stage,
            input_hash,
            platform,
            runner,
            IMAGE_QA_INPUTS_SCHEMA_VERSION,
        ),
    )?;
    Ok(())
}

/// Load expected QA input hashes for a stage.
///
/// # Errors
/// Returns an error if the query fails.
pub fn image_qa_inputs(
    conn: &Connection,
    stage: &str,
    platform: &str,
    runner: &str,
) -> Result<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT input_hash FROM image_qa_inputs_v1 \
         WHERE stage = ?1 AND platform = ?2 AND runner = ?3 \
         ORDER BY input_hash ASC",
    )?;
    let rows = stmt.query_map((stage, platform, runner), |row| row.get(0))?;
    let mut inputs = Vec::new();
    for row in rows {
        inputs.push(row?);
    }
    Ok(inputs)
}

/// Load distinct input hashes from existing image QA records.
///
/// # Errors
/// Returns an error if the query fails.
pub fn image_qa_input_hashes_from_records(
    conn: &Connection,
    stage: &str,
    platform: &str,
    runner: &str,
) -> Result<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT DISTINCT input_hash FROM image_qa_v1 \
         WHERE stage = ?1 AND platform = ?2 AND runner = ?3",
    )?;
    let rows = stmt.query_map((stage, platform, runner), |row| row.get(0))?;
    let mut inputs = Vec::new();
    for row in rows {
        inputs.push(row?);
    }
    Ok(inputs)
}

/// Check whether image QA passed for a tool/stage/image/platform.
///
/// # Errors
/// Returns an error if the query fails.
pub fn image_qa_passed(
    conn: &Connection,
    tool: &str,
    stage: &str,
    image_digest: &str,
    platform: &str,
    runner: &str,
    input_hash: &str,
) -> Result<bool> {
    let mut stmt = conn.prepare(
        "SELECT COUNT(1) FROM image_qa_v1 \
         WHERE tool = ?1 AND stage = ?2 AND image_digest = ?3 \
         AND platform = ?4 AND runner = ?5 AND input_hash = ?6 AND status = 'pass'",
    )?;
    let count: i64 = stmt.query_row(
        (tool, stage, image_digest, platform, runner, input_hash),
        |row| row.get(0),
    )?;
    Ok(count > 0)
}
