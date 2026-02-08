// Owner: bijux-dna-analyze
// Benchmark reporting helpers.

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use bijux_dna_infra::atomic_write_bytes;

use crate::aggregate::{
    derived_metric_spec, derived_metrics_for_stage, metric_kind_for_stage, metric_spec,
    stage_metric_spec, BenchmarkRecord, DerivedMetricId, FastqCorrectMetrics, FastqFilterMetrics,
    FastqMergeMetrics, FastqQcPostMetrics, FastqStatsMetrics, FastqTrimMetrics, FastqUmiMetrics,
    FastqValidateMetrics,
};
use crate::decision::score::{build_rankings, RankInput, RankingEntry};
use crate::failure::{classify_raw_failure, BenchmarkFailure};

pub fn gate_payload(failures: &[BenchmarkFailure]) -> serde_json::Value {
    let rationale: Vec<serde_json::Value> = failures
        .iter()
        .map(|failure| {
            serde_json::json!({
                "stage": failure.stage,
                "tool": failure.tool,
                "reason": failure.reason,
                "kind": format!("{:?}", failure.kind),
            })
        })
        .collect();
    serde_json::json!({
        "passes": failures.is_empty(),
        "rationale": rationale
    })
}
pub fn median(mut values: Vec<f64>) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mid = values.len() / 2;
    if values.len() % 2 == 0 {
        (values[mid - 1] + values[mid]) * 0.5
    } else {
        values[mid]
    }
}

#[allow(clippy::cast_precision_loss)]
pub fn u64_to_f64(value: u64) -> f64 {
    value as f64
}

pub fn ratio_u64(num: u64, denom: u64) -> f64 {
    if denom == 0 {
        0.0
    } else {
        u64_to_f64(num) / u64_to_f64(denom)
    }
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "snake_case")]
enum MetricValue {
    U64(u64),
    F64(f64),
}

#[derive(Debug, Clone, serde::Serialize)]
struct MetricDescriptor {
    value: MetricValue,
    meaning: &'static str,
    range: &'static str,
    notes: &'static str,
}

#[derive(Debug, Clone, serde::Serialize)]
struct IntegrityMetrics {
    reads_in: MetricDescriptor,
    reads_out: MetricDescriptor,
    bases_in: MetricDescriptor,
    bases_out: MetricDescriptor,
}

#[derive(Debug, Clone, serde::Serialize)]
struct RetentionMetrics {
    read_retention: MetricDescriptor,
    base_retention: MetricDescriptor,
}

#[derive(Debug, Clone, serde::Serialize)]
struct QualityShiftMetrics {
    mean_q_before: MetricDescriptor,
    mean_q_after: MetricDescriptor,
    delta_mean_q: MetricDescriptor,
}

#[derive(Debug, Clone, serde::Serialize)]
struct ContaminationMetrics {
    gc_before: MetricDescriptor,
    gc_after: MetricDescriptor,
    delta_gc: MetricDescriptor,
}

#[derive(Debug, Clone, serde::Serialize)]
struct SemanticMetrics {
    integrity: IntegrityMetrics,
    retention: RetentionMetrics,
    quality_shift: Option<QualityShiftMetrics>,
    contamination: Option<ContaminationMetrics>,
    interpretation: &'static str,
}

fn metric_u64(
    value: u64,
    meaning: &'static str,
    range: &'static str,
    notes: &'static str,
) -> MetricDescriptor {
    MetricDescriptor {
        value: MetricValue::U64(value),
        meaning,
        range,
        notes,
    }
}

fn metric_f64(
    value: f64,
    meaning: &'static str,
    range: &'static str,
    notes: &'static str,
) -> MetricDescriptor {
    MetricDescriptor {
        value: MetricValue::F64(value),
        meaning,
        range,
        notes,
    }
}

