use std::collections::BTreeMap;
use std::path::PathBuf;

use anyhow::{anyhow, Result};
use bijux_core::plan::execution_graph::{ExecutionEdge, ExecutionGraph};
use bijux_core::plan::PlanPolicy;
use bijux_core::StageId;
use bijux_domain_bam::BamStage;
use bijux_pipelines::bam::{bam_adna_capture_profile, bam_adna_shotgun_profile};
use bijux_pipelines::PipelineProfile;
use bijux_stage_contract::default_edges_for_stages;
use bijux_stage_contract::StagePlanV1;

pub const PLANNER_VERSION: &str = "bijux-planner-bam.v1";

mod report_stage;
mod stage_registry;
pub mod tool_adapters;
mod tool_registry;

pub mod stage_api {
    pub use crate::report_stage::report_stage_step;
    pub use crate::stage_registry::stage_registry;
    pub use crate::tool_registry::allowed_tools_for_stage;
    pub use crate::{plan_stage, StagePlanRequest};
    pub use bijux_stages_bam::stage_specs::*;
}

#[derive(Debug, Clone)]
pub struct BamPlanConfig {
    pub pipeline_id: String,
    pub policy: PlanPolicy,
    pub stages: Vec<StagePlanV1>,
}

pub struct BamPlanner;

impl BamPlanner {
    /// # Errors
    /// Returns an error if the plan lint fails.
    pub fn plan(config: &BamPlanConfig) -> Result<ExecutionGraph> {
        let edges = default_edges_for_stages(&config.stages);
        let graph = ExecutionGraph::new(
            config.pipeline_id.clone(),
            PLANNER_VERSION,
            config.policy,
            config
                .stages
                .iter()
                .map(bijux_stage_contract::execution_step_from_stage_plan)
                .collect(),
            edges
                .into_iter()
                .map(|edge| {
                    ExecutionEdge::new(
                        StageId::new(edge.from().to_string()),
                        StageId::new(edge.to().to_string()),
                    )
                })
                .collect(),
        )?;
        tracing::info!(
            target: "plan.graph",
            pipeline_id = %graph.pipeline_id(),
            steps = graph.steps().len(),
            edges = graph.edges().len(),
            "planned bam execution graph"
        );
        Ok(graph)
    }
}

#[derive(Debug, Clone)]
pub struct BamPipelineInputs {
    pub policy: PlanPolicy,
    pub tool_specs: BTreeMap<String, bijux_core::contract::ToolExecutionSpecV1>,
    pub params_overrides: BTreeMap<String, serde_json::Value>,
    pub bam: PathBuf,
    pub bam_index: Option<PathBuf>,
    pub reference: Option<PathBuf>,
    pub sample_id: Option<String>,
    pub out_dir: PathBuf,
}

pub struct StagePlanRequest<'a> {
    pub stage_id: &'a str,
    pub tool: &'a bijux_core::contract::ToolExecutionSpecV1,
    pub out_dir: &'a std::path::Path,
    pub bam: Option<&'a std::path::Path>,
    pub bam_index: Option<&'a std::path::Path>,
    pub r1: Option<&'a std::path::Path>,
    pub r2: Option<&'a std::path::Path>,
    pub reference: Option<&'a std::path::Path>,
    pub sample_id: Option<&'a str>,
    pub params: Option<&'a serde_json::Value>,
}

fn effective_params_for_stage(
    stage: bijux_domain_bam::BamStage,
    params: Option<&serde_json::Value>,
) -> Result<bijux_domain_bam::params::BamEffectiveParams> {
    if let Some(value) = params {
        return stage.parse_effective_params(value);
    }
    Ok(bijux_domain_bam::stage_spec(stage).default_params.clone())
}

