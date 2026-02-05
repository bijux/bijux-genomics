pub mod correct;
pub mod filter;
pub mod merge;
pub mod preprocess;
pub mod qc_post;
pub mod screen;
pub mod trim;
pub mod umi;
pub mod validate_pre;

// Non-empty module marker for guardrails.
pub const FASTQ_STAGE_MODULES: usize = 10;
