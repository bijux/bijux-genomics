//! Owner: bijux-dna-bench-model
//! Rank VCF resource observations and combine declared score components.

use super::BaseScoreRow;
use crate::{StageScoringCorrectnessSignal, StageScoringDecisionMode, VcfToolScoreStatus};
use std::collections::BTreeMap;

pub(super) fn build_runtime_scores(rows: &[BaseScoreRow]) -> BTreeMap<(String, String), f64> {
    let mut grouped = BTreeMap::<String, Vec<((String, String), f64)>>::new();
    for row in rows.iter().filter(|row| row.failure_class == "none") {
        if let Some(runtime_seconds) = row.runtime_seconds {
            grouped
                .entry(row.stage_id.clone())
                .or_default()
                .push(((row.stage_id.clone(), row.tool_id.clone()), runtime_seconds));
        }
    }
    let mut scores = BTreeMap::new();
    for values in grouped.into_values() {
        if values.len() == 1 {
            if let Some((key, _)) = values.first() {
                scores.insert(key.clone(), 1.0);
            }
            continue;
        }
        if values.len() < 2 {
            continue;
        }
        let min = values.iter().map(|(_, value)| *value).fold(f64::INFINITY, f64::min);
        let max = values.iter().map(|(_, value)| *value).fold(f64::NEG_INFINITY, f64::max);
        for (key, value) in values {
            scores.insert(key, normalize_lower_is_better(value, min, max));
        }
    }
    scores
}

pub(super) fn build_memory_scores(rows: &[BaseScoreRow]) -> BTreeMap<(String, String), f64> {
    let mut grouped = BTreeMap::<String, Vec<((String, String), f64)>>::new();
    for row in rows.iter().filter(|row| row.failure_class == "none") {
        if let Some(memory_mb) = row.effective_memory_mb {
            grouped
                .entry(row.stage_id.clone())
                .or_default()
                .push(((row.stage_id.clone(), row.tool_id.clone()), memory_mb));
        }
    }
    let mut scores = BTreeMap::new();
    for values in grouped.into_values() {
        if values.len() == 1 {
            if let Some((key, _)) = values.first() {
                scores.insert(key.clone(), 1.0);
            }
            continue;
        }
        if values.len() < 2 {
            continue;
        }
        let min = values.iter().map(|(_, value)| *value).fold(f64::INFINITY, f64::min);
        let max = values.iter().map(|(_, value)| *value).fold(f64::NEG_INFINITY, f64::max);
        for (key, value) in values {
            scores.insert(key, normalize_lower_is_better(value, min, max));
        }
    }
    scores
}

pub(super) fn completion_score_for_row(row: &BaseScoreRow) -> Option<f64> {
    if row.failure_class != "none" {
        return None;
    }
    if row.row_statuses.iter().all(|status| status == "present") {
        return Some(1.0);
    }
    None
}

pub(super) fn score_row(
    row: &BaseScoreRow,
    correctness_score: Option<f64>,
    scientific_threshold_score: Option<f64>,
    runtime_score: Option<f64>,
    memory_score: Option<f64>,
    completion_score: Option<f64>,
    failure_class_score: Option<f64>,
) -> (VcfToolScoreStatus, f64, Option<f64>) {
    if row.failure_class != "none" {
        return match row.failure_class.as_str() {
            "insufficient_data" => (VcfToolScoreStatus::InsufficientEvidence, 0.0, None),
            _ => (VcfToolScoreStatus::Blocked, 0.0, None),
        };
    }
    let components = [
        (correctness_score, row.weights.correctness),
        (scientific_threshold_score, row.weights.scientific_threshold),
        (runtime_score, row.weights.runtime),
        (memory_score, row.weights.memory),
        (completion_score, row.weights.completion),
        (failure_class_score, row.weights.failure_class),
    ];
    let mut weighted_sum = 0.0;
    let mut covered_weight = 0.0;
    for (value, weight) in components {
        if weight <= 0.0 {
            continue;
        }
        if let Some(value) = value {
            weighted_sum += value * weight;
            covered_weight += weight;
        }
    }
    if covered_weight <= 0.0 || correctness_score.is_none() {
        return (VcfToolScoreStatus::InsufficientEvidence, 0.0, None);
    }
    (VcfToolScoreStatus::Scored, covered_weight, Some(weighted_sum / covered_weight))
}

pub(super) fn decision_mode_label(mode: StageScoringDecisionMode) -> &'static str {
    match mode {
        StageScoringDecisionMode::SingleToolAcceptance => "single_tool_acceptance",
        StageScoringDecisionMode::MultiToolRanking => "multi_tool_ranking",
    }
}

pub(super) fn correctness_signal_label(signal: StageScoringCorrectnessSignal) -> &'static str {
    match signal {
        StageScoringCorrectnessSignal::ScientificComparableMetrics => {
            "scientific_comparable_metrics"
        }
        StageScoringCorrectnessSignal::OutputContract => "output_contract",
    }
}

pub(super) fn normalize_lower_is_better(value: f64, min: f64, max: f64) -> f64 {
    if !min.is_finite() || !max.is_finite() || (max - min).abs() < f64::EPSILON {
        return 1.0;
    }
    ((max - value) / (max - min)).clamp(0.0, 1.0)
}
