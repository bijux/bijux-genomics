use bijux_dna_core::ids::StageId;

#[must_use]
pub fn governed_qc_output_ids_for_stage(stage_id: &str) -> &'static [&'static str] {
    match stage_id {
        "fastq.validate_reads" => &["validation_report"],
        "fastq.detect_adapters" => &["adapter_report", "adapter_evidence_dir"],
        "fastq.profile_read_lengths" => &["length_distribution_tsv", "length_distribution_json"],
        "fastq.profile_overrepresented_sequences" => &[
            "overrepresented_sequences_tsv",
            "overrepresented_sequences_json",
        ],
        "fastq.profile_reads" => &["qc_json", "qc_tsv", "qc_plots_dir"],
        "fastq.deplete_rrna" => &["rrna_report_tsv", "rrna_report_json"],
        "fastq.screen_taxonomy" => &["screen_report_tsv", "classification_report_json"],
        "fastq.trim_reads" => &["report_json"],
        "fastq.merge_pairs" => &["report_json"],
        "fastq.remove_duplicates" => &["report_json"],
        "fastq.filter_low_complexity" => &["filter_report_json"],
        "fastq.deplete_host" => &["host_depletion_report_json"],
        "fastq.deplete_reference_contaminants" => &["contaminant_screen_report_json"],
        "fastq.correct_errors" => &["report_json"],
        "fastq.trim_terminal_damage" => &["report_json"],
        "fastq.trim_polyg_tails" => &["report_json"],
        "fastq.extract_umis" => &["report_json"],
        _ => &[],
    }
}

#[must_use]
pub fn governed_qc_producer_stage_ids() -> Vec<StageId> {
    bijux_dna_domain_fastq::STAGES
        .iter()
        .filter(|stage_id| !governed_qc_output_ids_for_stage(stage_id.as_str()).is_empty())
        .cloned()
        .collect()
}

#[must_use]
pub fn governed_qc_default_tool_ids() -> Vec<String> {
    let mut tool_ids = governed_qc_producer_stage_ids()
        .into_iter()
        .filter_map(|stage_id| crate::selection::default_tool_for_stage(&stage_id))
        .map(|tool_id| tool_id.to_string())
        .collect::<Vec<_>>();
    tool_ids.sort();
    tool_ids.dedup();
    tool_ids
}

#[cfg(test)]
mod tests {
    use super::{
        governed_qc_default_tool_ids, governed_qc_output_ids_for_stage,
        governed_qc_producer_stage_ids,
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
            tool_ids.iter().filter(|tool_id| tool_id.as_str() == "fastqc").count(),
            1
        );
    }

    #[test]
    fn report_qc_is_not_a_qc_producer() {
        assert!(governed_qc_output_ids_for_stage("fastq.report_qc").is_empty());
    }
}
