//! Owner: bijux-analyze
//! Scoring helpers for decision ranking.

use anyhow::{anyhow, Result};

use crate::decision::{DecisionMetricTrace, DecisionTrace};
use crate::semantics::resolve_semantics;

use crate::decision::score::{
    RankInput, RankingEntry, RankingMode, RankingPenalty, ScoreBreakdown,
};

pub(super) fn trace_for_input(
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
            effect: None,
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

pub(super) fn annotate_why_not_first(entries: &mut [RankingEntry], mode: RankingMode) {
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

pub(super) fn penalties_for_input(input: &RankInput) -> Vec<RankingPenalty> {
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

pub(super) fn format_optional(value: Option<f64>) -> String {
    value.map_or_else(|| "NA".to_string(), |val| format!("{val:.3}"))
}

pub(super) fn min_max<I: Iterator<Item = f64>>(mut iter: I) -> (f64, f64) {
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

pub(super) fn normalize_inverted(value: f64, min_val: f64, max_val: f64) -> f64 {
    if (max_val - min_val).abs() < f64::EPSILON {
        return 1.0;
    }
    let norm = (value - min_val) / (max_val - min_val);
    1.0 - norm
}

pub(super) fn assert_metric_semantics(metric_ids: &[&str]) -> Result<()> {
    for metric_id in metric_ids {
        resolve_semantics(metric_id).map_err(|err| {
            anyhow!("missing metric semantics for {metric_id}; remediation: {err}")
        })?;
    }
    Ok(())
}
