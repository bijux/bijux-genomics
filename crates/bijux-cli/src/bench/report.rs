use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use bijux_bench::{
    derived_metric_spec, derived_metrics_for_stage, metric_kind_for_stage, metric_spec,
    stage_metric_spec, BenchmarkRecord, DerivedMetricId, FastqCorrectMetrics, FastqFilterMetrics,
    FastqMergeMetrics, FastqQc2Metrics, FastqStatsMetrics, FastqTrimMetrics, FastqUmiMetrics,
    FastqValidateMetrics,
};

use super::failure::BenchmarkFailure;
use super::helpers::ratio_u64;
use super::ranking::{build_rankings, print_rank_explain, RankInput};

pub(crate) fn write_trim_report(
    base_dir: &Path,
    records: &[BenchmarkRecord<FastqTrimMetrics>],
    failures: &[BenchmarkFailure],
    explain: bool,
) -> Result<()> {
    let path = base_dir.join("report.json");
    let mut report = BTreeMap::new();
    report.insert("records", serde_json::to_value(records)?);
    report.insert("failures", serde_json::to_value(failures)?);
    report.insert(
        "sanity_flags",
        serde_json::to_value(sanity_flags_trim(records))?,
    );
    let derived: Vec<_> = records.iter().map(derived_trim_metrics).collect();
    report.insert("derived_metrics", serde_json::to_value(&derived)?);
    let rankings = rank_trim_tools(records);
    report.insert("rankings", serde_json::to_value(&rankings)?);
    let json = serde_json::to_string_pretty(&report)?;
    fs::write(&path, json).context("write report.json")?;
    if explain {
        print_rank_explain("fastq.trim", &rankings);
    }
    Ok(())
}

pub(crate) fn write_validate_report(
    base_dir: &Path,
    records: &[BenchmarkRecord<FastqValidateMetrics>],
    failures: &[BenchmarkFailure],
    explain: bool,
) -> Result<()> {
    let path = base_dir.join("report.json");
    let mut report = BTreeMap::new();
    report.insert("records", serde_json::to_value(records)?);
    report.insert("failures", serde_json::to_value(failures)?);
    report.insert(
        "sanity_flags",
        serde_json::to_value(sanity_flags_validate(records))?,
    );
    let rankings = rank_validate_tools(records);
    report.insert("rankings", serde_json::to_value(&rankings)?);
    let json = serde_json::to_string_pretty(&report)?;
    fs::write(&path, json).context("write report.json")?;
    if explain {
        print_rank_explain("fastq.validate", &rankings);
    }
    Ok(())
}

pub(crate) fn write_filter_report(
    base_dir: &Path,
    records: &[BenchmarkRecord<FastqFilterMetrics>],
    failures: &[BenchmarkFailure],
    explain: bool,
) -> Result<()> {
    let path = base_dir.join("report.json");
    let mut report = BTreeMap::new();
    report.insert("records", serde_json::to_value(records)?);
    report.insert("failures", serde_json::to_value(failures)?);
    report.insert(
        "sanity_flags",
        serde_json::to_value(sanity_flags_filter(records))?,
    );
    let derived: Vec<_> = records.iter().map(derived_filter_metrics).collect();
    report.insert("derived_metrics", serde_json::to_value(&derived)?);
    let rankings = rank_filter_tools(records);
    report.insert("rankings", serde_json::to_value(&rankings)?);
    let json = serde_json::to_string_pretty(&report)?;
    fs::write(&path, json).context("write report.json")?;
    if explain {
        print_rank_explain("fastq.filter", &rankings);
    }
    Ok(())
}

pub(crate) fn write_merge_report(
    base_dir: &Path,
    records: &[BenchmarkRecord<FastqMergeMetrics>],
    failures: &[BenchmarkFailure],
    explain: bool,
) -> Result<()> {
    let path = base_dir.join("report.json");
    let mut report = BTreeMap::new();
    report.insert("records", serde_json::to_value(records)?);
    report.insert("failures", serde_json::to_value(failures)?);
    report.insert(
        "sanity_flags",
        serde_json::to_value(sanity_flags_merge(records))?,
    );
    let derived: Vec<_> = records.iter().map(derived_merge_metrics).collect();
    report.insert("derived_metrics", serde_json::to_value(&derived)?);
    let rankings = rank_merge_tools(records);
    report.insert("rankings", serde_json::to_value(&rankings)?);
    let json = serde_json::to_string_pretty(&report)?;
    fs::write(&path, json).context("write report.json")?;
    if explain {
        print_rank_explain("fastq.merge", &rankings);
    }
    Ok(())
}

