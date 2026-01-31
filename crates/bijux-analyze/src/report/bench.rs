//! Owner: bijux-analyze
//! Benchmark reporting helpers.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use bijux_core::RawFailure;

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
        values[mid - 1].midpoint(values[mid])
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

/// Rank filter tools by metrics and execution stats.
///
/// # Errors
/// Returns an error if ranking computation fails.
pub fn rank_filter_tools(
    records: &[BenchmarkRecord<FastqFilterMetrics>],
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

/// Rank merge tools by metrics and execution stats.
///
/// # Errors
/// Returns an error if ranking computation fails.
pub fn rank_merge_tools(
    records: &[BenchmarkRecord<FastqMergeMetrics>],
) -> Result<BTreeMap<String, Vec<RankingEntry>>> {
    let inputs = records
        .iter()
        .map(|record| RankInput {
            tool: record.context.tool.clone(),
            runtime_s: record.execution.runtime_s,
            memory_mb: record.execution.memory_mb,
            read_retention: None,
            base_retention: None,
            error_reduction_proxy: Some(record.metrics.metrics.merge_rate),
        })
        .collect::<Vec<_>>();
    build_rankings(&inputs)
}

/// Rank correct tools by metrics and execution stats.
///
/// # Errors
/// Returns an error if ranking computation fails.
pub fn rank_correct_tools(
    records: &[BenchmarkRecord<FastqCorrectMetrics>],
) -> Result<BTreeMap<String, Vec<RankingEntry>>> {
    let inputs = records
        .iter()
        .map(|record| RankInput {
            tool: record.context.tool.clone(),
            runtime_s: record.execution.runtime_s,
            memory_mb: record.execution.memory_mb,
            read_retention: None,
            base_retention: None,
            error_reduction_proxy: Some(record.metrics.metrics.kmer_fix_rate),
        })
        .collect::<Vec<_>>();
    build_rankings(&inputs)
}

/// Rank UMI tools by metrics and execution stats.
///
/// # Errors
/// Returns an error if ranking computation fails.
pub fn rank_umi_tools(
    records: &[BenchmarkRecord<FastqUmiMetrics>],
) -> Result<BTreeMap<String, Vec<RankingEntry>>> {
    let inputs = records
        .iter()
        .map(|record| RankInput {
            tool: record.context.tool.clone(),
            runtime_s: record.execution.runtime_s,
            memory_mb: record.execution.memory_mb,
            read_retention: Some(ratio_u64(
                record.metrics.metrics.reads_out,
                record.metrics.metrics.reads_in,
            )),
            base_retention: None,
            error_reduction_proxy: None,
        })
        .collect::<Vec<_>>();
    build_rankings(&inputs)
}

pub fn sanity_flags_trim(records: &[BenchmarkRecord<FastqTrimMetrics>]) -> Vec<serde_json::Value> {
    let mut flags = Vec::new();
    let retention = records
        .iter()
        .map(|record| record.metrics.metrics.delta_metrics.read_retention)
        .collect::<Vec<_>>();
    let median_retention = median(retention);
    if median_retention < 0.85 {
        flags.push(serde_json::json!({
            "flag": "low_read_retention",
            "severity": "warning",
            "message": "Trim retains less than 85% of reads; check adapter or quality thresholds.",
        }));
    }
    flags
}

pub fn sanity_flags_filter(
    records: &[BenchmarkRecord<FastqFilterMetrics>],
) -> Vec<serde_json::Value> {
    let mut flags = Vec::new();
    let retention = records
        .iter()
        .map(|record| record.metrics.metrics.delta_metrics.read_retention)
        .collect::<Vec<_>>();
    let median_retention = median(retention);
    if median_retention < 0.85 {
        flags.push(serde_json::json!({
            "flag": "low_read_retention",
            "severity": "warning",
            "message": "Filter retains less than 85% of reads; check filtering thresholds.",
        }));
    }
    flags
}

pub fn sanity_flags_correct(
    records: &[BenchmarkRecord<FastqCorrectMetrics>],
) -> Vec<serde_json::Value> {
    let mut flags = Vec::new();
    let rates = records
        .iter()
        .map(|record| record.metrics.metrics.kmer_fix_rate)
        .collect::<Vec<_>>();
    let median_rate = median(rates);
    if median_rate < 0.2 {
        flags.push(serde_json::json!({
            "flag": "low_kmer_fix_rate",
            "severity": "warning",
            "message": "Correct fixes fewer than 20% of k-mer errors; check k-mer parameters.",
        }));
    }
    flags
}

pub fn sanity_flags_umi(records: &[BenchmarkRecord<FastqUmiMetrics>]) -> Vec<serde_json::Value> {
    let mut flags = Vec::new();
    let retention = records
        .iter()
        .map(|record| {
            ratio_u64(
                record.metrics.metrics.reads_out,
                record.metrics.metrics.reads_in,
            )
        })
        .collect::<Vec<_>>();
    let median_retention = median(retention);
    if median_retention < 0.85 {
        flags.push(serde_json::json!({
            "flag": "low_read_retention",
            "severity": "warning",
            "message": "UMI processing retains less than 85% of reads; verify UMI parameters.",
        }));
    }
    flags
}

pub fn sanity_flags_merge(
    records: &[BenchmarkRecord<FastqMergeMetrics>],
) -> Vec<serde_json::Value> {
    let mut flags = Vec::new();
    let merge_rates = records
        .iter()
        .map(|record| record.metrics.metrics.merge_rate)
        .collect::<Vec<_>>();
    let median_merge = median(merge_rates);
    if median_merge < 0.2 {
        flags.push(serde_json::json!({
            "flag": "low_merge_rate",
            "severity": "warning",
            "message": "Merge rate below 20%; check insert size vs read length.",
        }));
    }
    flags
}

pub fn sanity_flags_stats(
    records: &[BenchmarkRecord<FastqStatsMetrics>],
) -> Vec<serde_json::Value> {
    let mut flags = Vec::new();
    let gc = records
        .iter()
        .map(|record| record.metrics.metrics.gc_percent)
        .collect::<Vec<_>>();
    let median_gc = median(gc);
    if !(20.0..=80.0).contains(&median_gc) {
        flags.push(serde_json::json!({
            "flag": "gc_out_of_range",
            "severity": "warning",
            "message": "Median GC% is outside expected range; check sample composition.",
        }));
    }
    flags
}

pub fn sanity_flags_validate(
    records: &[BenchmarkRecord<FastqValidateMetrics>],
) -> Vec<serde_json::Value> {
    let mut flags = Vec::new();
    let retention = records
        .iter()
        .map(|record| {
            ratio_u64(
                record.metrics.metrics.reads_valid,
                record.metrics.metrics.reads_total,
            )
        })
        .collect::<Vec<_>>();
    let median_retention = median(retention);
    if median_retention < 0.9 {
        flags.push(serde_json::json!({
            "flag": "low_validation_pass_rate",
            "severity": "warning",
            "message": "More than 10% of reads are invalid; check read integrity.",
        }));
    }
    flags
}

pub fn sanity_flags_qc_post(
    records: &[BenchmarkRecord<FastqQcPostMetrics>],
) -> Vec<serde_json::Value> {
    let mut flags = Vec::new();
    let contamination = records
        .iter()
        .map(|record| record.metrics.metrics.contamination_rate)
        .collect::<Vec<_>>();
    let median_contamination = median(contamination);
    if median_contamination > 0.05 {
        flags.push(serde_json::json!({
            "flag": "high_contamination",
            "severity": "warning",
            "message": "Contamination rate > 5%; check contaminant panel or sample prep.",
        }));
    }
    flags
}

pub fn derived_trim_metrics(record: &BenchmarkRecord<FastqTrimMetrics>) -> serde_json::Value {
    let delta = &record.metrics.metrics.delta_metrics;
    serde_json::json!({
        "read_retention": delta.read_retention,
        "base_retention": delta.base_retention,
        "mean_q_delta": delta.mean_q_delta,
        "gc_delta": delta.gc_delta,
    })
}

pub fn derived_filter_metrics(record: &BenchmarkRecord<FastqFilterMetrics>) -> serde_json::Value {
    let delta = &record.metrics.metrics.delta_metrics;
    serde_json::json!({
        "read_retention": delta.read_retention,
        "base_retention": delta.base_retention,
        "mean_q_delta": delta.mean_q_delta,
        "gc_delta": delta.gc_delta,
    })
}

pub fn derived_merge_metrics(record: &BenchmarkRecord<FastqMergeMetrics>) -> serde_json::Value {
    serde_json::json!({
        "merge_rate": record.metrics.metrics.merge_rate,
        "reads_merged": record.metrics.metrics.reads_merged,
        "reads_unmerged": record.metrics.metrics.reads_unmerged,
    })
}

pub fn derived_correct_metrics(record: &BenchmarkRecord<FastqCorrectMetrics>) -> serde_json::Value {
    serde_json::json!({
        "kmer_fix_rate": record.metrics.metrics.kmer_fix_rate,
    })
}

pub fn derived_umi_metrics(record: &BenchmarkRecord<FastqUmiMetrics>) -> serde_json::Value {
    serde_json::json!({
        "read_retention": ratio_u64(
            record.metrics.metrics.reads_out,
            record.metrics.metrics.reads_in,
        ),
    })
}

#[must_use]
pub fn derived_metrics_for_stage_json(stage: &str) -> Vec<serde_json::Value> {
    let mut derived = Vec::new();
    for metric in derived_metrics_for_stage(stage) {
        let spec = derived_metric_spec(metric.id);
        let derived_metric = match metric.id {
            DerivedMetricId::ReadRetention => serde_json::json!({
                "id": spec.name,
                "meaning": spec.meaning,
                "direction": format!("{:?}", spec.direction),
                "range": spec.range.map(|range| serde_json::json!({
                    "min": range.min,
                    "max": range.max,
                })),
            }),
            DerivedMetricId::BaseRetention => serde_json::json!({
                "id": spec.name,
                "meaning": spec.meaning,
                "direction": format!("{:?}", spec.direction),
                "range": spec.range.map(|range| serde_json::json!({
                    "min": range.min,
                    "max": range.max,
                })),
            }),
            DerivedMetricId::MergeEfficiency => serde_json::json!({
                "id": spec.name,
                "meaning": spec.meaning,
                "direction": format!("{:?}", spec.direction),
                "range": spec.range.map(|range| serde_json::json!({
                    "min": range.min,
                    "max": range.max,
                })),
            }),
            DerivedMetricId::ErrorReductionProxy => serde_json::json!({
                "id": spec.name,
                "meaning": spec.meaning,
                "direction": format!("{:?}", spec.direction),
                "range": spec.range.map(|range| serde_json::json!({
                    "min": range.min,
                    "max": range.max,
                })),
            }),
        };
        derived.push(derived_metric);
    }
    derived
}

/// Print the benchmark schema for a stage.
///
/// # Errors
/// Returns an error if the schema cannot be rendered.
pub fn print_bench_schema(stage: &str) -> Result<()> {
    let json = bench_schema_json(stage)?;
    println!("{}", serde_json::to_string_pretty(&json)?);
    Ok(())
}

/// Build the benchmark schema as JSON for a stage.
///
/// # Errors
/// Returns an error if the stage is unknown or serialization fails.
pub fn bench_schema_json(stage: &str) -> Result<serde_json::Value> {
    let kind = metric_kind_for_stage(stage).ok_or_else(|| anyhow!("unknown stage {stage}"))?;
    let spec = stage_metric_spec(kind);
    let metrics: Vec<_> = spec
        .metrics
        .iter()
        .map(|metric_id| {
            let metric = metric_spec(*metric_id);
            serde_json::json!({
                "name": metric.name,
                "meaning": metric.meaning,
                "direction": format!("{:?}", metric.direction),
                "range": metric.range.map(|range| serde_json::json!({
                    "min": range.min,
                    "max": range.max
                })),
                "measured": metric.measured,
                "derived": metric.derived,
            })
        })
        .collect();
    let derived: Vec<_> = spec
        .metrics
        .iter()
        .filter_map(|metric_id| {
            let metric = metric_spec(*metric_id);
            if metric.derived {
                Some(metric.name.to_string())
            } else {
                None
            }
        })
        .collect();
    Ok(serde_json::json!({
        "stage": stage,
        "schema_version": format!("{}_v{}", stage.replace('.', "_"), spec.version),
        "metrics": metrics,
        "derived": derived,
        "invariants": spec.invariants,
    }))
}

/// Write the trim benchmark report.
///
/// # Errors
/// Returns an error if report serialization or file writes fail.
pub fn write_trim_report(
    base_dir: &Path,
    records: &[BenchmarkRecord<FastqTrimMetrics>],
    failures: &[RawFailure],
    explain: bool,
) -> Result<()> {
    let path = base_dir.join("report.json");
    let mut report = BTreeMap::new();
    report.insert("records", serde_json::to_value(records)?);
    let classified: Vec<BenchmarkFailure> = failures.iter().map(classify_raw_failure).collect();
    report.insert("failures", serde_json::to_value(&classified)?);
    report.insert("gate", gate_payload(&classified));
    report.insert(
        "sanity_flags",
        serde_json::to_value(sanity_flags_trim(records))?,
    );
    let derived: Vec<_> = records.iter().map(derived_trim_metrics).collect();
    report.insert("derived_metrics", serde_json::to_value(&derived)?);
    let semantic: Vec<_> = records
        .iter()
        .map(|record| semantic_trim(&record.metrics.metrics))
        .collect();
    report.insert("semantic_metrics", serde_json::to_value(&semantic)?);
    let rankings = rank_trim_tools(records)?;
    report.insert("rankings", serde_json::to_value(&rankings)?);
    let json = serde_json::to_string_pretty(&report)?;
    fs::write(&path, json).context("write report.json")?;
    if explain {
        crate::decision::score::print_rank_explain("fastq.trim", &rankings);
    }
    Ok(())
}

/// Write the validate benchmark report.
///
/// # Errors
/// Returns an error if report serialization or file writes fail.
pub fn write_validate_report(
    base_dir: &Path,
    records: &[BenchmarkRecord<FastqValidateMetrics>],
    failures: &[RawFailure],
    qc_class: Option<&str>,
    explain: bool,
) -> Result<()> {
    let path = base_dir.join("report.json");
    let mut report = BTreeMap::new();
    report.insert("records", serde_json::to_value(records)?);
    let classified: Vec<BenchmarkFailure> = failures.iter().map(classify_raw_failure).collect();
    report.insert("failures", serde_json::to_value(&classified)?);
    report.insert("gate", gate_payload(&classified));
    report.insert(
        "sanity_flags",
        serde_json::to_value(sanity_flags_validate(records))?,
    );
    let semantic: Vec<_> = records
        .iter()
        .map(|record| semantic_validate(&record.metrics.metrics))
        .collect();
    report.insert("semantic_metrics", serde_json::to_value(&semantic)?);
    if let Some(class) = qc_class {
        report.insert("qc_class", serde_json::to_value(class)?);
    }
    let rankings = rank_validate_tools(records)?;
    report.insert("rankings", serde_json::to_value(&rankings)?);
    let json = serde_json::to_string_pretty(&report)?;
    fs::write(&path, json).context("write report.json")?;
    if explain {
        crate::decision::score::print_rank_explain("fastq.validate_pre", &rankings);
    }
    Ok(())
}

/// Write the filter benchmark report.
///
/// # Errors
/// Returns an error if report serialization or file writes fail.
pub fn write_filter_report(
    base_dir: &Path,
    records: &[BenchmarkRecord<FastqFilterMetrics>],
    failures: &[RawFailure],
    explain: bool,
) -> Result<()> {
    let path = base_dir.join("report.json");
    let mut report = BTreeMap::new();
    report.insert("records", serde_json::to_value(records)?);
    let classified: Vec<BenchmarkFailure> = failures.iter().map(classify_raw_failure).collect();
    report.insert("failures", serde_json::to_value(&classified)?);
    report.insert("gate", gate_payload(&classified));
    report.insert(
        "sanity_flags",
        serde_json::to_value(sanity_flags_filter(records))?,
    );
    let derived: Vec<_> = records.iter().map(derived_filter_metrics).collect();
    report.insert("derived_metrics", serde_json::to_value(&derived)?);
    let semantic: Vec<_> = records
        .iter()
        .map(|record| semantic_filter(&record.metrics.metrics))
        .collect();
    report.insert("semantic_metrics", serde_json::to_value(&semantic)?);
    let rankings = rank_filter_tools(records)?;
    report.insert("rankings", serde_json::to_value(&rankings)?);
    let json = serde_json::to_string_pretty(&report)?;
    fs::write(&path, json).context("write report.json")?;
    if explain {
        crate::decision::score::print_rank_explain("fastq.filter", &rankings);
    }
    Ok(())
}

/// Write the merge benchmark report.
///
/// # Errors
/// Returns an error if report serialization or file writes fail.
pub fn write_merge_report(
    base_dir: &Path,
    records: &[BenchmarkRecord<FastqMergeMetrics>],
    failures: &[RawFailure],
    explain: bool,
) -> Result<()> {
    let path = base_dir.join("report.json");
    let mut report = BTreeMap::new();
    report.insert("records", serde_json::to_value(records)?);
    let classified: Vec<BenchmarkFailure> = failures.iter().map(classify_raw_failure).collect();
    report.insert("failures", serde_json::to_value(&classified)?);
    report.insert("gate", gate_payload(&classified));
    report.insert(
        "sanity_flags",
        serde_json::to_value(sanity_flags_merge(records))?,
    );
    let derived: Vec<_> = records.iter().map(derived_merge_metrics).collect();
    report.insert("derived_metrics", serde_json::to_value(&derived)?);
    let rankings = rank_merge_tools(records)?;
    report.insert("rankings", serde_json::to_value(&rankings)?);
    let json = serde_json::to_string_pretty(&report)?;
    fs::write(&path, json).context("write report.json")?;
    if explain {
        crate::decision::score::print_rank_explain("fastq.merge", &rankings);
    }
    Ok(())
}

/// Write the correct benchmark report.
///
/// # Errors
/// Returns an error if report serialization or file writes fail.
pub fn write_correct_report(
    base_dir: &Path,
    records: &[BenchmarkRecord<FastqCorrectMetrics>],
    failures: &[RawFailure],
    explain: bool,
) -> Result<()> {
    let path = base_dir.join("report.json");
    let mut report = BTreeMap::new();
    report.insert("records", serde_json::to_value(records)?);
    let classified: Vec<BenchmarkFailure> = failures.iter().map(classify_raw_failure).collect();
    report.insert("failures", serde_json::to_value(&classified)?);
    report.insert("gate", gate_payload(&classified));
    report.insert(
        "sanity_flags",
        serde_json::to_value(sanity_flags_correct(records))?,
    );
    let derived: Vec<_> = records.iter().map(derived_correct_metrics).collect();
    report.insert("derived_metrics", serde_json::to_value(&derived)?);
    let rankings = rank_correct_tools(records)?;
    report.insert("rankings", serde_json::to_value(&rankings)?);
    let json = serde_json::to_string_pretty(&report)?;
    fs::write(&path, json).context("write report.json")?;
    if explain {
        crate::decision::score::print_rank_explain("fastq.correct", &rankings);
    }
    Ok(())
}

/// Write the qc-post benchmark report.
///
/// # Errors
/// Returns an error if report serialization or file writes fail.
pub fn write_qc_post_report(
    base_dir: &Path,
    records: &[BenchmarkRecord<FastqQcPostMetrics>],
    failures: &[RawFailure],
    explain: bool,
) -> Result<()> {
    let path = base_dir.join("report.json");
    let mut report = BTreeMap::new();
    report.insert("records", serde_json::to_value(records)?);
    let classified: Vec<BenchmarkFailure> = failures.iter().map(classify_raw_failure).collect();
    report.insert("failures", serde_json::to_value(&classified)?);
    report.insert("gate", gate_payload(&classified));
    report.insert(
        "sanity_flags",
        serde_json::to_value(sanity_flags_qc_post(records))?,
    );
    let json = serde_json::to_string_pretty(&report)?;
    fs::write(&path, json).context("write report.json")?;
    if explain {
        crate::decision::score::print_rank_explain("fastq.qc_post", &BTreeMap::new());
    }
    Ok(())
}

/// Write the umi benchmark report.
///
/// # Errors
/// Returns an error if report serialization or file writes fail.
pub fn write_umi_report(
    base_dir: &Path,
    records: &[BenchmarkRecord<FastqUmiMetrics>],
    failures: &[RawFailure],
    explain: bool,
) -> Result<()> {
    let path = base_dir.join("report.json");
    let mut report = BTreeMap::new();
    report.insert("records", serde_json::to_value(records)?);
    let classified: Vec<BenchmarkFailure> = failures.iter().map(classify_raw_failure).collect();
    report.insert("failures", serde_json::to_value(&classified)?);
    report.insert("gate", gate_payload(&classified));
    report.insert(
        "sanity_flags",
        serde_json::to_value(sanity_flags_umi(records))?,
    );
    let derived: Vec<_> = records.iter().map(derived_umi_metrics).collect();
    report.insert("derived_metrics", serde_json::to_value(&derived)?);
    let rankings = rank_umi_tools(records)?;
    report.insert("rankings", serde_json::to_value(&rankings)?);
    let json = serde_json::to_string_pretty(&report)?;
    fs::write(&path, json).context("write report.json")?;
    if explain {
        crate::decision::score::print_rank_explain("fastq.umi", &rankings);
    }
    Ok(())
}

/// Write the stats benchmark report.
///
/// # Errors
/// Returns an error if report serialization or file writes fail.
pub fn write_stats_report(
    base_dir: &Path,
    records: &[BenchmarkRecord<FastqStatsMetrics>],
    failures: &[RawFailure],
    explain: bool,
) -> Result<()> {
    let path = base_dir.join("report.json");
    let mut report = BTreeMap::new();
    report.insert("records", serde_json::to_value(records)?);
    let classified: Vec<BenchmarkFailure> = failures.iter().map(classify_raw_failure).collect();
    report.insert("failures", serde_json::to_value(&classified)?);
    report.insert("gate", gate_payload(&classified));
    report.insert(
        "sanity_flags",
        serde_json::to_value(sanity_flags_stats(records))?,
    );
    let semantic: Vec<_> = records
        .iter()
        .map(|record| semantic_stats(&record.metrics.metrics))
        .collect();
    report.insert("semantic_metrics", serde_json::to_value(&semantic)?);
    let json = serde_json::to_string_pretty(&report)?;
    fs::write(&path, json).context("write report.json")?;
    if explain {
        crate::decision::score::print_rank_explain("fastq.stats_neutral", &BTreeMap::new());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aggregate::{FastqDeltaMetrics, FastqTrimMetrics, FastqValidateMetrics};

    #[test]
    fn semantic_trim_generates_summary() {
        let metrics = FastqTrimMetrics {
            reads_in: 100,
            reads_out: 80,
            bases_in: 1000,
            bases_out: 800,
            pairs_in: None,
            pairs_out: None,
            mean_q_before: 30.0,
            mean_q_after: 31.5,
            delta_metrics: FastqDeltaMetrics {
                read_retention: 0.8,
                base_retention: 0.8,
                mean_q_delta: 1.5,
                gc_delta: 0.1,
            },
            adapter_preset: Some("default".to_string()),
            adapter_bank_id: Some("bank.v1".to_string()),
            adapter_bank_hash: Some("sha256:abc".to_string()),
            adapter_overrides: None,
        };
        let summary = semantic_trim(&metrics);
        assert!(matches!(
            summary.integrity.reads_in.value,
            MetricValue::U64(100)
        ));
        assert!(matches!(
            summary.integrity.reads_out.value,
            MetricValue::U64(80)
        ));
        assert!(summary.quality_shift.is_some());
    }

    #[test]
    fn semantic_validate_generates_summary() {
        let metrics = FastqValidateMetrics {
            reads_in: 50,
            reads_out: 50,
            bases_in: 500,
            bases_out: 500,
            pairs_in: None,
            pairs_out: None,
            reads_total: 50,
            reads_valid: 45,
            reads_invalid: 5,
            mean_q: 32.0,
        };
        let summary = semantic_validate(&metrics);
        assert!(matches!(
            summary.integrity.reads_in.value,
            MetricValue::U64(50)
        ));
        assert!(matches!(
            summary.integrity.reads_out.value,
            MetricValue::U64(45)
        ));
        assert!(summary.quality_shift.is_none());
    }
}
