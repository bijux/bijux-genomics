// imports provided by queries_core.rs

/// Load a `qc_post` benchmark record from `SQLite` if present.
///
/// # Errors
/// Returns an error if the query or JSON parsing fails.
#[allow(clippy::too_many_arguments)]
pub fn fetch_fastq_qc_post_v1(
    conn: &Connection,
    tool: &str,
    tool_version: &str,
    image_digest: &str,
    runner: &str,
    platform: &str,
    input_hash: &str,
    params_hash: &str,
) -> Result<Option<BenchmarkRecord<FastqQcPostMetrics>>> {
    let mut stmt = conn.prepare(
        "SELECT tool, tool_version, image_digest, runner, platform, input_hash, params_hash,\
         parameters_json, runtime_s, memory_mb, exit_code, metrics_json \
         FROM bench_fastq_qc_post_v1 \
         WHERE tool = ?1 AND tool_version = ?2 AND image_digest = ?3\
         AND runner = ?4 AND platform = ?5 AND input_hash = ?6 AND params_hash = ?7\
         ORDER BY record_id DESC, inserted_at DESC LIMIT 1",
    )?;
    let row = stmt.query_row(
        params![
            tool,
            tool_version,
            image_digest,
            runner,
            platform,
            input_hash,
            params_hash
        ],
        benchmark_record_from_row::<FastqQcPostMetrics>,
    );
    match row {
        Ok(record) => Ok(Some(record)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(err) => Err(err.into()),
    }
}

/// Insert a `FastQ` umi benchmark record into the v1 table.
///
/// # Errors
/// Returns an error if the table cannot be created or the record cannot be inserted.
pub fn insert_fastq_umi_v1(
    conn: &Connection,
    record: &BenchmarkRecord<FastqUmiMetrics>,
) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS bench_fastq_umi_v1 (\
         record_id INTEGER PRIMARY KEY AUTOINCREMENT,\
         tool TEXT NOT NULL,\
         tool_version TEXT NOT NULL,\
         image_digest TEXT NOT NULL,\
         runner TEXT NOT NULL,\
         platform TEXT NOT NULL,\
         input_hash TEXT NOT NULL,\
         params_hash TEXT NOT NULL,\
         parameters_json TEXT NOT NULL,\
         schema_version INTEGER NOT NULL,\
         runtime_s REAL NOT NULL,\
         memory_mb REAL NOT NULL,\
         exit_code INTEGER NOT NULL,\
         metrics_json TEXT NOT NULL,\
         inserted_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))\
         )",
        [],
    )?;
    ensure_inserted_at_column(conn, "bench_fastq_umi_v1")?;
    ensure_record_id_column(conn, "bench_fastq_umi_v1")?;
    ensure_params_hash_column(conn, "bench_fastq_umi_v1")?;
    ensure_identity_index(conn, "bench_fastq_umi_v1")?;

    let metrics_json = serde_json::to_string(&record.metrics)?;
    let parameters_json = serde_json::to_string(&record.context.parameters)?;
    let params_hash = params_hash(&record.context.parameters)?;

    conn.execute(
        "INSERT INTO bench_fastq_umi_v1 (\
         tool, tool_version, image_digest, runner, platform, input_hash, params_hash,\
         parameters_json, schema_version, runtime_s, memory_mb, exit_code, metrics_json\
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
        (
            &record.context.tool,
            &record.context.tool_version,
            &record.context.image_digest,
            &record.context.runner,
            &record.context.platform,
            &record.context.input_hash,
            params_hash,
            parameters_json,
            record.metrics.version,
            record.execution.runtime_s,
            record.execution.memory_mb,
            record.execution.exit_code,
            metrics_json,
        ),
    )?;
    Ok(())
}

