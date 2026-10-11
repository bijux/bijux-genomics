//! Owner: bijux-dna-bench-workflows
//! Ancient DNA handoffs.
use super::{
    anyhow, AdnaMicroPipelineHandoff, AdnaMicroPipelineRow, AdnaMicroPipelineRowStatus, BTreeMap,
    Path, Result, Value,
};
use crate::execution::stage::StageIdentity;
use bijux_dna_core::id_catalog as stage;

pub(crate) fn succeeded_row(
    identity: StageIdentity<'_>,
    evidence_path: String,
    parsed_schema_version: &str,
    consumed_inputs: BTreeMap<String, String>,
    outputs: BTreeMap<String, String>,
    metrics: BTreeMap<String, Value>,
    reason: String,
) -> AdnaMicroPipelineRow {
    let StageIdentity { stage_id, domain, tool_id, execution_mode } = identity;

    AdnaMicroPipelineRow {
        stage_id: stage_id.to_string(),
        domain: domain.to_string(),
        tool_id: tool_id.to_string(),
        execution_mode: execution_mode.to_string(),
        status: AdnaMicroPipelineRowStatus::Succeeded,
        reason,
        evidence_path: Some(evidence_path),
        parsed_schema_version: Some(parsed_schema_version.to_string()),
        consumed_inputs,
        outputs,
        metrics,
    }
}

pub(crate) fn skipped_row(
    stage_id: &str,
    domain: &str,
    tool_id: &str,
    reason: &str,
) -> AdnaMicroPipelineRow {
    AdnaMicroPipelineRow {
        stage_id: stage_id.to_string(),
        domain: domain.to_string(),
        tool_id: tool_id.to_string(),
        execution_mode: "not_executed".to_string(),
        status: AdnaMicroPipelineRowStatus::Skipped,
        reason: reason.to_string(),
        evidence_path: None,
        parsed_schema_version: None,
        consumed_inputs: BTreeMap::new(),
        outputs: BTreeMap::new(),
        metrics: BTreeMap::new(),
    }
}

pub(crate) fn required_output(row: &AdnaMicroPipelineRow, key: &str) -> Result<String> {
    row.outputs
        .get(key)
        .cloned()
        .ok_or_else(|| anyhow!("{} is missing required output `{key}`", row.stage_id))
}

pub(crate) fn validate_handoff(
    repo_root: &Path,
    rows: &[AdnaMicroPipelineRow],
    source_stage_id: &str,
    source_output_id: &str,
    target_stage_id: &str,
    target_input_id: &str,
) -> Result<AdnaMicroPipelineHandoff> {
    let source_row = rows
        .iter()
        .find(|row| row.stage_id == source_stage_id)
        .ok_or_else(|| anyhow!("missing handoff source row `{source_stage_id}`"))?;
    let target_row = rows
        .iter()
        .find(|row| row.stage_id == target_stage_id)
        .ok_or_else(|| anyhow!("missing handoff target row `{target_stage_id}`"))?;
    let source_path = source_row.outputs.get(source_output_id).cloned().ok_or_else(|| {
        anyhow!("missing source output `{source_output_id}` on `{source_stage_id}`")
    })?;
    let target_path =
        target_row.consumed_inputs.get(target_input_id).cloned().ok_or_else(|| {
            anyhow!("missing target input `{target_input_id}` on `{target_stage_id}`")
        })?;
    let source_exists = repo_root.join(&source_path).exists();
    let target_exists = repo_root.join(&target_path).exists();
    let exact_path_match = source_path == target_path;
    let accepted = source_row.status == AdnaMicroPipelineRowStatus::Succeeded
        && target_row.status == AdnaMicroPipelineRowStatus::Succeeded
        && source_exists
        && target_exists
        && exact_path_match;
    Ok(AdnaMicroPipelineHandoff {
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
            "exact governed handoff observed".to_string()
        } else {
            "handoff missing, non-exact, or attached to a skipped stage".to_string()
        },
    })
}

