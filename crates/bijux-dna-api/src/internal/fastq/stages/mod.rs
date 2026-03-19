//! FASTQ stage wiring (internal).

pub(crate) mod correct_errors;
pub(crate) mod filter_reads;
pub(crate) mod merge_pairs;
pub(crate) mod preprocess;
pub(crate) mod profile_reads;
pub(crate) mod report_qc;
pub(crate) mod screen_taxonomy;
pub(crate) mod trim_bench_common;
pub(crate) mod trim_polyg_tails;
pub(crate) mod trim_reads;
pub(crate) mod trim_terminal_damage;
pub(crate) mod extract_umis;
pub(crate) mod validate_reads;
