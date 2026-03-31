mod manifest;
mod queries;

pub use queries::{
    stage_compatible_tool_ids, stage_input_ids, stage_output_ids,
    stage_output_ids_in_manifest_order, stage_parameter_ids,
};

#[cfg(test)]
mod tests {
    use super::{
        stage_compatible_tool_ids, stage_input_ids, stage_output_ids,
        stage_output_ids_in_manifest_order, stage_parameter_ids,
    };

    #[test]
    fn stage_ports_follow_governed_manifest_names() {
        assert_eq!(
            stage_input_ids("fastq.report_qc"),
            Some(["qc_artifacts"].into_iter().map(str::to_string).collect())
        );
        assert_eq!(
            stage_output_ids("fastq.trim_reads"),
            Some(
                [
                    "trimmed_reads_r1",
                    "trimmed_reads_r2",
                    "report_json",
                    "raw_backend_report_json",
                    "raw_backend_report_txt",
                ]
                .into_iter()
                .map(str::to_string)
                .collect()
            )
        );
        assert_eq!(
            stage_output_ids("fastq.trim_polyg_tails"),
            Some(
                [
                    "trimmed_reads_r1",
                    "trimmed_reads_r2",
                    "report_json",
                    "raw_backend_report_json",
                    "raw_backend_report_txt",
                ]
                .into_iter()
                .map(str::to_string)
                .collect()
            )
        );
        assert_eq!(
            stage_output_ids_in_manifest_order("fastq.report_qc"),
            Some(vec![
                "report_json".to_string(),
                "multiqc_report".to_string(),
                "multiqc_data".to_string(),
                "governed_qc_inputs_manifest".to_string()
            ])
        );
        assert_eq!(
            stage_output_ids_in_manifest_order("fastq.remove_chimeras"),
            Some(vec![
                "chimera_filtered_reads".to_string(),
                "report_json".to_string(),
                "chimera_metrics_json".to_string(),
                "chimeras_fasta".to_string(),
                "uchime_report_tsv".to_string(),
            ])
        );
        assert_eq!(
            stage_parameter_ids("fastq.trim_reads"),
            Some(
                [
                    "threads",
                    "min_length",
                    "quality_cutoff",
                    "adapter_policy",
                    "polyx_policy",
                    "n_policy",
                    "contaminant_policy",
                ]
                .into_iter()
                .map(str::to_string)
                .collect()
            )
        );
        assert_eq!(
            stage_parameter_ids("fastq.trim_polyg_tails"),
            Some(
                ["threads", "trim_polyg", "min_polyg_run"]
                    .into_iter()
                    .map(str::to_string)
                    .collect()
            )
        );
        assert_eq!(
            stage_compatible_tool_ids("fastq.remove_duplicates"),
            Some(vec!["fastuniq".to_string(), "clumpify".to_string()])
        );
    }
}
