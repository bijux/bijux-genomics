#![allow(missing_docs)]

use std::cmp::Ordering;

use crate::ids::StageId;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Objective {
    Speed,
    Memory,
    Retention,
    #[default]
    Balanced,
}

impl Objective {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Objective::Speed => "speed",
            Objective::Memory => "memory",
            Objective::Retention => "retention",
            Objective::Balanced => "balanced",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectiveWeights {
    pub runtime: f64,
    pub memory: f64,
    pub retention: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectiveSpec {
    pub name: String,
    pub weights: ObjectiveWeights,
}

#[derive(Debug, Clone)]
pub enum BenchResultStatus {
    Success,
    Failure,
    Missing,
}

#[derive(Debug, Clone)]
pub struct BenchResultRecord {
    pub dataset_id: String,
    pub tool: String,
    pub runtime_s: Option<f64>,
    pub memory_mb: Option<f64>,
    pub exit_code: Option<i64>,
    pub metrics: Option<JsonValue>,
    pub status: BenchResultStatus,
}

#[derive(Debug, Clone, Serialize)]
pub struct ToolScore {
    pub tool: String,
    pub score: f64,
    pub runtime_median: Option<f64>,
    pub memory_median: Option<f64>,
    pub retention_median: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Disqualification {
    pub tool: String,
    pub dataset_id: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct StageSelection {
    pub stage: StageId,
    pub selected: Option<String>,
    pub scores: Vec<ToolScore>,
    pub disqualified: Vec<Disqualification>,
}

pub use crate::contract::tooling::{
    BenchResultRecord as BenchRecord, BenchResultStatus as BenchStatus, Objective as ObjectiveKind,
    ObjectiveSpec as ObjectiveSchema, ObjectiveWeights as ObjectiveWeighting,
    StageSelection as Selection,
};

#[must_use]
pub fn objective_spec(objective: Objective) -> ObjectiveSpec {
    match objective {
        Objective::Speed => ObjectiveSpec {
            name: objective.as_str().to_string(),
            weights: ObjectiveWeights { runtime: 1.0, memory: 0.0, retention: 0.0 },
        },
        Objective::Memory => ObjectiveSpec {
            name: objective.as_str().to_string(),
            weights: ObjectiveWeights { runtime: 0.0, memory: 1.0, retention: 0.0 },
        },
        Objective::Retention => ObjectiveSpec {
            name: objective.as_str().to_string(),
            weights: ObjectiveWeights { runtime: 0.0, memory: 0.0, retention: -1.0 },
        },
        Objective::Balanced => ObjectiveSpec {
            name: objective.as_str().to_string(),
            weights: ObjectiveWeights { runtime: 1.0, memory: 1.0, retention: -100.0 },
        },
    }
}

#[must_use]
pub fn select_stage(
    stage: &crate::ids::StageId,
    tool_records: &[(String, Vec<BenchResultRecord>)],
    objective: &ObjectiveSpec,
    allow_partial: bool,
) -> StageSelection {
    let mut scores = Vec::new();
    let mut disqualified = Vec::new();

    for (tool, records) in tool_records {
        let mut failed = false;
        let mut runtime = Vec::new();
        let mut memory = Vec::new();
        let mut retention = Vec::new();

        for record in records {
            match record.status {
                BenchResultStatus::Success => {
                    if let Some(value) = record.runtime_s {
                        runtime.push(value);
                    }
                    if let Some(value) = record.memory_mb {
                        memory.push(value);
                    }
                    if let Some(value) = read_retention(record) {
                        retention.push(value);
                    }
                }
                BenchResultStatus::Failure => {
                    failed = true;
                    disqualified.push(Disqualification {
                        tool: tool.clone(),
                        dataset_id: record.dataset_id.clone(),
                        reason: "tool failed on dataset".to_string(),
                    });
                }
                BenchResultStatus::Missing => {
                    failed = true;
                    disqualified.push(Disqualification {
                        tool: tool.clone(),
                        dataset_id: record.dataset_id.clone(),
                        reason: "missing bench record".to_string(),
                    });
                }
            }
        }

        if failed && !allow_partial {
            continue;
        }

        let runtime_median = median(&runtime);
        let memory_median = median(&memory);
        let retention_median = median(&retention);
        let score = score_for_objective(objective, runtime_median, memory_median, retention_median);
        scores.push(ToolScore {
            tool: tool.clone(),
            score,
            runtime_median,
            memory_median,
            retention_median,
        });
    }

    scores.sort_by(|a, b| compare_score(a.score, b.score));
    let selected = scores.first().map(|entry| entry.tool.clone());

    StageSelection { stage: stage.clone(), selected, scores, disqualified }
}

fn score_for_objective(
    objective: &ObjectiveSpec,
    runtime: Option<f64>,
    memory: Option<f64>,
    retention: Option<f64>,
) -> f64 {
    let runtime = runtime.unwrap_or(f64::INFINITY);
    let memory = memory.unwrap_or(f64::INFINITY);
    let retention = retention.unwrap_or(0.0);
    weighted_component(runtime, objective.weights.runtime)
        + weighted_component(memory, objective.weights.memory)
        + weighted_component(retention, objective.weights.retention)
}

fn weighted_component(value: f64, weight: f64) -> f64 {
    if weight == 0.0 {
        0.0
    } else {
        value * weight
    }
}

fn compare_score(left: f64, right: f64) -> Ordering {
    left.partial_cmp(&right).unwrap_or(Ordering::Equal)
}

fn median(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
    let mid = sorted.len() / 2;
    if sorted.len() % 2 == 0 {
        Some((sorted[mid - 1] + sorted[mid]) * 0.5)
    } else {
        Some(sorted[mid])
    }
}

fn read_retention(record: &BenchResultRecord) -> Option<f64> {
    let metrics = record.metrics.as_ref()?;
    let retention = metrics.get("retention")?;
    let value = retention
        .get("value")
        .and_then(serde_json::Value::as_f64)
        .or_else(|| retention.as_f64())?;
    if value.is_finite() && (0.0..=1.0).contains(&value) {
        Some(value)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::{
        objective_spec, select_stage, BenchResultRecord, BenchResultStatus, Objective, ToolScore,
    };
    use crate::id_catalog::FASTQ_TRIM;
    use crate::ids::StageId;

    fn successful_record(runtime_s: Option<f64>, memory_mb: Option<f64>) -> BenchResultRecord {
        BenchResultRecord {
            dataset_id: "dataset-1".to_string(),
            tool: "tool".to_string(),
            runtime_s,
            memory_mb,
            exit_code: Some(0),
            metrics: None,
            status: BenchResultStatus::Success,
        }
    }

    fn score_for(scores: &[ToolScore], tool: &str) -> Option<f64> {
        scores.iter().find(|score| score.tool == tool).map(|score| score.score)
    }

    #[test]
    fn zero_weighted_missing_metrics_do_not_produce_nan_scores() {
        let stage = StageId::new(FASTQ_TRIM);
        let records = vec![
            ("fast".to_string(), vec![successful_record(Some(1.0), None)]),
            ("slow".to_string(), vec![successful_record(Some(2.0), None)]),
        ];

        let selection = select_stage(&stage, &records, &objective_spec(Objective::Speed), false);

        assert_eq!(selection.selected.as_deref(), Some("fast"));
        assert!(score_for(&selection.scores, "fast").is_some_and(f64::is_finite));
        assert!(score_for(&selection.scores, "slow").is_some_and(f64::is_finite));
    }

    #[test]
    fn retention_selection_ignores_invalid_fraction_values() {
        let stage = StageId::new(FASTQ_TRIM);
        let invalid = BenchResultRecord {
            metrics: Some(serde_json::json!({"retention": {"value": 1.2}})),
            ..successful_record(Some(1.0), Some(1.0))
        };
        let valid = BenchResultRecord {
            metrics: Some(serde_json::json!({"retention": 0.9})),
            ..successful_record(Some(5.0), Some(5.0))
        };
        let records =
            vec![("invalid".to_string(), vec![invalid]), ("valid".to_string(), vec![valid])];

        let selection =
            select_stage(&stage, &records, &objective_spec(Objective::Retention), false);

        assert_eq!(selection.selected.as_deref(), Some("valid"));
        assert_eq!(
            selection
                .scores
                .iter()
                .find(|score| score.tool == "invalid")
                .and_then(|score| score.retention_median),
            None
        );
    }
}
