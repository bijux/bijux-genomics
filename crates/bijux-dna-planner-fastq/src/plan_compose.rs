use std::collections::BTreeMap;
use std::path::PathBuf;

use anyhow::{anyhow, Result};
use bijux_dna_core::prelude::{
    ArtifactRef, ContainerImageRefV1, StageId, StepId, ToolExecutionSpecV1,
};
use bijux_dna_domain_fastq::params::{qc_post::QcAggregationScope, PairedMode};
use bijux_dna_domain_fastq::stages::ids::{
    STAGE_INDEX_REFERENCE, STAGE_PROFILE_OVERREPRESENTED_SEQUENCES, STAGE_PROFILE_READ_LENGTHS,
    STAGE_TRIM_POLYG_TAILS,
};
use bijux_dna_stage_contract::{PlanDecisionReason, PlanReasonKind, StagePlanV1};

use crate::{
    DepleteHostStageParams, DepleteReferenceContaminantsStageParams, DepleteRrnaStageParams,
    FastqStageBinding, FastqStageParameters, TrimTerminalDamageStageParams, STAGE_CLUSTER_OTUS,
    STAGE_CORRECT_ERRORS, STAGE_DEPLETE_HOST, STAGE_DEPLETE_REFERENCE_CONTAMINANTS,
    STAGE_DEPLETE_RRNA, STAGE_DETECT_ADAPTERS, STAGE_EXTRACT_UMIS, STAGE_FILTER_LOW_COMPLEXITY,
    STAGE_FILTER_READS, STAGE_INFER_ASVS, STAGE_MERGE_PAIRS, STAGE_NORMALIZE_ABUNDANCE,
    STAGE_NORMALIZE_PRIMERS, STAGE_PROFILE_READS, STAGE_REMOVE_CHIMERAS, STAGE_REMOVE_DUPLICATES,
    STAGE_REPORT_QC, STAGE_SCREEN_TAXONOMY, STAGE_TRIM_READS, STAGE_TRIM_TERMINAL_DAMAGE,
    STAGE_VALIDATE_READS,
};

#[derive(Debug, Clone)]
struct ReferenceIndexState {
    path: PathBuf,
    tool_id: String,
}

