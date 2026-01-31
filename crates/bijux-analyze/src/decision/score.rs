use std::collections::BTreeMap;

use anyhow::{anyhow, Result};
use serde::Serialize;

use crate::decision::{DecisionMetricTrace, DecisionTrace};
use crate::semantics::resolve_semantics;

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
    pub decision_trace: DecisionTrace,
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

/// # Errors
/// Returns an error if required metric semantics are missing.
pub fn build_rankings(inputs: &[RankInput]) -> Result<BTreeMap<String, Vec<RankingEntry>>> {
    assert_metric_semantics(&[
        "runtime_s",
        "memory_mb",
        "read_retention",
        "base_retention",
        "error_reduction_proxy",
        "merge_rate",
    ])?;
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
    Ok(rankings)
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

#[must_use]
pub fn decision_trace_for_input(mode: RankingMode, input: &RankInput) -> DecisionTrace {
    let breakdown = match mode {
        RankingMode::FastestAcceptable => vec![ScoreBreakdown {
            metric_id: "runtime_s".to_string(),
            value: input.runtime_s,
            weight: 1.0,
            contribution: input.runtime_s,
        }],
        RankingMode::MostConservative | RankingMode::BalancedPareto => vec![],
    };
    trace_for_input(input, mode, &breakdown, input.runtime_s)
}

fn rank_fastest(inputs: &[RankInput]) -> Vec<RankingEntry> {
    let mut entries: Vec<_> = inputs
        .iter()
        .map(|input| {
            let breakdown = vec![ScoreBreakdown {
                metric_id: "runtime_s".to_string(),
                value: input.runtime_s,
                weight: 1.0,
                contribution: input.runtime_s,
            }];
            RankingEntry {
                tool: input.tool.clone(),
                score: input.runtime_s,
                explain: format!(
                    "runtime_s={:.3} memory_mb={:.1} read_retention={}",
                    input.runtime_s,
                    input.memory_mb,
                    format_optional(input.read_retention)
                ),
                score_breakdown: breakdown.clone(),
                penalties: penalties_for_input(input),
                why_not_first: Vec::new(),
                decision_trace: trace_for_input(
                    input,
                    RankingMode::FastestAcceptable,
                    &breakdown,
                    input.runtime_s,
                ),
            }
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
            let breakdown = vec![
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
            ];
            RankingEntry {
                tool: input.tool.clone(),
                score,
                explain: format!(
                    "read_retention={} base_retention={} error_reduction_proxy={:.3}",
                    format_optional(input.read_retention),
                    format_optional(input.base_retention),
                    input.error_reduction_proxy.unwrap_or(0.0)
                ),
                score_breakdown: breakdown.clone(),
                penalties: penalties_for_input(input),
                why_not_first: Vec::new(),
                decision_trace: trace_for_input(
                    input,
                    RankingMode::MostConservative,
                    &breakdown,
                    score,
                ),
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
            let breakdown = vec![
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
            ];
            RankingEntry {
                tool: input.tool.clone(),
                score,
                explain: format!(
                    "runtime_norm={runtime_norm:.3} retention={retention_norm:.3} memory_norm={memory_norm:.3}"
                ),
                score_breakdown: breakdown.clone(),
                penalties: penalties_for_input(input),
                why_not_first: Vec::new(),
                decision_trace: trace_for_input(input, RankingMode::BalancedPareto, &breakdown, score),
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

fn trace_for_input(
    input: &RankInput,
    mode: RankingMode,
    breakdown: &[ScoreBreakdown],
    score: f64,
) -> DecisionTrace {
    let mut trace = DecisionTrace::empty();
    trace.per_metric = breakdown
        .iter()
        .map(|item| DecisionMetricTrace {
            metric_id: item.metric_id.clone(),
            value: Some(item.value),
            weight: item.weight,
            contribution: item.contribution,
        })
        .collect();
    if input.read_retention.is_none() {
        trace.missing.push("read_retention".to_string());
    }
    if input.base_retention.is_none() {
        trace.missing.push("base_retention".to_string());
    }
    if input.error_reduction_proxy.is_none() {
        trace.missing.push("error_reduction_proxy".to_string());
    }
    trace.penalties = penalties_for_input(input)
        .iter()
        .map(|penalty| format!("{}:{:?}", penalty.severity, penalty.reason))
        .collect();
    trace.tie_breaks.push(format!("{mode:?}:tool_id"));
    if !score.is_finite() {
        trace.penalties.push("score_non_finite".to_string());
    }
    trace
}

fn annotate_why_not_first(entries: &mut [RankingEntry], mode: RankingMode) {
    if entries.is_empty() {
        return;
    }
    let best_score = entries[0].score;
    for entry in entries.iter_mut().skip(1) {
        let explain = match mode {
            RankingMode::FastestAcceptable => "runtime is slower".to_string(),
            RankingMode::MostConservative => "retention/quality sum is lower".to_string(),
            RankingMode::BalancedPareto => "composite score is lower".to_string(),
        };
        entry.why_not_first.push(explain);
        if (entry.score - best_score).abs() <= f64::EPSILON {
            entry
                .why_not_first
                .push("tie broken by tool_id".to_string());
        }
    }
}

fn penalties_for_input(input: &RankInput) -> Vec<RankingPenalty> {
    let mut penalties = Vec::new();
    if input.runtime_s <= 0.0 {
        penalties.push(RankingPenalty {
            reason: "runtime_s missing or non-positive".to_string(),
            severity: "high".to_string(),
        });
    }
    if input.memory_mb <= 0.0 {
        penalties.push(RankingPenalty {
            reason: "memory_mb missing or non-positive".to_string(),
            severity: "medium".to_string(),
        });
    }
    if input.read_retention.is_none() {
        penalties.push(RankingPenalty {
            reason: "read_retention missing".to_string(),
            severity: "low".to_string(),
        });
    }
    penalties
}

fn format_optional(value: Option<f64>) -> String {
    value.map_or_else(|| "NA".to_string(), |val| format!("{val:.3}"))
}

fn min_max<I: Iterator<Item = f64>>(mut iter: I) -> (f64, f64) {
    let Some(first) = iter.next() else {
        return (0.0, 0.0);
    };
    let mut min_val = first;
    let mut max_val = first;
    for value in iter {
        if value < min_val {
            min_val = value;
        }
        if value > max_val {
            max_val = value;
        }
    }
    (min_val, max_val)
}

fn normalize_inverted(value: f64, min_val: f64, max_val: f64) -> f64 {
    if (max_val - min_val).abs() < f64::EPSILON {
        return 1.0;
    }
    let norm = (value - min_val) / (max_val - min_val);
    1.0 - norm
}

fn assert_metric_semantics(metric_ids: &[&str]) -> Result<()> {
    for metric_id in metric_ids {
        resolve_semantics(metric_id).map_err(|err| {
            anyhow!("missing metric semantics for {metric_id}; remediation: {err}")
        })?;
    }
    Ok(())
}
