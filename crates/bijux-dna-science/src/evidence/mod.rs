//! Scientific evidence derivation from governed observations.

mod fastq_read_lengths;

pub use fastq_read_lengths::{
    derive_fastq_read_length_truth, FastqInputLayout, FastqReadLengthTruth,
};
