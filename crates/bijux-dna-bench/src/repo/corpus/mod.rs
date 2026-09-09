//! Owner: bijux-dna-bench
//! Domain-specific corpus admission and normalized read metadata.

mod bam;
mod fastq;
mod normalized_fastq;

pub use bam::{
    admit_bam_benchmark_cohort, admit_bam_benchmark_corpus, admit_bam_genotyping_corpus,
    admit_benchmark_indexed_reference, admit_benchmark_reference, AdmittedBamGenotypingCorpus,
};
pub use fastq::{admit_alignment_truth_set, admit_fastq_benchmark_corpus};
pub use normalized_fastq::{
    discover_normalized_fastq_samples, expected_fastq_publication_cohort_counts,
    load_fastq_corpus_source_for_id, match_normalized_fastq_corpus_metadata,
    normalized_fastq_expected_sample_total, select_normalized_fastq_paired_samples,
    NormalizedFastqSample,
};
