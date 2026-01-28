use serde::Serialize;

use crate::core::RawFailure;

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
pub fn classify_raw_failure(raw: &RawFailure) -> BenchmarkFailure {
    let msg = raw.reason.to_lowercase();
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
    } else if (raw.stage == "fastq.validate" && msg.contains("strict validation failed"))
        || msg.contains("invalid fastq")
        || (msg.contains("fastq") && msg.contains("invalid"))
    {
        FailureClass::DataError
    } else {
        FailureClass::ToolError
    };
    BenchmarkFailure {
        stage: raw.stage.clone(),
        tool: raw.tool.clone(),
        class,
        reason: raw.reason.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_failure_detects_data_errors() {
        let raw = RawFailure {
            stage: "fastq.validate".to_string(),
            tool: "fastqvalidator".to_string(),
            reason: "strict validation failed for fastqvalidator".to_string(),
        };
        let failure = classify_raw_failure(&raw);
        assert!(matches!(failure.class, FailureClass::DataError));
    }

    #[test]
    fn classify_failure_detects_invariants() {
        let raw = RawFailure {
            stage: "fastq.trim".to_string(),
            tool: "fastp".to_string(),
            reason: "reads_out must be <= reads_in".to_string(),
        };
        let failure = classify_raw_failure(&raw);
        assert!(matches!(failure.class, FailureClass::InvariantViolation));
    }

    #[test]
    fn classify_failure_defaults_to_tool_error() {
        let raw = RawFailure {
            stage: "fastq.trim".to_string(),
            tool: "fastp".to_string(),
            reason: "unexpected crash".to_string(),
        };
        let failure = classify_raw_failure(&raw);
        assert!(matches!(failure.class, FailureClass::ToolError));
    }
}
