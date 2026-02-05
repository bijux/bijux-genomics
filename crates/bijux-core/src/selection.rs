use std::cmp::Ordering;

use crate::contract::{
    BenchResultRecord, BenchResultStatus, Disqualification, Objective, ObjectiveSpec,
    ObjectiveWeights, StageSelection, ToolScore,
};

pub use crate::contract::{
    BenchResultRecord as BenchRecord, BenchResultStatus as BenchStatus, Objective as ObjectiveKind,
    ObjectiveSpec as ObjectiveSchema, ObjectiveWeights as ObjectiveWeighting,
    StageSelection as Selection,
};

#[must_use]
pub fn objective_spec(objective: Objective) -> ObjectiveSpec {
    match objective {
        Objective::Speed => ObjectiveSpec {
            name: objective.as_str().to_string(),
            weights: ObjectiveWeights {
                runtime: 1.0,
                memory: 0.0,
                retention: 0.0,
            },
        },
        Objective::Memory => ObjectiveSpec {
            name: objective.as_str().to_string(),
            weights: ObjectiveWeights {
                runtime: 0.0,
                memory: 1.0,
                retention: 0.0,
            },
        },
        Objective::Retention => ObjectiveSpec {
            name: objective.as_str().to_string(),
            weights: ObjectiveWeights {
                runtime: 0.0,
                memory: 0.0,
                retention: -1.0,
            },
        },
        Objective::Balanced => ObjectiveSpec {
            name: objective.as_str().to_string(),
            weights: ObjectiveWeights {
                runtime: 1.0,
                memory: 1.0,
                retention: -100.0,
            },
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

    StageSelection {
        stage: stage.clone(),
        selected,
        scores,
        disqualified,
    }
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
    (runtime * objective.weights.runtime)
        + (memory * objective.weights.memory)
        + (retention * objective.weights.retention)
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
    retention
        .get("value")
        .and_then(serde_json::Value::as_f64)
        .or_else(|| retention.as_f64())
}
