//! BAM pipeline profile invariants and validation.

use std::collections::BTreeSet;

use bijux_dna_core::ids::{StageId, UdgTreatment};
use bijux_dna_core::prelude::id_catalog;
use serde::Serialize;

use crate::{
    InvariantSeverity, InvariantViolationV1, InvariantsPreset, InvariantsReportV1, PipelineProfile,
};

pub const BAM_INVARIANTS: &str = "bam-invariants.v1";

const CORE_BAM_STAGES: [&str; 5] = [
    id_catalog::BAM_VALIDATE,
    "bam.filter",
    id_catalog::BAM_COVERAGE,
    id_catalog::BAM_DAMAGE,
    "bam.mapping_summary",
];

const INDEX_DEPENDENT_BAM_STAGES: [&str; 9] = [
    id_catalog::BAM_COVERAGE,
    id_catalog::BAM_DAMAGE,
    id_catalog::BAM_AUTHENTICITY,
    id_catalog::BAM_CONTAMINATION,
    id_catalog::BAM_SEX,
    id_catalog::BAM_HAPLOGROUPS,
    "bam.genotyping",
    id_catalog::BAM_KINSHIP,
    id_catalog::BAM_INSERT_SIZE,
];

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct BamProfileViolation {
    pub code: &'static str,
    pub stage_id: Option<String>,
    pub severity: InvariantSeverity,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct BamProfileValidationReport {
    pub profile_id: String,
    pub invariants_version: &'static str,
    pub invariants_preset: Option<String>,
    pub valid: bool,
    pub violations: Vec<BamProfileViolation>,
}

impl BamProfileValidationReport {
    #[must_use]
    pub fn from_violations(
        profile: &PipelineProfile,
        violations: Vec<BamProfileViolation>,
    ) -> Self {
        Self {
            profile_id: profile.id.as_str().to_string(),
            invariants_version: BAM_INVARIANTS,
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

fn violation(
    code: &'static str,
    stage_id: Option<&str>,
    severity: InvariantSeverity,
    message: impl Into<String>,
) -> BamProfileViolation {
    BamProfileViolation {
        code,
        stage_id: stage_id.map(str::to_string),
        severity,
        message: message.into(),
    }
}

fn stage_set(profile: &PipelineProfile) -> BTreeSet<&str> {
    profile
        .capabilities
        .required_stages
        .iter()
        .copied()
        .collect()
}

fn has_stage_params(profile: &PipelineProfile, stage_id: &str) -> bool {
    profile
        .defaults
        .params
        .contains_key(&StageId::new(stage_id.to_string()))
}

#[must_use]
pub fn validate_bam_profile(profile: &PipelineProfile) -> BamProfileValidationReport {
    let mut violations = Vec::new();
    let required_stages = stage_set(profile);

    for stage in CORE_BAM_STAGES {
        if !required_stages.contains(stage) {
            violations.push(violation(
                "required_stage_missing",
                Some(stage),
                InvariantSeverity::Hard,
                format!("required BAM stage `{stage}` is missing"),
            ));
        }
    }

    if !profile
        .capabilities
        .required_metrics
        .contains(&"bam.metrics")
    {
        violations.push(violation(
            "required_metrics_missing",
            None,
            InvariantSeverity::Hard,
            "BAM profiles must require bam.metrics output",
        ));
    }

    for artifact in [
        "report.json",
        "run_manifest.json",
        "stage_summaries.json",
        "invariants_report.json",
    ] {
        if !profile.capabilities.required_artifacts.contains(&artifact) {
            violations.push(violation(
                "required_artifact_missing",
                None,
                InvariantSeverity::Hard,
                format!(
                    "BAM profiles must require `{artifact}` for metrics/provenance completeness"
                ),
            ));
        }
    }

    for stage in &required_stages {
        if !has_stage_params(profile, stage) {
            violations.push(violation(
                "required_params_missing",
                Some(stage),
                InvariantSeverity::Hard,
                format!("missing or invalid params for BAM stage `{stage}`"),
            ));
        }
    }

    if profile.invariants_preset == Some(InvariantsPreset::Adna)
        && !required_stages.contains(id_catalog::BAM_DAMAGE)
    {
        violations.push(violation(
            "adna_damage_stage_missing",
            Some(id_catalog::BAM_DAMAGE),
            InvariantSeverity::Hard,
            "aDNA BAM profile must include bam.damage unless explicitly disabled with justification",
        ));
    }
    if profile.invariants_preset == Some(InvariantsPreset::Adna)
        && profile.library_model.udg_treatment == UdgTreatment::Unknown
    {
        violations.push(violation(
            "library_udg_treatment_unknown",
            None,
            InvariantSeverity::Soft,
            "aDNA BAM profile should declare library_model.udg_treatment to calibrate damage expectations",
        ));
    }

    let has_indexing_gate = required_stages.contains(id_catalog::BAM_VALIDATE)
        || required_stages.contains("bam.qc_pre");
    if !has_indexing_gate {
        for stage in INDEX_DEPENDENT_BAM_STAGES {
            if required_stages.contains(stage) {
                violations.push(violation(
                    "index_prerequisite_missing",
                    Some(stage),
                    InvariantSeverity::Hard,
                    "index-dependent BAM QC stages require bam.validate or bam.qc_pre first",
                ));
            }
        }
    }

    BamProfileValidationReport::from_violations(profile, violations)
}
