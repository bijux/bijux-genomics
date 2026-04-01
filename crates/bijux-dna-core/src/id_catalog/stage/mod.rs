mod core;
mod prefixes;
mod report;

pub use core::CORE_PREPARE_REFERENCE;
pub use prefixes::{BAM_PREFIX, CORE_PREFIX, FASTQ_PREFIX, VCF_PREFIX};
pub use report::{REPORT_AGGREGATE_STAGE, REPORT_AGGREGATE_STEP};
