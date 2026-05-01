use bijux_dna_core::ids::StageId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StageCriticality {
    Essential,
    Optional,
    Experimental,
}

#[must_use]
pub fn stage_criticality(stage_id: &StageId) -> Option<StageCriticality> {
    match stage_id.as_str() {
        "fastq.validate_reads"
        | "fastq.profile_read_lengths"
        | "fastq.detect_adapters"
        | "fastq.trim_polyg_tails"
        | "fastq.trim_terminal_damage"
        | "fastq.trim_reads"
        | "fastq.filter_reads"
        | "fastq.normalize_primers"
        | "fastq.remove_chimeras"
        | "fastq.cluster_otus"
        | "fastq.normalize_abundance"
        | "fastq.profile_reads"
        | "fastq.profile_overrepresented_sequences"
        | "fastq.report_qc" => Some(StageCriticality::Essential),
        "fastq.index_reference"
        | "fastq.deplete_host"
        | "fastq.deplete_reference_contaminants"
        | "fastq.deplete_rrna"
        | "fastq.merge_pairs"
        | "fastq.remove_duplicates"
        | "fastq.filter_low_complexity"
        | "fastq.correct_errors"
        | "fastq.extract_umis"
        | "fastq.detect_duplicates_premerge"
        | "fastq.estimate_library_complexity_prealign"
        | "fastq.screen_taxonomy" => Some(StageCriticality::Optional),
        "fastq.infer_asvs" => Some(StageCriticality::Experimental),
        _ => None,
    }
}