pub(crate) fn write_correct_report(
    base_dir: &Path,
    records: &[BenchmarkRecord<FastqCorrectMetrics>],
    failures: &[BenchmarkFailure],
    explain: bool,
) -> Result<()> {
    let path = base_dir.join("report.json");
    let mut report = BTreeMap::new();
    report.insert("records", serde_json::to_value(records)?);
    report.insert("failures", serde_json::to_value(failures)?);
    report.insert(
        "sanity_flags",
        serde_json::to_value(sanity_flags_correct(records))?,
    );
    let derived: Vec<_> = records.iter().map(derived_correct_metrics).collect();
    report.insert("derived_metrics", serde_json::to_value(&derived)?);
    let rankings = rank_correct_tools(records);
    report.insert("rankings", serde_json::to_value(&rankings)?);
    let json = serde_json::to_string_pretty(&report)?;
    fs::write(&path, json).context("write report.json")?;
    if explain {
        print_rank_explain("fastq.correct", &rankings);
    }
    Ok(())
}

pub(crate) fn write_qc2_report(
    base_dir: &Path,
    records: &[BenchmarkRecord<FastqQc2Metrics>],
    failures: &[BenchmarkFailure],
    explain: bool,
) -> Result<()> {
    let path = base_dir.join("report.json");
    let mut report = BTreeMap::new();
    report.insert("records", serde_json::to_value(records)?);
    report.insert("failures", serde_json::to_value(failures)?);
    report.insert(
        "sanity_flags",
        serde_json::to_value(sanity_flags_qc2(records))?,
    );
    let json = serde_json::to_string_pretty(&report)?;
    fs::write(&path, json).context("write report.json")?;
    if explain {
        print_rank_explain("fastq.qc2", &BTreeMap::new());
    }
    Ok(())
}

pub(crate) fn write_umi_report(
    base_dir: &Path,
    records: &[BenchmarkRecord<FastqUmiMetrics>],
    failures: &[BenchmarkFailure],
    explain: bool,
) -> Result<()> {
    let path = base_dir.join("report.json");
    let mut report = BTreeMap::new();
    report.insert("records", serde_json::to_value(records)?);
    report.insert("failures", serde_json::to_value(failures)?);
    report.insert(
        "sanity_flags",
        serde_json::to_value(sanity_flags_umi(records))?,
    );
    let derived: Vec<_> = records.iter().map(derived_umi_metrics).collect();
    report.insert("derived_metrics", serde_json::to_value(&derived)?);
    let rankings = rank_umi_tools(records);
    report.insert("rankings", serde_json::to_value(&rankings)?);
    let json = serde_json::to_string_pretty(&report)?;
    fs::write(&path, json).context("write report.json")?;
    if explain {
        print_rank_explain("fastq.umi", &rankings);
    }
    Ok(())
}

pub(crate) fn write_stats_report(
    base_dir: &Path,
    records: &[BenchmarkRecord<FastqStatsMetrics>],
    failures: &[BenchmarkFailure],
    explain: bool,
) -> Result<()> {
    let path = base_dir.join("report.json");
    let mut report = BTreeMap::new();
    report.insert("records", serde_json::to_value(records)?);
    report.insert("failures", serde_json::to_value(failures)?);
    report.insert(
        "sanity_flags",
        serde_json::to_value(sanity_flags_stats(records))?,
    );
    let json = serde_json::to_string_pretty(&report)?;
    fs::write(&path, json).context("write report.json")?;
    if explain {
        print_rank_explain("fastq.stats", &BTreeMap::new());
    }
    Ok(())
}

pub fn print_bench_schema(stage: &str) -> Result<()> {
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
    let derived: Vec<_> = derived_metrics_for_stage(stage)
        .into_iter()
        .map(|metric| {
            serde_json::json!({
                "name": metric.name,
                "meaning": metric.meaning,
                "direction": format!("{:?}", metric.direction),
                "range": metric.range.map(|range| serde_json::json!({
                    "min": range.min,
                    "max": range.max
                })),
                "derived": true,
            })
        })
        .collect();
    let json = serde_json::json!({
        "stage": spec.stage,
        "metrics": metrics,
        "derived_metrics": derived,
        "invariants": spec.invariants,
    });
    println!("{}", serde_json::to_string_pretty(&json)?);
    Ok(())
}

fn median(mut values: Vec<f64>) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mid = values.len() / 2;
    if values.len() % 2 == 0 {
        f64::midpoint(values[mid - 1], values[mid])
    } else {
        values[mid]
    }
}

