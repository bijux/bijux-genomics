use std::collections::BTreeMap;
use std::path::PathBuf;

use anyhow::{anyhow, Result};
use bijux_dna_core::prelude::{ContainerImageRefV1, StageId, ToolExecutionSpecV1};
use bijux_dna_stage_contract::{PlanDecisionReason, PlanReasonKind, StagePlanV1};
use bijux_dna_domain_fastq::stages::ids::{
    STAGE_LENGTH_DISTRIBUTION_PRE, STAGE_OVERREPRESENTED_SEQUENCES, STAGE_POLYG_TAILING,
};

use crate::{
    STAGE_ABUNDANCE_NORMALIZATION, STAGE_ASV_INFERENCE, STAGE_CHIMERA_DETECTION, STAGE_CORRECT,
    STAGE_CONTAMINANT_SCREEN, STAGE_DEDUPLICATE, STAGE_DETECT_ADAPTERS, STAGE_FILTER,
    STAGE_HOST_DEPLETION, STAGE_LOW_COMPLEXITY, STAGE_MERGE, STAGE_OTU_CLUSTERING,
    STAGE_PRIMER_NORMALIZATION, STAGE_QC_POST, STAGE_RRNA, STAGE_SCREEN, STAGE_STATS_NEUTRAL,
    STAGE_TRIM, STAGE_UMI, STAGE_VALIDATE_PRE,
};

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
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
    if stages.len() != tools.len() {
        return Err(anyhow!(
            "pipeline stages/tools length mismatch: {} vs {}",
            stages.len(),
            tools.len()
        ));
    }
    let mut current_r1 = r1.to_path_buf();
    let raw_r1 = r1.to_path_buf();
    let mut current_r2 = r2.map(|path| path.to_path_buf());
    let mut plans = Vec::new();
    for (idx, (stage, tool)) in stages.iter().zip(tools.iter()).enumerate() {
        let out_dir = out_dir_for_stage(stage, tool, &current_r1, current_r2.as_deref())?;
        let stage_id: &str = stage;
        let (plan, next_r1, next_r2) = match stage_id {
            stage if stage == STAGE_DETECT_ADAPTERS.as_str() => {
                let plan = crate::tool_adapters::fastq::detect_adapters::plan(
                    tool,
                    &current_r1,
                    &out_dir,
                )?;
                (plan, current_r1.clone(), current_r2.clone())
            }
            stage if stage == STAGE_LENGTH_DISTRIBUTION_PRE.as_str() => {
                let plan = crate::tool_adapters::fastq::length_distribution_pre::plan(
                    tool,
                    &current_r1,
                    &out_dir,
                )?;
                (plan, current_r1.clone(), current_r2.clone())
            }
            stage if stage == STAGE_OVERREPRESENTED_SEQUENCES.as_str() => {
                let plan = crate::tool_adapters::fastq::overrepresented_sequences::plan(
                    tool,
                    &current_r1,
                    &out_dir,
                )?;
                (plan, current_r1.clone(), current_r2.clone())
            }
            stage if stage == STAGE_TRIM.as_str() => {
                let plan = crate::tool_adapters::fastq::trim::plan(
                    tool,
                    &current_r1,
                    &out_dir,
                    adapter_bank,
                    polyx_bank,
                    contaminant_bank,
                )?;
                let next_r1 = plan.io.outputs[0].path.clone();
                (plan, next_r1, None)
            }
            stage if stage == STAGE_FILTER.as_str() => {
                let mut filter_options =
                    crate::tool_adapters::fastq::filter::FilterPlanOptions::default();
                if adapter_bank.is_some() {
                    filter_options.redundant_filters.push("adapter".to_string());
                }
                if polyx_bank.is_some() {
                    filter_options.redundant_filters.push("polyx".to_string());
                }
                if enable_contaminant_removal && contaminant_bank.is_some() {
                    filter_options.kmer_ref =
                        crate::tool_adapters::fastq::filter::default_kmer_ref();
                }
                let plan = crate::tool_adapters::fastq::filter::plan_filter(
                    tool,
                    &current_r1,
                    &out_dir,
                    &filter_options,
                )?;
                let next_r1 = plan.io.outputs[0].path.clone();
                (plan, next_r1, None)
            }
            stage if stage == STAGE_DEDUPLICATE.as_str() => {
                let plan = crate::tool_adapters::fastq::deduplicate::plan_deduplicate(
                    tool,
                    &current_r1,
                    &out_dir,
                )?;
                let next_r1 = plan.io.outputs[0].path.clone();
                (plan, next_r1, None)
            }
            stage if stage == STAGE_HOST_DEPLETION.as_str() => {
                let plan = crate::tool_adapters::fastq::host_depletion::plan_host_depletion(
                    tool,
                    &current_r1,
                    &out_dir,
                )?;
                let next_r1 = plan.io.outputs[0].path.clone();
                (plan, next_r1, None)
            }
            stage if stage == STAGE_CONTAMINANT_SCREEN.as_str() => {
                let plan =
                    crate::tool_adapters::fastq::contaminant_screen::plan_contaminant_screen(
                        tool,
                        &current_r1,
                        &out_dir,
                    )?;
                let next_r1 = plan.io.outputs[0].path.clone();
                (plan, next_r1, None)
            }
            stage if stage == STAGE_LOW_COMPLEXITY.as_str() => {
                let plan = crate::tool_adapters::fastq::low_complexity::plan_low_complexity(
                    tool,
                    &current_r1,
                    &out_dir,
                )?;
                let next_r1 = plan.io.outputs[0].path.clone();
                (plan, next_r1, None)
            }
            stage if stage == STAGE_POLYG_TAILING.as_str() => {
                let plan = crate::tool_adapters::fastq::polyg_tailing::plan_polyg_tailing(
                    tool,
                    &current_r1,
                    &out_dir,
                )?;
                let next_r1 = plan.io.outputs[0].path.clone();
                (plan, next_r1, None)
            }
            stage if stage == STAGE_VALIDATE_PRE.as_str() => {
                if !matches!(tool.tool_id.as_str(), "fastqvalidator" | "seqkit") {
                    return Err(anyhow!(
                        "{} requires a canonical validator tool (fastqvalidator/seqkit); got {}",
                        STAGE_VALIDATE_PRE.as_str(),
                        tool.tool_id
                    ));
                }
                let plan =
                    crate::tool_adapters::fastq::validate_pre::plan(tool, &current_r1, &out_dir)?;
                (plan, current_r1.clone(), current_r2.clone())
            }
            stage if stage == STAGE_MERGE.as_str() => {
                let r2 = current_r2
                    .as_ref()
                    .ok_or_else(|| anyhow!("merge requires r2"))?;
                let plan = crate::tool_adapters::fastq::merge::plan_merge(
                    tool,
                    &current_r1,
                    r2,
                    &out_dir,
                )?;
                let next_r1 = plan.io.outputs[0].path.clone();
                (plan, next_r1, None)
            }
            stage if stage == STAGE_CORRECT.as_str() => {
                let r2 = current_r2
                    .as_ref()
                    .ok_or_else(|| anyhow!("correct requires r2"))?;
                let plan = crate::tool_adapters::fastq::correct::plan_correct(
                    tool,
                    &current_r1,
                    r2,
                    &out_dir,
                )?;
                let next_r1 = plan.io.outputs[0].path.clone();
                let next_r2 = plan.io.outputs[1].path.clone();
                (plan, next_r1, Some(next_r2))
            }
            stage if stage == STAGE_UMI.as_str() => {
                let r2 = current_r2
                    .as_ref()
                    .ok_or_else(|| anyhow!("umi requires r2"))?;
                let plan =
                    crate::tool_adapters::fastq::umi::plan_umi(tool, &current_r1, r2, &out_dir)?;
                let next_r1 = plan.io.outputs[0].path.clone();
                let next_r2 = plan.io.outputs[1].path.clone();
                (plan, next_r1, Some(next_r2))
            }
            stage if stage == STAGE_QC_POST.as_str() => {
                let mut stage_aux_images = std::collections::BTreeMap::new();
                if tool.tool_id.0 == "multiqc" {
                    for aux_tool in crate::tool_adapters::fastq::qc_post::aux_tool_ids() {
                        if let Some(image) = aux_images.get(*aux_tool) {
                            stage_aux_images.insert(aux_tool.to_string(), image.clone());
                        }
                    }
                }
                let plan = crate::tool_adapters::fastq::qc_post::plan_qc_post(
                    tool,
                    &current_r1,
                    &out_dir,
                    stage_aux_images,
                    Some(raw_r1.as_path()),
                )?;
                (plan, current_r1.clone(), current_r2.clone())
            }
            stage if stage == STAGE_RRNA.as_str() => {
                let plan = crate::tool_adapters::fastq::rrna::plan_rrna(
                    tool,
                    &current_r1,
                    &out_dir,
                )?;
                (plan, current_r1.clone(), current_r2.clone())
            }
            stage if stage == STAGE_SCREEN.as_str() => {
                let plan =
                    crate::tool_adapters::fastq::screen::plan_screen(tool, &current_r1, &out_dir)?;
                (plan, current_r1.clone(), current_r2.clone())
            }
            stage if stage == STAGE_STATS_NEUTRAL.as_str() => {
                let plan = crate::tool_adapters::fastq::stats_neutral::plan_stats_neutral(
                    tool,
                    &current_r1,
                    &out_dir,
                )?;
                (plan, current_r1.clone(), current_r2.clone())
            }
            stage if stage == STAGE_PRIMER_NORMALIZATION.as_str() => {
                if !matches!(tool.tool_id.as_str(), "cutadapt" | "seqkit") {
                    return Err(anyhow!(
                        "{} requires cutadapt/seqkit; got {}",
                        STAGE_PRIMER_NORMALIZATION.as_str(),
                        tool.tool_id
                    ));
                }
                let plan = plan_amplicon_stage(
                    stage,
                    tool,
                    &current_r1,
                    &out_dir,
                    serde_json::json!({
                        "orientation_policy": "normalize_to_forward_primer",
                        "primer_set_id": "default",
                        "mismatch_policy": {
                            "max_mismatches": 2,
                            "allow_iupac_codes": true,
                            "strict_5p_anchor": true
                        }
                    }),
                );
                let next_r1 = plan.io.outputs[0].path.clone();
                (plan, next_r1, None)
            }
            stage if stage == STAGE_CHIMERA_DETECTION.as_str() => {
                if tool.tool_id.as_str() != "vsearch" {
                    return Err(anyhow!(
                        "{} requires vsearch; got {}",
                        STAGE_CHIMERA_DETECTION.as_str(),
                        tool.tool_id
                    ));
                }
                let plan = plan_amplicon_stage(
                    stage,
                    tool,
                    &current_r1,
                    &out_dir,
                    serde_json::json!({
                        "method": "vsearch_denovo",
                        "chimera_removed_definition": "exclude flagged reads from downstream feature tables",
                        "metrics": ["chimera_fraction", "non_chimera_reads", "chimera_reads"]
                    }),
                );
                let next_r1 = plan.io.outputs[0].path.clone();
                (plan, next_r1, None)
            }
            stage if stage == STAGE_ASV_INFERENCE.as_str() => {
                if tool.tool_id.as_str() != "dada2" {
                    return Err(anyhow!(
                        "{} requires dada2 tool binding; got {}",
                        STAGE_ASV_INFERENCE.as_str(),
                        tool.tool_id
                    ));
                }
                let plan = plan_amplicon_stage(
                    stage,
                    tool,
                    &current_r1,
                    &out_dir,
                    serde_json::json!({
                        "requires_r_runtime": true,
                        "output_table_kind": "asv_abundance_table",
                        "runtime_constraints": {
                            "requires_r": true,
                            "min_r_major": 4
                        }
                    }),
                );
                (plan, current_r1.clone(), current_r2.clone())
            }
            stage if stage == STAGE_OTU_CLUSTERING.as_str() => {
                if tool.tool_id.as_str() != "vsearch" {
                    return Err(anyhow!(
                        "{} requires vsearch; got {}",
                        STAGE_OTU_CLUSTERING.as_str(),
                        tool.tool_id
                    ));
                }
                let plan = plan_amplicon_stage(
                    stage,
                    tool,
                    &current_r1,
                    &out_dir,
                    serde_json::json!({
                        "identity_threshold": 0.97,
                        "output_table_kind": "otu_abundance_table",
                        "output_naming": "deterministic"
                    }),
                );
                (plan, current_r1.clone(), current_r2.clone())
            }
            stage if stage == STAGE_ABUNDANCE_NORMALIZATION.as_str() => {
                if !matches!(tool.tool_id.as_str(), "seqfu" | "seqkit") {
                    return Err(anyhow!(
                        "{} requires seqfu/seqkit; got {}",
                        STAGE_ABUNDANCE_NORMALIZATION.as_str(),
                        tool.tool_id
                    ));
                }
                let plan = plan_amplicon_stage(
                    stage,
                    tool,
                    &current_r1,
                    &out_dir,
                    serde_json::json!({
                        "method": "relative_abundance",
                        "expected_columns": ["sample_id", "feature_id", "abundance"],
                        "compositional_rule": "per_sample_sum_to_one"
                    }),
                );
                (plan, current_r1.clone(), current_r2.clone())
            }
            _ => {
                return Err(anyhow!("unsupported stage in fastq pipeline: {stage}"));
            }
        };
        let mut plan = plan;
        if let Some(reasons) = tool_reasons {
            if let Some(reason) = reasons.get(idx) {
                plan.reason = reason.clone();
            }
        } else {
            plan.reason = PlanDecisionReason::new(
                PlanReasonKind::Default,
                format!("tool {} selected by planner", plan.tool_id.0),
            );
        }
        plans.push(plan);
        current_r1 = next_r1;
        current_r2 = next_r2;
    }
    Ok(plans)
}

