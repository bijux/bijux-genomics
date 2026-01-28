use std::cmp::Ordering;
use std::path::Path;

use anyhow::Result;
use serde::Serialize;

use super::corpus::BenchCorpus;
use super::objective::Objective;
use super::query::{get_results, BenchResultRecord, BenchResultStatus};

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

/// Rank tools for a stage based on a corpus and objective.
///
/// # Errors
/// Returns an error if bench results cannot be loaded or parsed.
pub fn rank_tools_for_stage(
    stage: &str,
    tools: &[String],
    objective: Objective,
    corpus: &BenchCorpus,
    out_dir: &Path,
    allow_partial: bool,
) -> Result<StageSelection> {
    let mut scores = Vec::new();
    let mut disqualified = Vec::new();

    for tool in tools {
        let records = get_results(stage, tool, corpus, out_dir)?;
        let mut failed = false;
        let mut runtime = Vec::new();
        let mut memory = Vec::new();
        let mut retention = Vec::new();

        for record in &records {
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

    scores.sort_by(|a, b| compare_score(objective, a.score, b.score));
    let selected = scores.first().map(|entry| entry.tool.clone());

    Ok(StageSelection {
        stage: stage.to_string(),
        selected,
        scores,
        disqualified,
    })
}

fn score_for_objective(
    objective: Objective,
    runtime: Option<f64>,
    memory: Option<f64>,
    retention: Option<f64>,
) -> f64 {
    match objective {
        Objective::Speed => runtime.unwrap_or(f64::INFINITY),
        Objective::Memory => memory.unwrap_or(f64::INFINITY),
        Objective::Retention => retention.map_or(f64::INFINITY, |v| -v),
        Objective::Balanced => {
            let runtime = runtime.unwrap_or(f64::INFINITY);
            let memory = memory.unwrap_or(f64::INFINITY);
            let retention = retention.unwrap_or(0.0);
            runtime + memory - (retention * 100.0)
        }
    }
}

fn compare_score(_objective: Objective, left: f64, right: f64) -> Ordering {
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
