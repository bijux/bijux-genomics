use anyhow::{anyhow, Result};

use crate::decision::{DecisionMetricTrace, DecisionTrace};
use crate::decision::score::{RankInput, RankingMode, ScoreBreakdown};
use crate::semantics::resolve_semantics;

use super::normalization::penalties_for_input;

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

pub(super) fn assert_metric_semantics(metric_ids: &[&str]) -> Result<()> {
    for metric_id in metric_ids {
        resolve_semantics(metric_id).map_err(|err| {
            anyhow!("missing metric semantics for {metric_id}; remediation: {err}")
        })?;
    }
    Ok(())
}
