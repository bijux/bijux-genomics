use bijux_dna_core::ids::StageId;
use crate::stages::ports::stage_output_ids_in_manifest_order;
use crate::types::FastqArtifactKind;

#[must_use]
pub fn governed_qc_output_ids_for_stage(stage_id: &StageId) -> Vec<String> {
    if stage_id.as_str() == "fastq.report_qc" {
        return Vec::new();
    }
    stage_output_ids_in_manifest_order(stage_id.as_str())
        .unwrap_or_default()
        .into_iter()
        .filter(|output_id| is_governed_qc_output_id(output_id))
        .collect()
}

#[must_use]
pub fn governed_qc_producer_stage_ids() -> Vec<StageId> {
    crate::FASTQ_STAGE_ID_CATALOG
        .iter()
        .map(|stage_id| StageId::new(*stage_id))
        .filter(|stage_id| !governed_qc_output_ids_for_stage(stage_id).is_empty())
        .collect()
}

#[must_use]
pub fn governed_qc_default_tool_ids() -> Vec<String> {
    let mut tool_ids = governed_qc_producer_stage_ids()
        .into_iter()
        .filter_map(|stage_id| crate::default_execution_tool_for_stage(&stage_id))
        .map(|tool_id| tool_id.to_string())
        .collect::<Vec<_>>();
    tool_ids.sort();
    tool_ids.dedup();
    tool_ids
}

#[must_use]
pub fn governed_qc_bench_contributor_stage_ids(paired_end: bool) -> Vec<StageId> {
    governed_qc_producer_stage_ids()
        .into_iter()
        .filter(|stage_id| stage_supports_governed_qc_bench_inputs(stage_id, paired_end))
        .collect()
}

fn stage_supports_governed_qc_bench_inputs(stage_id: &StageId, paired_end: bool) -> bool {
    let Some(contract) = crate::contract_for_stage(stage_id.as_str()) else {
        return false;
    };
    let required_kind = if paired_end {
        FastqArtifactKind::PairedEnd
    } else {
        FastqArtifactKind::SingleEnd
    };
    if !contract
        .accepted_input_kinds
        .iter()
        .any(|kind| kind == &required_kind)
    {
        return false;
    }
    let Some(input_ids) = crate::stage_input_ids(stage_id.as_str()) else {
        return false;
    };
    input_ids
        .into_iter()
        .all(|input_id| matches!(input_id.as_str(), "reads_r1" | "reads_r2"))
}

fn is_governed_qc_output_id(output_id: &str) -> bool {
    matches!(
        output_id,
        "validation_report"
            | "validated_reads_manifest"
            | "adapter_report"
            | "adapter_evidence_dir"
            | "length_distribution_tsv"
            | "length_distribution_json"
            | "overrepresented_sequences_tsv"
            | "overrepresented_sequences_json"
            | "qc_json"
            | "qc_tsv"
            | "qc_plots_dir"
            | "rrna_report_tsv"
            | "rrna_report_json"
            | "screen_report_tsv"
            | "classification_report_json"
            | "report_json"
            | "filter_report_json"
            | "host_depletion_report_json"
            | "contaminant_screen_report_json"
    )
}

#[cfg(test)]
mod tests {
    use super::{
        governed_qc_bench_contributor_stage_ids, governed_qc_default_tool_ids,
        governed_qc_output_ids_for_stage, governed_qc_producer_stage_ids,
    };
    use bijux_dna_core::ids::StageId;

    #[test]
    fn governed_qc_registry_includes_all_governed_qc_stage_families() {
        let producers = governed_qc_producer_stage_ids();
        assert!(producers.contains(&StageId::from_static("fastq.validate_reads")));
        assert!(producers.contains(&StageId::from_static("fastq.detect_adapters")));
        assert!(!producers.contains(&StageId::from_static("fastq.report_qc")));
        assert!(producers.contains(&StageId::from_static("fastq.deplete_rrna")));
        assert!(producers.contains(&StageId::from_static("fastq.trim_reads")));
        assert!(producers.contains(&StageId::from_static("fastq.correct_errors")));
        assert!(producers.contains(&StageId::from_static("fastq.trim_polyg_tails")));
    }

    #[test]
    fn governed_qc_default_tools_are_deduplicated() {
        let tool_ids = governed_qc_default_tool_ids();
        assert!(tool_ids.contains(&"fastqvalidator".to_string()));
        assert!(tool_ids.contains(&"fastqc".to_string()));
        assert!(!tool_ids.contains(&"multiqc".to_string()));
        assert_eq!(
            tool_ids
                .iter()
                .filter(|tool_id| tool_id.as_str() == "fastqc")
                .count(),
            1
        );
    }

    #[test]
    fn report_qc_is_not_a_qc_producer() {
        assert!(governed_qc_output_ids_for_stage(&StageId::from_static("fastq.report_qc")).is_empty());
    }

    #[test]
    fn bench_qc_contributors_cover_cleanup_and_pair_aware_stages() {
        let single_end = governed_qc_bench_contributor_stage_ids(false);
        assert!(single_end.contains(&StageId::from_static("fastq.trim_reads")));
        assert!(single_end.contains(&StageId::from_static("fastq.trim_terminal_damage")));
        assert!(single_end.contains(&StageId::from_static("fastq.trim_polyg_tails")));
        assert!(single_end.contains(&StageId::from_static("fastq.remove_duplicates")));
        assert!(single_end.contains(&StageId::from_static("fastq.profile_reads")));
        assert!(single_end.contains(&StageId::from_static("fastq.screen_taxonomy")));
        assert!(single_end.contains(&StageId::from_static("fastq.deplete_rrna")));
        assert!(single_end.contains(&StageId::from_static("fastq.correct_errors")));
        assert!(!single_end.contains(&StageId::from_static("fastq.deplete_host")));
        assert!(!single_end.contains(&StageId::from_static("fastq.deplete_reference_contaminants")));
        assert!(!single_end.contains(&StageId::from_static("fastq.normalize_abundance")));
        assert!(!single_end.contains(&StageId::from_static("fastq.merge_pairs")));

        let paired_end = governed_qc_bench_contributor_stage_ids(true);
        assert!(paired_end.contains(&StageId::from_static("fastq.correct_errors")));
        assert!(paired_end.contains(&StageId::from_static("fastq.merge_pairs")));
        assert!(paired_end.contains(&StageId::from_static("fastq.extract_umis")));
        assert!(!paired_end.contains(&StageId::from_static("fastq.deplete_host")));
    }
}
