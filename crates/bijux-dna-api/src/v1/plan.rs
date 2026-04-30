//! Planning API for v1.
//!
//! Stability: v1 (stable).

use anyhow::{anyhow, Result};

pub use crate::runtime::run::plan_run;
pub use crate::runtime::run::{select_pipeline, select_pipelines};
pub use crate::surface::explain::{
    explain_bundle, ExplainResponse, PlanExplainStageV1, PlanExplainV1,
};
pub use crate::surface::request_contracts::{
    BamRunArgs, BenchBamPipelineArgs, BenchBamStageArgs, FastqCrossArgs, PlanRunRequest,
    PlanRunResult,
};
pub use crate::v1::bam::plan::plan_for_bam_stage_with_profile;
pub use bijux_dna_core::contract::ExecutionGraph;
pub use bijux_dna_pipelines::bam::{
    validate_bam_profile, BamProfileValidationReport, BamProfileViolation, BAM_INVARIANTS,
};
pub use bijux_dna_pipelines::fastq::{
    validate_fastq_profile, FastqProfileValidationReport, FastqProfileViolation, FASTQ_INVARIANTS,
};
pub use bijux_dna_pipelines::registry::PipelineRegistry;
pub use bijux_dna_pipelines::vcf::{
    validate_vcf_profile, vcf_minimal_profile, VcfProfileValidationReport, VcfProfileViolation,
    VCF_INVARIANTS,
};
pub use bijux_dna_pipelines::{Domain, PipelineProfile};
pub use bijux_dna_pipelines::{
    cross::cross_workflow_template_by_id, cross::cross_workflow_templates,
    cross::cross_workflow_templates_for_pipeline,
};
pub use bijux_dna_planner_bam::{
    pipeline_id_catalog as bam_pipeline_id_catalog, plan_bam_to_bam__adna_capture__v1,
    plan_bam_to_bam__adna_shotgun__v1, BamPipelineInputs,
};
pub use bijux_dna_planner_fastq::{
    cross_fastq_to_bam_id_catalog, fastq_pipeline_id_catalog, plan_fastq_to_bam__default__v1,
    plan_fastq_to_fastq__default__v1, DefaultPipelineOptions, FastqPipelineInputs,
};

#[must_use]
pub fn explain(plan: &ExecutionGraph) -> PlanExplainV1 {
    PlanExplainV1::from_plan(plan)
}

/// Build the canonical explain-profile payload for a governed pipeline profile.
///
/// # Errors
/// Returns an error when the profile id is unknown.
pub fn explain_pipeline_profile(profile_id: &str) -> Result<serde_json::Value> {
    let profile = find_pipeline_profile(profile_id)?;
    let invariants = profile_invariants_json(&profile)?;
    let workflow_templates = cross_workflow_templates_for_pipeline(profile.id.as_str());
    Ok(serde_json::json!({
        "profile_id_input": profile_id,
        "profile_id_resolved": profile.id,
        "library_model": profile.library_model,
        "effective_params": profile.defaults.params,
        "effective_tools": profile.defaults.tools,
        "default_rationale": profile.defaults.rationales,
        "workflow_templates": workflow_templates,
        "supports_sample_sheet": profile.capabilities.supports_sample_sheet,
        "batch_semantics": profile.capabilities.batch_semantics,
        "fan_artifact_rules": profile.capabilities.fan_artifact_rules,
        "failure_policy": profile.capabilities.failure_policy,
        "evidence_summary": profile.capabilities.evidence_summary,
        "parameter_policy": profile.capabilities.parameter_policy,
        "rationale_links": [
            "docs/20-science/SCIENTIFIC_DEFAULTS.md",
            "docs/20-science/SCIENTIFIC_DECISIONS.md",
            "crates/bijux-dna-pipelines/docs/PROFILE_RATIONALE.md"
        ],
        "invariants": invariants,
    }))
}

