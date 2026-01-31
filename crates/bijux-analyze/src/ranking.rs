use std::collections::BTreeMap;

use serde::Serialize;

use bijux_core::metrics_registry::metric_semantics;

#[derive(Debug, Clone, Copy, Serialize)]
pub enum RankingMode {
    FastestAcceptable,
    MostConservative,
    BalancedPareto,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScoreBreakdown {
    pub metric_id: String,
    pub value: f64,
    pub weight: f64,
    pub contribution: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct RankingPenalty {
    pub reason: String,
    pub severity: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RankingEntry {
    pub tool: String,
    pub score: f64,
    pub explain: String,
    pub score_breakdown: Vec<ScoreBreakdown>,
    pub penalties: Vec<RankingPenalty>,
    pub why_not_first: Vec<String>,
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
    assert_metric_semantics(&[
        "runtime_s",
        "memory_mb",
        "read_retention",
        "base_retention",
        "error_reduction_proxy",
        "merge_rate",
    ]);
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
            score_breakdown: vec![ScoreBreakdown {
                metric_id: "runtime_s".to_string(),
                value: input.runtime_s,
                weight: 1.0,
                contribution: input.runtime_s,
            }],
            penalties: penalties_for_input(input),
            why_not_first: Vec::new(),
        })
        .collect();
    entries.sort_by(|a, b| {
        match a
            .score
            .partial_cmp(&b.score)
            .unwrap_or(std::cmp::Ordering::Equal)
        {
            std::cmp::Ordering::Equal => a.tool.cmp(&b.tool),
            ordering => ordering,
        }
    });
    annotate_why_not_first(&mut entries, RankingMode::FastestAcceptable);
    entries
}

fn rank_most_conservative(inputs: &[RankInput]) -> Vec<RankingEntry> {
    let mut entries: Vec<_> = inputs
        .iter()
        .map(|input| {
            let retention = input.read_retention.unwrap_or(0.0);
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
                score_breakdown: vec![
                    ScoreBreakdown {
                        metric_id: "read_retention".to_string(),
                        value: retention,
                        weight: 1.0,
                        contribution: retention,
                    },
                    ScoreBreakdown {
                        metric_id: "base_retention".to_string(),
                        value: base_retention,
                        weight: 1.0,
                        contribution: base_retention,
                    },
                    ScoreBreakdown {
                        metric_id: "error_reduction_proxy".to_string(),
                        value: input.error_reduction_proxy.unwrap_or(0.0),
                        weight: 1.0 / 45.0,
                        contribution: error_proxy,
                    },
                ],
                penalties: penalties_for_input(input),
                why_not_first: Vec::new(),
            }
        })
        .collect();
    entries.sort_by(|a, b| {
        match b
            .score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
        {
            std::cmp::Ordering::Equal => a.tool.cmp(&b.tool),
            ordering => ordering,
        }
    });
    annotate_why_not_first(&mut entries, RankingMode::MostConservative);
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
            let retention_norm = input.read_retention.unwrap_or(0.0);
            let score = 0.5 * runtime_norm + 0.3 * retention_norm + 0.2 * memory_norm;
            RankingEntry {
                tool: input.tool.clone(),
                score,
                explain: format!(
                    "runtime_norm={runtime_norm:.3} retention={retention_norm:.3} memory_norm={memory_norm:.3}"
                ),
                score_breakdown: vec![
                    ScoreBreakdown {
                        metric_id: "runtime_s".to_string(),
                        value: runtime_norm,
                        weight: 0.5,
                        contribution: 0.5 * runtime_norm,
                    },
                    ScoreBreakdown {
                        metric_id: "read_retention".to_string(),
                        value: retention_norm,
                        weight: 0.3,
                        contribution: 0.3 * retention_norm,
                    },
                    ScoreBreakdown {
                        metric_id: "memory_mb".to_string(),
                        value: memory_norm,
                        weight: 0.2,
                        contribution: 0.2 * memory_norm,
                    },
                ],
                penalties: penalties_for_input(input),
                why_not_first: Vec::new(),
            }
        })
        .collect();
    entries.sort_by(|a, b| {
        match b
            .score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
        {
            std::cmp::Ordering::Equal => a.tool.cmp(&b.tool),
            ordering => ordering,
        }
    });
    annotate_why_not_first(&mut entries, RankingMode::BalancedPareto);
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

fn penalties_for_input(input: &RankInput) -> Vec<RankingPenalty> {
    let mut penalties = Vec::new();
    if input.read_retention.is_none() {
        penalties.push(RankingPenalty {
            reason: "missing_metric:read_retention".to_string(),
            severity: "warning".to_string(),
        });
    }
    if input.base_retention.is_none() {
        penalties.push(RankingPenalty {
            reason: "missing_metric:base_retention".to_string(),
            severity: "warning".to_string(),
        });
    }
    if input.error_reduction_proxy.is_none() {
        penalties.push(RankingPenalty {
            reason: "missing_metric:error_reduction_proxy".to_string(),
            severity: "warning".to_string(),
        });
    }
    penalties
}

fn annotate_why_not_first(entries: &mut [RankingEntry], mode: RankingMode) {
    if entries.is_empty() {
        return;
    }
    let best = entries[0].clone();
    for entry in entries.iter_mut().skip(1) {
        let mut reasons = Vec::new();
        match mode {
            RankingMode::FastestAcceptable => {
                if entry.score > best.score {
                    reasons.push(format!(
                        "slower_runtime_s({:.3} > {:.3})",
                        entry.score, best.score
                    ));
                }
            }
            RankingMode::MostConservative => {
                if entry.score < best.score {
                    reasons.push(format!(
                        "lower_retention_score({:.3} < {:.3})",
                        entry.score, best.score
                    ));
                }
            }
            RankingMode::BalancedPareto => {
                if entry.score < best.score {
                    reasons.push(format!(
                        "lower_balanced_score({:.3} < {:.3})",
                        entry.score, best.score
                    ));
                }
            }
        }
        if reasons.is_empty() {
            reasons.push("tie_broken_by_tool_name".to_string());
        }
        entry.why_not_first = reasons;
    }
}

fn assert_metric_semantics(metric_ids: &[&str]) {
    for metric_id in metric_ids {
        assert!(
            metric_semantics(metric_id).is_some(),
            "missing metric semantics for {metric_id}"
        );
    }
}
