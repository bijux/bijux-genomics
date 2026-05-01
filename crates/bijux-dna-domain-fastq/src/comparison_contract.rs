mod catalog;
mod priorities;
mod trim;

pub use catalog::{
    benchmark_comparison_artifact_ids, comparison_artifact_ids_for_stage,
    comparison_contract_for_stage, comparison_input_artifact_ids_for_stage,
    StageComparisonContract,
};
pub use trim::{
    trim_backend_comparison_contract, TrimBackendComparisonContract, TrimComparisonToolProfile,
};

#[cfg(test)]
mod tests {
    use super::comparison_input_artifact_ids_for_stage;
    use bijux_dna_core::ids::StageId;

    #[test]
    fn validate_reads_comparison_inputs_prioritize_governed_report_then_lineage() {
        let artifact_ids =
            comparison_input_artifact_ids_for_stage(&StageId::from_static("fastq.validate_reads"));
        assert_eq!(
            artifact_ids,
            vec!["validation_report".to_string(), "validated_reads_manifest".to_string(),]
        );
    }

    #[test]
    fn screen_taxonomy_comparison_inputs_prioritize_governed_classification_report() {
        let artifact_ids =
            comparison_input_artifact_ids_for_stage(&StageId::from_static("fastq.screen_taxonomy"));
        assert_eq!(
            artifact_ids,
            vec!["classification_report_json".to_string(), "screen_report_tsv".to_string(),]
        );
    }

    #[test]
    fn trim_polyg_comparison_inputs_exclude_backend_native_reports() {
        let artifact_ids = comparison_input_artifact_ids_for_stage(&StageId::from_static(
            "fastq.trim_polyg_tails",
        ));
        assert_eq!(
            artifact_ids,
            vec![
                "trimmed_reads_r1".to_string(),
                "trimmed_reads_r2".to_string(),
                "report_json".to_string(),
            ]
        );
    }
}
