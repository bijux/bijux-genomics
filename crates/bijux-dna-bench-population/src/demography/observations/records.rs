//! Owner: bijux-dna-bench-population
//! Demography observations records.
use crate::demography::model::LocalVcfDemographyEstimate;
use anyhow::anyhow;
use anyhow::Result;

pub(crate) fn parse_time_bins(payload: &serde_json::Value) -> Result<Vec<u64>> {
    payload
        .get("time_bins")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| anyhow!("demography payload is missing `time_bins`"))?
        .iter()
        .map(|value| {
            value
                .as_u64()
                .ok_or_else(|| anyhow!("demography `time_bins` entries must be unsigned integers"))
        })
        .collect()
}

pub(crate) fn parse_ne_estimates(
    payload: &serde_json::Value,
) -> Result<Vec<LocalVcfDemographyEstimate>> {
    payload
        .get("ne_estimates")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| anyhow!("demography payload is missing `ne_estimates`"))?
        .iter()
        .map(|row| {
            Ok(LocalVcfDemographyEstimate {
                time_bin: row
                    .get("generation")
                    .and_then(serde_json::Value::as_u64)
                    .ok_or_else(|| anyhow!("demography Ne estimate is missing `generation`"))?,
                ne: row
                    .get("ne")
                    .and_then(serde_json::Value::as_f64)
                    .ok_or_else(|| anyhow!("demography Ne estimate is missing `ne`"))?,
                ci_low: row
                    .get("ci_low")
                    .and_then(serde_json::Value::as_f64)
                    .ok_or_else(|| anyhow!("demography Ne estimate is missing `ci_low`"))?,
                ci_high: row
                    .get("ci_high")
                    .and_then(serde_json::Value::as_f64)
                    .ok_or_else(|| anyhow!("demography Ne estimate is missing `ci_high`"))?,
            })
        })
        .collect()
}
