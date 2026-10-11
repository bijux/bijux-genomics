//! Owner: bijux-dna-bench-workflows
//! Core germline handoffs.
use super::{anyhow, CoreGermlineMicroPipelineHandoff, CoreGermlineMicroPipelineRow, Path, Result};
use bijux_dna_core::id_catalog as stage;

pub(crate) fn validate_handoff(
    repo_root: &Path,
    rows: &[CoreGermlineMicroPipelineRow],
    source_stage_id: &str,
    source_output_id: &str,
    target_stage_id: &str,
    target_input_id: &str,
) -> Result<CoreGermlineMicroPipelineHandoff> {
    let source_row = rows
        .iter()
        .find(|row| row.stage_id == source_stage_id)
        .ok_or_else(|| anyhow!("missing source stage row `{source_stage_id}`"))?;
    let target_row = rows
        .iter()
        .find(|row| row.stage_id == target_stage_id)
        .ok_or_else(|| anyhow!("missing target stage row `{target_stage_id}`"))?;
    let source_path = source_row.outputs.get(source_output_id).cloned().ok_or_else(|| {
        anyhow!("stage `{source_stage_id}` is missing output `{source_output_id}`")
    })?;
    let target_path =
        target_row.consumed_inputs.get(target_input_id).cloned().ok_or_else(|| {
            anyhow!("stage `{target_stage_id}` is missing consumed input `{target_input_id}`")
        })?;
    let source_exists = repo_root.join(&source_path).exists();
    let target_exists = repo_root.join(&target_path).exists();
    let exact_path_match = source_path == target_path;
    let accepted = source_exists && target_exists && exact_path_match;
    Ok(CoreGermlineMicroPipelineHandoff {
        handoff_id: format!(
            "{source_stage_id}:{source_output_id}->{target_stage_id}:{target_input_id}"
        ),
        source_stage_id: source_stage_id.to_string(),
        target_stage_id: target_stage_id.to_string(),
        source_output_id: source_output_id.to_string(),
        target_input_id: target_input_id.to_string(),
        source_path,
        target_path,
        source_exists,
        target_exists,
        exact_path_match,
        accepted,
        detail: if accepted {
            "target consumed the exact source artifact path".to_string()
        } else {
            "target did not consume the exact source artifact path".to_string()
        },
    })
}

pub(crate) fn required_output(
    row: &CoreGermlineMicroPipelineRow,
    output_id: &str,
) -> Result<String> {
    row.outputs
        .get(output_id)
        .cloned()
        .ok_or_else(|| anyhow!("stage `{}` is missing output `{output_id}`", row.stage_id))
}

