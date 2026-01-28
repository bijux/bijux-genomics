use std::collections::BTreeMap;

use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize)]
pub enum RankingMode {
    FastestAcceptable,
    MostConservative,
    BalancedPareto,
}

#[derive(Debug, Serialize)]
pub struct RankingEntry {
    pub tool: String,
    pub score: f64,
    pub explain: String,
}

#[derive(Debug)]
pub struct RankInput {
    pub tool: String,
    pub runtime_s: f64,
    pub memory_mb: f64,
    pub read_retention: Option<f64>,
    pub base_retention: Option<f64>,
    pub error_reduction_proxy: Option<f64>,
}

#[must_use]
pub fn build_rankings(inputs: &[RankInput]) -> BTreeMap<String, Vec<RankingEntry>> {
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

pub fn print_rank_explain(stage: &str, rankings: &BTreeMap<String, Vec<RankingEntry>>) {
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

fn normalize_inverted(value: f64, min: f64, max: f64) -> f64 {
    if (max - min).abs() < f64::EPSILON {
        1.0
    } else {
        1.0 - ((value - min) / (max - min)).clamp(0.0, 1.0)
    }
}

fn min_max(values: impl Iterator<Item = f64>) -> (f64, f64) {
    let mut min = f64::INFINITY;
    let mut max = f64::NEG_INFINITY;
    for value in values {
        if value < min {
            min = value;
        }
        if value > max {
            max = value;
        }
    }
    if min == f64::INFINITY {
        (0.0, 0.0)
    } else {
        (min, max)
    }
}

fn format_optional(value: Option<f64>) -> String {
    value.map_or_else(|| "n/a".to_string(), |v| format!("{v:.3}"))
}
