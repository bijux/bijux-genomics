use crate::aggregate::{
    BenchmarkRecord, FastqCorrectMetrics, FastqFilterMetrics, FastqMergeMetrics,
    FastqQcPostMetrics, FastqStatsMetrics, FastqTrimMetrics, FastqUmiMetrics, FastqValidateMetrics,
};

use super::math::{median, ratio_u64};

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
