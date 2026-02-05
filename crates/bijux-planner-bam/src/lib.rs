use std::collections::BTreeMap;
use std::path::PathBuf;

use anyhow::{anyhow, Result};
use bijux_core::explain::PlanExplainV1;
use bijux_core::plan::execution_plan::{default_edges_for_stages, ExecutionPlan, PlanPolicy};
use bijux_core::plan::stage_plan::StagePlanV1;
use bijux_domain_bam::BamStage;
use bijux_pipelines::bam::{bam_adna_capture_profile, bam_adna_shotgun_profile};
use bijux_pipelines::PipelineProfile;
use bijux_stages_bam::StagePlanRequest;

pub const PLANNER_VERSION: &str = "bijux-planner-bam.v1";

pub mod stage_api {
    pub use bijux_domain_bam::params;
    pub use bijux_domain_bam::types;
    pub use bijux_domain_bam::*;
    pub use bijux_domain_bam::{bam_stage_completeness, stage_spec, BamStage};
    pub use bijux_stages_bam::bam_tools_registry::allowed_tools_for_stage;
    pub use bijux_stages_bam::plan_stage;
    pub use bijux_stages_bam::StagePlanRequest;
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
    pub fn plan(config: &BamPlanConfig) -> Result<ExecutionPlan> {
        let edges = default_edges_for_stages(&config.stages);
        ExecutionPlan::new(
            config.pipeline_id.clone(),
            PLANNER_VERSION,
            config.policy,
            config.stages.clone(),
            edges,
        )
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

/// # Errors
/// Returns an error if pipeline planning fails.
#[allow(non_snake_case)]
pub fn plan_bam_to_bam__adna_shotgun__v1(inputs: &BamPipelineInputs) -> Result<ExecutionPlan> {
    let profile = bam_adna_shotgun_profile();
    build_bam_plan(&profile, inputs)
}

/// # Errors
/// Returns an error if pipeline planning fails.
#[allow(non_snake_case)]
pub fn plan_bam_to_bam__adna_capture__v1(inputs: &BamPipelineInputs) -> Result<ExecutionPlan> {
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
                        BamStage::Haplogroups | BamStage::Genotyping | BamStage::Kinship
                    )
                });
            }
            stages
        }
        _ => BamStage::all().to_vec(),
    }
}

#[must_use]
pub fn pipeline_stage_ids(profile_id: &str) -> Vec<String> {
    stage_order_for_profile(profile_id)
        .iter()
        .map(|stage| stage.as_str().to_string())
        .collect()
}

fn build_bam_plan(profile: &PipelineProfile, inputs: &BamPipelineInputs) -> Result<ExecutionPlan> {
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
        let plan = bijux_stages_bam::plan_stage(StagePlanRequest {
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
    ExecutionPlan::new(
        profile.id.as_str(),
        PLANNER_VERSION,
        inputs.policy,
        stages,
        edges,
    )
}

#[must_use]
pub fn explain_plan(plan: &ExecutionPlan) -> PlanExplainV1 {
    PlanExplainV1::from_plan(plan)
}
