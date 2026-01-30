use anyhow::Result;
use bijux_analyze::{fetch_fastq_trim_v1, metric_set, FastqDeltaMetrics, FastqTrimMetrics};
use rusqlite::Connection;

#[test]
fn fetch_latest_is_ordered() -> Result<()> {
    let conn = Connection::open_in_memory()?;
    conn.execute(
        "CREATE TABLE bench_fastq_trim_v1 (\
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

    let metrics = metric_set(FastqTrimMetrics {
        reads_in: 10,
        reads_out: 9,
        bases_in: 100,
        bases_out: 90,
        mean_q_before: 30.0,
        mean_q_after: 31.0,
        delta_metrics: FastqDeltaMetrics {
            read_retention: 0.9,
            base_retention: 0.9,
            mean_q_delta: 1.0,
            gc_delta: 0.1,
        },
        adapter_preset: None,
        adapter_bank_id: None,
        adapter_bank_hash: None,
        adapter_overrides: None,
    });
    let metrics_json = serde_json::to_string(&metrics)?;
    let parameters_json = serde_json::to_string(&serde_json::json!({"sample": "s1"}))?;

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
            &parameters_json,
            2,
            1.0,
            32.0,
            0,
            &metrics_json,
            "2024-01-01T00:00:00Z",
        ),
    )?;
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
            &parameters_json,
            2,
            2.0,
            32.0,
            0,
            &metrics_json,
            "2024-01-02T00:00:00Z",
        ),
    )?;

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
