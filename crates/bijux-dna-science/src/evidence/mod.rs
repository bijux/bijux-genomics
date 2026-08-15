//! Scientific evidence derivation from governed observations.

mod fastq_read_lengths;
mod genotypes;

pub use fastq_read_lengths::{
    derive_fastq_read_length_truth, summarize_fastq_read_length_histogram, FastqInputLayout,
    FastqReadLengthSummary, FastqReadLengthTruth,
};
pub use genotypes::genotypes_equivalent_unphased;
