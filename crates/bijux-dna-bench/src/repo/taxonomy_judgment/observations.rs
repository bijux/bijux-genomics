//! Owner: benchmark classifier-summary acquisition.
//! Read classifier summary bytes and delegate their grammar to the model owner.
use anyhow::{Context, Result};
use bijux_dna_bench_model::{
    decode_benchmark_taxonomy_observation, AdmittedBenchmarkTaxonomyObservation,
};
use std::fs;
use std::path::Path;

pub(super) fn load_observed_taxonomy_report(
    report_path: &Path,
) -> Result<AdmittedBenchmarkTaxonomyObservation> {
    let raw = fs::read_to_string(report_path)
        .with_context(|| format!("read {}", report_path.display()))?;
    decode_benchmark_taxonomy_observation(&raw, &report_path.display().to_string())
}
