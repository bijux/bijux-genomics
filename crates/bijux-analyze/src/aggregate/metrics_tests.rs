use super::*;
use crate::aggregate::Result;
use crate::model::JsonBlob;
use bijux_core::measure::ExecutionMetrics;

#[test]
fn record_serializes() -> Result<()> {
    let record = BenchmarkRecord {
        context: BenchmarkContext {
            tool: "fastp".to_string(),
            tool_version: "0.23.4".to_string(),
            image_digest: "sha256:abc".to_string(),
            runner: "docker".to_string(),
            platform: "local".to_string(),
            input_hash: "sha256:deadbeef".to_string(),
            parameters: JsonBlob::from_pairs(&[("adapter", "AGAT")]),
        },
        execution: ExecutionMetrics {
            runtime_s: 1.0,
            memory_mb: 10.0,
            exit_code: 0,
        },
        metrics: metric_set(FastqTrimMetrics {
            reads_in: 100,
            reads_out: 90,
            bases_in: 1000,
            bases_out: 900,
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
            adapter_preset: None,
            adapter_bank_id: None,
            adapter_bank_hash: None,
            adapter_overrides: None,
        }),
    };
    record.validate()?;
    let json = serde_json::to_string(&record)?;
    assert!(json.contains("fastp"));
    assert!(json.contains("metrics_schema"));
    Ok(())
}

#[test]
fn filter_metrics_invariants() -> Result<()> {
    let metrics = FastqFilterMetrics {
        reads_in: 100,
        reads_out: 80,
        reads_dropped: 20,
        reads_removed_by_n: 10,
        reads_removed_by_entropy: 5,
        reads_removed_low_complexity: 0,
        reads_removed_by_kmer: 5,
        reads_removed_contaminant_kmer: 0,
        reads_removed_by_length: 0,
        bases_in: 1000,
        bases_out: 800,
        pairs_in: None,
        pairs_out: None,
        mean_q_before: 30.0,
        mean_q_after: 29.0,
        delta_metrics: FastqDeltaMetrics {
            read_retention: 0.8,
            base_retention: 0.8,
            mean_q_delta: -1.0,
            gc_delta: 0.0,
        },
    };
    metrics.validate()?;
    let invalid = FastqFilterMetrics {
        reads_in: 100,
        reads_out: 81,
        reads_dropped: 20,
        reads_removed_by_n: 10,
        reads_removed_by_entropy: 10,
        reads_removed_low_complexity: 0,
        reads_removed_by_kmer: 5,
        reads_removed_contaminant_kmer: 0,
        reads_removed_by_length: 0,
        bases_in: 1000,
        bases_out: 810,
        pairs_in: None,
        pairs_out: None,
        mean_q_before: 30.0,
        mean_q_after: 31.0,
        delta_metrics: FastqDeltaMetrics {
            read_retention: 0.81,
            base_retention: 0.81,
            mean_q_delta: 1.0,
            gc_delta: 0.0,
        },
    };
    assert!(invalid.validate().is_err());
    Ok(())
}

#[test]
fn merge_metrics_invariants() -> Result<()> {
    let metrics = FastqMergeMetrics {
        reads_in: 100,
        reads_out: 60,
        bases_in: 1000,
        bases_out: 600,
        pairs_in: 100,
        pairs_out: 60,
        reads_r1: 100,
        reads_r2: 120,
        reads_merged: 60,
        reads_unmerged: 40,
        merge_rate: 0.6,
    };
    metrics.validate()?;
    let invalid = FastqMergeMetrics {
        reads_in: 100,
        reads_out: 80,
        bases_in: 1000,
        bases_out: 800,
        pairs_in: 100,
        pairs_out: 80,
        reads_r1: 100,
        reads_r2: 100,
        reads_merged: 80,
        reads_unmerged: 30,
        merge_rate: 1.2,
    };
    assert!(invalid.validate().is_err());
    Ok(())
}