#[allow(dead_code, clippy::too_many_arguments, clippy::too_many_lines)]
pub fn compose_fastq_pipeline_steps<F>(
    stages: &[String],
    tools: &[ToolExecutionSpecV1],
    aux_images: &BTreeMap<String, ContainerImageRefV1>,
    tool_reasons: Option<&[PlanDecisionReason]>,
    adapter_bank: Option<&serde_json::Value>,
    polyx_bank: Option<&serde_json::Value>,
    contaminant_bank: Option<&serde_json::Value>,
    enable_contaminant_removal: bool,
    r1: &std::path::Path,
    r2: Option<&std::path::Path>,
    reference_fasta: Option<&std::path::Path>,
    mut out_dir_for_stage: F,
) -> Result<Vec<StagePlanV1>>
where
    F: FnMut(
        &str,
        &ToolExecutionSpecV1,
        &std::path::Path,
        Option<&std::path::Path>,
    ) -> Result<PathBuf>,
{
    let stage_bindings = stages
        .iter()
        .zip(tools.iter())
        .enumerate()
        .map(|(idx, (stage_id, tool))| FastqStageBinding {
            stage_id: stage_id.clone(),
            stage_instance_id: None,
            tool: tool.clone(),
            reason: tool_reasons.and_then(|reasons| reasons.get(idx).cloned()),
            params: None,
        })
        .collect::<Vec<_>>();
    compose_fastq_stage_bindings(
        &stage_bindings,
        aux_images,
        adapter_bank,
        polyx_bank,
        contaminant_bank,
        enable_contaminant_removal,
        r1,
        r2,
        reference_fasta,
        |binding, current_r1, current_r2| {
            out_dir_for_stage(&binding.stage_id, &binding.tool, current_r1, current_r2)
        },
    )
}

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
pub fn compose_fastq_stage_bindings<F>(
    stage_bindings: &[FastqStageBinding],
    aux_images: &BTreeMap<String, ContainerImageRefV1>,
    adapter_bank: Option<&serde_json::Value>,
    polyx_bank: Option<&serde_json::Value>,
    contaminant_bank: Option<&serde_json::Value>,
    enable_contaminant_removal: bool,
    r1: &std::path::Path,
    r2: Option<&std::path::Path>,
    reference_fasta: Option<&std::path::Path>,
    mut out_dir_for_stage: F,
) -> Result<Vec<StagePlanV1>>
where
    F: FnMut(&FastqStageBinding, &std::path::Path, Option<&std::path::Path>) -> Result<PathBuf>,
{
    let mut current_r1 = r1.to_path_buf();
    let raw_r1 = r1.to_path_buf();
    let mut current_r2 = r2.map(|path| path.to_path_buf());
    let raw_r2 = r2.map(|path| path.to_path_buf());
    let mut current_feature_table: Option<PathBuf> = None;
    let mut current_reference_index: Option<ReferenceIndexState> = None;
    let mut current_qc_inputs: Vec<ArtifactRef> = Vec::new();
    let mut plans = Vec::new();
    for binding in stage_bindings {
        let out_dir = out_dir_for_stage(binding, &current_r1, current_r2.as_deref())?;
        let stage_id = binding.stage_id.as_str();
        let tool = &binding.tool;
        let (plan, next_r1, next_r2, next_feature_table) = match stage_id {
            stage if stage == STAGE_DETECT_ADAPTERS.as_str() => {
                let plan = crate::tool_adapters::fastq::detect_adapters::plan(
                    tool,
                    &current_r1,
                    current_r2.as_deref(),
                    &out_dir,
                )?;
                (
                    plan,
                    current_r1.clone(),
                    current_r2.clone(),
                    current_feature_table.clone(),
                )
            }
            stage if stage == STAGE_PROFILE_READ_LENGTHS.as_str() => {
                let plan = crate::tool_adapters::fastq::profile_read_lengths::plan(
                    tool,
                    &current_r1,
                    current_r2.as_deref(),
                    &out_dir,
                )?;
                (
                    plan,
                    current_r1.clone(),
                    current_r2.clone(),
                    current_feature_table.clone(),
                )
            }
            stage if stage == STAGE_PROFILE_OVERREPRESENTED_SEQUENCES.as_str() => {
                let plan = crate::tool_adapters::fastq::profile_overrepresented_sequences::plan(
                    tool,
                    &current_r1,
                    current_r2.as_deref(),
                    &out_dir,
                )?;
                (
                    plan,
                    current_r1.clone(),
                    current_r2.clone(),
                    current_feature_table.clone(),
                )
            }
            stage if stage == STAGE_INDEX_REFERENCE.as_str() => {
                let reference_fasta = reference_fasta
                    .ok_or_else(|| anyhow!("reference indexing requires reference_fasta input"))?;
                let plan = crate::tool_adapters::fastq::index_reference::plan(
                    tool,
                    reference_fasta,
                    &out_dir,
                )?;
                (
                    plan,
                    current_r1.clone(),
                    current_r2.clone(),
                    current_feature_table.clone(),
                )
            }
            stage if stage == STAGE_TRIM_READS.as_str() => {
                let plan = crate::tool_adapters::fastq::trim_reads::plan(
                    tool,
                    &current_r1,
                    current_r2.as_deref(),
                    &out_dir,
                    adapter_bank,
                    polyx_bank,
                    contaminant_bank,
                )?;
                let next_r1 = plan.io.outputs[0].path.clone();
                let next_r2 = if current_r2.is_some() {
                    Some(plan.io.outputs[1].path.clone())
                } else {
                    None
                };
                (plan, next_r1, next_r2, current_feature_table.clone())
            }
            stage if stage == STAGE_TRIM_TERMINAL_DAMAGE.as_str() => {
                let params = trim_terminal_damage_params(binding);
                let plan = crate::tool_adapters::fastq::trim_terminal_damage::plan_trim_terminal_damage_with_options(
                    tool,
                    &current_r1,
                    current_r2.as_deref(),
                    &out_dir,
                    &params,
                )?;
                let next_r1 = plan.io.outputs[0].path.clone();
                let next_r2 = if current_r2.is_some() {
                    Some(plan.io.outputs[1].path.clone())
                } else {
                    None
                };
                (plan, next_r1, next_r2, current_feature_table.clone())
            }
            stage if stage == STAGE_FILTER_READS.as_str() => {
                let mut filter_options =
                    crate::tool_adapters::fastq::filter_reads::FilterPlanOptions::default();
                if adapter_bank.is_some() {
                    filter_options.redundant_filters.push("adapter".to_string());
                }
                if polyx_bank.is_some() {
                    filter_options.redundant_filters.push("polyx".to_string());
                }
                if enable_contaminant_removal && contaminant_bank.is_some() {
                    filter_options.kmer_ref =
                        crate::tool_adapters::fastq::filter_reads::default_kmer_ref();
                }
                let plan = crate::tool_adapters::fastq::filter_reads::plan_filter(
                    tool,
                    &current_r1,
                    current_r2.as_deref(),
                    &out_dir,
                    &filter_options,
                )?;
                let next_r1 = plan.io.outputs[0].path.clone();
                let next_r2 = if current_r2.is_some() {
                    Some(plan.io.outputs[1].path.clone())
                } else {
                    None
                };
                (plan, next_r1, next_r2, current_feature_table.clone())
            }
            stage if stage == STAGE_REMOVE_DUPLICATES.as_str() => {
                let plan = crate::tool_adapters::fastq::remove_duplicates::plan_deduplicate(
                    tool,
                    &current_r1,
                    current_r2.as_deref(),
                    &out_dir,
                )?;
                let next_r1 = plan.io.outputs[0].path.clone();
                let next_r2 = if current_r2.is_some() {
                    Some(plan.io.outputs[1].path.clone())
                } else {
                    None
                };
                (plan, next_r1, next_r2, current_feature_table.clone())
            }
            stage if stage == STAGE_DEPLETE_HOST.as_str() => {
                let reference_index = current_reference_index.as_ref().ok_or_else(|| {
                    anyhow!("host depletion requires a prior reference index stage")
                })?;
                ensure_reference_index_backend(
                    STAGE_DEPLETE_HOST.as_str(),
                    tool.tool_id.as_str(),
                    &reference_index.tool_id,
                )?;
                let params = deplete_host_params(binding);
                let plan =
                    crate::tool_adapters::fastq::deplete_host::plan_host_depletion_with_options(
                        tool,
                        &current_r1,
                        current_r2.as_deref(),
                        &reference_index.path,
                        &out_dir,
                        &params,
                    )?;
                let next_r1 = plan.io.outputs[0].path.clone();
                let next_r2 = if current_r2.is_some() {
                    Some(plan.io.outputs[1].path.clone())
                } else {
                    None
                };
                (plan, next_r1, next_r2, current_feature_table.clone())
            }
            stage if stage == STAGE_DEPLETE_REFERENCE_CONTAMINANTS.as_str() => {
                let reference_index = current_reference_index.as_ref().ok_or_else(|| {
                    anyhow!(
                        "reference contaminant depletion requires a prior reference index stage"
                    )
                })?;
                ensure_reference_index_backend(
                    STAGE_DEPLETE_REFERENCE_CONTAMINANTS.as_str(),
                    tool.tool_id.as_str(),
                    &reference_index.tool_id,
                )?;
                let params = deplete_reference_contaminants_params(binding);
                let plan = crate::tool_adapters::fastq::deplete_reference_contaminants::plan_contaminant_screen_with_options(
                    tool,
                    &current_r1,
                    current_r2.as_deref(),
                    &reference_index.path,
                    &out_dir,
                    &params,
                )?;
                let next_r1 = plan.io.outputs[0].path.clone();
                let next_r2 = if current_r2.is_some() {
                    Some(plan.io.outputs[1].path.clone())
                } else {
                    None
                };
                (plan, next_r1, next_r2, current_feature_table.clone())
            }
            stage if stage == STAGE_FILTER_LOW_COMPLEXITY.as_str() => {
                let low_complexity_options =
                    crate::tool_adapters::fastq::filter_low_complexity::LowComplexityPlanOptions::default();
                let plan = crate::tool_adapters::fastq::filter_low_complexity::plan_low_complexity(
                    tool,
                    &current_r1,
                    current_r2.as_deref(),
                    &out_dir,
                    &low_complexity_options,
                )?;
                let next_r1 = plan.io.outputs[0].path.clone();
                let next_r2 = if current_r2.is_some() {
                    Some(plan.io.outputs[1].path.clone())
                } else {
                    None
                };
                (plan, next_r1, next_r2, current_feature_table.clone())
            }
            stage if stage == STAGE_TRIM_POLYG_TAILS.as_str() => {
                let plan = crate::tool_adapters::fastq::trim_polyg_tails::plan_trim_polyg_tails(
                    tool,
                    &current_r1,
                    current_r2.as_deref(),
                    &out_dir,
                )?;
                let next_r1 = plan.io.outputs[0].path.clone();
                let next_r2 = if current_r2.is_some() {
                    Some(plan.io.outputs[1].path.clone())
                } else {
                    None
                };
                (plan, next_r1, next_r2, current_feature_table.clone())
            }
            stage if stage == STAGE_VALIDATE_READS.as_str() => {
                if crate::tool_adapters::fastq::validate_reads::normalize_validate_tool_list(&[
                    tool.tool_id.as_str().to_string(),
                ])
                .is_err()
                {
                    return Err(anyhow!(
                        "{} requires a supported validator backend; got {}",
                        STAGE_VALIDATE_READS.as_str(),
                        tool.tool_id
                    ));
                }
                let plan = crate::tool_adapters::fastq::validate_reads::plan(
                    tool,
                    &current_r1,
                    current_r2.as_deref(),
                    &out_dir,
                )?;
                (
                    plan,
                    current_r1.clone(),
                    current_r2.clone(),
                    current_feature_table.clone(),
                )
            }
            stage if stage == STAGE_MERGE_PAIRS.as_str() => {
                let r2 = current_r2
                    .as_ref()
                    .ok_or_else(|| anyhow!("merge requires r2"))?;
                let plan = crate::tool_adapters::fastq::merge_pairs::plan_merge(
                    tool,
                    &current_r1,
                    r2,
                    &out_dir,
                )?;
                let next_r1 = plan.io.outputs[0].path.clone();
                (plan, next_r1, None, current_feature_table.clone())
            }
            stage if stage == STAGE_CORRECT_ERRORS.as_str() => {
                let r2 = current_r2
                    .as_ref()
                    .ok_or_else(|| anyhow!("correct requires r2"))?;
                let plan = crate::tool_adapters::fastq::correct_errors::plan_correct(
                    tool,
                    &current_r1,
                    r2,
                    &out_dir,
                )?;
                let next_r1 = plan.io.outputs[0].path.clone();
                let next_r2 = plan.io.outputs[1].path.clone();
                (plan, next_r1, Some(next_r2), current_feature_table.clone())
            }
            stage if stage == STAGE_EXTRACT_UMIS.as_str() => {
                let r2 = current_r2
                    .as_ref()
                    .ok_or_else(|| anyhow!("umi requires r2"))?;
                let plan = crate::tool_adapters::fastq::extract_umis::plan_umi(
                    tool,
                    &current_r1,
                    r2,
                    &out_dir,
                    None,
                )?;
                let next_r1 = plan.io.outputs[0].path.clone();
                let next_r2 = plan.io.outputs[1].path.clone();
                (plan, next_r1, Some(next_r2), current_feature_table.clone())
            }
            stage if stage == STAGE_REPORT_QC.as_str() => {
                let mut stage_aux_images = std::collections::BTreeMap::new();
                if tool.tool_id.0 == "multiqc" {
                    for aux_tool in crate::tool_adapters::fastq::report_qc::aux_tool_ids() {
                        if let Some(image) = aux_images.get(*aux_tool) {
                            stage_aux_images.insert(aux_tool.to_string(), image.clone());
                        }
                    }
                }
                let paired_mode = if current_r2.is_some() {
                    PairedMode::PairedEnd
                } else {
                    PairedMode::SingleEnd
                };
                if current_qc_inputs.is_empty() {
                    return Err(anyhow!(
                        "fastq.report_qc requires governed upstream QC artifacts; add contributing QC stages before report aggregation"
                    ));
                }
                let plan = crate::tool_adapters::fastq::report_qc::plan_qc_post_with_qc_inputs(
                    tool,
                    &current_qc_inputs,
                    &out_dir,
                    stage_aux_images,
                    paired_mode,
                    QcAggregationScope::GovernedQcArtifacts,
                    Some(raw_r1.as_path()),
                    raw_r2.as_deref(),
                )?;
                (
                    plan,
                    current_r1.clone(),
                    current_r2.clone(),
                    current_feature_table.clone(),
                )
            }
            stage if stage == STAGE_DEPLETE_RRNA.as_str() => {
                let params = deplete_rrna_params(binding);
                let plan = crate::tool_adapters::fastq::deplete_rrna::plan_rrna_with_options(
                    tool,
                    &current_r1,
                    current_r2.as_deref(),
                    &out_dir,
                    &params,
                )?;
                let next_r1 = plan.io.outputs[0].path.clone();
                let next_r2 = if current_r2.is_some() {
                    Some(plan.io.outputs[1].path.clone())
                } else {
                    None
                };
                (plan, next_r1, next_r2, current_feature_table.clone())
            }
            stage if stage == STAGE_SCREEN_TAXONOMY.as_str() => {
                let plan = crate::tool_adapters::fastq::screen_taxonomy::plan_screen(
                    tool,
                    &current_r1,
                    current_r2.as_deref(),
                    &out_dir,
                )?;
                (
                    plan,
                    current_r1.clone(),
                    current_r2.clone(),
                    current_feature_table.clone(),
                )
            }
            stage if stage == STAGE_PROFILE_READS.as_str() => {
                let plan = crate::tool_adapters::fastq::profile_reads::plan_stats_neutral(
                    tool,
                    &current_r1,
                    current_r2.as_deref(),
                    &out_dir,
                )?;
                (
                    plan,
                    current_r1.clone(),
                    current_r2.clone(),
                    current_feature_table.clone(),
                )
            }
            stage if stage == STAGE_NORMALIZE_PRIMERS.as_str() => {
                if !matches!(tool.tool_id.as_str(), "cutadapt" | "seqkit") {
                    return Err(anyhow!(
                        "{} requires cutadapt/seqkit; got {}",
                        STAGE_NORMALIZE_PRIMERS.as_str(),
                        tool.tool_id
                    ));
                }
                let plan = crate::tool_adapters::fastq::normalize_primers::plan(
                    tool,
                    &current_r1,
                    current_r2.as_deref(),
                    &out_dir,
                )?;
                let next_r1 = plan.io.outputs[0].path.clone();
                let next_r2 = if current_r2.is_some() {
                    Some(plan.io.outputs[1].path.clone())
                } else {
                    None
                };
                (plan, next_r1, next_r2, current_feature_table.clone())
            }
            stage if stage == STAGE_REMOVE_CHIMERAS.as_str() => {
                if tool.tool_id.as_str() != "vsearch" {
                    return Err(anyhow!(
                        "{} requires vsearch; got {}",
                        STAGE_REMOVE_CHIMERAS.as_str(),
                        tool.tool_id
                    ));
                }
                let plan = crate::tool_adapters::fastq::remove_chimeras::plan(
                    tool,
                    &current_r1,
                    current_r2.as_deref(),
                    &out_dir,
                )?;
                let next_r1 = plan.io.outputs[0].path.clone();
                (plan, next_r1, None, current_feature_table.clone())
            }
            stage if stage == STAGE_INFER_ASVS.as_str() => {
                if tool.tool_id.as_str() != "dada2" {
                    return Err(anyhow!(
                        "{} requires dada2 tool binding; got {}",
                        STAGE_INFER_ASVS.as_str(),
                        tool.tool_id
                    ));
                }
                let plan = crate::tool_adapters::fastq::infer_asvs::plan(
                    tool,
                    &current_r1,
                    current_r2.as_deref(),
                    &out_dir,
                )?;
                let next_feature_table = Some(plan.io.outputs[0].path.clone());
                (
                    plan,
                    current_r1.clone(),
                    current_r2.clone(),
                    next_feature_table,
                )
            }
            stage if stage == STAGE_CLUSTER_OTUS.as_str() => {
                if tool.tool_id.as_str() != "vsearch" {
                    return Err(anyhow!(
                        "{} requires vsearch; got {}",
                        STAGE_CLUSTER_OTUS.as_str(),
                        tool.tool_id
                    ));
                }
                let plan = crate::tool_adapters::fastq::cluster_otus::plan(
                    tool,
                    &current_r1,
                    current_r2.as_deref(),
                    &out_dir,
                )?;
                let next_feature_table = Some(plan.io.outputs[0].path.clone());
                (
                    plan,
                    current_r1.clone(),
                    current_r2.clone(),
                    next_feature_table,
                )
            }
            stage if stage == STAGE_NORMALIZE_ABUNDANCE.as_str() => {
                ensure_normalize_abundance_tool(tool.tool_id.as_str())?;
                let abundance_table = current_feature_table.as_ref().ok_or_else(|| {
                    anyhow!("fastq.normalize_abundance requires an upstream feature table")
                })?;
                let plan = crate::tool_adapters::fastq::normalize_abundance::plan(
                    tool,
                    abundance_table,
                    &out_dir,
                )?;
                let next_feature_table = Some(plan.io.outputs[0].path.clone());
                (
                    plan,
                    current_r1.clone(),
                    current_r2.clone(),
                    next_feature_table,
                )
            }
            _ => {
                return Err(anyhow!(
                    "unsupported stage in fastq pipeline: {}",
                    binding.stage_id
                ));
            }
        };
        let mut plan = plan;
        if let Some(reason) = binding.reason.as_ref() {
            plan.reason = reason.clone();
        } else {
            plan.reason = PlanDecisionReason::new(
                PlanReasonKind::Default,
                format!("tool {} selected by planner", plan.tool_id.0),
            );
        }
        if let Some(stage_instance_id) = binding.stage_instance_id.as_ref() {
            plan.stage_instance_id = Some(StepId::new(stage_instance_id.clone()));
        }
        current_qc_inputs.extend(qc_input_artifacts_for_stage(stage_id, &plan));
        plans.push(plan);
        if stage_id == STAGE_INDEX_REFERENCE.as_str() {
            let plan = plans.last().expect("stage just pushed");
            current_reference_index = Some(ReferenceIndexState {
                path: plan.io.outputs[0].path.clone(),
                tool_id: plan.tool_id.to_string(),
            });
        }
        current_r1 = next_r1;
        current_r2 = next_r2;
        current_feature_table = next_feature_table;
    }
    Ok(plans)
}

