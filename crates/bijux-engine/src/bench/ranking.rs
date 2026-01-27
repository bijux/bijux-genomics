use std::collections::BTreeMap;

use serde::Serialize;

use super::helpers::{format_optional, min_max, normalize_inverted};

#[derive(Debug, Clone, Copy, Serialize)]
pub(crate) enum RankingMode {
    FastestAcceptable,
    MostConservative,
    BalancedPareto,
}

#[derive(Debug, Serialize)]
pub(crate) struct RankingEntry {
    pub(crate) tool: String,
    pub(crate) score: f64,
    pub(crate) explain: String,
}

#[derive(Debug)]
pub(crate) struct RankInput {
    pub(crate) tool: String,
    pub(crate) runtime_s: f64,
    pub(crate) memory_mb: f64,
    pub(crate) read_retention: Option<f64>,
    pub(crate) base_retention: Option<f64>,
    pub(crate) error_reduction_proxy: Option<f64>,
}

pub(crate) fn build_rankings(inputs: &[RankInput]) -> BTreeMap<String, Vec<RankingEntry>> {
    let mut rankings = BTreeMap::new();
    rankings.insert(
        format!("{:?}", RankingMode::FastestAcceptable),
        rank_fastest(inputs),
    );
    rankings.insert(
        format!("{:?}", RankingMode::MostConservative),
        rank_most_conservative(inputs),
    );
    rankings.insert(
        format!("{:?}", RankingMode::BalancedPareto),
        rank_balanced(inputs),
    );
    rankings
}

pub(crate) fn print_rank_explain(stage: &str, rankings: &BTreeMap<String, Vec<RankingEntry>>) {
    println!("[bijux][rank] stage: {stage}");
    for (mode, entries) in rankings {
        println!("[bijux][rank] mode: {mode}");
        for entry in entries {
            println!("  {} -> {}", entry.tool, entry.explain);
        }
    }
}

fn rank_fastest(inputs: &[RankInput]) -> Vec<RankingEntry> {
    let mut entries: Vec<_> = inputs
        .iter()
        .map(|input| RankingEntry {
            tool: input.tool.clone(),
            score: input.runtime_s,
            explain: format!(
                "runtime_s={:.3} memory_mb={:.1} read_retention={}",
                input.runtime_s,
                input.memory_mb,
                format_optional(input.read_retention)
            ),
        })
        .collect();
    entries.sort_by(|a, b| {
        a.score
            .partial_cmp(&b.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    entries
}

fn rank_most_conservative(inputs: &[RankInput]) -> Vec<RankingEntry> {
    let mut entries: Vec<_> = inputs
        .iter()
        .map(|input| {
            let retention = input.read_retention.unwrap_or(1.0);
            let base_retention = input.base_retention.unwrap_or(retention);
            let error_proxy = input.error_reduction_proxy.unwrap_or(0.0) / 45.0;
            let score = retention + base_retention + error_proxy;
            RankingEntry {
                tool: input.tool.clone(),
                score,
                explain: format!(
                    "read_retention={} base_retention={} error_reduction_proxy={:.3}",
                    format_optional(input.read_retention),
                    format_optional(input.base_retention),
                    input.error_reduction_proxy.unwrap_or(0.0)
                ),
            }
        })
        .collect();
    entries.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    entries
}

fn rank_balanced(inputs: &[RankInput]) -> Vec<RankingEntry> {
    let (min_runtime, max_runtime) = min_max(inputs.iter().map(|input| input.runtime_s));
    let (min_memory, max_memory) = min_max(inputs.iter().map(|input| input.memory_mb));
    let mut entries: Vec<_> = inputs
        .iter()
        .map(|input| {
            let runtime_norm = normalize_inverted(input.runtime_s, min_runtime, max_runtime);
            let memory_norm = normalize_inverted(input.memory_mb, min_memory, max_memory);
            let retention_norm = input.read_retention.unwrap_or(1.0);
            let score = 0.5 * runtime_norm + 0.3 * retention_norm + 0.2 * memory_norm;
            RankingEntry {
                tool: input.tool.clone(),
                score,
                explain: format!(
                    "runtime_norm={runtime_norm:.3} retention={retention_norm:.3} memory_norm={memory_norm:.3}"
                ),
            }
        })
        .collect();
    entries.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    entries
}
