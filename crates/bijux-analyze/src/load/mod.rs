//! Owner: bijux-analyze
//! Load facts and run artifacts from disk.

use std::io::{BufRead, BufReader};
use std::path::Path;

use rusqlite::{params, Connection};
use serde::de::DeserializeOwned;
use serde_json::Value as JsonValue;
use sha2::{Digest, Sha256};

use bijux_core::measure::ExecutionMetrics;
use bijux_core::metrics::MetricSet;
use bijux_core::run_index::RunIndexLine;
use bijux_core::FactsRowV1;

use crate::aggregate::{
    BenchError, BenchmarkContext, BenchmarkRecord, FastqCorrectMetrics, FastqFilterMetrics,
    FastqMergeMetrics, FastqQcPostMetrics, FastqScreenMetrics, FastqStatsMetrics, FastqTrimMetrics,
    FastqUmiMetrics, FastqValidateMetrics, ImageQaRecord, Result, IMAGE_QA_INPUTS_SCHEMA_VERSION,
    IMAGE_QA_SCHEMA_VERSION,
};
use crate::facts::{stable_sort_records, RunSummaryV1};
use crate::model::{FactTable, JsonBlob};

#[derive(thiserror::Error, Debug)]
pub enum AnalyzeError {
    #[error("missing file: {path}")]
    MissingFile { path: String },
    #[error("invalid schema version: {found} (expected {expected})")]
    InvalidSchemaVersion { found: String, expected: String },
    #[error("invalid jsonl row at line {line}: {message}")]
    InvalidJsonlRow { line: usize, message: String },
    #[error("invalid json: {message}")]
    InvalidJson { message: String },
    #[error("unsupported parquet: {message}")]
    UnsupportedParquet { message: String },
}

/// # Errors
/// Returns an error if the facts file is missing or has invalid schema/rows.
pub fn load_facts(path: &Path) -> std::result::Result<Vec<FactsRowV1>, AnalyzeError> {
    if !path.exists() {
        return Err(AnalyzeError::MissingFile {
            path: path.display().to_string(),
        });
    }
    let file = std::fs::File::open(path).map_err(|err| AnalyzeError::InvalidJson {
        message: err.to_string(),
    })?;
    let reader = BufReader::new(file);
    let mut rows = Vec::new();
    for (idx, line) in reader.lines().enumerate() {
        let line = line.map_err(|err| AnalyzeError::InvalidJson {
            message: err.to_string(),
        })?;
        if line.trim().is_empty() {
            continue;
        }
        let parsed_row: FactsRowV1 =
            serde_json::from_str(&line).map_err(|err| AnalyzeError::InvalidJsonlRow {
                line: idx + 1,
                message: err.to_string(),
            })?;
        if parsed_row.schema_version != "bijux.facts.v1" {
            return Err(AnalyzeError::InvalidSchemaVersion {
                found: parsed_row.schema_version,
                expected: "bijux.facts.v1".to_string(),
            });
        }
        rows.push(parsed_row);
    }
    stable_sort_records(&mut rows, |row| {
        (
            row.run_id.as_str(),
            row.stage_id.as_str(),
            row.tool_id.as_str(),
            row.params_hash.as_str(),
            "",
        )
    });
    Ok(rows)
}

/// # Errors
/// Returns an error if the parquet reader is not enabled.
#[cfg(not(feature = "parquet"))]
pub fn load_facts_parquet(_path: &Path) -> std::result::Result<Vec<FactsRowV1>, AnalyzeError> {
    Err(AnalyzeError::UnsupportedParquet {
        message: "enable the parquet feature to read parquet facts".to_string(),
    })
}