fn semantic_trim(metrics: &FastqTrimMetrics) -> SemanticMetrics {
    let delta = &metrics.delta_metrics;
    let interpretation = if delta.read_retention < 0.8 {
        "Trim removed a substantial fraction of reads; likely adapter/low-quality trimming."
    } else if delta.mean_q_delta > 0.5 {
        "Trim preserved most reads while improving mean quality."
    } else {
        "Trim preserved reads with minimal quality shift."
    };
    SemanticMetrics {
        integrity: IntegrityMetrics {
            reads_in: metric_u64(
                metrics.reads_in,
                "Input reads",
                ">= 0",
                "Raw input read count.",
            ),
            reads_out: metric_u64(
                metrics.reads_out,
                "Output reads",
                ">= 0",
                "Reads after trimming.",
            ),
            bases_in: metric_u64(metrics.bases_in, "Input bases", ">= 0", "Raw input bases."),
            bases_out: metric_u64(
                metrics.bases_out,
                "Output bases",
                ">= 0",
                "Bases after trimming.",
            ),
        },
        retention: RetentionMetrics {
            read_retention: metric_f64(
                delta.read_retention,
                "Reads retained",
                "[0,1]",
                "Fraction of reads retained.",
            ),
            base_retention: metric_f64(
                delta.base_retention,
                "Bases retained",
                "[0,1]",
                "Fraction of bases retained.",
            ),
        },
        quality_shift: Some(QualityShiftMetrics {
            mean_q_before: metric_f64(
                metrics.mean_q_before,
                "Mean Q before",
                "[0,45]",
                "Mean quality before trimming.",
            ),
            mean_q_after: metric_f64(
                metrics.mean_q_after,
                "Mean Q after",
                "[0,45]",
                "Mean quality after trimming.",
            ),
            delta_mean_q: metric_f64(
                delta.mean_q_delta,
                "Mean Q shift",
                "(-45,45)",
                "Mean quality change.",
            ),
        }),
        contamination: None,
        interpretation,
    }
}

fn semantic_filter(metrics: &FastqFilterMetrics) -> SemanticMetrics {
    let delta = &metrics.delta_metrics;
    let removed = metrics.reads_dropped;
    let interpretation = if removed == 0 {
        "Filter removed no reads; thresholds may be permissive."
    } else if metrics.reads_removed_by_kmer > 0 {
        "Filter removed reads matching contaminant k-mers; check contaminant panel."
    } else if metrics.reads_removed_by_entropy > 0 {
        "Filter removed low-complexity reads; entropy threshold likely aggressive."
    } else {
        "Filter removed reads based on thresholds; review removal breakdown."
    };
    SemanticMetrics {
        integrity: IntegrityMetrics {
            reads_in: metric_u64(
                metrics.reads_in,
                "Input reads",
                ">= 0",
                "Raw input read count.",
            ),
            reads_out: metric_u64(
                metrics.reads_out,
                "Output reads",
                ">= 0",
                "Reads after filtering.",
            ),
            bases_in: metric_u64(
                metrics.reads_in,
                "Input bases",
                ">= 0",
                "Approx bases inferred from reads.",
            ),
            bases_out: metric_u64(
                metrics.reads_out,
                "Output bases",
                ">= 0",
                "Approx bases after filtering.",
            ),
        },
        retention: RetentionMetrics {
            read_retention: metric_f64(
                delta.read_retention,
                "Reads retained",
                "[0,1]",
                "Fraction of reads retained.",
            ),
            base_retention: metric_f64(
                delta.base_retention,
                "Bases retained",
                "[0,1]",
                "Fraction of bases retained.",
            ),
        },
        quality_shift: Some(QualityShiftMetrics {
            mean_q_before: metric_f64(
                metrics.mean_q_before,
                "Mean Q before",
                "[0,45]",
                "Mean quality before filtering.",
            ),
            mean_q_after: metric_f64(
                metrics.mean_q_after,
                "Mean Q after",
                "[0,45]",
                "Mean quality after filtering.",
            ),
            delta_mean_q: metric_f64(
                delta.mean_q_delta,
                "Mean Q shift",
                "(-45,45)",
                "Mean quality change.",
            ),
        }),
        contamination: None,
        interpretation,
    }
}

fn semantic_validate(metrics: &FastqValidateMetrics) -> SemanticMetrics {
    let reads_total = metrics.reads_total;
    let reads_valid = metrics.reads_valid;
    let read_retention = if reads_total == 0 {
        0.0
    } else {
        u64_to_f64(reads_valid) / u64_to_f64(reads_total)
    };
    SemanticMetrics {
        integrity: IntegrityMetrics {
            reads_in: metric_u64(reads_total, "Input reads", ">= 0", "Raw input read count."),
            reads_out: metric_u64(
                reads_valid,
                "Output reads",
                ">= 0",
                "Reads passing validation.",
            ),
            bases_in: metric_u64(0, "Input bases", ">= 0", "Base counts not reported."),
            bases_out: metric_u64(0, "Output bases", ">= 0", "Base counts not reported."),
        },
        retention: RetentionMetrics {
            read_retention: metric_f64(
                read_retention,
                "Reads retained",
                "[0,1]",
                "Fraction of reads retained.",
            ),
            base_retention: metric_f64(0.0, "Bases retained", "[0,1]", "Not reported."),
        },
        quality_shift: None,
        contamination: None,
        interpretation: "Validation reports invalid reads; no data is modified.",
    }
}