fn qc_input_artifacts_for_stage(stage_id: &str, plan: &StagePlanV1) -> Vec<ArtifactRef> {
    if stage_id == STAGE_REPORT_QC.as_str() {
        return Vec::new();
    }
    plan.io
        .outputs
        .iter()
        .filter(|artifact| is_governed_qc_artifact_role(artifact.role))
        .cloned()
        .collect()
}

fn is_governed_qc_artifact_role(role: bijux_dna_core::prelude::ArtifactRole) -> bool {
    matches!(
        role,
        bijux_dna_core::prelude::ArtifactRole::ReportJson
            | bijux_dna_core::prelude::ArtifactRole::MetricsJson
            | bijux_dna_core::prelude::ArtifactRole::MetricsEnvelope
            | bijux_dna_core::prelude::ArtifactRole::StageReport
            | bijux_dna_core::prelude::ArtifactRole::SummaryJson
            | bijux_dna_core::prelude::ArtifactRole::SummaryTsv
    )
}

fn trim_terminal_damage_params(binding: &FastqStageBinding) -> TrimTerminalDamageStageParams {
    match binding.params.as_ref() {
        Some(FastqStageParameters::TrimTerminalDamage(params)) => params.clone(),
        _ => TrimTerminalDamageStageParams::default(),
    }
}

