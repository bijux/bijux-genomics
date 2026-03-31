use bijux_dna_core::ids::StageId;

#[must_use]
pub fn canonical_stage_order() -> Vec<StageId> {
    vec![
        StageId::from_static("fastq.validate_reads"),
        StageId::from_static("fastq.profile_read_lengths"),
        StageId::from_static("fastq.detect_adapters"),
        StageId::from_static("fastq.trim_polyg_tails"),
        StageId::from_static("fastq.trim_terminal_damage"),
        StageId::from_static("fastq.trim_reads"),
        StageId::from_static("fastq.filter_reads"),
        StageId::from_static("fastq.correct_errors"),
        StageId::from_static("fastq.extract_umis"),
        StageId::from_static("fastq.index_reference"),
        StageId::from_static("fastq.deplete_host"),
        StageId::from_static("fastq.deplete_reference_contaminants"),
        StageId::from_static("fastq.deplete_rrna"),
        StageId::from_static("fastq.merge_pairs"),
        StageId::from_static("fastq.remove_duplicates"),
        StageId::from_static("fastq.filter_low_complexity"),
        StageId::from_static("fastq.profile_reads"),
        StageId::from_static("fastq.profile_overrepresented_sequences"),
        StageId::from_static("fastq.screen_taxonomy"),
        StageId::from_static("fastq.report_qc"),
    ]
}

#[must_use]
pub fn canonical_amplicon_stage_order() -> Vec<StageId> {
    vec![
        StageId::from_static("fastq.validate_reads"),
        StageId::from_static("fastq.profile_read_lengths"),
        StageId::from_static("fastq.detect_adapters"),
        StageId::from_static("fastq.trim_terminal_damage"),
        StageId::from_static("fastq.normalize_primers"),
        StageId::from_static("fastq.trim_reads"),
        StageId::from_static("fastq.filter_reads"),
        StageId::from_static("fastq.correct_errors"),
        StageId::from_static("fastq.extract_umis"),
        StageId::from_static("fastq.remove_chimeras"),
        StageId::from_static("fastq.cluster_otus"),
        StageId::from_static("fastq.normalize_abundance"),
        StageId::from_static("fastq.profile_reads"),
        StageId::from_static("fastq.profile_overrepresented_sequences"),
        StageId::from_static("fastq.screen_taxonomy"),
        StageId::from_static("fastq.report_qc"),
    ]
}

#[must_use]
pub fn default_shotgun_preprocess_stage_order() -> Vec<StageId> {
    vec![
        StageId::from_static("fastq.validate_reads"),
        StageId::from_static("fastq.profile_read_lengths"),
        StageId::from_static("fastq.detect_adapters"),
        StageId::from_static("fastq.trim_polyg_tails"),
        StageId::from_static("fastq.trim_terminal_damage"),
        StageId::from_static("fastq.trim_reads"),
        StageId::from_static("fastq.filter_reads"),
        StageId::from_static("fastq.profile_reads"),
        StageId::from_static("fastq.profile_overrepresented_sequences"),
        StageId::from_static("fastq.report_qc"),
    ]
}

#[must_use]
pub fn default_amplicon_preprocess_stage_order() -> Vec<StageId> {
    vec![
        StageId::from_static("fastq.validate_reads"),
        StageId::from_static("fastq.profile_read_lengths"),
        StageId::from_static("fastq.detect_adapters"),
        StageId::from_static("fastq.trim_terminal_damage"),
        StageId::from_static("fastq.normalize_primers"),
        StageId::from_static("fastq.trim_reads"),
        StageId::from_static("fastq.filter_reads"),
        StageId::from_static("fastq.remove_chimeras"),
        StageId::from_static("fastq.cluster_otus"),
        StageId::from_static("fastq.normalize_abundance"),
        StageId::from_static("fastq.profile_reads"),
        StageId::from_static("fastq.profile_overrepresented_sequences"),
        StageId::from_static("fastq.report_qc"),
    ]
}