/// Load a umi benchmark record from `SQLite` if present.
///
/// # Errors
/// Returns an error if the query or JSON parsing fails.
#[allow(clippy::too_many_arguments)]
pub fn fetch_fastq_umi_v1(
    conn: &Connection,
    tool: &str,
    tool_version: &str,
    image_digest: &str,
    runner: &str,
    platform: &str,
    input_hash: &str,
    params_hash: &str,
) -> Result<Option<BenchmarkRecord<FastqUmiMetrics>>> {
    let mut stmt = conn.prepare(
        "SELECT tool, tool_version, image_digest, runner, platform, input_hash, params_hash,\
         parameters_json, runtime_s, memory_mb, exit_code, metrics_json \
         FROM bench_fastq_umi_v1 \
         WHERE tool = ?1 AND tool_version = ?2 AND image_digest = ?3\
         AND runner = ?4 AND platform = ?5 AND input_hash = ?6 AND params_hash = ?7\
         ORDER BY record_id DESC, inserted_at DESC LIMIT 1",
    )?;
    let row = stmt.query_row(
        params![
            tool,
            tool_version,
            image_digest,
            runner,
            platform,
            input_hash,
            params_hash
        ],
        benchmark_record_from_row::<FastqUmiMetrics>,
    );
    match row {
        Ok(record) => Ok(Some(record)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(err) => Err(err.into()),
    }
}
// `SQLite` screen/stats benchmark helpers.

/// Insert a `FastQ` screen benchmark record into the v1 table.
///
/// # Errors
/// Returns an error if the table cannot be created or the record cannot be inserted.
pub fn insert_fastq_screen_v1(
    conn: &Connection,
    record: &BenchmarkRecord<FastqScreenMetrics>,
) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS bench_fastq_screen_v1 (\
         record_id INTEGER PRIMARY KEY AUTOINCREMENT,\
         tool TEXT NOT NULL,\
         tool_version TEXT NOT NULL,\
         image_digest TEXT NOT NULL,\
         runner TEXT NOT NULL,\
         platform TEXT NOT NULL,\
         input_hash TEXT NOT NULL,\
         params_hash TEXT NOT NULL,\
         parameters_json TEXT NOT NULL,\
         schema_version INTEGER NOT NULL,\
         runtime_s REAL NOT NULL,\
         memory_mb REAL NOT NULL,\
         exit_code INTEGER NOT NULL,\
         metrics_json TEXT NOT NULL,\
         inserted_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))\
         )",
        [],
    )?;
    ensure_inserted_at_column(conn, "bench_fastq_screen_v1")?;
    ensure_record_id_column(conn, "bench_fastq_screen_v1")?;
    ensure_params_hash_column(conn, "bench_fastq_screen_v1")?;
    ensure_identity_index(conn, "bench_fastq_screen_v1")?;

    let metrics_json = serde_json::to_string(&record.metrics)?;
    let parameters_json = serde_json::to_string(&record.context.parameters)?;
    let params_hash = params_hash(&record.context.parameters)?;

    conn.execute(
        "INSERT INTO bench_fastq_screen_v1 (\
         tool, tool_version, image_digest, runner, platform, input_hash, params_hash,\
         parameters_json, schema_version, runtime_s, memory_mb, exit_code, metrics_json\
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
        (
            &record.context.tool,
            &record.context.tool_version,
            &record.context.image_digest,
            &record.context.runner,
            &record.context.platform,
            &record.context.input_hash,
            params_hash,
            parameters_json,
            record.metrics.version,
            record.execution.runtime_s,
            record.execution.memory_mb,
            record.execution.exit_code,
            metrics_json,
        ),
    )?;
    Ok(())
}

/// Load a screen benchmark record from `SQLite` if present.
///
/// # Errors
/// Returns an error if the query or JSON parsing fails.
#[allow(clippy::too_many_arguments)]
pub fn fetch_fastq_screen_v1(
    conn: &Connection,
    tool: &str,
    tool_version: &str,
    image_digest: &str,
    runner: &str,
    platform: &str,
    input_hash: &str,
    params_hash: &str,
) -> Result<Option<BenchmarkRecord<FastqScreenMetrics>>> {
    let mut stmt = conn.prepare(
        "SELECT tool, tool_version, image_digest, runner, platform, input_hash, params_hash,\
         parameters_json, runtime_s, memory_mb, exit_code, metrics_json \
         FROM bench_fastq_screen_v1 \
         WHERE tool = ?1 AND tool_version = ?2 AND image_digest = ?3\
         AND runner = ?4 AND platform = ?5 AND input_hash = ?6 AND params_hash = ?7\
         ORDER BY record_id DESC, inserted_at DESC LIMIT 1",
    )?;
    let row = stmt.query_row(
        params![
            tool,
            tool_version,
            image_digest,
            runner,
            platform,
            input_hash,
            params_hash
        ],
        benchmark_record_from_row::<FastqScreenMetrics>,
    );
    match row {
        Ok(record) => Ok(Some(record)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(err) => Err(err.into()),
    }
}

/// Insert a `FastQ` stats benchmark record into the v1 table.
///
/// # Errors
/// Returns an error if the table cannot be created or the record cannot be inserted.
pub fn insert_fastq_stats_v1(
    conn: &Connection,
    record: &BenchmarkRecord<FastqStatsMetrics>,
) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS bench_fastq_stats_v1 (\
         record_id INTEGER PRIMARY KEY AUTOINCREMENT,\
         tool TEXT NOT NULL,\
         tool_version TEXT NOT NULL,\
         image_digest TEXT NOT NULL,\
         runner TEXT NOT NULL,\
         platform TEXT NOT NULL,\
         input_hash TEXT NOT NULL,\
         params_hash TEXT NOT NULL,\
         parameters_json TEXT NOT NULL,\
         schema_version INTEGER NOT NULL,\
         runtime_s REAL NOT NULL,\
         memory_mb REAL NOT NULL,\
         exit_code INTEGER NOT NULL,\
         metrics_json TEXT NOT NULL,\
         inserted_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))\
         )",
        [],
    )?;
    ensure_inserted_at_column(conn, "bench_fastq_stats_v1")?;
    ensure_record_id_column(conn, "bench_fastq_stats_v1")?;
    ensure_params_hash_column(conn, "bench_fastq_stats_v1")?;
    ensure_identity_index(conn, "bench_fastq_stats_v1")?;

    let metrics_json = serde_json::to_string(&record.metrics)?;
    let parameters_json = serde_json::to_string(&record.context.parameters)?;
    let params_hash = params_hash(&record.context.parameters)?;

    conn.execute(
        "INSERT INTO bench_fastq_stats_v1 (\
         tool, tool_version, image_digest, runner, platform, input_hash, params_hash,\
         parameters_json, schema_version, runtime_s, memory_mb, exit_code, metrics_json\
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
        (
            &record.context.tool,
            &record.context.tool_version,
            &record.context.image_digest,
            &record.context.runner,
            &record.context.platform,
            &record.context.input_hash,
            params_hash,
            parameters_json,
            record.metrics.version,
            record.execution.runtime_s,
            record.execution.memory_mb,
            record.execution.exit_code,
            metrics_json,
        ),
    )?;
    Ok(())
}
// `SQLite` stats benchmark helpers.

