//! Classifier-summary wire admission for benchmark judgment.
use crate::{AdmittedBenchmarkTaxonomyObservation, BenchmarkTaxonomyObservedTaxon};
use anyhow::{anyhow, Context, Result};
use std::collections::BTreeSet;

/// Decode supported classifier-summary aliases with deterministic taxon ordering.
/// This checks summary grammar and does not admit the full normalized stage report.
///
/// # Errors
/// Rejects malformed JSON, missing summary/label/percent fields, invalid percentages,
/// duplicate taxon labels and duplicate unclassified entries.
pub fn decode_benchmark_taxonomy_observation(
    raw: &str,
    source_label: &str,
) -> Result<AdmittedBenchmarkTaxonomyObservation> {
    let payload: serde_json::Value =
        serde_json::from_str(raw).with_context(|| format!("parse {source_label}"))?;
    let entries = payload
        .get("summary_entries")
        .or_else(|| payload.get("top_taxa"))
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| {
            anyhow!(
                "taxonomy output judgment report {source_label} must declare a `summary_entries` or `top_taxa` array"
            )
        })?;

    let mut seen_names = BTreeSet::new();
    let mut unclassified_percent = None::<f64>;
    let taxa = entries
        .iter()
        .map(|entry| {
            let name = entry
                .get("label")
                .or_else(|| entry.get("name"))
                .and_then(serde_json::Value::as_str)
                .map(str::trim)
                .ok_or_else(|| {
                    anyhow!(
                        "taxonomy output judgment report {source_label} contains an entry without a string `label` or `name`"
                    )
                })?;
            if name.is_empty() {
                return Err(anyhow!(
                    "taxonomy output judgment report {source_label} contains an empty taxon label"
                ));
            }
            let percent = entry
                .get("percent")
                .and_then(serde_json::Value::as_f64)
                .ok_or_else(|| {
                    anyhow!(
                        "taxonomy output judgment report {source_label} taxon `{name}` is missing numeric `percent`"
                    )
                })?;
            if !percent.is_finite() || percent.is_sign_negative() {
                return Err(anyhow!(
                    "taxonomy output judgment report {source_label} taxon `{name}` has invalid percent {percent}"
                ));
            }
            if name.eq_ignore_ascii_case("unclassified") {
                if unclassified_percent.replace(percent).is_some() {
                    return Err(anyhow!(
                        "taxonomy output judgment report {source_label} repeats `unclassified`"
                    ));
                }
                return Ok(None);
            }
            if !seen_names.insert(name.to_string()) {
                return Err(anyhow!(
                    "taxonomy output judgment report {source_label} repeats taxon `{name}`"
                ));
            }
            Ok(Some(BenchmarkTaxonomyObservedTaxon { name: name.to_string(), percent }))
        })
        .collect::<Result<Vec<_>>>()?;
    let mut taxa = taxa.into_iter().flatten().collect::<Vec<_>>();
    taxa.sort_by(|left, right| {
        right.percent.total_cmp(&left.percent).then_with(|| left.name.cmp(&right.name))
    });

    Ok(AdmittedBenchmarkTaxonomyObservation {
        taxa,
        unclassified_percent: unclassified_percent.unwrap_or(0.0),
    })
}
