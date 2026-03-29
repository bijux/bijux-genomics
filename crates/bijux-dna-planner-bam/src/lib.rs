use anyhow::{anyhow, Result};
use bijux_dna_core::contract::ExecutionGraph;
use bijux_dna_core::prelude::id_catalog;
use bijux_dna_domain_bam::BamStage;
use bijux_dna_pipelines::PipelineProfile;
use bijux_dna_stage_contract::{PlanDecisionReason, PlanReasonKind, StagePlanV1};
use serde_json::Value;

pub const PLANNER_VERSION: &str = "bijux-dna-planner-bam.v1";

mod api;
mod execution_graph;
mod params;
mod profile_catalog;
mod report_stage;
mod selection;
mod stage_activation;
mod stage_dispatch;
mod stages;
pub mod tool_adapters;

pub use api::{BamPipelineInputs, BamPlanConfig, StagePlanRequest};
pub use api::stage_api;
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
            "planned bam execution graph"
        )
    }
}

/// # Errors
/// Returns an error if the stage cannot be planned with the provided inputs.
#[allow(clippy::needless_pass_by_value, clippy::too_many_lines)]
pub fn plan_stage(request: StagePlanRequest<'_>) -> Result<StagePlanV1> {
    let stage = bijux_dna_domain_bam::BamStage::try_from(request.stage_id)?;
    let mut plan = match stage {
        bijux_dna_domain_bam::BamStage::Align
        | bijux_dna_domain_bam::BamStage::Validate
        | bijux_dna_domain_bam::BamStage::QcPre
        | bijux_dna_domain_bam::BamStage::MappingSummary
        | bijux_dna_domain_bam::BamStage::Filter
        | bijux_dna_domain_bam::BamStage::MapqFilter
        | bijux_dna_domain_bam::BamStage::LengthFilter
        | bijux_dna_domain_bam::BamStage::OverlapCorrection => {
            stage_dispatch::pre::plan(stage, &request)
        }
        bijux_dna_domain_bam::BamStage::Markdup
        | bijux_dna_domain_bam::BamStage::DuplicationMetrics
        | bijux_dna_domain_bam::BamStage::Complexity
        | bijux_dna_domain_bam::BamStage::Coverage
        | bijux_dna_domain_bam::BamStage::InsertSize
        | bijux_dna_domain_bam::BamStage::GcBias
        | bijux_dna_domain_bam::BamStage::EndogenousContent
        | bijux_dna_domain_bam::BamStage::Recalibration => {
            stage_dispatch::post::plan(stage, &request)
        }
        bijux_dna_domain_bam::BamStage::Damage
        | bijux_dna_domain_bam::BamStage::Authenticity
        | bijux_dna_domain_bam::BamStage::Contamination
        | bijux_dna_domain_bam::BamStage::Sex => stage_dispatch::adna::plan(stage, &request),
        bijux_dna_domain_bam::BamStage::BiasMitigation
        | bijux_dna_domain_bam::BamStage::Haplogroups
        | bijux_dna_domain_bam::BamStage::Genotyping
        | bijux_dna_domain_bam::BamStage::Kinship => {
            stage_dispatch::downstream::plan(stage, &request)
        }
    }?;
    let mut details = serde_json::Map::new();
    details.insert("defaults_diff".to_string(), serde_json::json!({}));
    if let Some(Ok(hash)) = bijux_dna_domain_bam::stage_contract_hash(request.stage_id) {
        details.insert("contract_hash".to_string(), Value::String(hash));
    }
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
pub fn plan_bam_to_bam__adna_shotgun__v1(inputs: &BamPipelineInputs) -> Result<ExecutionGraph> {
    let profile = profile_catalog::adna_shotgun_profile();
    build_bam_plan(&profile, inputs)
}

/// # Errors
/// Returns an error if pipeline planning fails.
#[allow(non_snake_case)]
pub fn plan_bam_to_bam__adna_capture__v1(inputs: &BamPipelineInputs) -> Result<ExecutionGraph> {
    let profile = profile_catalog::adna_capture_profile();
    build_bam_plan(&profile, inputs)
}

pub fn pipeline_id_catalog(profile_id: &str) -> Vec<String> {
    profile_catalog::pipeline_id_catalog(profile_id)
}

fn build_bam_plan(profile: &PipelineProfile, inputs: &BamPipelineInputs) -> Result<ExecutionGraph> {
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
        let default_params = profile
            .defaults
            .params
            .get(&stage_key)
            .map(|params| params.to_json());
        let params = inputs
            .params_overrides
            .get(stage_id)
            .or(default_params.as_ref());
        enforce_stage_tool_contracts(stage, &tool.tool_id.0, params, inputs.reference.as_deref())?;
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
        let next_bam = plan
            .io
            .outputs
            .iter()
            .find(|output| output.path.extension().is_some_and(|ext| ext == "bam"))
            .map(|output| output.path.clone());
        let next_bai = plan
            .io
            .outputs
            .iter()
            .find(|output| output.path.extension().is_some_and(|ext| ext == "bai"))
            .map(|output| output.path.clone());
        if let Some(path) = next_bam {
            bam = path;
        }
        if let Some(path) = next_bai {
            bam_index = Some(path);
        }
        stages.push(plan);
    }
    execution_graph::from_stage_plans(
        profile.id.as_str(),
        inputs.policy,
        &stages,
        "planned bam pipeline graph"
    )
}

