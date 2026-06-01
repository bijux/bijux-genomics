use anyhow::{anyhow, Result};
use bijux_dna_core::contract::ExecutionGraph;
use bijux_dna_pipelines::PipelineProfile;
use bijux_dna_stage_contract::{PlanDecisionReason, PlanReasonKind, StagePlanV1};
use serde_json::Value;
use std::collections::BTreeSet;

pub const PLANNER_VERSION: &str = "bijux-dna-planner-bam.v1";

mod api;
mod execution_graph;
mod local_readiness;
mod params;
mod profile_catalog;
mod report_stage;
mod selection;
mod stage_activation;
mod stage_dispatch;
mod stages;
pub mod tool_adapters;
mod tool_policy;

pub use api::stage_api;
pub use api::{BamPipelineInputs, BamPlanConfig, StagePlanRequest};
pub use report_stage::report_stage_step;

pub struct BamPlanner;

impl BamPlanner {
    /// # Errors
    /// Returns an error if the plan lint fails.
    pub fn plan(config: &BamPlanConfig) -> Result<ExecutionGraph> {
        execution_graph::from_stage_plans(
            &config.pipeline_id,
            config.policy,
            &config.stages,
            "planned bam execution graph",
        )
    }
}

/// # Errors
/// Returns an error if the stage cannot be planned with the provided inputs.
#[allow(clippy::needless_pass_by_value)]
pub fn plan_stage(request: StagePlanRequest<'_>) -> Result<StagePlanV1> {
    let stage = bijux_dna_domain_bam::BamStage::try_from(request.stage_id)?;
    tool_policy::enforce(stage, &request.tool.tool_id.0, request.params, request.reference)?;
    let mut plan = stage_dispatch::plan(stage, &request)?;
    let mut details = serde_json::Map::new();
    details.insert("defaults_diff".to_string(), serde_json::json!({}));
    if let Some(Ok(hash)) = bijux_dna_domain_bam::stage_contract_hash(request.stage_id) {
        details.insert("contract_hash".to_string(), Value::String(hash));
    }
    let input_bytes = request
        .bam
        .and_then(|path| std::fs::metadata(path).ok())
        .map_or(0, |metadata| metadata.len());
    details.insert(
        "resource_plan".to_string(),
        serde_json::to_value(bijux_dna_domain_bam::estimate_bam_stage_resources(
            request.stage_id,
            input_bytes,
        ))
        .unwrap_or_else(|error| serde_json::json!({ "serialization_error": error.to_string() })),
    );
    plan.reason = PlanDecisionReason::new(
        PlanReasonKind::Default,
        format!("tool {} selected by planner", plan.tool_id.0),
    );
    plan.reason.details = Value::Object(details);
    Ok(plan)
}

/// # Errors
/// Returns an error if pipeline planning fails.
#[allow(non_snake_case)]
pub fn plan_bam_to_bam__default__v1(inputs: &BamPipelineInputs) -> Result<ExecutionGraph> {
    let profile = profile_catalog::profile_by_id("bam-to-bam__default__v1")
        .ok_or_else(|| anyhow!("missing builtin BAM profile bam-to-bam__default__v1"))?;
    let stages = build_bam_stage_plans(&profile, inputs)?;
    execution_graph::from_stage_plans(
        profile.id.as_str(),
        inputs.policy,
        &stages,
        "planned bam pipeline graph",
    )
}

/// # Errors
/// Returns an error if pipeline planning fails.
#[allow(non_snake_case)]
pub fn plan_bam_to_bam__adna_shotgun__v1(inputs: &BamPipelineInputs) -> Result<ExecutionGraph> {
    let profile = profile_catalog::adna_shotgun_profile();
    let stages = build_bam_stage_plans(&profile, inputs)?;
    execution_graph::from_stage_plans(
        profile.id.as_str(),
        inputs.policy,
        &stages,
        "planned bam pipeline graph",
    )
}

/// # Errors
/// Returns an error if pipeline planning fails.
#[allow(non_snake_case)]
pub fn plan_bam_to_bam__adna_capture__v1(inputs: &BamPipelineInputs) -> Result<ExecutionGraph> {
    let profile = profile_catalog::adna_capture_profile();
    let stages = build_bam_stage_plans(&profile, inputs)?;
    execution_graph::from_stage_plans(
        profile.id.as_str(),
        inputs.policy,
        &stages,
        "planned bam pipeline graph",
    )
}

pub fn pipeline_id_catalog(profile_id: &str) -> Vec<String> {
    profile_catalog::pipeline_id_catalog(profile_id)
}

#[must_use]
pub fn bam_workflow_template_catalog() -> Vec<bijux_dna_domain_bam::BamWorkflowTemplateV1> {
    bijux_dna_domain_bam::bam_workflow_templates()
}

/// # Errors
/// Returns an error if the template is unknown or the underlying profile cannot be resolved.
pub fn plan_bam_workflow_template(
    template_id: &str,
    inputs: &BamPipelineInputs,
) -> Result<ExecutionGraph> {
    let template = bijux_dna_domain_bam::bam_workflow_template_by_id(template_id)
        .ok_or_else(|| anyhow!("unknown bam workflow template: {template_id}"))?;
    let profile = profile_catalog::profile_by_id(&template.profile_id).ok_or_else(|| {
        anyhow!(
            "bam workflow template {} references unknown profile {}",
            template.template_id,
            template.profile_id
        )
    })?;
    let stages = build_bam_stage_plans(&profile, inputs)?;
    execution_graph::from_stage_plans(
        profile.id.as_str(),
        inputs.policy,
        &stages,
        "planned bam pipeline graph",
    )
}

