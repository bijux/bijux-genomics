//! FASTQ stage wiring (internal).

pub(crate) mod correct_errors;
pub(crate) mod deplete_host;
pub(crate) mod deplete_rrna;
pub(crate) mod deplete_reference_contaminants;
pub(crate) mod detect_adapters;
pub(crate) mod filter_reads;
pub(crate) mod filter_low_complexity;
pub(crate) mod infer_asvs;
pub(crate) mod index_reference;
pub(crate) mod merge_pairs;
pub(crate) mod normalize_abundance;
pub(crate) mod normalize_primers;
pub(crate) mod preprocess;
pub(crate) mod profile_read_lengths;
pub(crate) mod profile_overrepresented_sequences;
pub(crate) mod profile_reads;
pub(crate) mod remove_chimeras;
pub(crate) mod remove_duplicates;
pub(crate) mod report_qc;
pub(crate) mod screen_taxonomy;
pub(crate) mod trim_bench_common;
pub(crate) mod trim_polyg_tails;
pub(crate) mod trim_reads;
pub(crate) mod trim_terminal_damage;
pub(crate) mod extract_umis;
pub(crate) mod validate_reads;