/// # Errors
/// Returns an error if the parquet reader fails.
#[cfg(feature = "parquet")]
pub fn load_facts_parquet(path: &Path) -> std::result::Result<Vec<FactsRowV1>, AnalyzeError> {
    use parquet::file::reader::{FileReader, SerializedFileReader};
    let file = std::fs::File::open(path).map_err(|err| AnalyzeError::InvalidJson {
        message: err.to_string(),
    })?;
    let reader = SerializedFileReader::new(file).map_err(|err| AnalyzeError::InvalidJson {
        message: err.to_string(),
    })?;
    let mut rows = Vec::new();
    let iter = reader
        .get_row_iter(None)
        .map_err(|err| AnalyzeError::InvalidJson {
            message: err.to_string(),
        })?;
    for record in iter {
        let record = record.map_err(|err| AnalyzeError::InvalidJson {
            message: err.to_string(),
        })?;
        let value = record.to_json_value();
        let parsed: FactsRowV1 =
            serde_json::from_value(value).map_err(|err| AnalyzeError::InvalidJson {
                message: err.to_string(),
            })?;
        if parsed.schema_version != "bijux.facts.v1" {
            return Err(AnalyzeError::InvalidSchemaVersion {
                found: parsed.schema_version,
                expected: "bijux.facts.v1".to_string(),
            });
        }
        rows.push(parsed);
    }
    stable_sort_records(&mut rows, |row| {
        (
            row.run_id.as_str(),
            row.stage_id.as_str(),
            row.tool_id.as_str(),
            row.params_hash.as_str(),
            "",
        )
    });
    Ok(rows)
}

/// # Errors
/// Returns an error if facts cannot be loaded from jsonl or parquet.
pub fn load_facts_auto(path: &Path) -> std::result::Result<Vec<FactsRowV1>, AnalyzeError> {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("parquet") => load_facts_parquet(path),
        _ => load_facts(path),
    }
}

/// # Errors
/// Returns an error if facts loading or validation fails.
pub fn load_fact_table(path: &Path) -> std::result::Result<FactTable, AnalyzeError> {
    let rows = load_facts(path)?;
    FactTable::from_facts(&rows).map_err(|err| AnalyzeError::InvalidJson {
        message: err.to_string(),
    })
}

/// # Errors
/// Returns an error if the run summary is missing or has invalid schema.
pub fn load_run_summary(path: &Path) -> std::result::Result<RunSummaryV1, AnalyzeError> {
    if !path.exists() {
        return Err(AnalyzeError::MissingFile {
            path: path.display().to_string(),
        });
    }
    let raw = std::fs::read_to_string(path).map_err(|err| AnalyzeError::InvalidJson {
        message: err.to_string(),
    })?;
    let summary: RunSummaryV1 =
        serde_json::from_str(&raw).map_err(|err| AnalyzeError::InvalidJson {
            message: err.to_string(),
        })?;
    if summary.schema_version != "bijux.run_summary.v1" {
        return Err(AnalyzeError::InvalidSchemaVersion {
            found: summary.schema_version,
            expected: "bijux.run_summary.v1".to_string(),
        });
    }
    Ok(summary)
}

/// # Errors
/// Returns an error if the run index is missing or rows are invalid.
pub fn load_run_index(path: &Path) -> std::result::Result<Vec<RunIndexLine>, AnalyzeError> {
    if !path.exists() {
        return Err(AnalyzeError::MissingFile {
            path: path.display().to_string(),
        });
    }
    let raw = std::fs::read_to_string(path).map_err(|err| AnalyzeError::InvalidJson {
        message: err.to_string(),
    })?;
    let mut rows = Vec::new();
    for (idx, line) in raw.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let parsed_row: RunIndexLine =
            serde_json::from_str(line).map_err(|err| AnalyzeError::InvalidJsonlRow {
                line: idx + 1,
                message: err.to_string(),
            })?;
        if parsed_row.schema_version != 1 {
            return Err(AnalyzeError::InvalidSchemaVersion {
                found: parsed_row.schema_version.to_string(),
                expected: "1".to_string(),
            });
        }
        rows.push(parsed_row);
    }
    Ok(rows)
}

fn json_from_str<T: DeserializeOwned>(value: &str) -> std::result::Result<T, rusqlite::Error> {
    serde_json::from_str(value).map_err(|err| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(err))
    })
}

/// # Errors
/// Returns an error if the database cannot be opened.
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
        return Err(BenchError::Validation(format!(
            "unsupported schema version {current}"
        )));
    }
    Ok(())
}

