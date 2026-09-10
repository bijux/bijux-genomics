//! Owner: bijux-dna-bench-population
//! Demography observations evidence.
use crate::demography::execution::artifacts::read_json;
use crate::demography::model::NormalizedDemographyEvidence;
use crate::demography::observations::records::parse_ne_estimates;
use crate::demography::observations::records::parse_time_bins;
use anyhow::anyhow;
use anyhow::bail;
use anyhow::Result;
use std::path::Path;

pub(crate) fn validate_primary_demography_evidence(
    evidence: &NormalizedDemographyEvidence,
) -> Result<()> {
    match evidence.status.as_str() {
        "complete" => {
            if evidence.ne_estimates.is_empty() {
                bail!("governed VCF demography smoke expected non-empty Ne estimates");
            }
            if evidence.time_bins.len() != evidence.ne_estimates.len() {
                bail!(
                    "governed VCF demography smoke time-bin count drifted from Ne estimate count"
                );
            }
            if evidence.insufficient_reason.is_some() {
                bail!("governed VCF demography smoke must not report `insufficient_reason` when status is complete");
            }
        }
        "insufficient_data" => {
            if evidence.insufficient_reason.is_none() {
                bail!(
                    "governed VCF demography smoke insufficient-data result must report a reason"
                );
            }
        }
        other => bail!("governed VCF demography smoke reported unsupported status `{other}`"),
    }
    Ok(())
}

pub(crate) fn load_demography_evidence(
    contract_path: &Path,
    metrics_path: &Path,
) -> Result<NormalizedDemographyEvidence> {
    let contract = read_json(contract_path)?;
    let metrics = read_json(metrics_path)?;
    let method = contract
        .get("method")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| anyhow!("demography contract is missing `method`"))?
        .to_string();
    let metrics_method = metrics
        .get("method")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| anyhow!("demography metrics are missing `method`"))?;
    if method != metrics_method {
        bail!("demography method drifted between contract and metrics");
    }
    let inference_status = contract
        .get("inference_status")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| anyhow!("demography contract is missing `inference_status`"))?
        .to_string();
    let metrics_inference_status = metrics
        .get("inference_status")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| anyhow!("demography metrics are missing `inference_status`"))?;
    if inference_status != metrics_inference_status {
        bail!("demography inference status drifted between contract and metrics");
    }
    let status = contract
        .get("status")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| anyhow!("demography contract is missing `status`"))?
        .to_string();
    let metrics_status = metrics
        .get("status")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| anyhow!("demography metrics are missing `status`"))?;
    if status != metrics_status {
        bail!("demography status drifted between contract and metrics");
    }
    let insufficient_reason = contract
        .get("insufficient_data_reason")
        .and_then(serde_json::Value::as_str)
        .map(str::to_string);
    let metrics_insufficient_reason = metrics
        .get("insufficient_data_reason")
        .and_then(serde_json::Value::as_str)
        .map(str::to_string);
    if insufficient_reason != metrics_insufficient_reason {
        bail!("demography insufficient-data reason drifted between contract and metrics");
    }
    let time_bins = parse_time_bins(&contract)?;
    let metrics_time_bins = parse_time_bins(&metrics)?;
    if time_bins != metrics_time_bins {
        bail!("demography time bins drifted between contract and metrics");
    }
    let ne_estimates = parse_ne_estimates(&contract)?;
    let metrics_ne_estimates = parse_ne_estimates(&metrics)?;
    if ne_estimates != metrics_ne_estimates {
        bail!("demography Ne estimates drifted between contract and metrics");
    }
    let tool_ok = metrics
        .pointer("/tool_attempts/ibdne")
        .and_then(serde_json::Value::as_bool)
        .ok_or_else(|| anyhow!("demography metrics are missing `tool_attempts.ibdne`"))?;

    Ok(NormalizedDemographyEvidence {
        method,
        inference_status,
        status,
        insufficient_reason,
        tool_ok,
        time_bins,
        ne_estimates,
    })
}
