//! Owner: bijux-dna-bench
//! Field and numeric projections shared by FASTQ report adapters.

use anyhow::{anyhow, Result};
use serde_json::Value;

pub(super) fn required_str<'a>(value: &'a Value, key: &str) -> Result<&'a str> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|text| !text.is_empty() && text.trim() == *text)
        .ok_or_else(|| anyhow!("missing or noncanonical string field `{key}`"))
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

pub(super) fn required_stage<'a>(value: &'a Value, expected: &str) -> Result<&'a str> {
    let stage = required_str(value, "stage_id")?;
    if stage != expected {
        return Err(anyhow!("expected stage `{expected}`, observed `{stage}`"));
    }
    Ok(stage)
}

pub(super) fn sum_report_counts(cases: &[Value], key: &str) -> Result<u64> {
    cases.iter().try_fold(0_u64, |total, case| {
        total
            .checked_add(required_u64(case, key)?)
            .ok_or_else(|| anyhow!("FASTQ report sum of `{key}` overflows u64"))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_identifiers_require_nonempty_canonical_strings() {
        for field in ["stage_id", "tool_id", "planned_tool_id"] {
            for text in ["", " fastp", "fastp "] {
                let value = serde_json::json!({field: text});
                assert!(required_str(&value, field).is_err(), "{field}: {text:?}");
            }
            let value = serde_json::json!({field: "fastp"});
            assert_eq!(required_str(&value, field).ok(), Some("fastp"));
        }
    }
}
