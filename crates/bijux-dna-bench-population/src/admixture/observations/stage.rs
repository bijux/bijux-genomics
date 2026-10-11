//! Owner: bijux-dna-bench-population
//! ADMIXTURE observations stage boundary.
use crate::admixture::admission::cohort::validate_sample_coverage;
use crate::admixture::execution::artifacts::read_json;
use crate::admixture::model::DEFAULT_OUTPUT_SOURCE_K_SELECTION_NAME;
use crate::admixture::model::DEFAULT_OUTPUT_SOURCE_LOGS_NAME;
use crate::admixture::model::DEFAULT_OUTPUT_SOURCE_Q_MATRIX_NAME;
use crate::admixture::observations::evidence::validate_admixture_manifest;
use crate::admixture::observations::records::parse_admixture_rows;
use crate::admixture::LocalVcfAdmixtureSmokeRow;
use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use std::path::Path;

pub(crate) struct AdmixtureEvidence {
    pub(crate) source_manifest: serde_json::Value,
    pub(crate) execution_mode: String,
    pub(crate) tool_ok: bool,
    pub(crate) status: String,
    pub(crate) insufficient_data_reason: Option<String>,
    pub(crate) selected_k: usize,
    pub(crate) sample_count: u64,
    pub(crate) population_count: u64,
    pub(crate) cluster_headers: Vec<String>,
    pub(crate) rows: Vec<LocalVcfAdmixtureSmokeRow>,
}

pub(crate) fn observe_stage(
    cohort: &super::super::admission::input::CohortAdmission,
    stage_outputs: &bijux_dna_stages_vcf::pipeline::AdmixtureStageOutputs,
    output_root: &Path,
) -> Result<AdmixtureEvidence> {
    let super::super::admission::input::CohortAdmission {
        expected_samples,
        metadata_by_sample,
        population_labels,
        ..
    } = cohort;
    let source_q_matrix_path = output_root.join(DEFAULT_OUTPUT_SOURCE_Q_MATRIX_NAME);
    bijux_dna_infra::copy_file(&stage_outputs.q_matrix_tsv, &source_q_matrix_path).with_context(
        || {
            format!(
                "copy {} to {}",
                stage_outputs.q_matrix_tsv.display(),
                source_q_matrix_path.display()
            )
        },
    )?;
    let source_k_selection_path = output_root.join(DEFAULT_OUTPUT_SOURCE_K_SELECTION_NAME);
    bijux_dna_infra::copy_file(&stage_outputs.k_selection_json, &source_k_selection_path)
        .with_context(|| {
            format!(
                "copy {} to {}",
                stage_outputs.k_selection_json.display(),
                source_k_selection_path.display()
            )
        })?;
    let source_logs_path = output_root.join(DEFAULT_OUTPUT_SOURCE_LOGS_NAME);
    bijux_dna_infra::copy_file(&stage_outputs.logs_txt, &source_logs_path).with_context(|| {
        format!("copy {} to {}", stage_outputs.logs_txt.display(), source_logs_path.display())
    })?;

    let source_manifest = read_json(&source_k_selection_path)?;
    let execution_mode = source_manifest
        .get("execution_mode")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| anyhow!("admixture manifest missing `execution_mode`"))?
        .to_string();
    let tool_ok = source_manifest
        .get("tool_ok")
        .and_then(serde_json::Value::as_bool)
        .ok_or_else(|| anyhow!("admixture manifest missing `tool_ok`"))?;
    let status = source_manifest
        .get("status")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| anyhow!("admixture manifest missing `status`"))?
        .to_string();
    let insufficient_data_reason = source_manifest
        .get("insufficient_data_reason")
        .and_then(serde_json::Value::as_str)
        .map(ToString::to_string);
    let selected_k = usize::try_from(
        source_manifest
            .get("selected_k")
            .and_then(serde_json::Value::as_u64)
            .ok_or_else(|| anyhow!("admixture manifest missing `selected_k`"))?,
    )
    .map_err(|_| anyhow!("selected_k overflow"))?;
    let sample_count = source_manifest
        .get("sample_count")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| anyhow!("admixture manifest missing `sample_count`"))?;
    let population_count = source_manifest
        .get("population_count")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| anyhow!("admixture manifest missing `population_count`"))?;
    let cluster_headers = source_manifest
        .get("cluster_headers")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| anyhow!("admixture manifest missing `cluster_headers`"))?
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(ToString::to_string)
                .ok_or_else(|| anyhow!("admixture manifest cluster_headers must be strings"))
        })
        .collect::<Result<Vec<_>>>()?;
    validate_admixture_manifest(&source_manifest, expected_samples, selected_k, &cluster_headers)?;

    let rows = parse_admixture_rows(
        &source_q_matrix_path,
        metadata_by_sample,
        population_labels,
        &cluster_headers,
        selected_k,
        &status,
    )?;
    validate_sample_coverage(&rows, expected_samples)?;

    Ok(AdmixtureEvidence {
        source_manifest,
        execution_mode,
        tool_ok,
        status,
        insufficient_data_reason,
        selected_k,
        sample_count,
        population_count,
        cluster_headers,
        rows,
    })
}
