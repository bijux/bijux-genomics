use std::collections::BTreeMap;
use std::path::PathBuf;

use anyhow::{anyhow, Result};
use bijux_dna_core::prelude::{ContainerImageRefV1, StepId};
use bijux_dna_domain_fastq::params::PairedMode;
use bijux_dna_domain_fastq::stages::ids::{
    STAGE_DETECT_DUPLICATES_PREMERGE, STAGE_ESTIMATE_LIBRARY_COMPLEXITY_PREALIGN,
    STAGE_INDEX_REFERENCE, STAGE_PROFILE_OVERREPRESENTED_SEQUENCES, STAGE_PROFILE_READ_LENGTHS,
    STAGE_TRIM_POLYG_TAILS,
};
use bijux_dna_stage_contract::{PlanDecisionReason, PlanReasonKind, StagePlanV1};

use crate::{
    FastqStageBinding, STAGE_CLUSTER_OTUS, STAGE_CORRECT_ERRORS, STAGE_DEPLETE_HOST,
    STAGE_DEPLETE_REFERENCE_CONTAMINANTS, STAGE_DEPLETE_RRNA, STAGE_DETECT_ADAPTERS,
    STAGE_EXTRACT_UMIS, STAGE_FILTER_LOW_COMPLEXITY, STAGE_FILTER_READS, STAGE_INFER_ASVS,
    STAGE_MERGE_PAIRS, STAGE_NORMALIZE_ABUNDANCE, STAGE_NORMALIZE_PRIMERS, STAGE_PROFILE_READS,
    STAGE_REMOVE_CHIMERAS, STAGE_REMOVE_DUPLICATES, STAGE_REPORT_QC, STAGE_SCREEN_TAXONOMY,
    STAGE_TRIM_READS, STAGE_TRIM_TERMINAL_DAMAGE, STAGE_VALIDATE_READS,
};

mod input_resolution;
mod lineage;
mod models;
mod qc_inputs;
mod stage_params;

