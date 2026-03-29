#![cfg(feature = "sqlite")]

use anyhow::Result;
use bijux_dna_analyze::load::sqlite::core_trim::fetch_fastq_trim_v1;
use bijux_dna_analyze::{metric_set, FastqDeltaMetrics, FastqTrimMetrics};

use rusqlite::Connection;

fn bench_trim_metrics() -> FastqTrimMetrics {
    FastqTrimMetrics {
        reads_in: 10,
        reads_out: 9,
        bases_in: 100,
        bases_out: 90,
        pairs_in: None,
        pairs_out: None,
        mean_q_before: 30.0,
        mean_q_after: 31.0,
        delta_metrics: FastqDeltaMetrics {
            read_retention: 0.9,
            base_retention: 0.9,
            mean_q_delta: 1.0,
            gc_delta: 0.1,
        },
        paired_mode: None,
        adapter_policy: None,
        polyx_policy: None,
        n_policy: None,
        contaminant_policy: None,
        raw_backend_report_format: None,
        adapter_preset: None,
        adapter_bank_id: None,
        adapter_bank_hash: None,
        adapter_overrides: None,
    }
}

fn create_trim_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE bench_fastq_trim_v1 (\
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
         inserted_at TEXT NOT NULL\
         )",
        [],
    )?;
    Ok(())
}

fn insert_trim_record(
    conn: &Connection,
    runtime_s: f64,
    metrics_json: &str,
    parameters_json: &str,
) -> Result<()> {
    conn.execute(
        "INSERT INTO bench_fastq_trim_v1 (\
         tool, tool_version, image_digest, runner, platform, input_hash, params_hash,\
         parameters_json, schema_version, runtime_s, memory_mb, exit_code, metrics_json, inserted_at\
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
        (
            "fastp",
            "0.23.4",
            "sha256:abc",
            "docker",
            "local",
            "sha256:input",
            "ph",
            parameters_json,
            2,
            runtime_s,
            32.0,
            0,
            metrics_json,
            if runtime_s < 2.0 {
                "2024-01-01T00:00:00Z"
            } else {
                "2024-01-02T00:00:00Z"
            },
        ),
    )?;
    Ok(())
}

#[test]
fn fetch_latest_is_ordered() -> Result<()> {
    let conn = Connection::open_in_memory()?;
    create_trim_table(&conn)?;

    let metrics = metric_set(bench_trim_metrics());
    let metrics_json = serde_json::to_string(&metrics)?;
    let parameters_json = serde_json::to_string(&serde_json::json!({"sample": "s1"}))?;

    insert_trim_record(&conn, 1.0, &metrics_json, &parameters_json)?;
    insert_trim_record(&conn, 2.0, &metrics_json, &parameters_json)?;

    let record = fetch_fastq_trim_v1(
        &conn,
        "fastp",
        "0.23.4",
        "sha256:abc",
        "docker",
        "local",
        "sha256:input",
        "ph",
    )?
    .ok_or_else(|| anyhow::anyhow!("missing record"))?;

    assert!((record.execution.runtime_s - 2.0).abs() < 1e-6);
    Ok(())
}
