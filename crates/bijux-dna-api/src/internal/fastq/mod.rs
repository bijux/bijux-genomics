//! Internal FASTQ wiring (non-public).

pub(crate) mod stage_ids;
pub(crate) mod stages;

#[allow(dead_code)]
pub(crate) const FASTQ_INTERNAL_BANNER: &str = "bijux-dna-api internal fastq";