/// Build the canonical validate-profile payload for a governed pipeline profile.
///
/// # Errors
/// Returns an error when the profile id is unknown.
pub fn validate_pipeline_profile(profile_id: &str) -> Result<serde_json::Value> {
    let profile = find_pipeline_profile(profile_id)?;
    let (has_fastq, has_bam, has_vcf) = profile_domain_flags(&profile);
    match (has_fastq, has_bam, has_vcf) {
        (true, false, false) => Ok(serde_json::to_value(validate_fastq_profile(&profile))?),
        (false, true, false) => Ok(serde_json::to_value(validate_bam_profile(&profile))?),
        (false, false, true) => Ok(serde_json::to_value(validate_vcf_profile(&profile))?),
        _ => Ok(validate_cross_pipeline_profile(&profile)),
    }
}

fn find_pipeline_profile(profile_id: &str) -> Result<PipelineProfile> {
    select_pipelines(None, true)
        .into_iter()
        .find(|profile| profile.id.as_str() == profile_id)
        .ok_or_else(|| anyhow!("unknown pipeline profile: {profile_id}"))
}

fn profile_domain_flags(profile: &PipelineProfile) -> (bool, bool, bool) {
    let has_fastq = profile
        .capabilities
        .required_stages
        .iter()
        .any(|stage| stage.starts_with("fastq."));
    let has_bam = profile
        .capabilities
        .required_stages
        .iter()
        .any(|stage| stage.starts_with("bam."));
    let has_vcf = profile
        .capabilities
        .required_stages
        .iter()
        .any(|stage| stage.starts_with("vcf."));
    (has_fastq, has_bam, has_vcf)
}

fn profile_invariants_json(profile: &PipelineProfile) -> Result<serde_json::Value> {
    let (has_fastq, has_bam, has_vcf) = profile_domain_flags(profile);
    match (has_fastq, has_bam, has_vcf) {
        (true, false, false) => Ok(serde_json::to_value(validate_fastq_profile(profile))?),
        (false, true, false) => Ok(serde_json::to_value(validate_bam_profile(profile))?),
        (false, false, true) => Ok(serde_json::to_value(validate_vcf_profile(profile))?),
        _ => Ok(validate_cross_pipeline_profile(profile)),
    }
}

fn validate_cross_pipeline_profile(profile: &PipelineProfile) -> serde_json::Value {
    let workflow_templates = cross_workflow_templates_for_pipeline(profile.id.as_str());
    let template_ids = workflow_templates
        .iter()
        .map(|template| template.template_id.clone())
        .collect::<Vec<_>>();
    let template_registry_consistent =
        template_ids == profile.capabilities.workflow_template_ids;
    let sample_sheet_consistent = profile.capabilities.supports_sample_sheet
        == workflow_templates
            .iter()
            .all(|template| template.sample_sheet_supported);
    let has_cross_evidence_story = profile.capabilities.evidence_summary.is_some();
    let mut violations = Vec::new();
    if workflow_templates.is_empty() {
        violations.push(serde_json::json!({
            "code": "missing_cross_template",
            "message": "cross-domain profile must expose at least one governed workflow template",
        }));
    }
    if !template_registry_consistent {
        violations.push(serde_json::json!({
            "code": "template_registry_mismatch",
            "message": "profile capability workflow_template_ids drifted from the template registry",
        }));
    }
    if !sample_sheet_consistent {
        violations.push(serde_json::json!({
            "code": "sample_sheet_contract_mismatch",
            "message": "sample-sheet support must stay aligned between the profile capability and template registry",
        }));
    }
    if !has_cross_evidence_story {
        violations.push(serde_json::json!({
            "code": "missing_evidence_story",
            "message": "cross-domain profile must expose a governed evidence summary contract",
        }));
    }
    serde_json::json!({
        "profile_id": profile.id,
        "valid": violations.is_empty(),
        "domain": "cross",
        "workflow_templates": workflow_templates,
        "supports_sample_sheet": profile.capabilities.supports_sample_sheet,
        "template_registry_consistent": template_registry_consistent,
        "sample_sheet_contract_consistent": sample_sheet_consistent,
        "has_cross_evidence_story": has_cross_evidence_story,
        "violations": violations,
    })
}
