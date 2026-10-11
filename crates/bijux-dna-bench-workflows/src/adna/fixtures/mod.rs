//! Owner: bijux-dna-bench-workflows
//! adna fixtures responsibilities.
mod reads;
pub(crate) use reads::{
    materialize_adna_pipeline_input_fastqs, read_named_reference_contig,
    write_mapped_sam_from_fastqs,
};
