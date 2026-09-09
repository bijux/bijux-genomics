//! Field and numeric projections shared by FASTQ report adapters.

use anyhow::{anyhow, Result};
use serde_json::Value;

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

pub(super) fn fraction_u64(numerator: u64, denominator: u64) -> Result<f64> {
    if denominator == 0 {
        Ok(0.0)
    } else {
        Ok(bijux_dna_core::prelude::numeric::checked_f64_from_u64(numerator, "fraction numerator")?
            / bijux_dna_core::prelude::numeric::checked_f64_from_u64(
                denominator,
                "fraction denominator",
            )?)
    }
}
