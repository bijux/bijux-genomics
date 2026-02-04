use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use bijux_core::{EffectiveConfigV1, ToolConstraints};

use bijux_engine::services::run_artifacts::PlanArtifacts;

#[allow(clippy::too_many_arguments)]
pub fn write_plan_artifacts(
    run_artifacts_dir: &Path,
    stage_id: &str,
    stage_version: i32,
    tool_id: &str,
    tool_version: &str,
    image_digest: Option<String>,
    runner: &str,
    platform: &str,
    resources: &ToolConstraints,
    inputs: &[PathBuf],
    outputs: &[PathBuf],
    params: &serde_json::Value,
    effective_params: &serde_json::Value,
    adapter_bank: Option<&bijux_core::metrics::AdapterBankProvenanceV1>,
    banks: Option<&serde_json::Value>,
    bank_assets: Option<&serde_json::Value>,
) -> Result<PlanArtifacts> {
    bijux_infra::ensure_dir(run_artifacts_dir).context("create run_artifacts dir")?;
    let plan_path = run_artifacts_dir.join("plan.json");
    let effective_config_path = run_artifacts_dir.join("effective_config.json");
    let config_dir = run_artifacts_dir.join("config");
    bijux_infra::ensure_dir(&config_dir).context("create config artifact dir")?;
    let stage_config_path = config_dir.join(format!("{stage_id}.effective.json"));
    let payload = serde_json::json!({
        "stage_id": stage_id,
        "stage_version": stage_version,
        "tool_id": tool_id,
        "inputs": inputs,
        "outputs": outputs,
        "parameters": params,
        "effective_params": effective_params,
    });
    bijux_infra::atomic_write_json(&plan_path, &payload).context("write plan.json")?;
    let effective_config = EffectiveConfigV1 {
        schema_version: "bijux.effective_config.v1".to_string(),
        stage_id: stage_id.to_string(),
        stage_version,
        tool_id: tool_id.to_string(),
        tool_version: tool_version.to_string(),
        image_digest,
        runner: runner.to_string(),
        platform: platform.to_string(),
        resources: resources.clone(),
        parameters_json: params.clone(),
        parameters_json_normalized: bijux_core::parameters_json_canonicalization(params),
        effective_params_json: effective_params.clone(),
        effective_params_json_normalized: bijux_core::parameters_json_canonicalization(
            effective_params,
        ),
        adapter_bank: adapter_bank.cloned(),
        banks: banks.cloned(),
        bank_assets: bank_assets.cloned(),
    };
    bijux_infra::atomic_write_json(&effective_config_path, &effective_config)
        .context("write effective_config.json")?;
    bijux_infra::atomic_write_json(&stage_config_path, &effective_config)
        .context("write effective config artifact")?;
    Ok(PlanArtifacts {
        plan_path,
        effective_config_path,
        stage_config_path,
    })
}