fn ensure_inserted_at_column(conn: &Connection, table: &str) -> Result<()> {
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

fn ensure_record_id_column(conn: &Connection, table: &str) -> Result<()> {
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

fn ensure_params_hash_column(conn: &Connection, table: &str) -> Result<()> {
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

fn ensure_identity_index(conn: &Connection, table: &str) -> Result<()> {
    let index_name = format!("{table}_identity_idx");
    let sql = format!(
        "CREATE UNIQUE INDEX IF NOT EXISTS {index_name} \
         ON {table} (tool, tool_version, image_digest, runner, platform, input_hash, params_hash)"
    );
    conn.execute(&sql, [])?;
    Ok(())
}

fn ensure_image_qa_identity_index(conn: &Connection) -> Result<()> {
    let sql = "CREATE UNIQUE INDEX IF NOT EXISTS image_qa_v1_identity_idx \
               ON image_qa_v1 (tool, stage, tool_version, image_digest, runner, platform, input_hash)";
    conn.execute(sql, [])?;
    Ok(())
}

fn params_hash(parameters: &JsonBlob) -> Result<String> {
    let canonical = bijux_core::parameters_json_canonicalization(parameters.as_value());
    let bytes = serde_json::to_vec(&canonical)?;
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    Ok(format!("{:x}", hasher.finalize()))
}

/// Insert a `FastQ` trim benchmark record into the v1 table.
///
/// # Errors
/// Returns an error if the table cannot be created or the record cannot be inserted.
pub fn insert_fastq_trim_v1(
    conn: &Connection,
    record: &BenchmarkRecord<FastqTrimMetrics>,
) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS bench_fastq_trim_v1 (\
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
    ensure_inserted_at_column(conn, "bench_fastq_trim_v1")?;
    ensure_record_id_column(conn, "bench_fastq_trim_v1")?;
    ensure_params_hash_column(conn, "bench_fastq_trim_v1")?;
    ensure_identity_index(conn, "bench_fastq_trim_v1")?;

    let metrics_json = serde_json::to_string(&record.metrics)?;
    let parameters_json = serde_json::to_string(&record.context.parameters)?;
    let params_hash = params_hash(&record.context.parameters)?;

    conn.execute(
        "INSERT INTO bench_fastq_trim_v1 (\
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

/// Insert a `FastQ` trim benchmark record into the v2 table.
///
/// # Errors
/// Returns an error if the table cannot be created or the record cannot be inserted.
pub fn insert_fastq_trim_v2(
    conn: &Connection,
    record: &BenchmarkRecord<FastqTrimMetrics>,
) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS bench_fastq_trim_v2 (\
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
    ensure_inserted_at_column(conn, "bench_fastq_trim_v2")?;
    ensure_record_id_column(conn, "bench_fastq_trim_v2")?;
    ensure_params_hash_column(conn, "bench_fastq_trim_v2")?;
    ensure_identity_index(conn, "bench_fastq_trim_v2")?;

    let metrics_json = serde_json::to_string(&record.metrics)?;
    let parameters_json = serde_json::to_string(&record.context.parameters)?;
    let params_hash = params_hash(&record.context.parameters)?;

    conn.execute(
        "INSERT INTO bench_fastq_trim_v2 (\
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

/// Load a trim benchmark record from `SQLite` if present.
///
/// # Errors
/// Returns an error if the query or JSON parsing fails.
/// Deterministic ordering: when multiple rows exist, pick the most recent by `inserted_at`.
#[allow(clippy::too_many_arguments)]
pub fn fetch_fastq_trim_v1(
    conn: &Connection,
    tool: &str,
    tool_version: &str,
    image_digest: &str,
    runner: &str,
    platform: &str,
    input_hash: &str,
    params_hash: &str,
) -> Result<Option<BenchmarkRecord<FastqTrimMetrics>>> {
    let mut stmt = conn.prepare(
        "SELECT tool, tool_version, image_digest, runner, platform, input_hash, params_hash,\
         parameters_json, runtime_s, memory_mb, exit_code, metrics_json \
         FROM bench_fastq_trim_v1 \
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
            let metrics: MetricSet<FastqTrimMetrics> = json_from_str(&metrics_json)?;
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

/// Load a trim benchmark record from `SQLite` v2 if present.
///
/// # Errors
/// Returns an error if the query or JSON parsing fails.
#[allow(clippy::too_many_arguments)]
pub fn fetch_fastq_trim_v2(
    conn: &Connection,
    tool: &str,
    tool_version: &str,
    image_digest: &str,
    runner: &str,
    platform: &str,
    input_hash: &str,
    params_hash: &str,
) -> Result<Option<BenchmarkRecord<FastqTrimMetrics>>> {
    let mut stmt = conn.prepare(
        "SELECT tool, tool_version, image_digest, runner, platform, input_hash, params_hash,\
         parameters_json, runtime_s, memory_mb, exit_code, metrics_json \
         FROM bench_fastq_trim_v2 \
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
            let metrics: MetricSet<FastqTrimMetrics> = json_from_str(&metrics_json)?;
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

/// Insert a `FastQ` validate benchmark record into the v1 table.
///
/// # Errors
/// Returns an error if the table cannot be created or the record cannot be inserted.
pub fn insert_fastq_validate_v1(
    conn: &Connection,
    record: &BenchmarkRecord<FastqValidateMetrics>,
) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS bench_fastq_validate_v1 (\
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
    ensure_inserted_at_column(conn, "bench_fastq_validate_v1")?;
    ensure_record_id_column(conn, "bench_fastq_validate_v1")?;
    ensure_params_hash_column(conn, "bench_fastq_validate_v1")?;
    ensure_identity_index(conn, "bench_fastq_validate_v1")?;

    let metrics_json = serde_json::to_string(&record.metrics)?;
    let parameters_json = serde_json::to_string(&record.context.parameters)?;
    let params_hash = params_hash(&record.context.parameters)?;

    conn.execute(
        "INSERT INTO bench_fastq_validate_v1 (\
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

/// Load a validate benchmark record from `SQLite` if present.
///
/// # Errors
/// Returns an error if the query or JSON parsing fails.
#[allow(clippy::too_many_arguments)]
pub fn fetch_fastq_validate_v1(
    conn: &Connection,
    tool: &str,
    tool_version: &str,
    image_digest: &str,
    runner: &str,
    platform: &str,
    input_hash: &str,
    params_hash: &str,
) -> Result<Option<BenchmarkRecord<FastqValidateMetrics>>> {
    let mut stmt = conn.prepare(
        "SELECT tool, tool_version, image_digest, runner, platform, input_hash, params_hash,\
         parameters_json, runtime_s, memory_mb, exit_code, metrics_json \
         FROM bench_fastq_validate_v1 \
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
            let metrics: MetricSet<FastqValidateMetrics> = json_from_str(&metrics_json)?;
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

/// Insert a `FastQ` filter benchmark record into the v1 table.
///
/// # Errors
/// Returns an error if the table cannot be created or the record cannot be inserted.
pub fn insert_fastq_filter_v1(
    conn: &Connection,
    record: &BenchmarkRecord<FastqFilterMetrics>,
) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS bench_fastq_filter_v1 (\
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
    ensure_inserted_at_column(conn, "bench_fastq_filter_v1")?;
    ensure_record_id_column(conn, "bench_fastq_filter_v1")?;
    ensure_params_hash_column(conn, "bench_fastq_filter_v1")?;
    ensure_identity_index(conn, "bench_fastq_filter_v1")?;

    let metrics_json = serde_json::to_string(&record.metrics)?;
    let parameters_json = serde_json::to_string(&record.context.parameters)?;
    let params_hash = params_hash(&record.context.parameters)?;

    conn.execute(
        "INSERT INTO bench_fastq_filter_v1 (\
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

/// Insert a `FastQ` filter benchmark record into the v2 table.
///
/// # Errors
/// Returns an error if the table cannot be created or the record cannot be inserted.
pub fn insert_fastq_filter_v2(
    conn: &Connection,
    record: &BenchmarkRecord<FastqFilterMetrics>,
) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS bench_fastq_filter_v2 (\
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
    ensure_inserted_at_column(conn, "bench_fastq_filter_v2")?;
    ensure_record_id_column(conn, "bench_fastq_filter_v2")?;
    ensure_params_hash_column(conn, "bench_fastq_filter_v2")?;
    ensure_identity_index(conn, "bench_fastq_filter_v2")?;

    let metrics_json = serde_json::to_string(&record.metrics)?;
    let parameters_json = serde_json::to_string(&record.context.parameters)?;
    let params_hash = params_hash(&record.context.parameters)?;

    conn.execute(
        "INSERT INTO bench_fastq_filter_v2 (\
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

/// Load a filter benchmark record from `SQLite` if present.
///
/// # Errors
/// Returns an error if the query or JSON parsing fails.
#[allow(clippy::too_many_arguments)]
pub fn fetch_fastq_filter_v1(
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
         FROM bench_fastq_filter_v1 \
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

/// Insert a `FastQ` correct benchmark record into the v1 table.
///
/// # Errors
/// Returns an error if the table cannot be created or the record cannot be inserted.
pub fn insert_fastq_correct_v1(
    conn: &Connection,
    record: &BenchmarkRecord<FastqCorrectMetrics>,
) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS bench_fastq_correct_v1 (\
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
    ensure_inserted_at_column(conn, "bench_fastq_correct_v1")?;
    ensure_record_id_column(conn, "bench_fastq_correct_v1")?;
    ensure_params_hash_column(conn, "bench_fastq_correct_v1")?;
    ensure_identity_index(conn, "bench_fastq_correct_v1")?;

    let metrics_json = serde_json::to_string(&record.metrics)?;
    let parameters_json = serde_json::to_string(&record.context.parameters)?;
    let params_hash = params_hash(&record.context.parameters)?;

    conn.execute(
        "INSERT INTO bench_fastq_correct_v1 (\
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

/// Load a correct benchmark record from `SQLite` if present.
///
/// # Errors
/// Returns an error if the query or JSON parsing fails.
#[allow(clippy::too_many_arguments)]
pub fn fetch_fastq_correct_v1(
    conn: &Connection,
    tool: &str,
    tool_version: &str,
    image_digest: &str,
    runner: &str,
    platform: &str,
    input_hash: &str,
    params_hash: &str,
) -> Result<Option<BenchmarkRecord<FastqCorrectMetrics>>> {
    let mut stmt = conn.prepare(
        "SELECT tool, tool_version, image_digest, runner, platform, input_hash, params_hash,\
         parameters_json, runtime_s, memory_mb, exit_code, metrics_json \
         FROM bench_fastq_correct_v1 \
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
            let metrics: MetricSet<FastqCorrectMetrics> = json_from_str(&metrics_json)?;
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

/// Insert a `FastQ` `qc_post` benchmark record into the v1 table.
///
/// # Errors
/// Returns an error if the table cannot be created or the record cannot be inserted.
pub fn insert_fastq_qc_post_v1(
    conn: &Connection,
    record: &BenchmarkRecord<FastqQcPostMetrics>,
) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS bench_fastq_qc_post_v1 (\
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
    ensure_inserted_at_column(conn, "bench_fastq_qc_post_v1")?;
    ensure_record_id_column(conn, "bench_fastq_qc_post_v1")?;
    ensure_params_hash_column(conn, "bench_fastq_qc_post_v1")?;
    ensure_identity_index(conn, "bench_fastq_qc_post_v1")?;

    let metrics_json = serde_json::to_string(&record.metrics)?;
    let parameters_json = serde_json::to_string(&record.context.parameters)?;
    let params_hash = params_hash(&record.context.parameters)?;

    conn.execute(
        "INSERT INTO bench_fastq_qc_post_v1 (\
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
            let metrics: MetricSet<FastqQcPostMetrics> = json_from_str(&metrics_json)?;
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
            let metrics: MetricSet<FastqUmiMetrics> = json_from_str(&metrics_json)?;
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
            let metrics: MetricSet<FastqScreenMetrics> = json_from_str(&metrics_json)?;
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
            let metrics: MetricSet<FastqStatsMetrics> = json_from_str(&metrics_json)?;
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

/// Insert an image QA record into the v1 table.
///
/// # Errors
/// Returns an error if the table cannot be created or the record cannot be inserted.
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
