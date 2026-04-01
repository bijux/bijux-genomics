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
            metrics.insert(metric.metric_id.clone(), metric.stats.median);
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
pub fn compare(summary_a: &BenchmarkSummary, summary_b: &BenchmarkSummary) -> Result<CompareReport> {
    compare_summaries(summary_a, summary_b)
}
