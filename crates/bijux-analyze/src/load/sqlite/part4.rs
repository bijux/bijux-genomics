//! Owner: bijux-analyze
//! `SQLite` benchmark helpers part 4.

use rusqlite::{params, Connection};
use serde_json::Value as JsonValue;

use bijux_core::measure::ExecutionMetrics;
use bijux_core::metrics::MetricSet;

use crate::aggregate::{
    BenchmarkContext, BenchmarkRecord, FastqFilterMetrics, FastqMergeMetrics, Result,
};
use crate::model::JsonBlob;

use super::base::{
    ensure_identity_index, ensure_inserted_at_column, ensure_params_hash_column,
    ensure_record_id_column, json_from_str, params_hash,
};

/// Load a filter benchmark record from `SQLite` v2 if present.
///
/// # Errors
/// Returns an error if the query or JSON parsing fails.
#[allow(clippy::too_many_arguments)]
pub fn fetch_fastq_filter_v2(
    conn: &Connection,
    tool: &str,
    tool_version: &str,
    image_digest: &str,
    runner: &str,
    platform: &str,
    input_hash: &str,
    params_hash: &str,
) -> Result<Option<BenchmarkRecord<FastqFilterMetrics>>> {
    let mut stmt = conn.prepare(
        "SELECT tool, tool_version, image_digest, runner, platform, input_hash, params_hash,\
         parameters_json, runtime_s, memory_mb, exit_code, metrics_json \
         FROM bench_fastq_filter_v2 \
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
        |row| {
            let tool: String = row.get(0)?;
            let tool_version: String = row.get(1)?;
            let image_digest: String = row.get(2)?;
            let runner: String = row.get(3)?;
            let platform: String = row.get(4)?;
            let input_hash: String = row.get(5)?;
            let _params_hash: String = row.get(6)?;
            let parameters_json: String = row.get(7)?;
            let runtime_s: f64 = row.get(8)?;
            let memory_mb: f64 = row.get(9)?;
            let exit_code: i64 = row.get(10)?;
            let metrics_json: String = row.get(11)?;
            let parameters: JsonValue = json_from_str(&parameters_json)?;
            let metrics: MetricSet<FastqFilterMetrics> = json_from_str(&metrics_json)?;
            Ok(BenchmarkRecord {
                context: BenchmarkContext {
                    tool,
                    tool_version,
                    image_digest,
                    runner,
                    platform,
                    input_hash,
                    parameters: JsonBlob::from(parameters),
                },
                execution: ExecutionMetrics {
                    runtime_s,
                    memory_mb,
                    exit_code: i32::try_from(exit_code).unwrap_or(i32::MAX),
                },
                metrics,
            })
        },
    );
    match row {
        Ok(record) => Ok(Some(record)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(err) => Err(err.into()),
    }
}

/// Insert a `FastQ` merge benchmark record into the v1 table.
///
/// # Errors
/// Returns an error if the table cannot be created or the record cannot be inserted.
pub fn insert_fastq_merge_v1(
    conn: &Connection,
    record: &BenchmarkRecord<FastqMergeMetrics>,
) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS bench_fastq_merge_v1 (\
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
    ensure_inserted_at_column(conn, "bench_fastq_merge_v1")?;
    ensure_record_id_column(conn, "bench_fastq_merge_v1")?;
    ensure_params_hash_column(conn, "bench_fastq_merge_v1")?;
    ensure_identity_index(conn, "bench_fastq_merge_v1")?;

    let metrics_json = serde_json::to_string(&record.metrics)?;
    let parameters_json = serde_json::to_string(&record.context.parameters)?;
    let params_hash = params_hash(&record.context.parameters)?;

    conn.execute(
        "INSERT INTO bench_fastq_merge_v1 (\
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

/// Load a merge benchmark record from `SQLite` if present.
///
/// # Errors
/// Returns an error if the query or JSON parsing fails.
#[allow(clippy::too_many_arguments)]
pub fn fetch_fastq_merge_v1(
    conn: &Connection,
    tool: &str,
    tool_version: &str,
    image_digest: &str,
    runner: &str,
    platform: &str,
    input_hash: &str,
    params_hash: &str,
) -> Result<Option<BenchmarkRecord<FastqMergeMetrics>>> {
    let mut stmt = conn.prepare(
        "SELECT tool, tool_version, image_digest, runner, platform, input_hash, params_hash,\
         parameters_json, runtime_s, memory_mb, exit_code, metrics_json \
         FROM bench_fastq_merge_v1 \
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
        |row| {
            let tool: String = row.get(0)?;
            let tool_version: String = row.get(1)?;
            let image_digest: String = row.get(2)?;
            let runner: String = row.get(3)?;
            let platform: String = row.get(4)?;
            let input_hash: String = row.get(5)?;
            let _params_hash: String = row.get(6)?;
            let parameters_json: String = row.get(7)?;
            let runtime_s: f64 = row.get(8)?;
            let memory_mb: f64 = row.get(9)?;
            let exit_code: i64 = row.get(10)?;
            let metrics_json: String = row.get(11)?;
            let parameters: JsonValue = json_from_str(&parameters_json)?;
            let metrics: MetricSet<FastqMergeMetrics> = json_from_str(&metrics_json)?;
            Ok(BenchmarkRecord {
                context: BenchmarkContext {
                    tool,
                    tool_version,
                    image_digest,
                    runner,
                    platform,
                    input_hash,
                    parameters: JsonBlob::from(parameters),
                },
                execution: ExecutionMetrics {
                    runtime_s,
                    memory_mb,
                    exit_code: i32::try_from(exit_code).unwrap_or(i32::MAX),
                },
                metrics,
            })
        },
    );
    match row {
        Ok(record) => Ok(Some(record)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(err) => Err(err.into()),
    }
}
