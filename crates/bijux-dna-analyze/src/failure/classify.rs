use bijux_dna_core::prelude::errors::ErrorCategory;
use bijux_dna_core::prelude::RawFailure;

use super::hints::remediation_hints_for_failure;
use super::{BenchmarkFailure, FailureClass, FailureKind};

#[must_use]
pub fn failure_class(kind: FailureKind) -> FailureClass {
    match kind {
        FailureKind::DataInvalid | FailureKind::ContractViolation => FailureClass::ContractError,
        FailureKind::ImageError | FailureKind::ResourceExhaustion => FailureClass::EnvironmentError,
        FailureKind::ObserverParse | FailureKind::ToolExit => FailureClass::ToolError,
    }
}

#[must_use]
pub fn error_category(kind: FailureKind) -> ErrorCategory {
    match kind {
        FailureKind::DataInvalid | FailureKind::ContractViolation => ErrorCategory::ContractError,
        FailureKind::ImageError | FailureKind::ResourceExhaustion => ErrorCategory::InfraError,
        FailureKind::ObserverParse => ErrorCategory::ParseError,
        FailureKind::ToolExit => ErrorCategory::ToolError,
    }
}

#[must_use]
pub fn classify_raw_failure(raw: &RawFailure) -> BenchmarkFailure {
    let msg = raw.reason.to_lowercase();
    let kind = match raw.category {
        ErrorCategory::PlanError => FailureKind::DataInvalid,
        ErrorCategory::ContractError => FailureKind::ContractViolation,
        ErrorCategory::ParseError => FailureKind::ObserverParse,
        ErrorCategory::ToolError => FailureKind::ToolExit,
        ErrorCategory::InfraError => FailureKind::ResourceExhaustion,
    };
    let kind = if msg.contains("timeout") || msg.contains("out of memory") {
        FailureKind::ResourceExhaustion
    } else if msg.contains("docker image not found")
        || msg.contains("missing runtime dependency")
        || msg.contains("docker run failed")
        || msg.contains("image not found")
    {
        FailureKind::ImageError
    } else if msg.contains("validation error")
        || msg.contains("invariant")
        || msg.contains("must be <=")
        || msg.contains("must equal")
    {
        FailureKind::ContractViolation
    } else if (raw.stage == "fastq.validate_reads" && msg.contains("strict validation failed"))
        || msg.contains("invalid fastq")
        || (msg.contains("fastq") && msg.contains("invalid"))
    {
        FailureKind::DataInvalid
    } else if msg.contains("parse") || msg.contains("observer") {
        FailureKind::ObserverParse
    } else {
        kind
    };
    BenchmarkFailure {
        stage: raw.stage.clone(),
        tool: raw.tool.clone(),
        kind,
        reason: raw.reason.clone(),
        hints: remediation_hints_for_failure(raw),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_failure_detects_data_errors() {
        let raw = RawFailure {
            stage: "fastq.validate_reads".to_string(),
            tool: "fastqvalidator".to_string(),
            reason: "strict validation failed for fastqvalidator".to_string(),
            category: ErrorCategory::ContractError,
        };
        let failure = classify_raw_failure(&raw);
        assert!(matches!(failure.kind, FailureKind::DataInvalid));
    }

    #[test]
    fn classify_failure_detects_invariants() {
        let raw = RawFailure {
            stage: "fastq.trim_reads".to_string(),
            tool: "fastp".to_string(),
            reason: "invariant failed: reads_out must be <= reads_in".to_string(),
            category: ErrorCategory::ContractError,
        };
        let failure = classify_raw_failure(&raw);
        assert!(matches!(failure.kind, FailureKind::ContractViolation));
    }

    #[test]
    fn classify_failure_defaults_to_tool_exit() {
        let raw = RawFailure {
            stage: "fastq.trim_reads".to_string(),
            tool: "fastp".to_string(),
            reason: "unexpected crash".to_string(),
            category: ErrorCategory::ToolError,
        };
        let failure = classify_raw_failure(&raw);
        assert!(matches!(failure.kind, FailureKind::ToolExit));
    }

    #[test]
    fn classify_failure_includes_remediation_hints() {
        let raw = RawFailure {
            stage: "fastq.trim_reads".to_string(),
            tool: "fastp".to_string(),
            reason: "adapter preset missing".to_string(),
            category: ErrorCategory::ContractError,
        };
        let failure = classify_raw_failure(&raw);
        assert!(failure
            .hints
            .iter()
            .any(|hint| hint.id.contains("adapter_preset")));
    }
}
