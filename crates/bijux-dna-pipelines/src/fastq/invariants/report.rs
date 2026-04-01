use serde::Serialize;

use crate::{InvariantSeverity, InvariantViolationV1, InvariantsReportV1, PipelineProfile};

pub const FASTQ_INVARIANTS: &str = "fastq-invariants.v2";

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct FastqProfileViolation {
    pub code: &'static str,
    pub stage_id: Option<String>,
    pub severity: InvariantSeverity,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct FastqProfileValidationReport {
    pub profile_id: String,
    pub invariants_version: &'static str,
    pub invariants_preset: Option<String>,
    pub valid: bool,
    pub violations: Vec<FastqProfileViolation>,
}

impl FastqProfileValidationReport {
    #[must_use]
    pub fn from_violations(
        profile: &PipelineProfile,
        violations: Vec<FastqProfileViolation>,
    ) -> Self {
        Self {
            profile_id: profile.id.as_str().to_string(),
            invariants_version: FASTQ_INVARIANTS,
            invariants_preset: profile
                .invariants_preset
                .map(|preset| preset.as_str().to_string()),
            valid: violations.is_empty(),
            violations,
        }
    }

    #[must_use]
    pub fn as_invariants_report(&self) -> InvariantsReportV1 {
        InvariantsReportV1 {
            schema_version: "bijux.invariants_report.v1".to_string(),
            profile_id: self.profile_id.clone(),
            invariants_version: self.invariants_version.to_string(),
            valid: self.valid,
            blocking: self
                .violations
                .iter()
                .any(|v| v.severity == InvariantSeverity::Hard),
            violations: self
                .violations
                .iter()
                .map(|v| InvariantViolationV1 {
                    code: v.code.to_string(),
                    stage_id: v.stage_id.clone(),
                    severity: v.severity,
                    message: v.message.clone(),
                })
                .collect(),
        }
    }
}

pub(super) fn violation(
    code: &'static str,
    stage_id: Option<&str>,
    severity: InvariantSeverity,
    message: impl Into<String>,
) -> FastqProfileViolation {
    FastqProfileViolation {
        code,
        stage_id: stage_id.map(str::to_string),
        severity,
        message: message.into(),
    }
}