const BINDINGS: &[crate::execution::stage::HandoffBinding] = &[
    crate::execution::stage::HandoffBinding {
        source_stage: stage::FASTQ_VALIDATE_READS,
        source_output: "validated_reads_r1_path",
        target_stage: stage::FASTQ_TRIM_TERMINAL_DAMAGE,
        target_input: "validated_reads_r1_path",
    },
    crate::execution::stage::HandoffBinding {
        source_stage: stage::FASTQ_VALIDATE_READS,
        source_output: "validated_reads_r2_path",
        target_stage: stage::FASTQ_TRIM_TERMINAL_DAMAGE,
        target_input: "validated_reads_r2_path",
    },
    crate::execution::stage::HandoffBinding {
        source_stage: stage::FASTQ_TRIM_TERMINAL_DAMAGE,
        source_output: "terminal_damage_trimmed_reads_r1_path",
        target_stage: stage::FASTQ_DEDUPLICATE,
        target_input: "terminal_damage_trimmed_reads_r1_path",
    },
    crate::execution::stage::HandoffBinding {
        source_stage: stage::FASTQ_TRIM_TERMINAL_DAMAGE,
        source_output: "terminal_damage_trimmed_reads_r2_path",
        target_stage: stage::FASTQ_DEDUPLICATE,
        target_input: "terminal_damage_trimmed_reads_r2_path",
    },
    crate::execution::stage::HandoffBinding {
        source_stage: stage::FASTQ_DEDUPLICATE,
        source_output: "deduplicated_reads_r1_path",
        target_stage: stage::BAM_ALIGN,
        target_input: "deduplicated_reads_r1_path",
    },
    crate::execution::stage::HandoffBinding {
        source_stage: stage::FASTQ_DEDUPLICATE,
        source_output: "deduplicated_reads_r2_path",
        target_stage: stage::BAM_ALIGN,
        target_input: "deduplicated_reads_r2_path",
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
        target_stage: stage::BAM_MAPPING_SUMMARY,
        target_input: "bam_validation_report",
    },
    crate::execution::stage::HandoffBinding {
        source_stage: stage::BAM_ALIGN,
        source_output: "align_sam",
        target_stage: stage::BAM_MAPPING_SUMMARY,
        target_input: "align_sam",
    },
    crate::execution::stage::HandoffBinding {
        source_stage: stage::BAM_ALIGN,
        source_output: "align_sam",
        target_stage: stage::BAM_COVERAGE,
        target_input: "align_sam",
    },
    crate::execution::stage::HandoffBinding {
        source_stage: stage::BAM_MAPPING_SUMMARY,
        source_output: "mapping_summary_report_json",
        target_stage: stage::BAM_COVERAGE,
        target_input: "mapping_summary_report_json",
    },
    crate::execution::stage::HandoffBinding {
        source_stage: stage::BAM_ALIGN,
        source_output: "aligned_bam",
        target_stage: stage::BAM_DAMAGE,
        target_input: "aligned_bam",
    },
    crate::execution::stage::HandoffBinding {
        source_stage: stage::BAM_DAMAGE,
        source_output: "damage_report_json",
        target_stage: stage::BAM_AUTHENTICITY,
        target_input: "damage_report_json",
    },
    crate::execution::stage::HandoffBinding {
        source_stage: stage::BAM_ALIGN,
        source_output: "aligned_bam",
        target_stage: stage::VCF_CALL_PSEUDOHAPLOID,
        target_input: "aligned_bam",
    },
    crate::execution::stage::HandoffBinding {
        source_stage: stage::BAM_COVERAGE,
        source_output: "coverage_report_json",
        target_stage: stage::VCF_CALL_PSEUDOHAPLOID,
        target_input: "coverage_report_json",
    },
    crate::execution::stage::HandoffBinding {
        source_stage: stage::BAM_DAMAGE,
        source_output: "damage_report_json",
        target_stage: stage::VCF_CALL_PSEUDOHAPLOID,
        target_input: "damage_report_json",
    },
    crate::execution::stage::HandoffBinding {
        source_stage: stage::BAM_AUTHENTICITY,
        source_output: "authenticity_report_json",
        target_stage: stage::VCF_CALL_PSEUDOHAPLOID,
        target_input: "authenticity_report_json",
    },
    crate::execution::stage::HandoffBinding {
        source_stage: stage::VCF_CALL_PSEUDOHAPLOID,
        source_output: "pseudohaploid_vcf",
        target_stage: stage::VCF_DAMAGE_FILTER,
        target_input: "pseudohaploid_vcf",
    },
    crate::execution::stage::HandoffBinding {
        source_stage: stage::VCF_CALL_PSEUDOHAPLOID,
        source_output: "pseudohaploid_vcf_tbi",
        target_stage: stage::VCF_DAMAGE_FILTER,
        target_input: "pseudohaploid_vcf_tbi",
    },
    crate::execution::stage::HandoffBinding {
        source_stage: stage::VCF_DAMAGE_FILTER,
        source_output: "damage_filtered_vcf",
        target_stage: stage::VCF_STATS,
        target_input: "damage_filtered_vcf",
    },
    crate::execution::stage::HandoffBinding {
        source_stage: stage::VCF_DAMAGE_FILTER,
        source_output: "damage_filtered_vcf_tbi",
        target_stage: stage::VCF_STATS,
        target_input: "damage_filtered_vcf_tbi",
    },
];

pub(crate) fn build_handoffs(
    repo_root: &Path,
    rows: &[AdnaMicroPipelineRow],
) -> Result<Vec<AdnaMicroPipelineHandoff>> {
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
