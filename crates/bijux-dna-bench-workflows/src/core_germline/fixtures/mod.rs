//! Owner: bijux-dna-bench-workflows
//! Core germline fixtures responsibilities.
mod reads;
pub(crate) use reads::{materialize_pipeline_input_fastqs, write_mapped_sam_from_fastqs};
