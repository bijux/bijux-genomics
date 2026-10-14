//! Pure abundance-table transforms and observations for application artifact composition.
//! Acquisition, publication, backend execution, and benchmark persistence belong to callers.

use crate::params::edna::AbundanceNormalizationEffectiveParams;
use anyhow::{anyhow, Result};
use std::collections::{BTreeMap, BTreeSet};

#[cfg(test)]
mod contracts;

/// Numeric observations of an abundance table in declared sample and feature units.
///
/// Stability: stable domain contract.
#[derive(Debug, Clone, PartialEq)]
pub struct AbundanceTableObservation {
    pub table_rows: u64,
    pub sample_count: u64,
    pub feature_count: u64,
    pub zero_fraction: f64,
    pub per_sample_sums: Vec<(String, f64)>,
}

/// Normalized TSV bytes and their observations before caller-owned publication.
///
/// Stability: stable domain contract.
#[derive(Debug, Clone, PartialEq)]
pub struct AbundanceTableNormalization {
    pub table_tsv: String,
    pub observation: AbundanceTableObservation,
}

/// Normalize table text using its declared columns and scaling parameters.
///
/// Samples are sorted; rows within a sample retain their input order, including
/// duplicates. Malformed numeric cells become zero and non-positive sample totals
/// produce zero rows. Numeric output uses six decimal places; observations retain
/// the arithmetic values before formatting. Method labels remain metadata.
///
/// # Errors
/// Returns an error if a present input header differs from the declared columns.
pub fn normalize_abundance_table_text(
    raw: &str,
    effective_params: &AbundanceNormalizationEffectiveParams,
) -> Result<AbundanceTableNormalization> {
    let expected_header = effective_params.expected_columns.join("\t");
    if let Some(header) = raw.lines().next() {
        if header.trim() != expected_header {
            return Err(anyhow!(
                "abundance table header mismatch: expected `{expected_header}`, got `{header}`"
            ));
        }
    }
    let mut by_sample = BTreeMap::<String, Vec<(String, f64)>>::new();
    let mut feature_ids = BTreeSet::<String>::new();
    for line in raw.lines().skip(1) {
        let cols = line.split('\t').collect::<Vec<_>>();
        if cols.len() < 3 {
            continue;
        }
        let sample = cols[0].trim().to_string();
        let feature = cols[1].trim().to_string();
        let abundance = cols[2].trim().parse::<f64>().unwrap_or(0.0);
        feature_ids.insert(feature.clone());
        by_sample.entry(sample).or_default().push((feature, abundance));
    }
    let mut out = format!("sample_id\tfeature_id\t{}\n", effective_params.normalized_value_column);
    let scale_factor = effective_params.scale_factor.unwrap_or(1.0);
    let sample_count = by_sample.len() as u64;
    let mut table_rows = 0_u64;
    let mut zeros = 0_u64;
    let mut per_sample_sums = Vec::new();
    for (sample, rows) in by_sample {
        let total = rows.iter().map(|(_, abundance)| *abundance).sum::<f64>();
        let mut sample_sum = 0.0_f64;
        for (feature, abundance) in rows {
            let normalized = if total > 0.0 { (abundance * scale_factor) / total } else { 0.0 };
            table_rows += 1;
            if normalized == 0.0 {
                zeros += 1;
            }
            sample_sum += normalized;
            out.push_str(&sample);
            out.push('\t');
            out.push_str(&feature);
            out.push('\t');
            let normalized_text = format!("{normalized:.6}");
            out.push_str(&normalized_text);
            out.push('\n');
        }
        per_sample_sums.push((sample, sample_sum));
    }
    let observation = AbundanceTableObservation {
        table_rows,
        sample_count,
        feature_count: feature_ids.len() as u64,
        zero_fraction: if table_rows == 0 {
            0.0
        } else {
            u64_to_f64(zeros) / u64_to_f64(table_rows)
        },
        per_sample_sums,
    };
    Ok(AbundanceTableNormalization { table_tsv: out, observation })
}

/// Observe normalized text without changing rows, values, or declared units.
///
/// Malformed numeric cells become zero, short rows are skipped, and duplicate
/// rows contribute individually to observations. Sample totals are sorted.
///
/// # Errors
/// Returns an error if a present output header differs from the declared column.
pub fn observe_normalized_abundance_table_text(
    raw: &str,
    effective_params: &AbundanceNormalizationEffectiveParams,
) -> Result<AbundanceTableObservation> {
    let expected_header =
        format!("sample_id\tfeature_id\t{}", effective_params.normalized_value_column);
    if let Some(header) = raw.lines().next() {
        if header.trim() != expected_header {
            return Err(anyhow!(
                "normalized abundance table header mismatch: expected `{expected_header}`, got `{header}`"
            ));
        }
    }
    let mut rows = 0_u64;
    let mut zeros = 0_u64;
    let mut samples = BTreeMap::<String, f64>::new();
    let mut feature_ids = BTreeSet::<String>::new();
    for line in raw.lines().skip(1) {
        let cols = line.split('\t').collect::<Vec<_>>();
        if cols.len() < 3 {
            continue;
        }
        rows += 1;
        feature_ids.insert(cols[1].trim().to_string());
        let value = cols[2].trim().parse::<f64>().unwrap_or(0.0);
        *samples.entry(cols[0].trim().to_string()).or_default() += value;
        if value == 0.0 {
            zeros += 1;
        }
    }
    Ok(AbundanceTableObservation {
        table_rows: rows,
        sample_count: samples.len() as u64,
        feature_count: feature_ids.len() as u64,
        zero_fraction: if rows == 0 { 0.0 } else { u64_to_f64(zeros) / u64_to_f64(rows) },
        per_sample_sums: samples.into_iter().collect(),
    })
}

fn u64_to_f64(value: u64) -> f64 {
    value.to_string().parse::<f64>().unwrap_or(0.0)
}