fn deplete_rrna_params(binding: &FastqStageBinding) -> DepleteRrnaStageParams {
    match binding.params.as_ref() {
        Some(FastqStageParameters::DepleteRrna(params)) => params.clone(),
        _ => DepleteRrnaStageParams::default(),
    }
}

fn deplete_host_params(binding: &FastqStageBinding) -> DepleteHostStageParams {
    match binding.params.as_ref() {
        Some(FastqStageParameters::DepleteHost(params)) => params.clone(),
        _ => DepleteHostStageParams::default(),
    }
}

fn deplete_reference_contaminants_params(
    binding: &FastqStageBinding,
) -> DepleteReferenceContaminantsStageParams {
    match binding.params.as_ref() {
        Some(FastqStageParameters::DepleteReferenceContaminants(params)) => params.clone(),
        _ => DepleteReferenceContaminantsStageParams::default(),
    }
}

fn ensure_reference_index_backend(
    stage_id: &str,
    depletion_tool_id: &str,
    index_tool_id: &str,
) -> Result<()> {
    let depletion_tool_id = bijux_dna_core::ids::ToolId::new(depletion_tool_id.to_string());
    let index_tool_id = bijux_dna_core::ids::ToolId::new(index_tool_id.to_string());
    let compatible_backends =
        bijux_dna_domain_fastq::reference_index_backends_for_tool(&depletion_tool_id);
    if compatible_backends.is_empty() {
        return Err(anyhow!(
            "unsupported reference-aware depletion backend for {stage_id}: {}",
            depletion_tool_id.as_str()
        ));
    }
    if bijux_dna_domain_fastq::is_reference_index_backend_compatible(
        &depletion_tool_id,
        &index_tool_id,
    ) {
        return Ok(());
    }
    Err(anyhow!(
        "{stage_id} requires one of [{}] as reference index backend, but upstream fastq.index_reference used {}",
        compatible_backends
            .iter()
            .map(|tool_id| tool_id.as_str().to_string())
            .collect::<Vec<_>>()
            .join(", "),
        index_tool_id.as_str()
    ))
}

