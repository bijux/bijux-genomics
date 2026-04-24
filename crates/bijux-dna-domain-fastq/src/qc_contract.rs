use crate::stages::ports::stage_output_ids_in_manifest_order;
use crate::types::FastqArtifactKind;
use bijux_dna_core::ids::StageId;

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
    let required_kind =
        if paired_end { FastqArtifactKind::PairedEnd } else { FastqArtifactKind::SingleEnd };
    if !contract.accepted_input_kinds.iter().any(|kind| kind == &required_kind) {
        return false;
    }
    let Some(input_ids) = crate::stage_input_ids(stage_id.as_str()) else {
        return false;
    };
    input_ids.into_iter().all(|input_id| matches!(input_id.as_str(), "reads_r1" | "reads_r2"))
}

fn is_governed_qc_output_id(output_id: &str) -> bool {
    if output_id == "validated_reads_manifest" {
        return true;
    }
    if output_id.contains("reads")
        || output_id.contains("reference")
        || output_id.contains("table")
        || output_id.contains("fasta")
    {
        return false;
    }
    output_id.ends_with("_manifest")
        || output_id.ends_with("_report")
        || output_id.ends_with("_report_json")
        || output_id.ends_with("_report_tsv")
        || output_id.ends_with("_json")
        || output_id.ends_with("_tsv")
        || output_id.ends_with("_dir")
}

#[cfg(test)]
mod tests {
    use super::{
        governed_qc_bench_contributor_stage_ids, governed_qc_default_tool_ids,
        governed_qc_output_ids_for_stage, governed_qc_producer_stage_ids,
    };
    use bijux_dna_core::ids::StageId;

    #[test]
    fn governed_qc_producers_include_all_governed_qc_stage_families() {
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
        assert_eq!(tool_ids.iter().filter(|tool_id| tool_id.as_str() == "fastqc").count(), 1);
    }

    #[test]
    fn report_qc_is_not_a_qc_producer() {
        assert!(
            governed_qc_output_ids_for_stage(&StageId::from_static("fastq.report_qc")).is_empty()
        );
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
