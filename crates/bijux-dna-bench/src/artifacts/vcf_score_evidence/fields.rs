//! Owner: bijux-dna-bench
//! Shared VCF observation field and numeric projections.

use anyhow::{Context, Result};
use bijux_dna_core::prelude::numeric::checked_f64_from_u64;
use serde_json::Value;

pub(super) fn required_object<'a>(
    value: &'a Value,
    key: &str,
) -> Result<&'a serde_json::Map<String, Value>> {
    value
        .get(key)
        .and_then(Value::as_object)
        .with_context(|| format!("missing object field `{key}`"))
}

pub(super) fn required_array<'a>(value: &'a Value, key: &str) -> Result<&'a [Value]> {
    value
        .get(key)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .with_context(|| format!("missing array field `{key}`"))
}

pub(super) fn required_str<'a>(value: &'a Value, key: &str) -> Result<&'a str> {
    value.get(key).and_then(Value::as_str).with_context(|| format!("missing string field `{key}`"))
}

pub(super) fn required_u64(value: &Value, key: &str) -> Result<u64> {
    value.get(key).and_then(Value::as_u64).with_context(|| format!("missing integer field `{key}`"))
}

pub(super) fn required_u64_from_object(
    value: &serde_json::Map<String, Value>,
    key: &str,
) -> Result<u64> {
    value.get(key).and_then(Value::as_u64).with_context(|| format!("missing integer field `{key}`"))
}

pub(super) fn required_i64(value: &Value, key: &str) -> Result<i64> {
    value.get(key).and_then(Value::as_i64).with_context(|| format!("missing integer field `{key}`"))
}

pub(super) fn required_f64(value: &Value, key: &str) -> Result<f64> {
    value.get(key).and_then(Value::as_f64).with_context(|| format!("missing float field `{key}`"))
}

pub(super) fn required_bool(value: &Value, key: &str) -> Result<bool> {
    value.get(key).and_then(Value::as_bool).with_context(|| format!("missing bool field `{key}`"))
}

pub(super) fn optional_f64(value: &Value, key: &str) -> Option<f64> {
    value.get(key).and_then(Value::as_f64)
}

pub(super) fn checked_count_sum(left: u64, right: u64, field: &str) -> Result<u64> {
    left.checked_add(right).with_context(|| format!("VCF observation `{field}` count overflow"))
}

pub(super) fn fraction_u64(numerator: u64, denominator: u64) -> Result<f64> {
    if denominator == 0 {
        return Ok(0.0);
    }
    Ok(checked_f64_from_u64(numerator, "fraction numerator")?
        / checked_f64_from_u64(denominator, "fraction denominator")?)
}

pub(super) fn format_f64(value: f64) -> String {
    format!("{value:.6}")
}

pub(super) fn format_u64_as_string(value: u64) -> String {
    value.to_string()
}

pub(super) fn summarize_metrics(values: Vec<String>) -> String {
    values.into_iter().filter(|value| !value.is_empty()).collect::<Vec<_>>().join("; ")
}

pub(super) fn unique_strings(values: impl IntoIterator<Item = String>) -> Vec<String> {
    values.into_iter().collect::<std::collections::BTreeSet<_>>().into_iter().collect::<Vec<_>>()
}

/// Admit the expected envelope before any scientific projection enters the score index.
pub(super) fn required_stage<'a>(value: &'a Value, expected: &str) -> Result<&'a str> {
    let stage = required_identifier(value, "stage_id")?;
    anyhow::ensure!(
        stage == expected,
        "expected VCF report stage `{expected}`, observed `{stage}`"
    );
    Ok(stage)
}

pub(super) fn required_identifier<'a>(value: &'a Value, key: &str) -> Result<&'a str> {
    let id = required_str(value, key)?;
    anyhow::ensure!(
        !id.is_empty() && id.trim() == id,
        "missing or noncanonical VCF report identifier `{key}`"
    );
    Ok(id)
}

pub(super) fn required_report_schema(value: &Value, expected: &str) -> Result<()> {
    let schema = required_identifier(value, "schema_version")?;
    anyhow::ensure!(
        schema == expected,
        "expected VCF report schema `{expected}`, observed `{schema}`"
    );
    Ok(())
}