/// Load a stats benchmark record from `SQLite` if present.
///
/// # Errors
/// Returns an error if the query or JSON parsing fails.
#[allow(clippy::too_many_arguments)]
pub fn fetch_fastq_stats_v1(
    conn: &Connection,
    tool: &str,
    tool_version: &str,
    image_digest: &str,
    runner: &str,
    platform: &str,
    input_hash: &str,
    params_hash: &str,
) -> Result<Option<BenchmarkRecord<FastqStatsMetrics>>> {
    let mut stmt = conn.prepare(
        "SELECT tool, tool_version, image_digest, runner, platform, input_hash, params_hash,\
         parameters_json, runtime_s, memory_mb, exit_code, metrics_json \
         FROM bench_fastq_stats_v1 \
         WHERE tool = ?1 AND tool_version = ?2 AND image_digest = ?3\
         AND runner = ?4 AND platform = ?5 AND input_hash = ?6 AND params_hash = ?7\
         ORDER BY record_id DESC, inserted_at DESC LIMIT 1",
    )?;
    let row = stmt.query_row(
        params![
            tool,
            tool_version,
            image_digest,
            runner,
            platform,
            input_hash,
            params_hash
        ],
        benchmark_record_from_row::<FastqStatsMetrics>,
    );
    match row {
        Ok(record) => Ok(Some(record)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(err) => Err(err.into()),
    }
}
// Image QA storage in `SQLite`.

/// Insert an image QA record.
///
/// # Errors
/// Returns an error if the insert or schema setup fails.
pub fn insert_image_qa_v1(conn: &Connection, record: &ImageQaRecord) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS image_qa_v1 (\
         record_id INTEGER PRIMARY KEY AUTOINCREMENT,\
         tool TEXT NOT NULL,\
         stage TEXT NOT NULL,\
         tool_version TEXT NOT NULL,\
         image_digest TEXT NOT NULL,\
         runner TEXT NOT NULL,\
         platform TEXT NOT NULL,\
         input_hash TEXT NOT NULL,\
         status TEXT NOT NULL,\
         failure_reason TEXT,\
         schema_version INTEGER NOT NULL,\
         outcome_json TEXT NOT NULL\
         )",
        [],
    )?;
    ensure_record_id_column(conn, "image_qa_v1")?;
    ensure_image_qa_identity_index(conn)?;

    let outcome_json = serde_json::to_string(&record.outcome)?;
    conn.execute(
        "INSERT INTO image_qa_v1 (\
         tool, stage, tool_version, image_digest, runner, platform, input_hash,\
         status, failure_reason, schema_version, outcome_json\
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        (
            &record.tool,
            &record.stage,
            &record.tool_version,
            &record.image_digest,
            &record.runner,
            &record.platform,
            &record.input_hash,
            record.outcome.status(),
            record.outcome.failure_reason(),
            IMAGE_QA_SCHEMA_VERSION,
            outcome_json,
        ),
    )?;
    Ok(())
}

/// Ensure image QA tables exist in `SQLite`.
///
/// # Errors
/// Returns an error if the tables cannot be created.
pub fn ensure_image_qa_tables(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS image_qa_v1 (\
         record_id INTEGER PRIMARY KEY AUTOINCREMENT,\
         tool TEXT NOT NULL,\
         stage TEXT NOT NULL,\
         tool_version TEXT NOT NULL,\
         image_digest TEXT NOT NULL,\
         runner TEXT NOT NULL,\
         platform TEXT NOT NULL,\
         input_hash TEXT NOT NULL,\
         status TEXT NOT NULL,\
         failure_reason TEXT,\
         schema_version INTEGER NOT NULL,\
         outcome_json TEXT NOT NULL\
         )",
        [],
    )?;
    ensure_record_id_column(conn, "image_qa_v1")?;
    ensure_image_qa_identity_index(conn)?;
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
    Ok(())
}
// imports provided by queries_core.rs
