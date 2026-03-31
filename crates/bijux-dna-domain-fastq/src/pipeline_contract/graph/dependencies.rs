use std::collections::BTreeSet;

#[must_use]
pub fn first_present_stage<'a>(
    present: &BTreeSet<String>,
    candidates: &'a [&'a str],
) -> Option<&'a str> {
    candidates
        .iter()
        .copied()
        .find(|stage_id| present.contains(*stage_id))
}

#[must_use]
pub fn primary_upstream_candidates(stage_id: &str) -> &'static [&'static str] {
    match stage_id {
        "fastq.profile_read_lengths" | "fastq.detect_adapters" => &["fastq.validate_reads"],
        "fastq.trim_polyg_tails" => &["fastq.detect_adapters", "fastq.validate_reads"],
        "fastq.trim_terminal_damage" => &[
            "fastq.trim_polyg_tails",
            "fastq.detect_adapters",
            "fastq.validate_reads",
        ],
        "fastq.normalize_primers" => &[
            "fastq.trim_terminal_damage",
            "fastq.detect_adapters",
            "fastq.validate_reads",
        ],
        "fastq.trim_reads" => &[
            "fastq.normalize_primers",
            "fastq.trim_terminal_damage",
            "fastq.trim_polyg_tails",
            "fastq.detect_adapters",
            "fastq.validate_reads",
        ],
        "fastq.filter_reads" => &[
            "fastq.trim_reads",
            "fastq.normalize_primers",
            "fastq.trim_terminal_damage",
            "fastq.validate_reads",
        ],
        "fastq.correct_errors" | "fastq.extract_umis" | "fastq.deplete_rrna" => &[
            "fastq.filter_reads",
            "fastq.trim_reads",
            "fastq.validate_reads",
        ],
        "fastq.profile_reads"
        | "fastq.profile_overrepresented_sequences"
        | "fastq.screen_taxonomy" => &[
            "fastq.filter_low_complexity",
            "fastq.remove_duplicates",
            "fastq.merge_pairs",
            "fastq.deplete_rrna",
            "fastq.deplete_reference_contaminants",
            "fastq.deplete_host",
            "fastq.correct_errors",
            "fastq.filter_reads",
            "fastq.trim_reads",
            "fastq.validate_reads",
        ],
        "fastq.merge_pairs" | "fastq.remove_chimeras" => {
            &["fastq.filter_reads", "fastq.trim_reads"]
        }
        "fastq.remove_duplicates" => &[
            "fastq.merge_pairs",
            "fastq.filter_reads",
            "fastq.trim_reads",
        ],
        "fastq.filter_low_complexity" => &[
            "fastq.remove_duplicates",
            "fastq.filter_reads",
            "fastq.trim_reads",
        ],
        "fastq.cluster_otus" => &["fastq.remove_chimeras", "fastq.filter_reads"],
        "fastq.normalize_abundance" => &["fastq.cluster_otus"],
        _ => &[],
    }
}