pub(crate) fn ensure_normalize_abundance_tool(tool_id: &str) -> Result<()> {
    if tool_id == "seqkit" {
        return Ok(());
    }
    Err(anyhow!(
        "{} requires seqkit; got {}",
        STAGE_NORMALIZE_ABUNDANCE.as_str(),
        tool_id
    ))
}

#[allow(dead_code)]
fn plan_fastq_transform_stage(
    stage_id: &str,
    tool: &ToolExecutionSpecV1,
    input: &std::path::Path,
    out_dir: &std::path::Path,
    output_name: &str,
    effective_params: serde_json::Value,
) -> StagePlanV1 {
    StagePlanV1 {
        stage_id: StageId::new(stage_id),
        stage_instance_id: None,
        stage_version: bijux_dna_core::prelude::StageVersion(1),
        tool_id: tool.tool_id.clone(),
        tool_version: tool.tool_version.clone(),
        image: tool.image.clone(),
        command: bijux_dna_core::prelude::CommandSpecV1 {
            template: tool.command.template.to_vec(),
        },
        resources: tool.resources.clone(),
        io: bijux_dna_stage_contract::StageIO {
            inputs: vec![bijux_dna_stage_contract::ArtifactRef::required(
                bijux_dna_core::prelude::ArtifactId::from_static("reads"),
                input.to_path_buf(),
                bijux_dna_core::prelude::ArtifactRole::Reads,
            )],
            outputs: vec![bijux_dna_stage_contract::ArtifactRef::required(
                bijux_dna_core::prelude::ArtifactId::from_static("trimmed_reads"),
                out_dir.join(output_name),
                bijux_dna_core::prelude::ArtifactRole::Reads,
            )],
        },
        out_dir: out_dir.to_path_buf(),
        params: serde_json::json!({}),
        effective_params,
        aux_images: std::collections::BTreeMap::new(),
        reason: PlanDecisionReason::new(
            PlanReasonKind::Fallback,
            "fastq transform stage contract default".to_string(),
        ),
    }
}
