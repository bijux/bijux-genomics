//! FASTQ stage wiring (internal).

pub(crate) mod correct;
pub(crate) mod filter_reads;
pub(crate) mod merge;
pub(crate) mod preprocess;
pub(crate) mod profile_reads;
pub(crate) mod report_qc;
pub(crate) mod screen_taxonomy;
pub(crate) mod trim_reads;
pub(crate) mod umi;
pub(crate) mod validate_reads;
