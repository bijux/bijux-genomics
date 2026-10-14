//! Presence, false-positive and count projections against admitted corpus expectations.
use crate::{
    BenchmarkTaxonomyExpectationRowJudgment, BenchmarkTaxonomyJudgmentInput,
    BenchmarkTaxonomyOutputJudgmentReport, BenchmarkTaxonomySampleJudgment,
    BENCHMARK_TAXONOMY_OUTPUT_JUDGMENT_SCHEMA_VERSION,
};
use anyhow::{anyhow, Result};
use bijux_dna_bench_corpus::edna::{EdnaExpectedPresence, EdnaExpectedTaxonRow};
use bijux_dna_core::prelude::numeric::rounded_f64_to_u64;
use std::collections::BTreeSet;

/// Evaluate decoded observations using corpus-admitted declarations and matrix rows.
/// Declared sample and expected-row order and display labels are retained.
/// Aggregate validity requires every admitted sample judgment to be valid.
///
/// # Errors
/// Rejects a missing declared sample observation or an invalid inferred read count.
pub fn evaluate_benchmark_taxonomy_outputs(
    input: BenchmarkTaxonomyJudgmentInput<'_>,
) -> Result<BenchmarkTaxonomyOutputJudgmentReport> {
    let BenchmarkTaxonomyJudgmentInput {
        manifest_path,
        expected_taxa_path,
        manifest,
        expected_rows,
        mut observed_reports,
    } = input;
    let mut matched_expectation_count = 0usize;
    let mut mismatched_expectation_count = 0usize;
    let mut samples = Vec::with_capacity(manifest.samples.len());

    for sample in &manifest.samples {
        let (report_path, observed) =
            observed_reports.remove(&sample.sample_id).ok_or_else(|| {
                anyhow!("missing observed report for sample_id `{}`", sample.sample_id)
            })?;
        let observed_presence = observed
            .taxa
            .iter()
            .filter(|entry| entry.percent > 0.0)
            .map(|entry| entry.name.as_str())
            .collect::<BTreeSet<_>>();
        let expected_present_taxa = expected_rows
            .iter()
            .filter(|row| {
                row.sample_id == sample.sample_id
                    && matches!(row.expected_presence, EdnaExpectedPresence::Present)
            })
            .map(|row| row.name.as_str())
            .collect::<BTreeSet<_>>();
        let rows = expected_rows
            .iter()
            .filter(|row| row.sample_id == sample.sample_id)
            .map(|row| row_judgment(row, &observed_presence))
            .collect::<Vec<_>>();
        let unexpected_taxa = observed
            .taxa
            .iter()
            .filter(|entry| {
                entry.percent > 0.0 && !expected_present_taxa.contains(entry.name.as_str())
            })
            .cloned()
            .collect::<Vec<_>>();
        let sample_matched = rows.iter().filter(|row| row.matched).count();
        let sample_mismatched = rows.len().saturating_sub(sample_matched);
        let false_positive_count = unexpected_taxa.len();
        let observed_unclassified_read_count = rounded_f64_to_u64(
            (sample.expected_read_count.to_string().parse::<f64>()?
                * observed.unclassified_percent)
                / 100.0,
            "observed unclassified read count",
        )?;
        matched_expectation_count += sample_matched;
        mismatched_expectation_count += sample_mismatched;
        samples.push(BenchmarkTaxonomySampleJudgment {
            sample_id: sample.sample_id.clone(),
            report_path,
            observed_taxa: observed.taxa,
            observed_unclassified_percent: observed.unclassified_percent,
            observed_unclassified_read_count,
            expectation_count: rows.len(),
            matched_expectation_count: sample_matched,
            mismatched_expectation_count: sample_mismatched,
            false_positive_count,
            unexpected_taxa,
            valid: sample_mismatched == 0 && false_positive_count == 0,
            rows,
        });
    }

    Ok(BenchmarkTaxonomyOutputJudgmentReport {
        schema_version: BENCHMARK_TAXONOMY_OUTPUT_JUDGMENT_SCHEMA_VERSION,
        manifest_path: manifest_path.to_string(),
        expected_taxa_path: expected_taxa_path.to_string(),
        sample_count: samples.len(),
        expectation_count: expected_rows.len(),
        matched_expectation_count,
        mismatched_expectation_count,
        valid: samples.iter().all(|sample| sample.valid),
        samples,
    })
}

fn row_judgment(
    row: &EdnaExpectedTaxonRow,
    observed_presence: &BTreeSet<&str>,
) -> BenchmarkTaxonomyExpectationRowJudgment {
    let observed = observed_presence.contains(row.name.as_str());
    let matched = match row.expected_presence {
        EdnaExpectedPresence::Present => observed,
        EdnaExpectedPresence::Absent => !observed,
    };
    BenchmarkTaxonomyExpectationRowJudgment {
        sample_id: row.sample_id.clone(),
        taxon_id: row.taxon_id,
        name: row.name.clone(),
        rank: row.rank.clone(),
        expected_presence: row.expected_presence,
        observed_presence: observed,
        matched,
    }
}
