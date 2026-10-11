//! Component normalization and weighted decisions.

use super::BaseScoreRow;
use crate::{FastqToolScoreStatus, StageScoringCorrectnessSignal, StageScoringDecisionMode};
use std::collections::BTreeMap;

pub(super) fn build_runtime_scores(rows: &[BaseScoreRow]) -> BTreeMap<(String, String), f64> {
    let mut per_stage = BTreeMap::<String, Vec<&BaseScoreRow>>::new();
    for row in rows.iter().filter(|row| row.failure_class == "none") {
        per_stage.entry(row.stage_id.clone()).or_default().push(row);
    }
    let mut scores = BTreeMap::new();
    for stage_rows in per_stage.into_values() {
        let Some(reference) = stage_rows.first() else {
            continue;
        };
        let values = stage_rows
            .iter()
            .filter_map(|row| {
                row.runtime_seconds
                    .map(|value| ((row.stage_id.clone(), row.tool_id.clone()), value))
            })
            .collect::<Vec<_>>();
        if reference.decision_mode == StageScoringDecisionMode::SingleToolAcceptance {
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
    let mut per_stage = BTreeMap::<String, Vec<&BaseScoreRow>>::new();
    for row in rows.iter().filter(|row| row.failure_class == "none") {
        per_stage.entry(row.stage_id.clone()).or_default().push(row);
    }
    let mut scores = BTreeMap::new();
    for stage_rows in per_stage.into_values() {
        let Some(reference) = stage_rows.first() else {
            continue;
        };
        let values = stage_rows
            .iter()
            .filter_map(|row| {
                row.effective_memory_mb
                    .map(|value| ((row.stage_id.clone(), row.tool_id.clone()), value))
            })
            .collect::<Vec<_>>();
        if reference.decision_mode == StageScoringDecisionMode::SingleToolAcceptance {
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
) -> (FastqToolScoreStatus, f64, Option<f64>) {
    if row.failure_class != "none" {
        return match row.failure_class.as_str() {
            "insufficient_data" => (FastqToolScoreStatus::InsufficientEvidence, 0.0, None),
            _ => (FastqToolScoreStatus::Blocked, 0.0, None),
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
        return (FastqToolScoreStatus::InsufficientEvidence, 0.0, None);
    }
    (FastqToolScoreStatus::Scored, covered_weight, Some(weighted_sum / covered_weight))
}

fn normalize_lower_is_better(value: f64, min: f64, max: f64) -> f64 {
    if (max - min).abs() <= 1e-12 {
        1.0
    } else {
        ((max - value) / (max - min)).clamp(0.0, 1.0)
    }
}

pub(super) fn decision_mode_label(value: StageScoringDecisionMode) -> &'static str {
    match value {
        StageScoringDecisionMode::MultiToolRanking => "multi_tool_ranking",
        StageScoringDecisionMode::SingleToolAcceptance => "single_tool_acceptance",
    }
}

pub(super) fn correctness_signal_label(value: StageScoringCorrectnessSignal) -> &'static str {
    match value {
        StageScoringCorrectnessSignal::ScientificComparableMetrics => {
            "scientific_comparable_metrics"
        }
        StageScoringCorrectnessSignal::OutputContract => "output_contract",
    }
}