/// # Errors
/// Returns an error if the stage cannot be planned with the provided inputs.
#[allow(clippy::needless_pass_by_value, clippy::too_many_lines)]
pub fn plan_stage(request: StagePlanRequest<'_>) -> Result<StagePlanV1> {
    let stage = bijux_domain_bam::BamStage::try_from(request.stage_id)?;
    match stage {
        bijux_domain_bam::BamStage::Align => {
            let r1 = request.r1.ok_or_else(|| anyhow!("align requires r1"))?;
            let reference = request
                .reference
                .ok_or_else(|| anyhow!("align requires reference"))?;
            let sample_id = request
                .sample_id
                .ok_or_else(|| anyhow!("align requires sample_id"))?;
            let params = effective_params_for_stage(stage, request.params)?;
            let bijux_domain_bam::params::BamEffectiveParams::Align(params) = params else {
                return Err(anyhow!("align params mismatch"));
            };
            tool_adapters::stages_pre::align::plan(
                request.tool,
                r1,
                request.r2,
                reference,
                sample_id,
                &params,
                request.out_dir,
            )
        }
        bijux_domain_bam::BamStage::Validate => {
            let bam = request
                .bam
                .ok_or_else(|| anyhow!("validate requires bam"))?;
            tool_adapters::stages_pre::validate::plan(
                request.tool,
                bam,
                request.bam_index,
                request.reference,
                request.out_dir,
            )
        }
        bijux_domain_bam::BamStage::QcPre => {
            let bam = request.bam.ok_or_else(|| anyhow!("qc_pre requires bam"))?;
            tool_adapters::stages_pre::qc_pre::plan(request.tool, bam, request.out_dir)
        }
        bijux_domain_bam::BamStage::Filter => {
            let bam = request.bam.ok_or_else(|| anyhow!("filter requires bam"))?;
            let params = effective_params_for_stage(stage, request.params)?;
            let bijux_domain_bam::params::BamEffectiveParams::Filter(params) = params else {
                return Err(anyhow!("filter params mismatch"));
            };
            tool_adapters::stages_pre::filter::plan(request.tool, bam, request.out_dir, &params)
        }
        bijux_domain_bam::BamStage::Markdup => {
            let bam = request.bam.ok_or_else(|| anyhow!("markdup requires bam"))?;
            let params = effective_params_for_stage(stage, request.params)?;
            let bijux_domain_bam::params::BamEffectiveParams::Markdup(params) = params else {
                return Err(anyhow!("markdup params mismatch"));
            };
            tool_adapters::stages_post::markdup::plan(request.tool, bam, request.out_dir, &params)
        }
        bijux_domain_bam::BamStage::Complexity => {
            let bam = request
                .bam
                .ok_or_else(|| anyhow!("complexity requires bam"))?;
            let params = effective_params_for_stage(stage, request.params)?;
            let bijux_domain_bam::params::BamEffectiveParams::Complexity(params) = params else {
                return Err(anyhow!("complexity params mismatch"));
            };
            tool_adapters::stages_post::complexity::plan(
                request.tool,
                bam,
                request.out_dir,
                &params,
            )
        }
        bijux_domain_bam::BamStage::Coverage => {
            let bam = request
                .bam
                .ok_or_else(|| anyhow!("coverage requires bam"))?;
            let params = effective_params_for_stage(stage, request.params)?;
            let bijux_domain_bam::params::BamEffectiveParams::Coverage(params) = params else {
                return Err(anyhow!("coverage params mismatch"));
            };
            tool_adapters::stages_post::coverage::plan(request.tool, bam, request.out_dir, &params)
        }
        bijux_domain_bam::BamStage::Recalibration => {
            let bam = request
                .bam
                .ok_or_else(|| anyhow!("recalibration requires bam"))?;
            let params = effective_params_for_stage(stage, request.params)?;
            let bijux_domain_bam::params::BamEffectiveParams::Recalibration(params) = params else {
                return Err(anyhow!("recalibration params mismatch"));
            };
            tool_adapters::stages_post::recalibration::plan(
                request.tool,
                bam,
                request.out_dir,
                &params,
            )
        }
        bijux_domain_bam::BamStage::Damage => {
            let bam = request.bam.ok_or_else(|| anyhow!("damage requires bam"))?;
            let params = effective_params_for_stage(stage, request.params)?;
            let bijux_domain_bam::params::BamEffectiveParams::Damage(params) = params else {
                return Err(anyhow!("damage params mismatch"));
            };
            tool_adapters::stages_adna::damage::plan(request.tool, bam, request.out_dir, &params)
        }
        bijux_domain_bam::BamStage::Authenticity => {
            let bam = request
                .bam
                .ok_or_else(|| anyhow!("authenticity requires bam"))?;
            let params = effective_params_for_stage(stage, request.params)?;
            let bijux_domain_bam::params::BamEffectiveParams::Authenticity(params) = params else {
                return Err(anyhow!("authenticity params mismatch"));
            };
            tool_adapters::stages_adna::authenticity::plan(
                request.tool,
                bam,
                request.out_dir,
                &params,
            )
        }
        bijux_domain_bam::BamStage::Contamination => {
            let bam = request
                .bam
                .ok_or_else(|| anyhow!("contamination requires bam"))?;
            let params = effective_params_for_stage(stage, request.params)?;
            let bijux_domain_bam::params::BamEffectiveParams::Contamination(params) = params else {
                return Err(anyhow!("contamination params mismatch"));
            };
            tool_adapters::stages_adna::contamination::plan(
                request.tool,
                bam,
                request.out_dir,
                &params,
            )
        }
        bijux_domain_bam::BamStage::Sex => {
            let bam = request.bam.ok_or_else(|| anyhow!("sex requires bam"))?;
            let params = effective_params_for_stage(stage, request.params)?;
            let bijux_domain_bam::params::BamEffectiveParams::Sex(params) = params else {
                return Err(anyhow!("sex params mismatch"));
            };
            tool_adapters::stages_adna::sex::plan(request.tool, bam, request.out_dir, &params)
        }
        bijux_domain_bam::BamStage::BiasMitigation => {
            #[cfg(feature = "bam_downstream")]
            {
                let bam = request
                    .bam
                    .ok_or_else(|| anyhow!("bias_mitigation requires bam"))?;
                let params = effective_params_for_stage(stage, request.params)?;
                let bijux_domain_bam::params::BamEffectiveParams::BiasMitigation(params) = params
                else {
                    return Err(anyhow!("bias_mitigation params mismatch"));
                };
                tool_adapters::stages_downstream::bias_mitigation::plan(
                    request.tool,
                    bam,
                    request.out_dir,
                    &params,
                )
            }
            #[cfg(not(feature = "bam_downstream"))]
            {
                Err(anyhow!("bias_mitigation requires bam_downstream feature"))
            }
        }
        bijux_domain_bam::BamStage::Haplogroups => {
            #[cfg(feature = "bam_downstream")]
            {
                let bam = request
                    .bam
                    .ok_or_else(|| anyhow!("haplogroups requires bam"))?;
                let params = effective_params_for_stage(stage, request.params)?;
                let bijux_domain_bam::params::BamEffectiveParams::Haplogroups(params) = params
                else {
                    return Err(anyhow!("haplogroups params mismatch"));
                };
                tool_adapters::stages_downstream::haplogroups::plan(
                    request.tool,
                    bam,
                    request.out_dir,
                    &params,
                )
            }
            #[cfg(not(feature = "bam_downstream"))]
            {
                Err(anyhow!("haplogroups requires bam_downstream feature"))
            }
        }
        bijux_domain_bam::BamStage::Genotyping => {
            #[cfg(feature = "bam_downstream")]
            {
                let bam = request
                    .bam
                    .ok_or_else(|| anyhow!("genotyping requires bam"))?;
                let params = effective_params_for_stage(stage, request.params)?;
                let bijux_domain_bam::params::BamEffectiveParams::Genotyping(params) = params
                else {
                    return Err(anyhow!("genotyping params mismatch"));
                };
                tool_adapters::stages_downstream::genotyping::plan(
                    request.tool,
                    bam,
                    request.out_dir,
                    &params,
                )
            }
            #[cfg(not(feature = "bam_downstream"))]
            {
                Err(anyhow!("genotyping requires bam_downstream feature"))
            }
        }
        bijux_domain_bam::BamStage::Kinship => {
            #[cfg(feature = "bam_downstream")]
            {
                let bam = request.bam.ok_or_else(|| anyhow!("kinship requires bam"))?;
                let params = effective_params_for_stage(stage, request.params)?;
                let bijux_domain_bam::params::BamEffectiveParams::Kinship(params) = params else {
                    return Err(anyhow!("kinship params mismatch"));
                };
                tool_adapters::stages_downstream::kinship::plan(
                    request.tool,
                    bam,
                    request.out_dir,
                    &params,
                )
            }
            #[cfg(not(feature = "bam_downstream"))]
            {
                Err(anyhow!("kinship requires bam_downstream feature"))
            }
        }
    }
}

