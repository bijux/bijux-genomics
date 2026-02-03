use bijux_bench::{
    insert_fastq_trim_v1, open_sqlite, BenchmarkContext, BenchmarkRecord, ExecutionMetrics,
    FastqTrimMetrics, StageMetricSchema,
};

#[test]
fn fastq_trim_metrics_invariants_fail() {
    let metrics = FastqTrimMetrics {
        reads_in: 10,
        reads_out: 11,
        bases_in: 100,
        bases_out: 90,
        mean_q_before: 30.0,
        mean_q_after: 31.0,
    };
    let err = match metrics.validate() {
        Ok(()) => panic!("expected invariant failure"),
        Err(err) => err,
    };
    let msg = err.to_string();
    assert!(msg.contains("reads_out"));
}

#[test]
fn sqlite_insert_fastq_trim_v1() -> Result<(), Box<dyn std::error::Error>> {
    let record = BenchmarkRecord {
        context: BenchmarkContext {
            tool: "fastp".to_string(),
            tool_version: "0.23.4".to_string(),
            image_digest: "sha256:abc".to_string(),
            runner: "docker".to_string(),
            platform: "local".to_string(),
            input_hash: "sha256:deadbeef".to_string(),
            parameters: serde_json::json!({"sample_id": "s1", "r1": "reads.fastq.gz"}),
        },
        execution: ExecutionMetrics {
            runtime_s: 1.2,
            memory_mb: 42.0,
            exit_code: 0,
        },
        metrics: FastqTrimMetrics {
            reads_in: 100,
            reads_out: 90,
            bases_in: 1000,
            bases_out: 900,
            mean_q_before: 30.0,
            mean_q_after: 31.0,
        },
    };

    let conn = open_sqlite(std::path::Path::new(":memory:"))?;
    insert_fastq_trim_v1(&conn, &record)?;

    let count: i64 = conn.query_row("SELECT COUNT(*) FROM bench_fastq_trim_v1", [], |row| {
        row.get(0)
    })?;
    assert_eq!(count, 1);
    Ok(())
}
