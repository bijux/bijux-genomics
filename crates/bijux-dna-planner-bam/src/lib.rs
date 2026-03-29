use anyhow::{anyhow, Result};
use bijux_dna_core::contract::{ExecutionEdge, ExecutionGraph};
use bijux_dna_core::prelude::{id_catalog, StepId};
use bijux_dna_domain_bam::BamStage;
use bijux_dna_pipelines::bam::{bam_adna_capture_profile, bam_adna_shotgun_profile};
use bijux_dna_pipelines::PipelineProfile;
use bijux_dna_stage_contract::default_edges_for_stages;
use bijux_dna_stage_contract::{PlanDecisionReason, PlanReasonKind, StagePlanV1};
use serde_json::Value;

pub const PLANNER_VERSION: &str = "bijux-dna-planner-bam.v1";

mod api;
mod params;
mod report_stage;
mod selection;
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
        let edges = default_edges_for_stages(&config.stages);
        let graph = ExecutionGraph::new(
            config.pipeline_id.clone(),
            PLANNER_VERSION,
            config.policy,
            config
                .stages
                .iter()
                .map(bijux_dna_stage_contract::execution_step_from_stage_plan)
                .collect(),
            edges
                .into_iter()
                .map(|edge| {
                    ExecutionEdge::new(
                        StepId::new(edge.from().to_string()),
                        StepId::new(edge.to().to_string()),
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
        bijux_dna_domain_bam::BamStage::Markdup => {
            let bam = request.bam.ok_or_else(|| anyhow!("markdup requires bam"))?;
            let params = params::effective_params_for_stage(stage, request.params)?;
            let bijux_dna_domain_bam::params::BamEffectiveParams::Markdup(params) = params else {
                return Err(anyhow!("markdup params mismatch"));
            };
            let mut plan = tool_adapters::stages_post::markdup::plan(
                request.tool,
                bam,
                request.out_dir,
                &params,
            )?;
            plan.stage_id = bijux_dna_core::ids::StageId::new(stage.as_str().to_string());
            Ok(plan)
        }
        bijux_dna_domain_bam::BamStage::DuplicationMetrics => {
            let bam = request
                .bam
                .ok_or_else(|| anyhow!("duplication_metrics requires bam"))?;
            let params = params::effective_params_for_stage(stage, request.params)?;
            let bijux_dna_domain_bam::params::BamEffectiveParams::DuplicationMetrics(params) =
                params
            else {
                return Err(anyhow!("duplication_metrics params mismatch"));
            };
            let mut plan = tool_adapters::stages_post::duplication_metrics::plan(
                request.tool,
                bam,
                request.out_dir,
                &params,
            )?;
            plan.stage_id = bijux_dna_core::ids::StageId::new(stage.as_str().to_string());
            Ok(plan)
        }
        bijux_dna_domain_bam::BamStage::Complexity => {
            let bam = request
                .bam
                .ok_or_else(|| anyhow!("complexity requires bam"))?;
            let params = params::effective_params_for_stage(stage, request.params)?;
            let bijux_dna_domain_bam::params::BamEffectiveParams::Complexity(params) = params
            else {
                return Err(anyhow!("complexity params mismatch"));
            };
            tool_adapters::stages_post::complexity::plan(
                request.tool,
                bam,
                request.out_dir,
                &params,
            )
        }
        bijux_dna_domain_bam::BamStage::Coverage => {
            let bam = request
                .bam
                .ok_or_else(|| anyhow!("coverage requires bam"))?;
            let params = params::effective_params_for_stage(stage, request.params)?;
            let bijux_dna_domain_bam::params::BamEffectiveParams::Coverage(params) = params else {
                return Err(anyhow!("coverage params mismatch"));
            };
            tool_adapters::stages_post::coverage::plan(request.tool, bam, request.out_dir, &params)
        }
        bijux_dna_domain_bam::BamStage::InsertSize => {
            let bam = request
                .bam
                .ok_or_else(|| anyhow!("insert_size requires bam"))?;
            let params = params::effective_params_for_stage(stage, request.params)?;
            let bijux_dna_domain_bam::params::BamEffectiveParams::InsertSize(params) = params
            else {
                return Err(anyhow!("insert_size params mismatch"));
            };
            tool_adapters::stages_post::insert_size::plan(
                request.tool,
                bam,
                request.out_dir,
                &params,
            )
        }
        bijux_dna_domain_bam::BamStage::GcBias => {
            let bam = request.bam.ok_or_else(|| anyhow!("gc_bias requires bam"))?;
            let reference = request
                .reference
                .ok_or_else(|| anyhow!("gc_bias requires reference"))?;
            let params = params::effective_params_for_stage(stage, request.params)?;
            let bijux_dna_domain_bam::params::BamEffectiveParams::GcBias(params) = params else {
                return Err(anyhow!("gc_bias params mismatch"));
            };
            tool_adapters::stages_post::gc_bias::plan(
                request.tool,
                bam,
                reference,
                request.out_dir,
                &params,
            )
        }
        bijux_dna_domain_bam::BamStage::EndogenousContent => {
            let bam = request
                .bam
                .ok_or_else(|| anyhow!("endogenous_content requires bam"))?;
            let params = params::effective_params_for_stage(stage, request.params)?;
            let bijux_dna_domain_bam::params::BamEffectiveParams::EndogenousContent(params) =
                params
            else {
                return Err(anyhow!("endogenous_content params mismatch"));
            };
            let mut plan = tool_adapters::stages_post::coverage::plan(
                request.tool,
                bam,
                request.out_dir,
                &params,
            )?;
            plan.stage_id = bijux_dna_core::ids::StageId::new(stage.as_str().to_string());
            Ok(plan)
        }
        bijux_dna_domain_bam::BamStage::Recalibration => {
            let bam = request
                .bam
                .ok_or_else(|| anyhow!("recalibration requires bam"))?;
            let params = params::effective_params_for_stage(stage, request.params)?;
            let bijux_dna_domain_bam::params::BamEffectiveParams::Recalibration(params) = params
            else {
                return Err(anyhow!("recalibration params mismatch"));
            };
            tool_adapters::stages_post::recalibration::plan(
                request.tool,
                bam,
                request.out_dir,
                &params,
            )
        }
        bijux_dna_domain_bam::BamStage::Damage => {
            let bam = request.bam.ok_or_else(|| anyhow!("damage requires bam"))?;
            let params = params::effective_params_for_stage(stage, request.params)?;
            let bijux_dna_domain_bam::params::BamEffectiveParams::Damage(params) = params else {
                return Err(anyhow!("damage params mismatch"));
            };
            tool_adapters::stages_adna::damage::plan(request.tool, bam, request.out_dir, &params)
        }
        bijux_dna_domain_bam::BamStage::Authenticity => {
            let bam = request
                .bam
                .ok_or_else(|| anyhow!("authenticity requires bam"))?;
            let params = params::effective_params_for_stage(stage, request.params)?;
            let bijux_dna_domain_bam::params::BamEffectiveParams::Authenticity(params) = params
            else {
                return Err(anyhow!("authenticity params mismatch"));
            };
            tool_adapters::stages_adna::authenticity::plan(
                request.tool,
                bam,
                request.out_dir,
                &params,
            )
        }
        bijux_dna_domain_bam::BamStage::Contamination => {
            let bam = request
                .bam
                .ok_or_else(|| anyhow!("contamination requires bam"))?;
            let params = params::effective_params_for_stage(stage, request.params)?;
            let bijux_dna_domain_bam::params::BamEffectiveParams::Contamination(params) = params
            else {
                return Err(anyhow!("contamination params mismatch"));
            };
            tool_adapters::stages_adna::contamination::plan(
                request.tool,
                bam,
                request.out_dir,
                &params,
            )
        }
        bijux_dna_domain_bam::BamStage::Sex => {
            let bam = request.bam.ok_or_else(|| anyhow!("sex requires bam"))?;
            let params = params::effective_params_for_stage(stage, request.params)?;
            let bijux_dna_domain_bam::params::BamEffectiveParams::Sex(params) = params else {
                return Err(anyhow!("sex params mismatch"));
            };
            tool_adapters::stages_adna::sex::plan(request.tool, bam, request.out_dir, &params)
        }
        bijux_dna_domain_bam::BamStage::BiasMitigation => {
            #[cfg(feature = "bam_downstream")]
            {
                let bam = request
                    .bam
                    .ok_or_else(|| anyhow!("bias_mitigation requires bam"))?;
                let params = params::effective_params_for_stage(stage, request.params)?;
                let bijux_dna_domain_bam::params::BamEffectiveParams::BiasMitigation(params) =
                    params
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
        bijux_dna_domain_bam::BamStage::Haplogroups => {
            #[cfg(feature = "bam_downstream")]
            {
                let bam = request
                    .bam
                    .ok_or_else(|| anyhow!("haplogroups requires bam"))?;
                let params = params::effective_params_for_stage(stage, request.params)?;
                let bijux_dna_domain_bam::params::BamEffectiveParams::Haplogroups(params) = params
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
        bijux_dna_domain_bam::BamStage::Genotyping => {
            #[cfg(feature = "bam_downstream")]
            {
                let bam = request
                    .bam
                    .ok_or_else(|| anyhow!("genotyping requires bam"))?;
                let params = params::effective_params_for_stage(stage, request.params)?;
                let bijux_dna_domain_bam::params::BamEffectiveParams::Genotyping(params) = params
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
        bijux_dna_domain_bam::BamStage::Kinship => {
            #[cfg(feature = "bam_downstream")]
            {
                let bam = request.bam.ok_or_else(|| anyhow!("kinship requires bam"))?;
                let params = params::effective_params_for_stage(stage, request.params)?;
                let bijux_dna_domain_bam::params::BamEffectiveParams::Kinship(params) = params
                else {
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

fn stage_order_for_profile(profile: &PipelineProfile) -> Result<Vec<BamStage>> {
    profile
        .capabilities
        .required_stages
        .iter()
        .map(|stage_id| BamStage::try_from(stage_id.as_str()))
        .collect()
}

pub fn pipeline_id_catalog(profile_id: &str) -> Vec<String> {
    let profile = match profile_id {
        "bam-to-bam__default__v1" => bijux_dna_pipelines::bam::bam_default_profile(),
        "bam-to-bam__adna_shotgun__v1" => bam_adna_shotgun_profile(),
        "bam-to-bam__adna_capture__v1" => bam_adna_capture_profile(),
        _ => return Vec::new(),
    };
    stage_order_for_profile(&profile)
        .unwrap_or_default()
        .iter()
        .map(|stage| stage.as_str().to_string())
        .collect()
}

fn build_bam_plan(profile: &PipelineProfile, inputs: &BamPipelineInputs) -> Result<ExecutionGraph> {
    let mut bam = inputs.bam.clone();
    let mut bam_index = inputs.bam_index.clone();
    let mut stages = Vec::new();
    for stage in stage_order_for_profile(profile)? {
        let stage_id = stage.as_str();
        enforce_stage_status(stage_id, inputs.allow_planned)?;
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
    let edges = default_edges_for_stages(&stages);
    let graph = ExecutionGraph::new(
        profile.id.as_str(),
        PLANNER_VERSION,
        inputs.policy,
        stages
            .iter()
            .map(bijux_dna_stage_contract::execution_step_from_stage_plan)
            .collect(),
        edges
            .into_iter()
            .map(|edge| {
                ExecutionEdge::new(
                    StepId::new(edge.from().to_string()),
                    StepId::new(edge.to().to_string()),
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

fn stage_status(stage_id: &str) -> Option<String> {
    let cwd = std::env::current_dir().ok()?;
    let path = bijux_dna_infra::configs_file(&cwd, "ci/stages/stages.toml");
    let raw = std::fs::read_to_string(path).ok()?;
    let parsed = raw.parse::<toml::Value>().ok()?;
    let entries = parsed.get("stages")?.as_array()?;
    entries.iter().find_map(|entry| {
        let id = entry.get("id").and_then(toml::Value::as_str)?;
        if id == stage_id {
            entry
                .get("status")
                .and_then(toml::Value::as_str)
                .map(std::string::ToString::to_string)
        } else {
            None
        }
    })
}

fn enforce_stage_status(stage_id: &str, allow_planned: bool) -> Result<()> {
    match stage_status(stage_id).as_deref() {
        Some("supported") | None => Ok(()),
        Some("planned") | Some("out_of_scope") if allow_planned => Ok(()),
        Some("planned") | Some("out_of_scope") => Err(anyhow!(
            "stage {stage_id} is not active in current scope; re-run with --allow-planned to override"
        )),
        Some(other) => Err(anyhow!("stage {stage_id} has unknown status {other}")),
    }
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
