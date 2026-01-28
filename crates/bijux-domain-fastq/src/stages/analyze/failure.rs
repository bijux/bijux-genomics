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

#[must_use]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_failure_detects_data_errors() {
        let err = anyhow::anyhow!("strict validation failed for fastqvalidator");
        let failure = classify_failure("fastq.validate", "fastqvalidator", &err);
        assert!(matches!(failure.class, FailureClass::DataError));
    }

    #[test]
    fn classify_failure_detects_invariants() {
        let err = anyhow::anyhow!("reads_out must be <= reads_in");
        let failure = classify_failure("fastq.trim", "fastp", &err);
        assert!(matches!(failure.class, FailureClass::InvariantViolation));
    }

    #[test]
    fn classify_failure_defaults_to_tool_error() {
        let err = anyhow::anyhow!("unexpected crash");
        let failure = classify_failure("fastq.trim", "fastp", &err);
        assert!(matches!(failure.class, FailureClass::ToolError));
    }
}