/// # Errors
/// Returns an error if pipeline planning fails.
#[allow(non_snake_case)]
pub fn plan_bam_to_bam__adna_shotgun__v1(inputs: &BamPipelineInputs) -> Result<ExecutionGraph> {
    let profile = bam_adna_shotgun_profile();
    build_bam_plan(&profile, inputs)
}

/// # Errors
/// Returns an error if pipeline planning fails.
#[allow(non_snake_case)]
pub fn plan_bam_to_bam__adna_capture__v1(inputs: &BamPipelineInputs) -> Result<ExecutionGraph> {
    let profile = bam_adna_capture_profile();
    build_bam_plan(&profile, inputs)
}

fn stage_order_for_profile(profile_id: &str) -> Vec<BamStage> {
    match profile_id {
        "bam-to-bam__default__v1" => vec![
            BamStage::Validate,
            BamStage::QcPre,
            BamStage::Filter,
            BamStage::Coverage,
            BamStage::Damage,
        ],
        "bam-to-bam__adna_shotgun__v1" | "bam-to-bam__adna_capture__v1" => {
            let mut stages = BamStage::all().to_vec();
            stages.retain(|stage| *stage != BamStage::Align);
            stages.retain(|stage| *stage != BamStage::Recalibration);
            if !cfg!(feature = "bam_downstream") {
                stages.retain(|stage| {
                    !matches!(
                        stage,
                        BamStage::BiasMitigation
                            | BamStage::Haplogroups
                            | BamStage::Genotyping
                            | BamStage::Kinship
                    )
                });
            }
            stages
        }
        _ => BamStage::all().to_vec(),
    }
}

