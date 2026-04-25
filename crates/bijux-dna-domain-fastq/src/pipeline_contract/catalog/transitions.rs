use bijux_dna_core::ids::StageId;

#[must_use]
pub fn optional_branches() -> Vec<(StageId, Vec<StageId>)> {
    vec![
        (
            StageId::from_static("fastq.merge_pairs"),
            vec![
                StageId::from_static("fastq.trim_reads"),
                StageId::from_static("fastq.filter_reads"),
            ],
        ),
        (
            StageId::from_static("fastq.correct_errors"),
            vec![StageId::from_static("fastq.trim_reads")],
        ),
        (
            StageId::from_static("fastq.extract_umis"),
            vec![StageId::from_static("fastq.validate_reads")],
        ),
        (
            StageId::from_static("fastq.report_qc"),
            vec![StageId::from_static("fastq.validate_reads")],
        ),
        (
            StageId::from_static("fastq.screen_taxonomy"),
            vec![StageId::from_static("fastq.validate_reads")],
        ),
    ]
}

#[must_use]
pub fn forbidden_transitions() -> Vec<(StageId, StageId)> {
    vec![
        (StageId::from_static("fastq.validate_reads"), StageId::from_static("fastq.merge_pairs")),
        (StageId::from_static("fastq.profile_reads"), StageId::from_static("fastq.trim_reads")),
        (StageId::from_static("fastq.profile_reads"), StageId::from_static("fastq.filter_reads")),
        (StageId::from_static("fastq.profile_reads"), StageId::from_static("fastq.merge_pairs")),
        (StageId::from_static("fastq.trim_reads"), StageId::from_static("fastq.extract_umis")),
        (StageId::from_static("fastq.filter_reads"), StageId::from_static("fastq.extract_umis")),
        (StageId::from_static("fastq.infer_asvs"), StageId::from_static("fastq.cluster_otus")),
        (StageId::from_static("fastq.cluster_otus"), StageId::from_static("fastq.infer_asvs")),
    ]
}
