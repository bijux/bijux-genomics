use crate::aggregate::{
    derived_metric_spec, derived_metrics_for_stage, BenchmarkRecord, DerivedMetricId,
    FastqCorrectMetrics, FastqFilterMetrics, FastqMergeMetrics, FastqTrimMetrics, FastqUmiMetrics,
};

use super::math::ratio_u64;

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