fn enforce_stage_tool_contracts(
    stage: BamStage,
    tool_id: &str,
    params: Option<&serde_json::Value>,
    reference: Option<&std::path::Path>,
) -> Result<()> {
    match stage {
        BamStage::Authenticity if tool_id == id_catalog::TOOL_PMDTOOLS => {
            if reference.is_none() {
                return Err(anyhow!(
                    "{} with pmdtools requires reference input",
                    id_catalog::BAM_AUTHENTICITY
                ));
            }
        }
        BamStage::Contamination => {
            let effective = params
                .map(|value| stage.parse_effective_params(value))
                .transpose()?
                .and_then(|effective| match effective {
                    bijux_dna_domain_bam::params::BamEffectiveParams::Contamination(c) => Some(c),
                    _ => None,
                });
            let scope = effective
                .as_ref()
                .map(|contamination| contamination.scope)
                .unwrap_or(bijux_dna_domain_bam::params::ContaminationScope::Both);
            match tool_id {
                id_catalog::TOOL_SCHMUTZI
                    if !matches!(
                        scope,
                        bijux_dna_domain_bam::params::ContaminationScope::Mito
                            | bijux_dna_domain_bam::params::ContaminationScope::Both
                    ) =>
                {
                    return Err(anyhow!(
                        "{} tool schmutzi requires scope mito/both",
                        id_catalog::BAM_CONTAMINATION
                    ));
                }
                id_catalog::TOOL_SCHMUTZI if reference.is_none() => {
                    return Err(anyhow!(
                        "{} tool schmutzi requires mitochondrial reference input",
                        id_catalog::BAM_CONTAMINATION
                    ));
                }
                id_catalog::TOOL_VERIFYBAMID2 | id_catalog::TOOL_CONTAMMIX
                    if !matches!(
                        scope,
                        bijux_dna_domain_bam::params::ContaminationScope::Nuclear
                            | bijux_dna_domain_bam::params::ContaminationScope::Both
                    ) =>
                {
                    return Err(anyhow!(
                        "{} tool {tool_id} requires scope nuclear/both",
                        id_catalog::BAM_CONTAMINATION
                    ));
                }
                id_catalog::TOOL_VERIFYBAMID2 | id_catalog::TOOL_CONTAMMIX
                    if effective
                        .as_ref()
                        .is_some_and(|contamination| contamination.reference_panels.is_empty()) =>
                {
                    return Err(anyhow!(
                        "{} tool {tool_id} requires non-empty reference_panels",
                        id_catalog::BAM_CONTAMINATION
                    ));
                }
                _ => {}
            }
        }
        _ => {}
    }
    Ok(())
}