fn semantic_stats(metrics: &FastqStatsMetrics) -> SemanticMetrics {
    let reads_total = metrics.reads_total;
    let bases_total = metrics.bases_total;
    let read_retention = if reads_total == 0 { 0.0 } else { 1.0 };
    let base_retention = if bases_total == 0 { 0.0 } else { 1.0 };
    SemanticMetrics {
        integrity: IntegrityMetrics {
            reads_in: metric_u64(reads_total, "Input reads", ">= 0", "Raw input read count."),
            reads_out: metric_u64(reads_total, "Output reads", ">= 0", "Reads after stats."),
            bases_in: metric_u64(bases_total, "Input bases", ">= 0", "Raw input bases."),
            bases_out: metric_u64(bases_total, "Output bases", ">= 0", "Bases after stats."),
        },
        retention: RetentionMetrics {
            read_retention: metric_f64(
                read_retention,
                "Reads retained",
                "[0,1]",
                "Fraction of reads retained.",
            ),
            base_retention: metric_f64(
                base_retention,
                "Bases retained",
                "[0,1]",
                "Fraction of bases retained.",
            ),
        },
        quality_shift: Some(QualityShiftMetrics {
            mean_q_before: metric_f64(
                metrics.mean_q,
                "Mean Q before",
                "[0,45]",
                "Mean quality before stats.",
            ),
            mean_q_after: metric_f64(
                metrics.mean_q,
                "Mean Q after",
                "[0,45]",
                "Mean quality after stats.",
            ),
            delta_mean_q: metric_f64(0.0, "Mean Q shift", "(-45,45)", "Mean quality change."),
        }),
        contamination: Some(ContaminationMetrics {
            gc_before: metric_f64(
                metrics.gc_percent,
                "GC% before",
                "[0,100]",
                "GC percent before stats.",
            ),
            gc_after: metric_f64(
                metrics.gc_percent,
                "GC% after",
                "[0,100]",
                "GC percent after stats.",
            ),
            delta_gc: metric_f64(0.0, "GC% shift", "(-100,100)", "GC percent change."),
        }),
        interpretation: "Stats summarizes input reads; no data is modified.",
    }
}

/// Rank trim tools by metrics and execution stats.
///
/// # Errors
/// Returns an error if ranking computation fails.
pub fn rank_trim_tools(
    records: &[BenchmarkRecord<FastqTrimMetrics>],
) -> Result<BTreeMap<String, Vec<RankingEntry>>> {
    let inputs = records
        .iter()
        .map(|record| RankInput {
            tool: record.context.tool.clone(),
            runtime_s: record.execution.runtime_s,
            memory_mb: record.execution.memory_mb,
            read_retention: Some(record.metrics.metrics.delta_metrics.read_retention),
            base_retention: Some(record.metrics.metrics.delta_metrics.base_retention),
            error_reduction_proxy: None,
        })
        .collect::<Vec<_>>();
    build_rankings(&inputs)
}

/// Rank validate tools by metrics and execution stats.
///
/// # Errors
/// Returns an error if ranking computation fails.
pub fn rank_validate_tools(
    records: &[BenchmarkRecord<FastqValidateMetrics>],
) -> Result<BTreeMap<String, Vec<RankingEntry>>> {
    let inputs = records
        .iter()
        .map(|record| {
            let retention = ratio_u64(
                record.metrics.metrics.reads_valid,
                record.metrics.metrics.reads_total,
            );
            RankInput {
                tool: record.context.tool.clone(),
                runtime_s: record.execution.runtime_s,
                memory_mb: record.execution.memory_mb,
                read_retention: Some(retention),
                base_retention: None,
                error_reduction_proxy: None,
            }
        })
        .collect::<Vec<_>>();
    build_rankings(&inputs)
}
