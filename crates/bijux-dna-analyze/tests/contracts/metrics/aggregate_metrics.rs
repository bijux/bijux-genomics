use bijux_dna_analyze::model::JsonBlob;
use bijux_dna_analyze::{
    metric_set, BenchmarkContext, BenchmarkRecord, FastqDeltaMetrics, FastqFilterMetrics,
    FastqMergeMetrics, FastqTrimMetrics, Result, StageMetricSchema,
};
use bijux_dna_core::prelude::measure::ExecutionMetrics;

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
        execution: ExecutionMetrics { runtime_s: 1.0, memory_mb: 10.0, exit_code: 0 },
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

fn refuses_unrepresentable_counts<T: StageMetricSchema + std::panic::RefUnwindSafe>(
    label: &str,
    metrics: &T,
) -> bool {
    let outcome = std::panic::catch_unwind(|| metrics.validate());
    let refused = matches!(outcome, Ok(Err(_)));
    eprintln!("count_admission {label}: {outcome:?}");
    refused
}

#[test]
fn unrepresentable_metric_count_sums_refuse_without_panicking() -> anyhow::Result<()> {
    use bijux_dna_analyze::{FastqLowComplexityMetrics, FastqValidateMetrics};
    let mut validate: FastqValidateMetrics = serde_json::from_value(serde_json::json!({
        "reads_in": u64::MAX, "reads_out": u64::MAX,
        "bases_in": u64::MAX, "bases_out": u64::MAX,
        "reads_total": u64::MAX, "reads_valid": u64::MAX - 1,
        "reads_invalid": 1, "mean_q": 32.0
    }))?;
    let mut filter: FastqFilterMetrics = serde_json::from_value(serde_json::json!({
        "reads_in": u64::MAX, "reads_out": u64::MAX - 1, "reads_dropped": 1,
        "reads_removed_by_n": 1, "bases_in": u64::MAX, "bases_out": 1,
        "mean_q_before": 30.0, "mean_q_after": 30.0
    }))?;
    let mut breakdown = filter.clone();
    breakdown.reads_out = 0;
    breakdown.reads_dropped = u64::MAX;
    breakdown.reads_removed_by_n = u64::MAX;
    let mut complexity: FastqLowComplexityMetrics = serde_json::from_value(serde_json::json!({
        "reads_in": u64::MAX, "reads_out": u64::MAX - 1,
        "reads_removed_low_complexity": 1, "bases_in": u64::MAX, "bases_out": 1,
        "mean_q_before": 30.0, "mean_q_after": 30.0
    }))?;
    let mut merge: FastqMergeMetrics = serde_json::from_value(serde_json::json!({
        "reads_r1": u64::MAX, "reads_r2": u64::MAX,
        "reads_merged": u64::MAX - 1, "reads_unmerged": 1, "merge_rate": 0.5
    }))?;
    validate.validate()?;
    filter.validate()?;
    breakdown.validate()?;
    complexity.validate()?;
    merge.validate()?;
    let mut malformed = serde_json::to_value(&validate)?;
    malformed["reads_valid"] = serde_json::json!(-1);
    assert!(serde_json::from_value::<FastqValidateMetrics>(malformed).is_err());
    let mut ordinary_invalid = validate.clone();
    ordinary_invalid.reads_valid = 0;
    ordinary_invalid.reads_invalid = 0;
    ordinary_invalid.mean_q = f64::NAN;
    assert!(ordinary_invalid
        .validate()
        .expect_err("invalid sum")
        .to_string()
        .contains("reads_valid + reads_invalid must equal reads_total"));

    validate.reads_valid = u64::MAX;
    validate.reads_total = 0;
    filter.reads_out = u64::MAX;
    filter.reads_in = 0;
    breakdown.reads_removed_by_entropy = 1;
    complexity.reads_out = u64::MAX;
    complexity.reads_in = 0;
    merge.reads_merged = u64::MAX;
    let observations = [
        refuses_unrepresentable_counts("validate total", &validate),
        refuses_unrepresentable_counts("filter total", &filter),
        refuses_unrepresentable_counts("filter breakdown", &breakdown),
        refuses_unrepresentable_counts("low-complexity total", &complexity),
        refuses_unrepresentable_counts("merge total", &merge),
    ];
    assert!(
        observations.into_iter().all(|refused| refused),
        "unrepresentable count sums must return validation errors in debug and release"
    );
    Ok(())
}
