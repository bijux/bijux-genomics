use bijux_dna_core::ids::StageId;

#[must_use]
pub fn governed_qc_output_ids_for_stage(stage_id: &str) -> Vec<String> {
    bijux_dna_domain_fastq::governed_qc_output_ids_for_stage(&StageId::new(stage_id))
}

#[must_use]
pub fn governed_qc_producer_stage_ids() -> Vec<StageId> {
    bijux_dna_domain_fastq::governed_qc_producer_stage_ids()
}

#[must_use]
pub fn governed_qc_default_tool_ids() -> Vec<String> {
    bijux_dna_domain_fastq::governed_qc_default_tool_ids()
}

#[must_use]
pub fn governed_qc_bench_contributor_stage_ids(paired_end: bool) -> Vec<StageId> {
    bijux_dna_domain_fastq::governed_qc_bench_contributor_stage_ids(paired_end)
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
        assert_eq!(tool_ids.iter().filter(|tool_id| tool_id.as_str() == "fastqc").count(), 1);
    }

    #[test]
    fn report_qc_is_not_a_qc_producer() {
        assert!(governed_qc_output_ids_for_stage("fastq.report_qc").is_empty());
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
