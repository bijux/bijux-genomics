//! Owner: bijux-dna-bench-population
//! Population structure observations evidence.
use crate::execution::stage_result::path_relative_to_repo;
use crate::population_structure::model::{
    AdmixtureSampleRow, ConsumedAdmixtureReport, ConsumedPcaReport, PcaSampleRow,
};
use anyhow::anyhow;
use anyhow::bail;
use anyhow::Result;
use std::path::Path;

pub(crate) fn summarize_consumed_pca(
    repo_root: &Path,
    report_path: &Path,
    payload: &serde_json::Value,
) -> Result<ConsumedPcaReport> {
    Ok(ConsumedPcaReport {
        report_path: path_relative_to_repo(repo_root, report_path),
        sample_count: payload
            .get("sample_count")
            .and_then(serde_json::Value::as_u64)
            .ok_or_else(|| anyhow!("consumed PCA report missing `sample_count`"))?,
        execution_mode: payload
            .get("execution_mode")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| anyhow!("consumed PCA report missing `execution_mode`"))?
            .to_string(),
        tool_ok: payload
            .get("tool_ok")
            .and_then(serde_json::Value::as_bool)
            .ok_or_else(|| anyhow!("consumed PCA report missing `tool_ok`"))?,
    })
}

pub(crate) fn summarize_consumed_admixture(
    repo_root: &Path,
    report_path: &Path,
    payload: &serde_json::Value,
) -> Result<ConsumedAdmixtureReport> {
    Ok(ConsumedAdmixtureReport {
        report_path: path_relative_to_repo(repo_root, report_path),
        sample_count: payload
            .get("sample_count")
            .and_then(serde_json::Value::as_u64)
            .ok_or_else(|| anyhow!("consumed admixture report missing `sample_count`"))?,
        selected_k: payload
            .get("selected_k")
            .and_then(serde_json::Value::as_u64)
            .ok_or_else(|| anyhow!("consumed admixture report missing `selected_k`"))?,
        execution_mode: payload
            .get("execution_mode")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| anyhow!("consumed admixture report missing `execution_mode`"))?
            .to_string(),
        tool_ok: payload
            .get("tool_ok")
            .and_then(serde_json::Value::as_bool)
            .ok_or_else(|| anyhow!("consumed admixture report missing `tool_ok`"))?,
        status: payload
            .get("status")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| anyhow!("consumed admixture report missing `status`"))?
            .to_string(),
    })
}

pub(crate) fn validate_consumed_stage_inputs(
    stage_payload: &serde_json::Value,
    pca_rows: &[PcaSampleRow],
    admixture_rows: &[AdmixtureSampleRow],
) -> Result<()> {
    let stage_sample_ids = stage_payload
        .get("sample_ids")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| anyhow!("population-structure stage report missing `sample_ids`"))?
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(ToString::to_string)
                .ok_or_else(|| anyhow!("population-structure stage sample_ids must be strings"))
        })
        .collect::<Result<Vec<_>>>()?;
    let pca_sample_ids = pca_rows.iter().map(|row| row.sample_id.clone()).collect::<Vec<_>>();
    let admixture_sample_ids =
        admixture_rows.iter().map(|row| row.sample_id.clone()).collect::<Vec<_>>();
    if stage_sample_ids != pca_sample_ids {
        bail!(
            "population-structure stage sample_ids drifted from consumed PCA rows: {stage_sample_ids:?} vs {pca_sample_ids:?}"
        );
    }
    if stage_sample_ids != admixture_sample_ids {
        bail!(
            "population-structure stage sample_ids drifted from consumed admixture rows: {stage_sample_ids:?} vs {admixture_sample_ids:?}"
        );
    }
    Ok(())
}

pub(crate) fn required_string_at(
    payload: &serde_json::Value,
    pointer: &str,
    label: &str,
) -> Result<String> {
    payload
        .pointer(pointer)
        .and_then(serde_json::Value::as_str)
        .map(ToString::to_string)
        .ok_or_else(|| anyhow!("{label} is missing at `{pointer}`"))
}

pub(crate) fn required_bool_at(
    payload: &serde_json::Value,
    pointer: &str,
    label: &str,
) -> Result<bool> {
    payload
        .pointer(pointer)
        .and_then(serde_json::Value::as_bool)
        .ok_or_else(|| anyhow!("{label} is missing at `{pointer}`"))
}