pub fn pipeline_stage_ids(profile_id: &str) -> Vec<String> {
    stage_order_for_profile(profile_id)
        .iter()
        .map(|stage| stage.as_str().to_string())
        .collect()
}

fn build_bam_plan(profile: &PipelineProfile, inputs: &BamPipelineInputs) -> Result<ExecutionGraph> {
    let mut bam = inputs.bam.clone();
    let mut bam_index = inputs.bam_index.clone();
    let mut stages = Vec::new();
    for stage in stage_order_for_profile(profile.id.as_str()) {
        let stage_id = stage.as_str();
        let tool = inputs
            .tool_specs
            .get(stage_id)
            .ok_or_else(|| anyhow!("missing tool spec for stage {stage_id}"))?;
        let params = inputs
            .params_overrides
            .get(stage_id)
            .or_else(|| profile.defaults.params.get(stage_id));
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
    let edges = default_edges_for_stages(&stages);
    let graph = ExecutionGraph::new(
        profile.id.as_str(),
        PLANNER_VERSION,
        inputs.policy,
        stages
            .iter()
            .map(bijux_stage_contract::execution_step_from_stage_plan)
            .collect(),
        edges
            .into_iter()
            .map(|edge| {
                ExecutionEdge::new(
                    StageId::new(edge.from().to_string()),
                    StageId::new(edge.to().to_string()),
                )
            })
            .collect(),
    )?;
    tracing::info!(
        target: "plan.graph",
        pipeline_id = %graph.pipeline_id(),
        steps = graph.steps().len(),
        edges = graph.edges().len(),
        "planned bam pipeline graph"
    );
    Ok(graph)
}