const BINDINGS: &[crate::execution::stage::HandoffBinding] = &[
    crate::execution::stage::HandoffBinding {
        source_stage: stage::FASTQ_VALIDATE_READS,
        source_output: "validated_reads_r1_path",
        target_stage: stage::FASTQ_STATS_NEUTRAL,
        target_input: "validated_reads_r1_path",
    },
    crate::execution::stage::HandoffBinding {
        source_stage: stage::FASTQ_VALIDATE_READS,
        source_output: "validated_reads_r2_path",
        target_stage: stage::FASTQ_STATS_NEUTRAL,
        target_input: "validated_reads_r2_path",
    },
    crate::execution::stage::HandoffBinding {
        source_stage: stage::FASTQ_VALIDATE_READS,
        source_output: "validated_reads_r1_path",
        target_stage: stage::FASTQ_TRIM,
        target_input: "validated_reads_r1_path",
    },
    crate::execution::stage::HandoffBinding {
        source_stage: stage::FASTQ_VALIDATE_READS,
        source_output: "validated_reads_r2_path",
        target_stage: stage::FASTQ_TRIM,
        target_input: "validated_reads_r2_path",
    },
    crate::execution::stage::HandoffBinding {
        source_stage: stage::FASTQ_TRIM,
        source_output: "trimmed_reads_r1_path",
        target_stage: stage::FASTQ_FILTER,
        target_input: "trimmed_reads_r1_path",
    },
    crate::execution::stage::HandoffBinding {
        source_stage: stage::FASTQ_TRIM,
        source_output: "trimmed_reads_r2_path",
        target_stage: stage::FASTQ_FILTER,
        target_input: "trimmed_reads_r2_path",
    },
    crate::execution::stage::HandoffBinding {
        source_stage: stage::FASTQ_FILTER,
        source_output: "filtered_reads_r1_path",
        target_stage: stage::BAM_ALIGN,
        target_input: "filtered_reads_r1_path",
    },
    crate::execution::stage::HandoffBinding {
        source_stage: stage::FASTQ_FILTER,
        source_output: "filtered_reads_r2_path",
        target_stage: stage::BAM_ALIGN,
        target_input: "filtered_reads_r2_path",
    },
    crate::execution::stage::HandoffBinding {
        source_stage: stage::BAM_ALIGN,
        source_output: "aligned_bam",
        target_stage: stage::BAM_VALIDATE,
        target_input: "aligned_bam",
    },
    crate::execution::stage::HandoffBinding {
        source_stage: stage::BAM_VALIDATE,
        source_output: "bam_validation_report",
        target_stage: stage::BAM_QC_PRE,
        target_input: "bam_validation_report",
    },
    crate::execution::stage::HandoffBinding {
        source_stage: stage::BAM_QC_PRE,
        source_output: "qc_pre_report",
        target_stage: stage::BAM_COVERAGE,
        target_input: "qc_pre_report",
    },
    crate::execution::stage::HandoffBinding {
        source_stage: stage::BAM_ALIGN,
        source_output: "aligned_bam",
        target_stage: stage::VCF_CALL,
        target_input: "aligned_bam",
    },
    crate::execution::stage::HandoffBinding {
        source_stage: stage::BAM_COVERAGE,
        source_output: "coverage_report_json",
        target_stage: stage::VCF_CALL,
        target_input: "coverage_report_json",
    },
    crate::execution::stage::HandoffBinding {
        source_stage: stage::BAM_COVERAGE,
        source_output: "coverage_regions_json",
        target_stage: stage::VCF_CALL,
        target_input: "coverage_regions_json",
    },
    crate::execution::stage::HandoffBinding {
        source_stage: stage::VCF_CALL,
        source_output: "call_metrics_json",
        target_stage: stage::VCF_FILTER,
        target_input: "call_metrics_json",
    },
    crate::execution::stage::HandoffBinding {
        source_stage: stage::VCF_CALL,
        source_output: "called_vcf",
        target_stage: stage::VCF_FILTER,
        target_input: "called_vcf",
    },
    crate::execution::stage::HandoffBinding {
        source_stage: stage::VCF_FILTER,
        source_output: "filter_report_json",
        target_stage: stage::VCF_STATS,
        target_input: "filter_report_json",
    },
    crate::execution::stage::HandoffBinding {
        source_stage: stage::VCF_FILTER,
        source_output: "filtered_vcf",
        target_stage: stage::VCF_STATS,
        target_input: "filtered_vcf",
    },
    crate::execution::stage::HandoffBinding {
        source_stage: stage::VCF_FILTER,
        source_output: "filtered_vcf",
        target_stage: stage::VCF_QC,
        target_input: "filtered_vcf",
    },
    crate::execution::stage::HandoffBinding {
        source_stage: stage::VCF_STATS,
        source_output: "stats_json",
        target_stage: stage::VCF_QC,
        target_input: "stats_json",
    },
];

pub(crate) fn build_handoffs(
    repo_root: &Path,
    rows: &[CoreGermlineMicroPipelineRow],
) -> Result<Vec<CoreGermlineMicroPipelineHandoff>> {
    BINDINGS
        .iter()
        .map(|edge| {
            validate_handoff(
                repo_root,
                rows,
                edge.source_stage,
                edge.source_output,
                edge.target_stage,
                edge.target_input,
            )
        })
        .collect()
}
