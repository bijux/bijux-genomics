use super::model::VcfAdapterMissingInputTestRow;
use anyhow::{anyhow, Result};
use std::path::Path;
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ProbeInput {
    pub(super) artifact_id: String,
    pub(super) path: String,
}

pub(super) fn build_adapter_probe_row(
    repo_root: &Path,
    stage_id: &str,
    tool_id: &str,
    benchmark_status: &str,
    artifact_id: &str,
    row_reason: &str,
    inputs: &[ProbeInput],
) -> Result<VcfAdapterMissingInputTestRow> {
    let probe =
        inputs.iter().find(|input| input.artifact_id == artifact_id).cloned().ok_or_else(|| {
            anyhow!("VCF {stage_id} / {tool_id} row is missing required probe `{artifact_id}`")
        })?;
    let (expected_error_fragment, observed_error, passed) =
        run_missing_input_probe(repo_root, stage_id, inputs, artifact_id);

    Ok(VcfAdapterMissingInputTestRow {
        contract_surface: "adapter_contract".to_string(),
        stage_id: stage_id.to_string(),
        tool_id: tool_id.to_string(),
        benchmark_status: benchmark_status.to_string(),
        missing_input_role: artifact_id.to_string(),
        artifact_id: probe.artifact_id,
        artifact_path: probe.path,
        expected_error_fragment,
        observed_error,
        passed,
        reason: format!(
            "{row_reason}; required-input admission probes `{artifact_id}` on this retained adapter surface before tool execution"
        ),
    })
}
pub(super) fn build_support_probe_row(
    repo_root: &Path,
    artifact_id: &str,
    artifact_path: &str,
    reason: &str,
) -> VcfAdapterMissingInputTestRow {
    let inputs =
        vec![ProbeInput { artifact_id: artifact_id.to_string(), path: artifact_path.to_string() }];
    let (expected_error_fragment, observed_error, passed) =
        run_missing_input_probe(repo_root, "vcf.corpus_fixture", &inputs, artifact_id);
    VcfAdapterMissingInputTestRow {
        contract_surface: "fixture_support".to_string(),
        stage_id: "vcf.corpus_fixture".to_string(),
        tool_id: "fixture_contract".to_string(),
        benchmark_status: "support_required".to_string(),
        missing_input_role: artifact_id.to_string(),
        artifact_id: artifact_id.to_string(),
        artifact_path: artifact_path.to_string(),
        expected_error_fragment,
        observed_error,
        passed,
        reason: reason.to_string(),
    }
}
pub(super) fn run_missing_input_probe(
    repo_root: &Path,
    stage_id: &str,
    inputs: &[ProbeInput],
    artifact_id: &str,
) -> (String, String, bool) {
    let mut mutated_inputs = inputs.to_vec();
    if let Some(probe) = mutated_inputs.iter_mut().find(|input| input.artifact_id == artifact_id) {
        probe.path = format!(
            "artifacts/benchmark-smoke/readiness/adapters/probes/{stage_id}/{}.missing",
            probe.artifact_id
        );
    }
    let expected_error_fragment = format!("required input `{artifact_id}`");
    let observed_error = match validate_required_inputs(repo_root, stage_id, &mutated_inputs) {
        Ok(()) => format!(
            "VCF missing-input probe unexpectedly accepted `{artifact_id}` for `{stage_id}`"
        ),
        Err(error) => error.to_string(),
    };
    let passed = observed_error.contains(&expected_error_fragment);
    (expected_error_fragment, observed_error, passed)
}
fn validate_required_inputs(repo_root: &Path, stage_id: &str, inputs: &[ProbeInput]) -> Result<()> {
    crate::adapters::inputs::validate_required_files(
        repo_root,
        stage_id,
        crate::adapters::inputs::InputContext::Audit,
        inputs.iter().map(|input| (input.artifact_id.as_str(), input.path.as_str())),
    )
}