fn sanity_flags_trim(records: &[BenchmarkRecord<FastqTrimMetrics>]) -> Vec<serde_json::Value> {
    let runtime_median = median(records.iter().map(|r| r.execution.runtime_s).collect());
    records
        .iter()
        .map(|record| {
            let mut flags = Vec::new();
            if record.metrics.reads_in > 0 {
                let retention = ratio_u64(record.metrics.reads_out, record.metrics.reads_in);
                if retention < 0.1 {
                    flags.push("reads_retention_lt_0.1");
                }
            }
            if runtime_median > 0.0 && record.execution.runtime_s > 10.0 * runtime_median {
                flags.push("runtime_gt_10x_median");
            }
            serde_json::json!({ "tool": record.context.tool, "flags": flags })
        })
        .collect()
}

fn sanity_flags_filter(records: &[BenchmarkRecord<FastqFilterMetrics>]) -> Vec<serde_json::Value> {
    let runtime_median = median(records.iter().map(|r| r.execution.runtime_s).collect());
    records
        .iter()
        .map(|record| {
            let mut flags = Vec::new();
            if record.metrics.reads_in > 0 {
                let retention = ratio_u64(record.metrics.reads_out, record.metrics.reads_in);
                if retention < 0.1 {
                    flags.push("reads_retention_lt_0.1");
                }
            }
            if runtime_median > 0.0 && record.execution.runtime_s > 10.0 * runtime_median {
                flags.push("runtime_gt_10x_median");
            }
            serde_json::json!({ "tool": record.context.tool, "flags": flags })
        })
        .collect()
}

fn sanity_flags_correct(
    records: &[BenchmarkRecord<FastqCorrectMetrics>],
) -> Vec<serde_json::Value> {
    let runtime_median = median(records.iter().map(|r| r.execution.runtime_s).collect());
    records
        .iter()
        .map(|record| {
            let mut flags = Vec::new();
            if record.metrics.reads_in > 0 {
                let retention = ratio_u64(record.metrics.reads_out, record.metrics.reads_in);
                if retention < 0.1 {
                    flags.push("reads_retention_lt_0.1");
                }
            }
            if runtime_median > 0.0 && record.execution.runtime_s > 10.0 * runtime_median {
                flags.push("runtime_gt_10x_median");
            }
            serde_json::json!({ "tool": record.context.tool, "flags": flags })
        })
        .collect()
}

fn sanity_flags_umi(records: &[BenchmarkRecord<FastqUmiMetrics>]) -> Vec<serde_json::Value> {
    let runtime_median = median(records.iter().map(|r| r.execution.runtime_s).collect());
    records
        .iter()
        .map(|record| {
            let mut flags = Vec::new();
            if record.metrics.reads_in > 0 {
                let retention = ratio_u64(record.metrics.reads_out, record.metrics.reads_in);
                if retention < 0.1 {
                    flags.push("reads_retention_lt_0.1");
                }
            }
            if runtime_median > 0.0 && record.execution.runtime_s > 10.0 * runtime_median {
                flags.push("runtime_gt_10x_median");
            }
            serde_json::json!({ "tool": record.context.tool, "flags": flags })
        })
        .collect()
}

fn sanity_flags_merge(records: &[BenchmarkRecord<FastqMergeMetrics>]) -> Vec<serde_json::Value> {
    let runtime_median = median(records.iter().map(|r| r.execution.runtime_s).collect());
    records
        .iter()
        .map(|record| {
            let mut flags = Vec::new();
            if runtime_median > 0.0 && record.execution.runtime_s > 10.0 * runtime_median {
                flags.push("runtime_gt_10x_median");
            }
            serde_json::json!({ "tool": record.context.tool, "flags": flags })
        })
        .collect()
}

fn sanity_flags_stats(records: &[BenchmarkRecord<FastqStatsMetrics>]) -> Vec<serde_json::Value> {
    let runtime_median = median(records.iter().map(|r| r.execution.runtime_s).collect());
    let gc_median = median(records.iter().map(|r| r.metrics.gc_percent).collect());
    records
        .iter()
        .map(|record| {
            let mut flags = Vec::new();
            if (record.metrics.gc_percent - gc_median).abs() > 10.0 {
                flags.push("gc_shift_gt_10");
            }
            if runtime_median > 0.0 && record.execution.runtime_s > 10.0 * runtime_median {
                flags.push("runtime_gt_10x_median");
            }
            serde_json::json!({ "tool": record.context.tool, "flags": flags })
        })
        .collect()
}

