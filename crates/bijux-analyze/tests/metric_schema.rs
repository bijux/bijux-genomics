use bijux_analyze::{BenchmarkRecord, FastqDeltaMetrics, FastqTrimMetrics, MetricSet};
use bijux_measure::ExecutionMetrics;

fn base_record(metrics: MetricSet<FastqTrimMetrics>) -> BenchmarkRecord<FastqTrimMetrics> {
    BenchmarkRecord {
        context: bijux_analyze::BenchmarkContext {
            tool: "fastp".to_string(),
            tool_version: "0.23.4".to_string(),
            image_digest: "sha256:abc".to_string(),
            runner: "docker".to_string(),
            platform: "local".to_string(),
            input_hash: "sha256:deadbeef".to_string(),
            parameters: serde_json::json!({"sample_id": "s1"}),
        },
        execution: ExecutionMetrics {
            runtime_s: 1.0,
            memory_mb: 32.0,
            exit_code: 0,
        },
        metrics,
    }
}

#[test]
fn metrics_schema_matches_stage_and_version() {
    let record = base_record(MetricSet::new(FastqTrimMetrics {
        reads_in: 100,
        reads_out: 90,
        bases_in: 1000,
        bases_out: 900,
        mean_q_before: 30.0,
        mean_q_after: 31.0,
        delta_metrics: FastqDeltaMetrics {
            read_retention: 0.9,
            base_retention: 0.9,
            mean_q_delta: 1.0,
            gc_delta: 0.1,
        },
    }));
    assert!(record.validate().is_ok());
    let schema = record.metrics.metrics_schema;
    assert_eq!(schema, "fastq_trim_v2");
}

#[test]
fn metrics_schema_rejects_unknown() {
    let mut metrics = MetricSet::new(FastqTrimMetrics {
        reads_in: 100,
        reads_out: 90,
        bases_in: 1000,
        bases_out: 900,
        mean_q_before: 30.0,
        mean_q_after: 31.0,
        delta_metrics: FastqDeltaMetrics {
            read_retention: 0.9,
            base_retention: 0.9,
            mean_q_delta: 1.0,
            gc_delta: 0.1,
        },
    });
    metrics.metrics_schema = "fastq_trim_v1".to_string();
    let record = base_record(metrics);
    match record.validate() {
        Ok(()) => panic!("expected schema mismatch"),
        Err(err) => assert!(err.to_string().contains("metric schema mismatch")),
    }
}

#[test]
fn metrics_schema_rejects_mixed_stage() {
    let mut metrics = MetricSet::new(FastqTrimMetrics {
        reads_in: 100,
        reads_out: 90,
        bases_in: 1000,
        bases_out: 900,
        mean_q_before: 30.0,
        mean_q_after: 31.0,
        delta_metrics: FastqDeltaMetrics {
            read_retention: 0.9,
            base_retention: 0.9,
            mean_q_delta: 1.0,
            gc_delta: 0.1,
        },
    });
    metrics.metrics_schema = "fastq_validate_v1".to_string();
    let record = base_record(metrics);
    match record.validate() {
        Ok(()) => panic!("expected schema mismatch"),
        Err(err) => assert!(err.to_string().contains("metric schema mismatch")),
    }
}

#[test]
fn execution_metrics_require_positive_values() {
    let mut record = base_record(MetricSet::new(FastqTrimMetrics {
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
    }));
    record.execution.runtime_s = 0.0;
    match record.validate() {
        Ok(()) => panic!("expected runtime error"),
        Err(err) => assert!(err.to_string().contains("runtime_s")),
    }
    record.execution.runtime_s = 1.0;
    record.execution.memory_mb = 0.0;
    match record.validate() {
        Ok(()) => panic!("expected memory error"),
        Err(err) => assert!(err.to_string().contains("memory_mb")),
    }
}
