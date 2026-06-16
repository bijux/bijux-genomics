//! Owner: bijux-dna-bench
//! Read-only evaluation entrypoints built on finished benchmark summaries.

use std::collections::BTreeMap;

use anyhow::Result;
use bijux_dna_bench_model::compare::{compare_summaries, CompareReport};
use bijux_dna_bench_model::policy::{GateDecision, GatePolicy};
use bijux_dna_bench_model::BenchmarkSummary;

/// Gate summary rows using a policy.
#[must_use]
pub fn gate(policy: &GatePolicy, summary: &BenchmarkSummary) -> Vec<GateDecision> {
    let mut decisions = Vec::new();
    for row in &summary.rows {
        let mut metrics = BTreeMap::new();
        metrics.insert("runtime_s".to_string(), row.runtime.stats.median);
        metrics.insert("memory_mb".to_string(), row.memory.stats.median);
        for metric in &row.metrics {
            metrics.entry(metric.metric_id.clone()).or_insert(metric.stats.median);
        }
        decisions.push(policy.decide(
            &row.dataset_id,
            &row.stage_id,
            &row.tool_id,
            &row.params_hash,
            &metrics,
        ));
    }
    decisions
}

/// Compare two summaries.
pub fn compare(
    summary_a: &BenchmarkSummary,
    summary_b: &BenchmarkSummary,
) -> Result<CompareReport> {
    compare_summaries(summary_a, summary_b)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use bijux_dna_core::metrics::MetricDirection;
    use bijux_dna_bench_model::{
        robust_stats, BenchmarkSummary, GatePolicy, MetricSummary, SummaryRow,
    };

    use super::gate;

    fn metric_summary(metric_id: &str, median: f64) -> MetricSummary {
        MetricSummary {
            metric_id: metric_id.to_string(),
            n: 3,
            stats: robust_stats(&[median, median, median]),
            ci_low: None,
            ci_high: None,
            outlier_count: 0,
            outlier_replicates: Vec::new(),
            practical_threshold: None,
            power_warning: false,
        }
    }

    #[test]
    fn gate_keeps_core_runtime_metric_authoritative() {
        let row = SummaryRow {
            dataset_id: "dataset-1".to_string(),
            dataset_class: "trueseq".to_string(),
            read_layout: "paired".to_string(),
            stage_id: "fastq.trim_reads".to_string(),
            stage_instance_id: None,
            lineage_id: None,
            tool_id: "fastp".to_string(),
            params_hash: "params-a".to_string(),
            runtime: metric_summary("runtime_s", 1.0),
            memory: metric_summary("memory_mb", 100.0),
            metrics: vec![metric_summary("runtime_s", 99.0)],
            failure_rate: 0.0,
            completeness: 1.0,
            n_effective: 3,
            low_power: false,
        };
        let summary =
            BenchmarkSummary::v1("suite-1".to_string(), vec![row], Vec::new(), Vec::new());
        let mut thresholds = BTreeMap::new();
        thresholds.insert("runtime_s".to_string(), 2.0);
        let mut semantics_overrides = BTreeMap::new();
        semantics_overrides.insert("runtime_s".to_string(), MetricDirection::LowerBetter);
        let policy = GatePolicy {
            objective: "balanced".to_string(),
            required_metrics: Vec::new(),
            thresholds,
            allowed_regressions: BTreeMap::new(),
            must_not_regress: Vec::new(),
            semantics_overrides,
            stage_overrides: BTreeMap::new(),
        };

        let decisions = gate(&policy, &summary);

        assert_eq!(decisions.len(), 1);
        assert!(decisions[0].passes);
    }
}
