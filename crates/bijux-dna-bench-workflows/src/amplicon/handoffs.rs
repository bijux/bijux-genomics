//! Owner: bijux-dna-bench-workflows
//! Amplicon handoffs.
use super::{
    AmpliconMicroPipelineHandoff, AmpliconMicroPipelineRow, AmpliconMicroPipelineRowStatus,
    BTreeMap, Path, Value,
};
use crate::execution::stage::StageIdentity;

pub(crate) fn build_handoffs(
    repo_root: &Path,
    stages: &super::execution::AmpliconStageArtifacts,
    judgment_row: &AmpliconMicroPipelineRow,
) -> Vec<AmpliconMicroPipelineHandoff> {
    vec![
        handoff_from_rows(
            repo_root,
            &stages.corpus.row,
            "primers_tsv_path",
            &stages.normalize.row,
            "primer_contract",
        ),
        handoff_from_rows(
            repo_root,
            &stages.truth.row,
            "expected_path",
            judgment_row,
            "truth_bundle_path",
        ),
        handoff_from_rows(
            repo_root,
            &stages.normalize.row,
            "normalized_reads_r1",
            judgment_row,
            "normalized_reads_r1",
        ),
        handoff_from_rows(
            repo_root,
            &stages.infer.row,
            "asv_table_tsv",
            judgment_row,
            "asv_table_tsv",
        ),
        handoff_from_rows(
            repo_root,
            &stages.infer.row,
            "representatives_fasta",
            &stages.chimera.row,
            "asv_representatives",
        ),
        handoff_from_rows(
            repo_root,
            &stages.infer.row,
            "representatives_fasta",
            judgment_row,
            "asv_representatives",
        ),
        handoff_from_rows(
            repo_root,
            &stages.chimera.row,
            "chimeras_tsv",
            judgment_row,
            "chimeras_tsv",
        ),
        handoff_from_rows(
            repo_root,
            &stages.chimera.row,
            "non_chimeric_fasta",
            &stages.otu.row,
            "non_chimeric_representatives",
        ),
        handoff_from_rows(
            repo_root,
            &stages.chimera.row,
            "non_chimeric_fasta",
            judgment_row,
            "non_chimeric_representatives",
        ),
        handoff_from_rows(
            repo_root,
            &stages.otu.row,
            "otu_table_tsv",
            judgment_row,
            "otu_table_tsv",
        ),
        handoff_from_rows(
            repo_root,
            &stages.otu.row,
            "otu_representatives_fasta",
            judgment_row,
            "otu_representatives",
        ),
        handoff_from_rows(
            repo_root,
            &stages.abundance.row,
            "otu_abundance_table",
            judgment_row,
            "otu_abundance_table",
        ),
        handoff_from_rows(
            repo_root,
            &stages.abundance.row,
            "normalized_abundance_tsv",
            judgment_row,
            "normalized_abundance_tsv",
        ),
    ]
}

pub(crate) fn handoff_from_rows(
    repo_root: &Path,
    source_row: &AmpliconMicroPipelineRow,
    source_output_id: &str,
    target_row: &AmpliconMicroPipelineRow,
    target_input_id: &str,
) -> AmpliconMicroPipelineHandoff {
    let source_path = source_row.outputs.get(source_output_id).cloned().unwrap_or_default();
    let target_path = target_row.consumed_inputs.get(target_input_id).cloned().unwrap_or_default();
    let source_exists = !source_path.is_empty() && repo_root.join(&source_path).exists();
    let target_exists = !target_path.is_empty() && repo_root.join(&target_path).exists();
    let exact_path_match = source_path == target_path;
    let accepted = source_exists && target_exists && exact_path_match;
    AmpliconMicroPipelineHandoff {
        handoff_id: format!(
            "{}:{}->{}:{}",
            source_row.stage_id, source_output_id, target_row.stage_id, target_input_id
        ),
        source_stage_id: source_row.stage_id.clone(),
        target_stage_id: target_row.stage_id.clone(),
        source_output_id: source_output_id.to_string(),
        target_input_id: target_input_id.to_string(),
        source_path,
        target_path,
        source_exists,
        target_exists,
        exact_path_match,
        accepted,
        detail: if accepted {
            "micro pipeline keeps the governed artifact handoff explicit".to_string()
        } else {
            "micro pipeline handoff drifted from the governed artifact path".to_string()
        },
    }
}

pub(crate) fn row(
    identity: StageIdentity<'_>,
    reason: &str,
    evidence_path: Option<String>,
    parsed_schema_version: Option<String>,
    consumed_inputs: BTreeMap<String, String>,
    outputs: BTreeMap<String, String>,
    metrics: BTreeMap<String, Value>,
) -> AmpliconMicroPipelineRow {
    let StageIdentity { stage_id, domain, tool_id, execution_mode } = identity;

    AmpliconMicroPipelineRow {
        stage_id: stage_id.to_string(),
        domain: domain.to_string(),
        tool_id: tool_id.to_string(),
        execution_mode: execution_mode.to_string(),
        status: AmpliconMicroPipelineRowStatus::Succeeded,
        reason: reason.to_string(),
        evidence_path,
        parsed_schema_version,
        consumed_inputs,
        outputs,
        metrics,
    }
}