fn sanity_flags_validate(
    records: &[BenchmarkRecord<FastqValidateMetrics>],
) -> Vec<serde_json::Value> {
    let runtime_median = median(records.iter().map(|r| r.execution.runtime_s).collect());
    records
        .iter()
        .map(|record| {
            let mut flags = Vec::new();
            if runtime_median > 0.0 && record.execution.runtime_s > 10.0 * runtime_median {
                flags.push("runtime_gt_10x_median");
            }
            serde_json::json!({ "tool": record.context.tool, "flags": flags })
        })
        .collect()
}

fn sanity_flags_qc2(records: &[BenchmarkRecord<FastqQc2Metrics>]) -> Vec<serde_json::Value> {
    let runtime_median = median(records.iter().map(|r| r.execution.runtime_s).collect());
    records
        .iter()
        .map(|record| {
            let mut flags = Vec::new();
            if runtime_median > 0.0 && record.execution.runtime_s > 10.0 * runtime_median {
                flags.push("runtime_gt_10x_median");
            }
            serde_json::json!({ "tool": record.context.tool, "flags": flags })
        })
        .collect()
}

fn derived_trim_metrics(record: &BenchmarkRecord<FastqTrimMetrics>) -> serde_json::Value {
    let reads_in = record.metrics.reads_in;
    let bases_in = record.metrics.bases_in;
    let read_retention = if reads_in > 0 {
        ratio_u64(record.metrics.reads_out, reads_in)
    } else {
        0.0
    };
    let base_retention = if bases_in > 0 {
        ratio_u64(record.metrics.bases_out, bases_in)
    } else {
        0.0
    };
    let error_reduction_proxy =
        (record.metrics.mean_q_after - record.metrics.mean_q_before).max(0.0);
    serde_json::json!({
        "tool": record.context.tool,
        derived_metric_spec(DerivedMetricId::ReadRetention).name: read_retention,
        derived_metric_spec(DerivedMetricId::BaseRetention).name: base_retention,
        derived_metric_spec(DerivedMetricId::ErrorReductionProxy).name: error_reduction_proxy,
    })
}

fn derived_filter_metrics(record: &BenchmarkRecord<FastqFilterMetrics>) -> serde_json::Value {
    let reads_in = record.metrics.reads_in;
    let read_retention = if reads_in > 0 {
        ratio_u64(record.metrics.reads_out, reads_in)
    } else {
        0.0
    };
    let base_retention = read_retention;
    let error_reduction_proxy =
        (record.metrics.mean_q_after - record.metrics.mean_q_before).max(0.0);
    serde_json::json!({
        "tool": record.context.tool,
        derived_metric_spec(DerivedMetricId::ReadRetention).name: read_retention,
        derived_metric_spec(DerivedMetricId::BaseRetention).name: base_retention,
        derived_metric_spec(DerivedMetricId::ErrorReductionProxy).name: error_reduction_proxy,
    })
}

fn derived_merge_metrics(record: &BenchmarkRecord<FastqMergeMetrics>) -> serde_json::Value {
    let min_reads = record.metrics.reads_r1.min(record.metrics.reads_r2);
    let merge_efficiency = if min_reads > 0 {
        ratio_u64(record.metrics.reads_merged, min_reads)
    } else {
        0.0
    };
    serde_json::json!({
        "tool": record.context.tool,
        derived_metric_spec(DerivedMetricId::MergeEfficiency).name: merge_efficiency,
    })
}

fn derived_correct_metrics(record: &BenchmarkRecord<FastqCorrectMetrics>) -> serde_json::Value {
    let reads_in = record.metrics.reads_in;
    let bases_in = record.metrics.bases_in;
    let read_retention = if reads_in > 0 {
        ratio_u64(record.metrics.reads_out, reads_in)
    } else {
        0.0
    };
    let base_retention = if bases_in > 0 {
        ratio_u64(record.metrics.bases_out, bases_in)
    } else {
        0.0
    };
    let error_reduction_proxy =
        (record.metrics.mean_q_after - record.metrics.mean_q_before).max(0.0);
    serde_json::json!({
        "tool": record.context.tool,
        derived_metric_spec(DerivedMetricId::ReadRetention).name: read_retention,
        derived_metric_spec(DerivedMetricId::BaseRetention).name: base_retention,
        derived_metric_spec(DerivedMetricId::ErrorReductionProxy).name: error_reduction_proxy,
    })
}

