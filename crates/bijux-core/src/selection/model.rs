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
    pub stage: String,
    pub selected: Option<String>,
    pub scores: Vec<ToolScore>,
    pub disqualified: Vec<Disqualification>,
}
