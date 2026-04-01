use std::collections::BTreeSet;

use anyhow::Result;

use bijux_dna_bench_model::stats::{mad_outliers, robust_stats};
use bijux_dna_bench_model::{
    BenchmarkObservation, BenchmarkSuiteSpec, MetricSummary, SummaryRow,
};

use super::super::options::BenchRunOptions;
use super::super::summary_statistics::{bootstrap_if_enabled, indices_to_replicates};

#[allow(clippy::too_many_arguments)]
pub(super) fn build_summary_row(
    suite: &BenchmarkSuiteSpec,
    options: &BenchRunOptions,
    dataset_id: String,
    stage_id: String,
    stage_instance_id: Option<String>,
    lineage_id: Option<String>,
    tool_id: String,
    params_hash: String,
    group: Vec<&BenchmarkObservation>,
) -> Result<Option<(SummaryRow, Vec<String>)>> {
    let tool = tool_id.as_str();
    let runtimes: Vec<f64> = group.iter().map(|o| o.runtime_s).collect();
    let memories: Vec<f64> = group.iter().map(|o| o.memory_mb).collect();

    let runtime_stats = robust_stats(&runtimes);
    let memory_stats = robust_stats(&memories);

    let runtime_outliers = mad_outliers(&runtimes, 3.5);
    let memory_outliers = mad_outliers(&memories, 3.5);

    let mut warnings = Vec::new();
    let runtime_ci = bootstrap_if_enabled(
        suite,
        &stage_id,
        &tool_id,
        "runtime_s",
        &runtimes,
        options.ci_bootstrap,
    );
    if options.ci_bootstrap.is_some() && runtimes.len() < 5 {
        warnings.push(format!("ci_min_n:runtime_s:{stage_id}:{tool}"));
    }
    let memory_ci = bootstrap_if_enabled(
        suite,
        &stage_id,
        &tool_id,
        "memory_mb",
        &memories,
        options.ci_bootstrap,
    );
    if options.ci_bootstrap.is_some() && memories.len() < 5 {
        warnings.push(format!("ci_min_n:memory_mb:{stage_id}:{tool}"));
    }

    let runtime_summary = MetricSummary {
        metric_id: "runtime_s".to_string(),
        n: runtimes.len(),
        stats: runtime_stats,
        ci_low: runtime_ci.map(|ci| ci.0),
        ci_high: runtime_ci.map(|ci| ci.1),
        outlier_count: runtime_outliers.outlier_count,
        outlier_replicates: indices_to_replicates(&runtime_outliers.outlier_indices, &group),
        practical_threshold: Some(0.05),
        power_warning: runtimes.len() < 5,
    };
    let memory_summary = MetricSummary {
        metric_id: "memory_mb".to_string(),
        n: memories.len(),
        stats: memory_stats,
        ci_low: memory_ci.map(|ci| ci.0),
        ci_high: memory_ci.map(|ci| ci.1),
        outlier_count: memory_outliers.outlier_count,
        outlier_replicates: indices_to_replicates(&memory_outliers.outlier_indices, &group),
        practical_threshold: Some(0.05),
        power_warning: memories.len() < 5,
    };

    let mut metric_summaries = Vec::new();
    let mut metric_ids = BTreeSet::new();
    for obs in &group {
        metric_ids.extend(obs.metrics.values.keys().cloned());
    }
    for metric_id in metric_ids {
        let values: Vec<f64> = group
            .iter()
            .filter_map(|obs| obs.metrics.values.get(&metric_id).copied())
            .collect();
        let stats = robust_stats(&values);
        let outliers = mad_outliers(&values, 3.5);
        let ci = bootstrap_if_enabled(
            suite,
            &stage_id,
            &tool_id,
            &metric_id,
            &values,
            options.ci_bootstrap,
        );
        if options.ci_bootstrap.is_some() && values.len() < 5 {
            warnings.push(format!("ci_min_n:{metric_id}:{stage_id}:{tool}"));
        }
        metric_summaries.push(MetricSummary {
            metric_id,
            n: values.len(),
            stats,
            ci_low: ci.map(|c| c.0),
            ci_high: ci.map(|c| c.1),
            outlier_count: outliers.outlier_count,
            outlier_replicates: indices_to_replicates(&outliers.outlier_indices, &group),
            practical_threshold: Some(0.05),
            power_warning: values.len() < 5,
        });
    }

    let failures = group.iter().filter(|obs| obs.exit_code != 0).count();
    let failure_rate = if group.is_empty() {
        0.0
    } else {
        failures as f64 / group.len() as f64
    };
    let n_effective = group.len().saturating_sub(failures);
    let low_power = n_effective < 3;
    if low_power {
        warnings.push(format!("low_power:{stage_id}:{tool}:{dataset_id}"));
    }
    let completeness = if group.is_empty() {
        0.0
    } else {
        n_effective as f64 / group.len() as f64
    };

    let Some(first) = group.first().copied() else {
        return Ok(None);
    };
    let dataset_class = first.dataset_class.clone();
    let read_layout = first.read_layout.clone();

    Ok(Some((
        SummaryRow {
            dataset_id,
            dataset_class,
            read_layout,
            stage_id,
            stage_instance_id,
            lineage_id,
            tool_id,
            params_hash,
            runtime: runtime_summary,
            memory: memory_summary,
            metrics: metric_summaries,
            failure_rate,
            completeness,
            n_effective,
            low_power,
        },
        warnings,
    )))
}
