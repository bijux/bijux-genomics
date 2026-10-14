//! Benchmark taxonomy expectations and judgment projections.
use bijux_dna_bench_corpus::edna::{
    EdnaCorpusFixtureManifest, EdnaExpectedPresence, EdnaExpectedTaxonRow,
};
use serde::Serialize;
use std::collections::BTreeMap;

pub const BENCHMARK_TAXONOMY_OUTPUT_JUDGMENT_SCHEMA_VERSION: &str =
    "bijux.bench.local_taxonomy_output_judgment.v1";

#[derive(Debug, Clone, Serialize)]
pub struct BenchmarkTaxonomyObservedTaxon {
    pub name: String,
    pub percent: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct BenchmarkTaxonomyExpectationRowJudgment {
    pub sample_id: String,
    pub taxon_id: u64,
    pub name: String,
    pub rank: String,
    pub expected_presence: EdnaExpectedPresence,
    pub observed_presence: bool,
    pub matched: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct BenchmarkTaxonomySampleJudgment {
    pub sample_id: String,
    pub report_path: String,
    pub observed_taxa: Vec<BenchmarkTaxonomyObservedTaxon>,
    pub observed_unclassified_percent: f64,
    pub observed_unclassified_read_count: u64,
    pub expectation_count: usize,
    pub matched_expectation_count: usize,
    pub mismatched_expectation_count: usize,
    pub false_positive_count: usize,
    pub unexpected_taxa: Vec<BenchmarkTaxonomyObservedTaxon>,
    pub valid: bool,
    pub rows: Vec<BenchmarkTaxonomyExpectationRowJudgment>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BenchmarkTaxonomyOutputJudgmentReport {
    pub schema_version: &'static str,
    pub manifest_path: String,
    pub expected_taxa_path: String,
    pub sample_count: usize,
    pub expectation_count: usize,
    pub matched_expectation_count: usize,
    pub mismatched_expectation_count: usize,
    pub valid: bool,
    pub samples: Vec<BenchmarkTaxonomySampleJudgment>,
}

/// A decoded classifier summary, not full stage-report or biological admission.
#[derive(Debug, Clone)]
pub struct AdmittedBenchmarkTaxonomyObservation {
    pub(crate) taxa: Vec<BenchmarkTaxonomyObservedTaxon>,
    pub(crate) unclassified_percent: f64,
}

/// Corpus-admitted expectations and acquired summary observations for pure evaluation.
/// Source labels are presentation identities; evaluation does not inspect their files.
#[derive(Debug)]
pub struct BenchmarkTaxonomyJudgmentInput<'a> {
    pub manifest_path: &'a str,
    pub expected_taxa_path: &'a str,
    pub manifest: &'a EdnaCorpusFixtureManifest,
    pub expected_rows: &'a [EdnaExpectedTaxonRow],
    pub observed_reports: BTreeMap<String, (String, AdmittedBenchmarkTaxonomyObservation)>,
}
