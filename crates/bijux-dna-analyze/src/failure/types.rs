use serde::Serialize;

use bijux_dna_core::prelude::errors::ErrorHintV1;

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureKind {
    ToolExit,
    ContractViolation,
    ObserverParse,
    DataInvalid,
    ResourceExhaustion,
    ImageError,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureClass {
    ContractError,
    ToolError,
    EnvironmentError,
}

#[derive(Debug, Clone, Serialize)]
pub struct BenchmarkFailure {
    pub stage: String,
    pub tool: String,
    pub kind: FailureKind,
    pub reason: String,
    pub hints: Vec<ErrorHintV1>,
}
