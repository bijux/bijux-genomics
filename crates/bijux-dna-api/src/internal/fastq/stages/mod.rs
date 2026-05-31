//! FASTQ stage wiring (internal).

#[allow(dead_code)]
pub(crate) const STAGE_MODULES: &[&str] = &[
    "cluster_otus",
    "correct_errors",
    "deplete_host",
    "deplete_reference_contaminants",
    "deplete_rrna",
    "detect_adapters",
    "detect_duplicates_premerge",
    "extract_umis",
    "filter_low_complexity",
    "filter_reads",
    "index_reference",
    "infer_asvs",
    "merge_pairs",
    "normalize_abundance",
    "normalize_primers",
    "preprocess",
    "profile_overrepresented_sequences",
    "profile_read_lengths",
    "profile_reads",
    "record_identity",
    "remove_chimeras",
    "remove_duplicates",
    "report_qc",
    "screen_taxonomy",
    "trim_bench_common",
    "trim_polyg_tails",
    "trim_reads",
    "trim_terminal_damage",
    "validate_reads",
];

pub(crate) mod cluster_otus;
pub(crate) mod correct_errors;
pub(crate) mod deplete_host;
pub(crate) mod deplete_reference_contaminants;
pub(crate) mod deplete_rrna;
pub(crate) mod detect_adapters;
pub(crate) mod detect_duplicates_premerge;
pub(crate) mod extract_umis;
pub(crate) mod filter_low_complexity;
pub(crate) mod filter_reads;
pub(crate) mod index_reference;
pub(crate) mod infer_asvs;
pub(crate) mod merge_pairs;
pub(crate) mod normalize_abundance;
pub(crate) mod normalize_primers;
pub(crate) mod preprocess;
pub(crate) mod profile_overrepresented_sequences;
pub(crate) mod profile_read_lengths;
pub(crate) mod profile_reads;
pub(crate) mod record_identity;
pub(crate) mod remove_chimeras;
pub(crate) mod remove_duplicates;
pub(crate) mod report_qc;
pub(crate) mod screen_taxonomy;
pub(crate) mod trim_bench_common;
pub(crate) mod trim_polyg_tails;
pub(crate) mod trim_reads;
pub(crate) mod trim_terminal_damage;
pub(crate) mod validate_reads;
