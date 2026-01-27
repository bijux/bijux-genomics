use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureClass {
    ImageError,
    ToolError,
    DataError,
    InvariantViolation,
    ResourceExhaustion,
}

#[derive(Debug, Clone, Serialize)]
pub struct BenchmarkFailure {
    pub stage: String,
    pub tool: String,
    pub class: FailureClass,
    pub reason: String,
}

pub fn classify_failure(stage: &str, tool: &str, err: &anyhow::Error) -> BenchmarkFailure {
    let reason = err.to_string();
    let msg = reason.to_lowercase();
    let class = if msg.contains("timeout") {
        FailureClass::ResourceExhaustion
    } else if msg.contains("docker image not found")
        || msg.contains("missing runtime dependency")
        || msg.contains("docker run failed")
        || msg.contains("image not found")
    {
        FailureClass::ImageError
    } else if msg.contains("validation error")
        || msg.contains("invariant")
        || msg.contains("must be <=")
        || msg.contains("must equal")
    {
        FailureClass::InvariantViolation
    } else if (stage == "fastq.validate" && msg.contains("strict validation failed"))
        || msg.contains("invalid fastq")
        || (msg.contains("fastq") && msg.contains("invalid"))
    {
        FailureClass::DataError
    } else {
        FailureClass::ToolError
    };
    BenchmarkFailure {
        stage: stage.to_string(),
        tool: tool.to_string(),
        class,
        reason,
    }
}