fn plan_amplicon_stage(
    stage_id: &str,
    tool: &ToolExecutionSpecV1,
    input: &std::path::Path,
    out_dir: &std::path::Path,
    effective_params: serde_json::Value,
) -> StagePlanV1 {
    let outputs = match stage_id {
        "fastq.primer_normalization" => vec![
                (
                    "normalized_reads",
                    out_dir.join("primer_normalized.fastq.gz"),
                    bijux_dna_core::prelude::ArtifactRole::Reads,
                ),
                (
                    "primer_orientation_report",
                    out_dir.join("primer_orientation.tsv"),
                    bijux_dna_core::prelude::ArtifactRole::SummaryTsv,
                ),
            ],
        "fastq.chimera_detection" => vec![
                (
                    "chimera_filtered_reads",
                    out_dir.join("chimera_filtered.fastq.gz"),
                    bijux_dna_core::prelude::ArtifactRole::Reads,
                ),
                (
                    "chimera_metrics_json",
                    out_dir.join("chimera_metrics.json"),
                    bijux_dna_core::prelude::ArtifactRole::MetricsJson,
                ),
            ],
        "fastq.asv_inference" => vec![
                (
                    "asv_table_tsv",
                    out_dir.join("asv_abundance.tsv"),
                    bijux_dna_core::prelude::ArtifactRole::SummaryTsv,
                ),
                (
                    "asv_sequences_fasta",
                    out_dir.join("asv_sequences.fasta"),
                    bijux_dna_core::prelude::ArtifactRole::Reads,
                ),
            ],
        "fastq.otu_clustering" => vec![
                (
                    "otu_table_tsv",
                    out_dir.join("otu_abundance.tsv"),
                    bijux_dna_core::prelude::ArtifactRole::SummaryTsv,
                ),
                (
                    "otu_sequences_fasta",
                    out_dir.join("otu_representatives.fasta"),
                    bijux_dna_core::prelude::ArtifactRole::Reads,
                ),
            ],
        "fastq.abundance_normalization" => vec![(
                "normalized_abundance_tsv",
                out_dir.join("abundance_normalized.tsv"),
                bijux_dna_core::prelude::ArtifactRole::SummaryTsv,
            )],
        _ => vec![(
                "stage_output",
                out_dir.join(format!(
                    "{}.out",
                    stage_id
                        .split_once('.')
                        .map(|(_, suffix)| suffix)
                        .unwrap_or(stage_id)
                        .replace('.', "_")
                )),
                bijux_dna_core::prelude::ArtifactRole::SummaryTsv,
            )],
    };
    StagePlanV1 {
        stage_id: StageId::new(stage_id),
        stage_version: bijux_dna_core::prelude::StageVersion(1),
        tool_id: tool.tool_id.clone(),
        tool_version: tool.tool_version.clone(),
        image: tool.image.clone(),
        command: tool.command.clone(),
        resources: tool.resources.clone(),
        io: bijux_dna_stage_contract::StageIO {
            inputs: vec![bijux_dna_stage_contract::ArtifactRef::required(
                bijux_dna_core::prelude::ArtifactId::from_static("reads"),
                input.to_path_buf(),
                bijux_dna_core::prelude::ArtifactRole::Reads,
            )],
            outputs: outputs
                .into_iter()
                .map(|(name, path, role)| {
                    bijux_dna_stage_contract::ArtifactRef::required(
                        bijux_dna_core::prelude::ArtifactId::new(name.to_string()),
                        path,
                        role,
                    )
                })
                .collect(),
        },
        out_dir: out_dir.to_path_buf(),
        params: serde_json::json!({}),
        effective_params,
        aux_images: std::collections::BTreeMap::new(),
        reason: PlanDecisionReason::new(
            PlanReasonKind::Fallback,
            "amplicon stage contract default".to_string(),
        ),
    }
}
