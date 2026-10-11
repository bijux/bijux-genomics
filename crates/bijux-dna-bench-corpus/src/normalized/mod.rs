//! Normalized FASTQ discovery, source identity, publication cohorts and bounded content admission.
//! Read-record semantics remain a separate domain contract.
mod cohorts;
mod discovery;
mod loading;
mod manifest;
mod metadata;
mod model;
pub use cohorts::{
    expected_fastq_publication_cohort_counts, normalized_fastq_expected_sample_total,
    select_normalized_fastq_paired_samples,
};
pub use discovery::discover_normalized_fastq_samples;
pub use loading::{load_fastq_corpus_source_for_id, load_fastq_corpus_source_spec};
pub use metadata::match_normalized_fastq_corpus_metadata;
pub use model::NormalizedFastqSample;
