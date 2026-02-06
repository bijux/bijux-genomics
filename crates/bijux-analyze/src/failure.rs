//! Owner: bijux-analyze
//! Failure classification and structured remediation hints.
//! Owns stable failure IDs and remediation guidance.
//! Must not perform IO or depend on pipeline/report layers.
//! Invariants: failure kinds are stable and hints are structured.

use serde::Serialize;

use bijux_core::foundation::errors::{ErrorCategory, ErrorHintV1, HintSeverity};
use bijux_core::foundation::RawFailure;

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
    } else if (raw.stage == "fastq.validate_pre" && msg.contains("strict validation failed"))
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

#[must_use]
fn remediation_hints_for_failure(raw: &RawFailure) -> Vec<ErrorHintV1> {
    let msg = raw.reason.to_lowercase();
    let mut hints = Vec::new();
    if msg.contains("adapter") || msg.contains("adapter preset") {
        hints.push(ErrorHintV1 {
            id: "adapter_preset_missing".to_string(),
            category: ErrorCategory::ContractError,
            severity: HintSeverity::Medium,
            message: "Adapter preset missing or invalid".to_string(),
            suggested_action: "Configure a valid adapter preset or supply an adapter file"
                .to_string(),
            docs_link_key: Some("adapters".to_string()),
        });
    }
    if msg.contains("polyg") || msg.contains("poly-g") {
        hints.push(ErrorHintV1 {
            id: "polyg_artifact".to_string(),
            category: ErrorCategory::ContractError,
            severity: HintSeverity::Low,
            message: "Poly-G artifact suspected".to_string(),
            suggested_action: "Enable illumina_twocolor or configure polyG filtering".to_string(),
            docs_link_key: Some("polyg".to_string()),
        });
    }
    if raw.stage == "fastq.screen" || msg.contains("contaminant") {
        hints.push(ErrorHintV1 {
            id: "contamination_screen".to_string(),
            category: ErrorCategory::ContractError,
            severity: HintSeverity::Medium,
            message: "Potential contaminant signal detected".to_string(),
            suggested_action: "Review contaminant screen output and adjust contaminant bank"
                .to_string(),
            docs_link_key: Some("contamination".to_string()),
        });
    }
    if msg.contains("missing outputs") {
        hints.push(ErrorHintV1 {
            id: "missing_outputs".to_string(),
            category: ErrorCategory::ToolError,
            severity: HintSeverity::High,
            message: "Expected outputs missing".to_string(),
            suggested_action: "Check tool output paths, permissions, and working directory"
                .to_string(),
            docs_link_key: Some("outputs".to_string()),
        });
    }
    hints
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_failure_detects_data_errors() {
        let raw = RawFailure {
            stage: "fastq.validate_pre".to_string(),
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
            stage: "fastq.trim".to_string(),
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
            stage: "fastq.trim".to_string(),
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
            stage: "fastq.trim".to_string(),
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
