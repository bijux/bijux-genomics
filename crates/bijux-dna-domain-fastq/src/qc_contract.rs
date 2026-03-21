use bijux_dna_core::ids::StageId;

type GovernedQcArtifactRecord = (&'static str, &'static [&'static str]);

const GOVERNED_QC_ARTIFACT_IDS: &[GovernedQcArtifactRecord] = &[
    (
        "fastq.validate_reads",
        &["validation_report", "validated_reads_manifest"],
    ),
    (
        "fastq.detect_adapters",
        &["adapter_report", "adapter_evidence_dir"],
    ),
    (
        "fastq.profile_read_lengths",
        &["length_distribution_tsv", "length_distribution_json"],
    ),
    (
        "fastq.profile_overrepresented_sequences",
        &[
            "overrepresented_sequences_tsv",
            "overrepresented_sequences_json",
        ],
    ),
    ("fastq.profile_reads", &["qc_json", "qc_tsv", "qc_plots_dir"]),
    ("fastq.deplete_rrna", &["rrna_report_tsv", "rrna_report_json"]),
    (
        "fastq.screen_taxonomy",
        &["screen_report_tsv", "classification_report_json"],
    ),
    ("fastq.trim_reads", &["report_json"]),
    ("fastq.merge_pairs", &["report_json"]),
    ("fastq.remove_duplicates", &["report_json"]),
    ("fastq.filter_low_complexity", &["filter_report_json"]),
    ("fastq.deplete_host", &["host_depletion_report_json"]),
    (
        "fastq.deplete_reference_contaminants",
        &["contaminant_screen_report_json"],
    ),
    ("fastq.correct_errors", &["report_json"]),
    ("fastq.trim_terminal_damage", &["report_json"]),
    ("fastq.trim_polyg_tails", &["report_json"]),
    ("fastq.extract_umis", &["report_json"]),
];

#[must_use]
pub fn governed_qc_output_ids_for_stage(stage_id: &StageId) -> Vec<&'static str> {
    GOVERNED_QC_ARTIFACT_IDS
        .iter()
        .find(|(candidate_stage_id, _)| *candidate_stage_id == stage_id.as_str())
        .map(|(_, artifact_ids)| artifact_ids.to_vec())
        .unwrap_or_default()
}

#[must_use]
pub fn governed_qc_producer_stage_ids() -> Vec<StageId> {
    GOVERNED_QC_ARTIFACT_IDS
        .iter()
        .map(|(stage_id, _)| StageId::new(*stage_id))
        .collect()
}
