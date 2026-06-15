use crate::stages::ports::stage_output_ids_in_manifest_order;

pub(super) fn comparison_input_artifact_ids_for_manifest_stage(stage_id: &str) -> Vec<String> {
    let mut artifact_ids = stage_output_ids_in_manifest_order(stage_id)
        .unwrap_or_default()
        .into_iter()
        .filter(|artifact_id| !artifact_id.starts_with("raw_backend_report"))
        .collect::<Vec<_>>();
    prioritize_provenance_artifact(stage_id, &mut artifact_ids);
    artifact_ids
}

fn prioritize_provenance_artifact(stage_id: &str, artifact_ids: &mut Vec<String>) {
    let prioritized_artifact_ids: &[&str] = match stage_id {
        "fastq.validate_reads" => &["validation_report", "validated_reads_manifest"],
        "fastq.filter_reads" => &["report_json", "filtered_reads_r1", "filtered_reads_r2"],
        "fastq.filter_low_complexity" => {
            &["filter_report_json", "filtered_fastq_r1", "filtered_fastq_r2"]
        }
        "fastq.merge_pairs" => {
            &["report_json", "merged_reads", "unmerged_reads_r1", "unmerged_reads_r2"]
        }
        "fastq.profile_read_lengths" => {
            &["report_json", "length_distribution_tsv", "length_distribution_json"]
        }
        "fastq.profile_overrepresented_sequences" => {
            &["report_json", "overrepresented_sequences_tsv", "overrepresented_sequences_json"]
        }
        "fastq.profile_reads" => &["qc_json", "qc_tsv"],
        "fastq.screen_taxonomy" => &["classification_report_json", "screen_report_tsv"],
        "fastq.normalize_primers" => {
            &["report_json", "primer_orientation_report", "primer_stats_json"]
        }
        "fastq.infer_asvs" => &[
            "report_json",
            "asv_table_tsv",
            "asv_sequences_fasta",
            "taxonomy_ready_fasta",
            "taxonomy_ready_fastq",
        ],
        "fastq.cluster_otus" => &[
            "report_json",
            "otu_table",
            "otu_representatives",
            "taxonomy_ready_fasta",
            "taxonomy_ready_fastq",
        ],
        "fastq.index_reference" => &["report_json", "reference_index"],
        "fastq.detect_adapters" => &["report_json", "adapter_evidence_dir"],
        "fastq.deplete_host" => &[
            "host_depletion_report_json",
            "host_depleted_reads_r1",
            "host_depleted_reads_r2",
            "removed_host_reads_r1",
            "removed_host_reads_r2",
        ],
        "fastq.deplete_rrna" => &[
            "rrna_report_json",
            "rrna_report_tsv",
            "rrna_filtered_reads_r1",
            "rrna_filtered_reads_r2",
            "rrna_removed_reads_r1",
            "rrna_removed_reads_r2",
        ],
        "fastq.deplete_reference_contaminants" => &[
            "contaminant_screen_report_json",
            "contaminant_screened_reads_r1",
            "contaminant_screened_reads_r2",
            "removed_contaminant_reads_r1",
            "removed_contaminant_reads_r2",
        ],
        "fastq.correct_errors" => &["report_json", "corrected_reads_r1", "corrected_reads_r2"],
        "fastq.normalize_abundance" => &["report_json", "normalized_abundance_tsv"],
        "fastq.extract_umis" => &["report_json", "umi_reads_r1", "umi_reads_r2"],
        "fastq.remove_duplicates" => {
            &["report_json", "duplicate_provenance_json", "duplicate_classes_tsv"]
        }
        "fastq.remove_chimeras" => &["report_json", "uchime_report_tsv", "chimera_metrics_json"],
        "fastq.report_qc" => {
            &["report_json", "governed_qc_inputs_manifest", "multiqc_report", "multiqc_data"]
        }
        _ => &[],
    };
    if prioritized_artifact_ids.is_empty() {
        return;
    }
    for prioritized_artifact_id in prioritized_artifact_ids.iter().rev() {
        let Some(position) =
            artifact_ids.iter().position(|artifact_id| artifact_id == prioritized_artifact_id)
        else {
            continue;
        };
        let artifact = artifact_ids.remove(position);
        artifact_ids.insert(0, artifact);
    }
}