use input_resolution::{
    explicit_abundance_table, explicit_reads_input_path, explicit_reference_index_state,
    explicit_report_qc_inputs, has_explicit_input, resolved_stage_input_artifacts,
    stage_node_id_for_binding,
};
use lineage::{
    inherited_lineage, lineage_input_artifacts_for_stage, merge_lineage_input_artifacts,
};
use models::{PlannedStageLineage, ReferenceIndexState};
pub use models::{
    StageArtifactInputBinding, StageArtifactInputPolicy, StageDependencyPolicy,
    SyntheticStageArtifact, SyntheticStageArtifactPolicy,
};
use qc_inputs::qc_input_artifacts_for_stage;
use stage_params::{
    cluster_otus_params, correct_errors_params, deplete_host_params,
    deplete_reference_contaminants_params, deplete_rrna_params, detect_adapters_params,
    extract_umis_params, filter_low_complexity_plan_options, filter_reads_plan_options,
    index_reference_params, infer_asvs_params, merge_pairs_plan_options,
    normalize_abundance_plan_options, normalize_primers_plan_options,
    profile_overrepresented_params, profile_read_lengths_params, profile_reads_params,
    remove_chimeras_params, remove_duplicates_params, report_qc_params, screen_params,
    trim_polyg_options, trim_reads_options, trim_terminal_damage_params, validate_reads_params,
};

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
    explicit_stage_inputs: Option<&StageArtifactInputPolicy>,
    out_dir_for_stage: F,
) -> Result<Vec<StagePlanV1>>
where
    F: FnMut(&FastqStageBinding, &std::path::Path, Option<&std::path::Path>) -> Result<PathBuf>,
{
    compose_fastq_stage_bindings_with_dependencies(
        stage_bindings,
        aux_images,
        adapter_bank,
        polyx_bank,
        contaminant_bank,
        enable_contaminant_removal,
        r1,
        r2,
        reference_fasta,
        explicit_stage_inputs,
        None,
        None,
        out_dir_for_stage,
    )
}

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
pub fn compose_fastq_stage_bindings_with_dependencies<F>(
    stage_bindings: &[FastqStageBinding],
    aux_images: &BTreeMap<String, ContainerImageRefV1>,
    adapter_bank: Option<&serde_json::Value>,
    polyx_bank: Option<&serde_json::Value>,
    contaminant_bank: Option<&serde_json::Value>,
    enable_contaminant_removal: bool,
    r1: &std::path::Path,
    r2: Option<&std::path::Path>,
    reference_fasta: Option<&std::path::Path>,
    explicit_stage_inputs: Option<&StageArtifactInputPolicy>,
    synthetic_stage_artifacts: Option<&SyntheticStageArtifactPolicy>,
    stage_dependencies: Option<&StageDependencyPolicy>,
    mut out_dir_for_stage: F,
) -> Result<Vec<StagePlanV1>>
where
    F: FnMut(&FastqStageBinding, &std::path::Path, Option<&std::path::Path>) -> Result<PathBuf>,
{
    let raw_r1 = r1.to_path_buf();
    let raw_r2 = r2.map(std::path::Path::to_path_buf);
    let mut plans = Vec::new();
    let mut lineage_by_node_id = BTreeMap::<String, PlannedStageLineage>::new();
    let mut latest_lineage_node_id = None::<String>;
    for binding in stage_bindings {
        let resolved_inputs = resolved_stage_input_artifacts(
            binding,
            explicit_stage_inputs,
            synthetic_stage_artifacts,
            &plans,
        )?;
        let inherited = inherited_lineage(
            binding,
            stage_dependencies,
            &lineage_by_node_id,
            latest_lineage_node_id.as_deref(),
            &raw_r1,
            raw_r2.as_deref(),
        )?;
        let stage_r1 = explicit_reads_input_path(&resolved_inputs, "reads_r1")?
            .unwrap_or_else(|| inherited.reads_r1.clone());
        let stage_r2 = explicit_reads_input_path(&resolved_inputs, "reads_r2")?.or_else(|| {
            if has_explicit_input(&resolved_inputs, "reads_r1") {
                None
            } else {
                inherited.reads_r2.clone()
            }
        });
        let out_dir = out_dir_for_stage(binding, &stage_r1, stage_r2.as_deref())?;
        let stage_id = binding.stage_id.as_str();
        let tool = &binding.tool;
        crate::tool_policy::enforce_stage_tool(stage_id, &tool.tool_id)?;
        stage_params::ensure_stage_params_match(binding)?;
        crate::tool_policy::enforce_input_layout(stage_id, &tool.tool_id, stage_r2.is_some())?;
        let (plan, next_r1, next_r2, next_feature_table) = match stage_id {
            stage if stage == STAGE_DETECT_ADAPTERS.as_str() => {
                let plan = crate::tool_adapters::fastq::detect_adapters::plan_with_options(
                    tool,
                    &stage_r1,
                    stage_r2.as_deref(),
                    &out_dir,
                    &detect_adapters_params(binding),
                )?;
                (plan, stage_r1.clone(), stage_r2.clone(), inherited.feature_table.clone())
            }
            stage if stage == STAGE_DETECT_DUPLICATES_PREMERGE.as_str() => {
                let plan = crate::tool_adapters::fastq::detect_duplicates_premerge::plan(
                    tool,
                    &stage_r1,
                    stage_r2.as_deref(),
                    &out_dir,
                )?;
                (plan, stage_r1.clone(), stage_r2.clone(), inherited.feature_table.clone())
            }
            stage if stage == STAGE_ESTIMATE_LIBRARY_COMPLEXITY_PREALIGN.as_str() => {
                let plan = crate::tool_adapters::fastq::estimate_library_complexity_prealign::plan(
                    tool,
                    &stage_r1,
                    stage_r2.as_deref(),
                    &out_dir,
                    None,
                )?;
                (plan, stage_r1.clone(), stage_r2.clone(), inherited.feature_table.clone())
            }
            stage if stage == STAGE_PROFILE_READ_LENGTHS.as_str() => {
                let plan = if let Some(params) = profile_read_lengths_params(binding) {
                    crate::tool_adapters::fastq::profile_read_lengths::plan_with_options(
                        tool,
                        &stage_r1,
                        stage_r2.as_deref(),
                        &out_dir,
                        Some(params.threads),
                        Some(params.histogram_bins),
                    )?
                } else {
                    crate::tool_adapters::fastq::profile_read_lengths::plan(
                        tool,
                        &stage_r1,
                        stage_r2.as_deref(),
                        &out_dir,
                    )?
                };
                (plan, stage_r1.clone(), stage_r2.clone(), inherited.feature_table.clone())
            }
            stage if stage == STAGE_PROFILE_OVERREPRESENTED_SEQUENCES.as_str() => {
                let plan = if let Some(params) = profile_overrepresented_params(binding) {
                    crate::tool_adapters::fastq::profile_overrepresented_sequences::plan_with_options(
                        tool,
                        &stage_r1,
                        stage_r2.as_deref(),
                        &out_dir,
                        Some(params.threads),
                        Some(params.top_k),
                    )?
                } else {
                    crate::tool_adapters::fastq::profile_overrepresented_sequences::plan(
                        tool,
                        &stage_r1,
                        stage_r2.as_deref(),
                        &out_dir,
                    )?
                };
                (plan, stage_r1.clone(), stage_r2.clone(), inherited.feature_table.clone())
            }
            stage if stage == STAGE_INDEX_REFERENCE.as_str() => {
                let reference_fasta = reference_fasta
                    .ok_or_else(|| anyhow!("reference indexing requires reference_fasta input"))?;
                let plan = crate::tool_adapters::fastq::index_reference::plan_with_options(
                    tool,
                    reference_fasta,
                    &out_dir,
                    &index_reference_params(binding),
                )?;
                (
                    plan,
                    inherited.reads_r1.clone(),
                    inherited.reads_r2.clone(),
                    inherited.feature_table.clone(),
                )
            }
            stage if stage == STAGE_TRIM_READS.as_str() => {
                let plan = if let Some(options) = trim_reads_options(binding) {
                    crate::tool_adapters::fastq::trim_reads::plan_with_options(
                        tool,
                        &stage_r1,
                        stage_r2.as_deref(),
                        &out_dir,
                        adapter_bank,
                        polyx_bank,
                        contaminant_bank,
                        &options,
                    )?
                } else {
                    crate::tool_adapters::fastq::trim_reads::plan(
                        tool,
                        &stage_r1,
                        stage_r2.as_deref(),
                        &out_dir,
                        adapter_bank,
                        polyx_bank,
                        contaminant_bank,
                    )?
                };
                let next_r1 = plan.io.outputs[0].path.clone();
                let next_r2 =
                    if stage_r2.is_some() { Some(plan.io.outputs[1].path.clone()) } else { None };
                (plan, next_r1, next_r2, inherited.feature_table.clone())
            }
            stage if stage == STAGE_TRIM_TERMINAL_DAMAGE.as_str() => {
                let params = trim_terminal_damage_params(binding);
                let plan = crate::tool_adapters::fastq::trim_terminal_damage::plan_trim_terminal_damage_with_options(
                    tool,
                    &stage_r1,
                    stage_r2.as_deref(),
                    &out_dir,
                    &params,
                )?;
                let next_r1 = plan.io.outputs[0].path.clone();
                let next_r2 =
                    if stage_r2.is_some() { Some(plan.io.outputs[1].path.clone()) } else { None };
                (plan, next_r1, next_r2, inherited.feature_table.clone())
            }
            stage if stage == STAGE_FILTER_READS.as_str() => {
                let mut filter_options = filter_reads_plan_options(binding);
                if adapter_bank.is_some()
                    && !filter_options.redundant_filters.iter().any(|filter| filter == "adapter")
                {
                    filter_options.redundant_filters.push("adapter".to_string());
                }
                if polyx_bank.is_some()
                    && !filter_options.redundant_filters.iter().any(|filter| filter == "polyx")
                {
                    filter_options.redundant_filters.push("polyx".to_string());
                }
                if enable_contaminant_removal
                    && contaminant_bank.is_some()
                    && filter_options.kmer_ref.is_none()
                {
                    filter_options.kmer_ref =
                        crate::tool_adapters::fastq::filter_reads::default_kmer_ref();
                }
                let plan = crate::tool_adapters::fastq::filter_reads::plan_filter(
                    tool,
                    &stage_r1,
                    stage_r2.as_deref(),
                    &out_dir,
                    &filter_options,
                )?;
                let next_r1 = plan.io.outputs[0].path.clone();
                let next_r2 =
                    if stage_r2.is_some() { Some(plan.io.outputs[1].path.clone()) } else { None };
                (plan, next_r1, next_r2, inherited.feature_table.clone())
            }
            stage if stage == STAGE_REMOVE_DUPLICATES.as_str() => {
                let plan = if let Some(params) = remove_duplicates_params(binding) {
                    crate::tool_adapters::fastq::remove_duplicates::plan_deduplicate_with_options(
                        tool,
                        &stage_r1,
                        stage_r2.as_deref(),
                        &out_dir,
                        &crate::tool_adapters::fastq::remove_duplicates::RemoveDuplicatesPlanOptions {
                            dedup_mode: params.dedup_mode.clone(),
                            keep_order: params.keep_order,
                            threads_override: Some(params.threads),
                        },
                    )?
                } else {
                    crate::tool_adapters::fastq::remove_duplicates::plan_deduplicate(
                        tool,
                        &stage_r1,
                        stage_r2.as_deref(),
                        &out_dir,
                    )?
                };
                let next_r1 = plan.io.outputs[0].path.clone();
                let next_r2 =
                    if stage_r2.is_some() { Some(plan.io.outputs[1].path.clone()) } else { None };
                (plan, next_r1, next_r2, inherited.feature_table.clone())
            }
            stage if stage == STAGE_DEPLETE_HOST.as_str() => {
                let explicit_reference_index =
                    explicit_reference_index_state(&resolved_inputs, "reference_index")?;
                let reference_index = explicit_reference_index
                    .as_ref()
                    .or(inherited.reference_index.as_ref())
                    .ok_or_else(|| {
                        anyhow!("host depletion requires a prior reference index stage")
                    })?;
                ensure_reference_index_backend(
                    STAGE_DEPLETE_HOST.as_str(),
                    tool.tool_id.as_str(),
                    &reference_index.tool_id,
                )?;
                let params = deplete_host_params(binding);
                let plan =
                    crate::tool_adapters::fastq::deplete_host::plan_host_depletion_with_index_backend(
                        tool,
                        &stage_r1,
                        stage_r2.as_deref(),
                        &reference_index.path,
                        &out_dir,
                        &params,
                        &reference_index.tool_id,
                    )?;
                let next_r1 = plan.io.outputs[0].path.clone();
                let next_r2 =
                    if stage_r2.is_some() { Some(plan.io.outputs[1].path.clone()) } else { None };
                (plan, next_r1, next_r2, inherited.feature_table.clone())
            }
            stage if stage == STAGE_DEPLETE_REFERENCE_CONTAMINANTS.as_str() => {
                let explicit_reference_index =
                    explicit_reference_index_state(&resolved_inputs, "reference_index")?;
                let reference_index = explicit_reference_index
                    .as_ref()
                    .or(inherited.reference_index.as_ref())
                    .ok_or_else(|| {
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
                let plan = crate::tool_adapters::fastq::deplete_reference_contaminants::plan_contaminant_screen_with_index_backend(
                    tool,
                    &stage_r1,
                    stage_r2.as_deref(),
                    &reference_index.path,
                    &out_dir,
                    &params,
                    &reference_index.tool_id,
                )?;
                let next_r1 = plan.io.outputs[0].path.clone();
                let next_r2 =
                    if stage_r2.is_some() { Some(plan.io.outputs[1].path.clone()) } else { None };
                (plan, next_r1, next_r2, inherited.feature_table.clone())
            }
            stage if stage == STAGE_FILTER_LOW_COMPLEXITY.as_str() => {
                let low_complexity_options = filter_low_complexity_plan_options(binding);
                let plan = crate::tool_adapters::fastq::filter_low_complexity::plan_low_complexity(
                    tool,
                    &stage_r1,
                    stage_r2.as_deref(),
                    &out_dir,
                    &low_complexity_options,
                )?;
                let next_r1 = plan.io.outputs[0].path.clone();
                let next_r2 =
                    if stage_r2.is_some() { Some(plan.io.outputs[1].path.clone()) } else { None };
                (plan, next_r1, next_r2, inherited.feature_table.clone())
            }
            stage if stage == STAGE_TRIM_POLYG_TAILS.as_str() => {
                let plan = if let Some(options) = trim_polyg_options(binding) {
                    crate::tool_adapters::fastq::trim_polyg_tails::plan_trim_polyg_tails_with_options(
                        tool,
                        &stage_r1,
                        stage_r2.as_deref(),
                        &out_dir,
                        &options,
                    )?
                } else {
                    crate::tool_adapters::fastq::trim_polyg_tails::plan_trim_polyg_tails(
                        tool,
                        &stage_r1,
                        stage_r2.as_deref(),
                        &out_dir,
                    )?
                };
                let next_r1 = plan.io.outputs[0].path.clone();
                let next_r2 =
                    if stage_r2.is_some() { Some(plan.io.outputs[1].path.clone()) } else { None };
                (plan, next_r1, next_r2, inherited.feature_table.clone())
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
                let params = validate_reads_params(binding, stage_r2.is_some());
                let plan = crate::tool_adapters::fastq::validate_reads::plan_with_options(
                    tool,
                    &stage_r1,
                    stage_r2.as_deref(),
                    &out_dir,
                    &crate::tool_adapters::fastq::validate_reads::ValidateReadsPlanOptions {
                        threads: Some(params.threads),
                        validation_mode: params.validation_mode.clone(),
                        pair_sync_policy: params.pair_sync_policy.clone(),
                    },
                )?;
                (plan, stage_r1.clone(), stage_r2.clone(), inherited.feature_table.clone())
            }
            stage if stage == STAGE_MERGE_PAIRS.as_str() => {
                let r2 = stage_r2.as_ref().ok_or_else(|| anyhow!("merge requires r2"))?;
                let plan = crate::tool_adapters::fastq::merge_pairs::plan_merge_with_options(
                    tool,
                    &stage_r1,
                    r2,
                    &out_dir,
                    &merge_pairs_plan_options(binding),
                )?;
                let next_r1 = plan.io.outputs[0].path.clone();
                (plan, next_r1, None, inherited.feature_table.clone())
            }
            stage if stage == STAGE_CORRECT_ERRORS.as_str() => {
                let params = correct_errors_params(binding);
                let plan = crate::tool_adapters::fastq::correct_errors::plan_correct_with_options(
                    tool,
                    &stage_r1,
                    stage_r2.as_deref(),
                    &out_dir,
                    &params,
                )?;
                let next_r1 = plan.io.outputs[0].path.clone();
                let next_r2 = stage_r2
                    .as_ref()
                    .and_then(|_| plan.io.outputs.get(1).map(|artifact| artifact.path.clone()));
                (plan, next_r1, next_r2, inherited.feature_table.clone())
            }
            stage if stage == STAGE_EXTRACT_UMIS.as_str() => {
                let r2 = stage_r2.as_ref().ok_or_else(|| anyhow!("umi requires r2"))?;
                let plan = crate::tool_adapters::fastq::extract_umis::plan_umi_with_options(
                    tool,
                    &stage_r1,
                    r2,
                    &out_dir,
                    &extract_umis_params(binding),
                )?;
                let next_r1 = plan.io.outputs[0].path.clone();
                let next_r2 = plan.io.outputs[1].path.clone();
                (plan, next_r1, Some(next_r2), inherited.feature_table.clone())
            }
            stage if stage == STAGE_REPORT_QC.as_str() => {
                let report_qc_inputs = explicit_report_qc_inputs(&resolved_inputs)?
                    .unwrap_or_else(|| inherited.qc_inputs.clone());
                let mut stage_aux_images = std::collections::BTreeMap::new();
                if tool.tool_id.0 == "multiqc" {
                    for aux_tool in
                        crate::tool_adapters::fastq::report_qc::aux_tool_ids_for_qc_inputs(
                            &report_qc_inputs,
                        )
                    {
                        if let Some(image) = aux_images.get(aux_tool.as_str()) {
                            stage_aux_images.insert(aux_tool, image.clone());
                        }
                    }
                }
                let paired_mode =
                    if stage_r2.is_some() { PairedMode::PairedEnd } else { PairedMode::SingleEnd };
                let qc_post_params = report_qc_params(binding, paired_mode);
                if report_qc_inputs.is_empty() {
                    return Err(anyhow!(
                        "fastq.report_qc requires governed upstream QC artifacts; add contributing QC stages before report aggregation"
                    ));
                }
                let plan = crate::tool_adapters::fastq::report_qc::plan_qc_post_with_qc_inputs(
                    tool,
                    &report_qc_inputs,
                    &out_dir,
                    stage_aux_images,
                    qc_post_params.paired_mode,
                    qc_post_params.aggregation_engine,
                    qc_post_params.aggregation_scope,
                    Some(raw_r1.as_path()),
                    raw_r2.as_deref(),
                )?;
                (plan, stage_r1.clone(), stage_r2.clone(), inherited.feature_table.clone())
            }
            stage if stage == STAGE_DEPLETE_RRNA.as_str() => {
                let params = deplete_rrna_params(binding);
                let plan = crate::tool_adapters::fastq::deplete_rrna::plan_rrna_with_options(
                    tool,
                    &stage_r1,
                    stage_r2.as_deref(),
                    &out_dir,
                    &params,
                )?;
                let next_r1 = plan.io.outputs[0].path.clone();
                let next_r2 =
                    if stage_r2.is_some() { Some(plan.io.outputs[1].path.clone()) } else { None };
                (plan, next_r1, next_r2, inherited.feature_table.clone())
            }
            stage if stage == STAGE_SCREEN_TAXONOMY.as_str() => {
                let params = screen_params(binding, stage_r2.is_some());
                let plan = crate::tool_adapters::fastq::screen_taxonomy::plan_screen_with_effective_params(
                    tool,
                    &stage_r1,
                    stage_r2.as_deref(),
                    &out_dir,
                    &params,
                )?;
                (plan, stage_r1.clone(), stage_r2.clone(), inherited.feature_table.clone())
            }
            stage if stage == STAGE_PROFILE_READS.as_str() => {
                let plan = if let Some(params) = profile_reads_params(binding) {
                    crate::tool_adapters::fastq::profile_reads::plan_stats_with_threads(
                        tool,
                        &stage_r1,
                        stage_r2.as_deref(),
                        &out_dir,
                        Some(params.threads),
                    )?
                } else {
                    crate::tool_adapters::fastq::profile_reads::plan_stats_neutral(
                        tool,
                        &stage_r1,
                        stage_r2.as_deref(),
                        &out_dir,
                    )?
                };
                (plan, stage_r1.clone(), stage_r2.clone(), inherited.feature_table.clone())
            }
            stage if stage == STAGE_NORMALIZE_PRIMERS.as_str() => {
                if !matches!(tool.tool_id.as_str(), "cutadapt" | "seqkit") {
                    return Err(anyhow!(
                        "{} requires cutadapt/seqkit; got {}",
                        STAGE_NORMALIZE_PRIMERS.as_str(),
                        tool.tool_id
                    ));
                }
                let plan = crate::tool_adapters::fastq::normalize_primers::plan_with_options(
                    tool,
                    &stage_r1,
                    stage_r2.as_deref(),
                    &out_dir,
                    &normalize_primers_plan_options(binding),
                )?;
                let next_r1 = plan.io.outputs[0].path.clone();
                let next_r2 =
                    if stage_r2.is_some() { Some(plan.io.outputs[1].path.clone()) } else { None };
                (plan, next_r1, next_r2, inherited.feature_table.clone())
            }
            stage if stage == STAGE_REMOVE_CHIMERAS.as_str() => {
                if tool.tool_id.as_str() != "vsearch" {
                    return Err(anyhow!(
                        "{} requires vsearch; got {}",
                        STAGE_REMOVE_CHIMERAS.as_str(),
                        tool.tool_id
                    ));
                }
                let plan = if let Some(params) = remove_chimeras_params(binding) {
                    crate::tool_adapters::fastq::remove_chimeras::plan_with_effective_params(
                        tool,
                        &stage_r1,
                        stage_r2.as_deref(),
                        &out_dir,
                        &params,
                    )?
                } else {
                    crate::tool_adapters::fastq::remove_chimeras::plan(
                        tool,
                        &stage_r1,
                        stage_r2.as_deref(),
                        &out_dir,
                    )?
                };
                let next_r1 = plan.io.outputs[0].path.clone();
                (plan, next_r1, None, inherited.feature_table.clone())
            }
            stage if stage == STAGE_INFER_ASVS.as_str() => {
                if tool.tool_id.as_str() != "dada2" {
                    return Err(anyhow!(
                        "{} requires dada2 tool binding; got {}",
                        STAGE_INFER_ASVS.as_str(),
                        tool.tool_id
                    ));
                }
                let infer_params = infer_asvs_params(binding);
                let plan = crate::tool_adapters::fastq::infer_asvs::plan_with_options(
                    tool,
                    &stage_r1,
                    stage_r2.as_deref(),
                    &out_dir,
                    &infer_params,
                )?;
                let next_feature_table = Some(plan.io.outputs[0].path.clone());
                (plan, stage_r1.clone(), stage_r2.clone(), next_feature_table)
            }
            stage if stage == STAGE_CLUSTER_OTUS.as_str() => {
                if tool.tool_id.as_str() != "vsearch" {
                    return Err(anyhow!(
                        "{} requires vsearch; got {}",
                        STAGE_CLUSTER_OTUS.as_str(),
                        tool.tool_id
                    ));
                }
                let cluster_params = cluster_otus_params(binding);
                let plan = crate::tool_adapters::fastq::cluster_otus::plan_with_options(
                    tool,
                    &stage_r1,
                    stage_r2.as_deref(),
                    &out_dir,
                    &cluster_params,
                )?;
                let next_feature_table = Some(plan.io.outputs[0].path.clone());
                (plan, stage_r1.clone(), stage_r2.clone(), next_feature_table)
            }
            stage if stage == STAGE_NORMALIZE_ABUNDANCE.as_str() => {
                ensure_normalize_abundance_tool(tool.tool_id.as_str())?;
                let abundance_table = explicit_abundance_table(&resolved_inputs)?
                    .or(inherited.feature_table.clone())
                    .ok_or_else(|| {
                        anyhow!("fastq.normalize_abundance requires an upstream feature table")
                    })?;
                let plan = crate::tool_adapters::fastq::normalize_abundance::plan_with_options(
                    tool,
                    &abundance_table,
                    &out_dir,
                    &normalize_abundance_plan_options(binding),
                )?;
                let next_feature_table = Some(plan.io.outputs[0].path.clone());
                (plan, stage_r1.clone(), stage_r2.clone(), next_feature_table)
            }
            _ => {
                return Err(anyhow!("unsupported stage in fastq pipeline: {}", binding.stage_id));
            }
        };
        let mut plan = plan;
        merge_lineage_input_artifacts(&mut plan, &inherited.lineage_inputs);
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
        let mut next_qc_inputs = inherited.qc_inputs.clone();
        next_qc_inputs.extend(qc_input_artifacts_for_stage(stage_id, &plan));
        next_qc_inputs.sort_by(|left, right| {
            left.name.as_str().cmp(right.name.as_str()).then_with(|| left.path.cmp(&right.path))
        });
        next_qc_inputs.dedup_by(|left, right| left.name == right.name && left.path == right.path);
        let mut next_lineage_inputs = inherited.lineage_inputs.clone();
        next_lineage_inputs.extend(lineage_input_artifacts_for_stage(&plan));
        next_lineage_inputs.sort_by(|left, right| {
            left.name.as_str().cmp(right.name.as_str()).then_with(|| left.path.cmp(&right.path))
        });
        next_lineage_inputs
            .dedup_by(|left, right| left.name == right.name && left.path == right.path);
        let reference_index = if stage_id == STAGE_INDEX_REFERENCE.as_str() {
            let reference_index_artifact = plan
                .io
                .outputs
                .iter()
                .find(|artifact| artifact.name.as_str() == "reference_index")
                .ok_or_else(|| {
                    anyhow!(
                        "{} plan from {} must publish a reference_index output artifact",
                        STAGE_INDEX_REFERENCE.as_str(),
                        plan.tool_id.as_str()
                    )
                })?;
            Some(ReferenceIndexState {
                path: reference_index_artifact.path.clone(),
                tool_id: plan.tool_id.to_string(),
            })
        } else {
            inherited.reference_index.clone()
        };
        plans.push(plan);
        let stage_node_id = stage_node_id_for_binding(binding);
        lineage_by_node_id.insert(
            stage_node_id.clone(),
            PlannedStageLineage {
                reads_r1: next_r1,
                reads_r2: next_r2,
                feature_table: next_feature_table,
                reference_index,
                qc_inputs: next_qc_inputs,
                lineage_inputs: next_lineage_inputs,
            },
        );
        latest_lineage_node_id = Some(stage_node_id);
    }
    Ok(plans)
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
    Err(anyhow!("{} requires seqkit; got {}", STAGE_NORMALIZE_ABUNDANCE.as_str(), tool_id))
}