fn derived_umi_metrics(record: &BenchmarkRecord<FastqUmiMetrics>) -> serde_json::Value {
    let reads_in = record.metrics.reads_in;
    let read_retention = if reads_in > 0 {
        ratio_u64(record.metrics.reads_out, reads_in)
    } else {
        0.0
    };
    serde_json::json!({
        "tool": record.context.tool,
        derived_metric_spec(DerivedMetricId::ReadRetention).name: read_retention,
    })
}

fn rank_trim_tools(
    records: &[BenchmarkRecord<FastqTrimMetrics>],
) -> BTreeMap<String, Vec<super::ranking::RankingEntry>> {
    let inputs: Vec<_> = records
        .iter()
        .map(|record| RankInput {
            tool: record.context.tool.clone(),
            runtime_s: record.execution.runtime_s,
            memory_mb: record.execution.memory_mb,
            read_retention: Some(ratio_u64(record.metrics.reads_out, record.metrics.reads_in)),
            base_retention: Some(ratio_u64(record.metrics.bases_out, record.metrics.bases_in)),
            error_reduction_proxy: Some(
                (record.metrics.mean_q_after - record.metrics.mean_q_before).max(0.0),
            ),
        })
        .collect();
    build_rankings(&inputs)
}

fn rank_filter_tools(
    records: &[BenchmarkRecord<FastqFilterMetrics>],
) -> BTreeMap<String, Vec<super::ranking::RankingEntry>> {
    let inputs: Vec<_> = records
        .iter()
        .map(|record| RankInput {
            tool: record.context.tool.clone(),
            runtime_s: record.execution.runtime_s,
            memory_mb: record.execution.memory_mb,
            read_retention: Some(ratio_u64(record.metrics.reads_out, record.metrics.reads_in)),
            base_retention: Some(ratio_u64(record.metrics.reads_out, record.metrics.reads_in)),
            error_reduction_proxy: Some(
                (record.metrics.mean_q_after - record.metrics.mean_q_before).max(0.0),
            ),
        })
        .collect();
    build_rankings(&inputs)
}

fn rank_merge_tools(
    records: &[BenchmarkRecord<FastqMergeMetrics>],
) -> BTreeMap<String, Vec<super::ranking::RankingEntry>> {
    let inputs: Vec<_> = records
        .iter()
        .map(|record| RankInput {
            tool: record.context.tool.clone(),
            runtime_s: record.execution.runtime_s,
            memory_mb: record.execution.memory_mb,
            read_retention: None,
            base_retention: None,
            error_reduction_proxy: None,
        })
        .collect();
    build_rankings(&inputs)
}

fn rank_correct_tools(
    records: &[BenchmarkRecord<FastqCorrectMetrics>],
) -> BTreeMap<String, Vec<super::ranking::RankingEntry>> {
    let inputs: Vec<_> = records
        .iter()
        .map(|record| RankInput {
            tool: record.context.tool.clone(),
            runtime_s: record.execution.runtime_s,
            memory_mb: record.execution.memory_mb,
            read_retention: Some(ratio_u64(record.metrics.reads_out, record.metrics.reads_in)),
            base_retention: Some(ratio_u64(record.metrics.bases_out, record.metrics.bases_in)),
            error_reduction_proxy: Some(
                (record.metrics.mean_q_after - record.metrics.mean_q_before).max(0.0),
            ),
        })
        .collect();
    build_rankings(&inputs)
}

fn rank_umi_tools(
    records: &[BenchmarkRecord<FastqUmiMetrics>],
) -> BTreeMap<String, Vec<super::ranking::RankingEntry>> {
    let inputs: Vec<_> = records
        .iter()
        .map(|record| RankInput {
            tool: record.context.tool.clone(),
            runtime_s: record.execution.runtime_s,
            memory_mb: record.execution.memory_mb,
            read_retention: Some(ratio_u64(record.metrics.reads_out, record.metrics.reads_in)),
            base_retention: None,
            error_reduction_proxy: None,
        })
        .collect();
    build_rankings(&inputs)
}

fn rank_validate_tools(
    records: &[BenchmarkRecord<FastqValidateMetrics>],
) -> BTreeMap<String, Vec<super::ranking::RankingEntry>> {
    let inputs: Vec<_> = records
        .iter()
        .map(|record| RankInput {
            tool: record.context.tool.clone(),
            runtime_s: record.execution.runtime_s,
            memory_mb: record.execution.memory_mb,
            read_retention: None,
            base_retention: None,
            error_reduction_proxy: None,
        })
        .collect();
    build_rankings(&inputs)
}