/// # Errors
/// Returns an error if stage planning fails for the requested BAM profile.
pub fn plan_bam_stage_plans_for_profile_id(
    profile_id: &str,
    inputs: &BamPipelineInputs,
) -> Result<Vec<StagePlanV1>> {
    let profile = profile_catalog::profile_by_id(profile_id)
        .ok_or_else(|| anyhow!("unknown BAM profile: {profile_id}"))?;
    build_bam_stage_plans(&profile, inputs)
}

/// # Errors
/// Returns an error if stage planning fails for the requested BAM template.
pub fn plan_bam_workflow_template_stage_plans(
    template_id: &str,
    inputs: &BamPipelineInputs,
) -> Result<Vec<StagePlanV1>> {
    let template = bijux_dna_domain_bam::bam_workflow_template_by_id(template_id)
        .ok_or_else(|| anyhow!("unknown bam workflow template: {template_id}"))?;
    let profile = profile_catalog::profile_by_id(&template.profile_id).ok_or_else(|| {
        anyhow!(
            "bam workflow template {} references unknown profile {}",
            template.template_id,
            template.profile_id
        )
    })?;
    build_bam_stage_plans(&profile, inputs)
}

/// # Errors
/// Returns an error if stage planning fails for the requested BAM profile.
#[allow(non_snake_case)]
pub fn plan_bam_to_bam__default__v1_stage_plans(
    inputs: &BamPipelineInputs,
) -> Result<Vec<StagePlanV1>> {
    plan_bam_stage_plans_for_profile_id("bam-to-bam__default__v1", inputs)
}

/// # Errors
/// Returns an error if stage planning fails for the requested BAM profile.
#[allow(non_snake_case)]
pub fn plan_bam_to_bam__adna_shotgun__v1_stage_plans(
    inputs: &BamPipelineInputs,
) -> Result<Vec<StagePlanV1>> {
    let profile = profile_catalog::adna_shotgun_profile();
    build_bam_stage_plans(&profile, inputs)
}

/// # Errors
/// Returns an error if stage planning fails for the requested BAM profile.
#[allow(non_snake_case)]
pub fn plan_bam_to_bam__adna_capture__v1_stage_plans(
    inputs: &BamPipelineInputs,
) -> Result<Vec<StagePlanV1>> {
    let profile = profile_catalog::adna_capture_profile();
    build_bam_stage_plans(&profile, inputs)
}

fn build_bam_stage_plans(
    profile: &PipelineProfile,
    inputs: &BamPipelineInputs,
) -> Result<Vec<StagePlanV1>> {
    validate_pipeline_input_maps(profile, inputs)?;
    let mut bam = inputs.bam.clone();
    let mut bam_index = inputs.bam_index.clone();
    let mut stages = Vec::new();
    for stage in profile_catalog::ordered_stages(profile)? {
        let stage_id = stage.as_str();
        stage_activation::enforce(stage_id, inputs.allow_planned)?;
        let tool = inputs
            .tool_specs
            .get(stage_id)
            .ok_or_else(|| anyhow!("missing tool spec for stage {stage_id}"))?;
        let stage_key = bijux_dna_core::ids::StageId::from_static(stage_id);
        let default_params = profile.defaults.params.get(&stage_key).map(|params| params.to_json());
        let params = inputs.params_overrides.get(stage_id).or(default_params.as_ref());
        let stage_dir = inputs.out_dir.join(stage_id.replace('.', "_"));
        let plan = plan_stage(StagePlanRequest {
            stage_id,
            tool,
            out_dir: &stage_dir,
            bam: Some(&bam),
            bam_index: bam_index.as_deref(),
            r1: None,
            r2: None,
            reference: inputs.reference.as_deref(),
            sample_id: inputs.sample_id.as_deref(),
            params,
        })?;
        let next_bam =
            next_required_output_path(&plan, bijux_dna_core::contract::ArtifactRole::Bam);
        let next_bai =
            next_required_output_path(&plan, bijux_dna_core::contract::ArtifactRole::Index);
        if let Some(path) = next_bam {
            bam = path;
        }
        if let Some(path) = next_bai {
            bam_index = Some(path);
        }
        stages.push(plan);
    }
    Ok(stages)
}

fn validate_pipeline_input_maps(
    profile: &PipelineProfile,
    inputs: &BamPipelineInputs,
) -> Result<()> {
    let stage_ids = profile
        .capabilities
        .required_stages
        .iter()
        .map(|stage_id| stage_id.as_str())
        .collect::<BTreeSet<_>>();
    for stage_id in inputs.params_overrides.keys() {
        if !stage_ids.contains(stage_id.as_str()) {
            return Err(anyhow!(
                "params override for stage {stage_id} is not part of profile {}",
                profile.id.as_str()
            ));
        }
    }
    for stage_id in inputs.tool_specs.keys() {
        if !stage_ids.contains(stage_id.as_str()) {
            return Err(anyhow!(
                "tool spec for stage {stage_id} is not part of profile {}",
                profile.id.as_str()
            ));
        }
    }
    Ok(())
}

fn next_required_output_path(
    plan: &StagePlanV1,
    role: bijux_dna_core::contract::ArtifactRole,
) -> Option<std::path::PathBuf> {
    plan.io
        .outputs
        .iter()
        .find(|output| !output.optional && output.role == role)
        .map(|output| output.path.clone())
}
