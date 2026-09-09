//! Owner: bijux-dna-bench
//! BAM score report field and numeric projections.

use anyhow::{anyhow, Context, Result};
use bijux_dna_core::prelude::numeric::checked_f64_from_u64;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn required_array<'a>(value: &'a Value, key: &str) -> Result<&'a [Value]> {
    value
        .get(key)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .ok_or_else(|| anyhow!("missing array field `{key}`"))
}

pub(super) fn required_str<'a>(value: &'a Value, key: &str) -> Result<&'a str> {
    value.get(key).and_then(Value::as_str).ok_or_else(|| anyhow!("missing string field `{key}`"))
}

pub(super) fn optional_str<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str)
}

pub(super) fn required_u64(value: &Value, key: &str) -> Result<u64> {
    value.get(key).and_then(Value::as_u64).ok_or_else(|| anyhow!("missing integer field `{key}`"))
}

pub(super) fn optional_u64(value: &Value, key: &str) -> Option<u64> {
    value.get(key).and_then(Value::as_u64)
}

pub(super) fn required_f64(value: &Value, key: &str) -> Result<f64> {
    value.get(key).and_then(Value::as_f64).ok_or_else(|| anyhow!("missing numeric field `{key}`"))
}

pub(super) fn required_bool(value: &Value, key: &str) -> Result<bool> {
    value.get(key).and_then(Value::as_bool).ok_or_else(|| anyhow!("missing boolean field `{key}`"))
}

pub(super) fn required_tsv<'a>(row: &'a BTreeMap<String, String>, key: &str) -> Result<&'a str> {
    row.get(key)
        .map(String::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| anyhow!("missing TSV field `{key}`"))
}

pub(super) fn parse_u64(value: &str) -> Result<u64> {
    value.parse::<u64>().with_context(|| format!("parse integer `{value}`"))
}

pub(super) fn parse_f64(value: &str) -> Result<f64> {
    let parsed = value.parse::<f64>().with_context(|| format!("parse number `{value}`"))?;
    anyhow::ensure!(parsed.is_finite(), "BAM observation number `{value}` must be finite");
    Ok(parsed)
}

pub(super) fn parse_bool(value: &str) -> Result<bool> {
    value.parse::<bool>().with_context(|| format!("parse bool `{value}`"))
}

pub(super) fn fraction_u64(numerator: u64, denominator: u64) -> Result<f64> {
    if denominator == 0 {
        Ok(0.0)
    } else {
        Ok(checked_f64_from_u64(numerator, "fraction numerator")?
            / checked_f64_from_u64(denominator, "fraction denominator")?)
    }
}

pub(super) fn summarize_metrics(values: Vec<String>) -> Option<String> {
    let values = values
        .into_iter()
        .map(|value| sanitize_tsv_cell(&value))
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    if values.is_empty() {
        None
    } else {
        Some(values.join("; "))
    }
}

pub(super) fn format_f64(value: f64) -> String {
    format!("{value:.6}")
}

pub(super) fn unique_strings<I>(values: I) -> Vec<String>
where
    I: IntoIterator<Item = String>,
{
    let mut seen = BTreeSet::new();
    let mut rows = Vec::new();
    for value in values {
        if !value.is_empty() && seen.insert(value.clone()) {
            rows.push(value);
        }
    }
    rows
}

pub(super) fn sanitize_tsv_cell(value: &str) -> String {
    value.replace(['\t', '\n', '\r'], " ")
}

pub(crate) fn parse_tsv_rows(
    raw: &str,
    path: &std::path::Path,
) -> Result<Vec<BTreeMap<String, String>>> {
    let mut lines = raw.lines();
    let headers = lines
        .next()
        .ok_or_else(|| anyhow!("{} is missing a header row", path.display()))?
        .split('\t')
        .map(str::to_string)
        .collect::<Vec<_>>();
    let mut rows = Vec::new();
    for line in lines.filter(|line| !line.trim().is_empty()) {
        let values = line.split('\t').map(str::to_string).collect::<Vec<_>>();
        let row = headers
            .iter()
            .cloned()
            .zip(values.into_iter().chain(std::iter::repeat(String::new())))
            .take(headers.len())
            .collect::<BTreeMap<_, _>>();
        rows.push(row);
    }
    Ok(rows)
}

pub(super) fn checked_observation_sum(left: f64, right: f64, field: &str) -> Result<f64> {
    let sum = left + right;
    anyhow::ensure!(sum.is_finite(), "BAM observation {field} total must remain finite");
    Ok(sum)
}

/// Admit stage identity before any scientific projection can enter the score index.
pub(super) fn required_stage<'a>(value: &'a Value, expected: &str) -> Result<&'a str> {
    let stage = required_identifier(value, "stage_id")?;
    anyhow::ensure!(
        stage == expected,
        "expected BAM report stage `{expected}`, observed `{stage}`"
    );
    Ok(stage)
}

pub(super) fn required_identifier<'a>(value: &'a Value, key: &str) -> Result<&'a str> {
    let id = required_str(value, key)?;
    anyhow::ensure!(
        !id.is_empty() && id.trim() == id,
        "missing or noncanonical BAM report identifier `{key}`"
    );
    Ok(id)
}
