//! Owner: bijux-dna-bench-workflows
//! amplicon stages responsibilities.
mod amplicon_corpus_fixture;
pub(crate) use amplicon_corpus_fixture::run_amplicon_corpus_fixture_stage;
mod amplicon_truth_fixture;
pub(crate) use amplicon_truth_fixture::run_amplicon_truth_fixture_stage;
mod normalize_primers;
pub(crate) use normalize_primers::run_normalize_primers_stage;
mod infer_asvs;
pub(crate) use infer_asvs::run_infer_asvs_stage;
mod remove_chimeras;
pub(crate) use remove_chimeras::run_remove_chimeras_stage;
mod cluster_otus;
pub(crate) use cluster_otus::run_cluster_otus_stage;
mod normalize_abundance;
pub(crate) use normalize_abundance::run_normalize_abundance_stage;
mod amplicon_output_judgment;
pub(crate) use amplicon_output_judgment::run_amplicon_output_judgment_stage;
