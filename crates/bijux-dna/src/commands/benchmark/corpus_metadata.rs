use std::path::Path;

use crate::commands::benchmark_workspace::benchmark_corpus_spec_path;
use anyhow::Result;
use bijux_dna_api::v1::api::bench::load_fastq_corpus_source_for_id;

pub(crate) use bijux_dna_api::v1::api::bench::{
    discover_normalized_fastq_samples as discover_normalized_samples,
    expected_fastq_publication_cohort_counts as expected_counts_for_scope,
    match_normalized_fastq_corpus_metadata as validate_corpus_contract,
    normalized_fastq_expected_sample_total as corpus_expected_sample_total,
    select_normalized_fastq_paired_samples as select_paired_samples,
    FastqCorpusSourceSample as CorpusSpecSample, FastqCorpusSourceSpec as CorpusSpec,
    NormalizedFastqSample as CorpusNormalizedSample,
};

pub(crate) fn load_corpus_spec(
    repo_root: &Path,
    config_path: Option<&Path>,
    corpus_id: &str,
) -> Result<CorpusSpec> {
    let path = benchmark_corpus_spec_path(repo_root, config_path, corpus_id)?;
    load_fastq_corpus_source_for_id(&path, corpus_id)
}
