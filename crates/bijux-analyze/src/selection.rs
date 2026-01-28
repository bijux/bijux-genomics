use std::cmp::Ordering;

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
    pub stage: String,
    pub selected: Option<String>,
    pub scores: Vec<ToolScore>,
    pub disqualified: Vec<Disqualification>,
}

#[must_use]
pub fn select_stage(
    stage: &str,
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
        stage: stage.to_string(),
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
        Some(f64::midpoint(sorted[mid - 1], sorted[mid]))
    } else {
        Some(sorted[mid])
    }
}

fn read_retention(record: &BenchResultRecord) -> Option<f64> {
    let metrics = record.metrics.as_ref()?;
    metrics
        .get("metrics")
        .and_then(|value| value.get("delta_metrics"))
        .and_then(|value| value.get("read_retention"))
        .and_then(serde_json::Value::as_f64)
}

#[derive(Debug, Serialize)]
pub struct SelectionReport {
    pub objective: String,
    pub weights: ObjectiveWeights,
    pub corpus_id: String,
    pub stages: Vec<StageSelection>,
}

/// Write the selection report to disk.
///
/// # Errors
/// Returns an error if the report cannot be serialized or written.
pub fn write_selection_report(
    out_dir: &std::path::Path,
    objective: &ObjectiveSpec,
    corpus_id: &str,
    stages: Vec<StageSelection>,
) -> anyhow::Result<()> {
    let report = SelectionReport {
        objective: objective.name.clone(),
        weights: objective.weights.clone(),
        corpus_id: corpus_id.to_string(),
        stages,
    };
    let path = out_dir.join("selection_report.json");
    let payload = serde_json::to_string_pretty(&report)?;
    std::fs::write(&path, payload)?;
    Ok(())
}
