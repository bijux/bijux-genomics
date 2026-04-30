use std::path::Path;

use anyhow::{Context, Result};
use bijux_dna_core::contract::ExecutionStep;
use bijux_dna_core::metrics::{ToolInvocationSpecV1, ToolInvocationV1};
use bijux_dna_environment::api::RuntimeKind;

use super::effects::{runner_failure, RunnerEffectKind};
use super::identity::{infer_tool_version_from_image, runtime_platform_identity};

fn selected_environment(out_dir: &Path) -> std::collections::BTreeMap<String, String> {
    let mut environment = std::collections::BTreeMap::from([(
        "working_directory".to_string(),
        out_dir.display().to_string(),
    )]);
    for key in ["TMPDIR", "PATH", "BIJUX_ALLOW_NETWORK"] {
        if let Ok(value) = std::env::var(key) {
            environment.insert(key.to_string(), value);
        }
    }
    environment
}

pub(super) fn write_minimum_run_artifacts(
    step: &ExecutionStep,
    input_hashes: &[String],
    output_hashes: &[String],
    runner: RuntimeKind,
    command: &str,
    stdout: &str,
    stderr: &str,
    run_id: &str,
    params_fingerprint: &str,
) -> Result<()> {
    let run_artifacts_dir = step.out_dir.join("run_artifacts");
    bijux_dna_infra::ensure_dir(&run_artifacts_dir)
        .map_err(|err| runner_failure(RunnerEffectKind::Filesystem, err.to_string()))?;

    let metrics_path = run_artifacts_dir.join("metrics.json");
    if !metrics_path.exists() {
        bijux_dna_infra::atomic_write_json(&metrics_path, &serde_json::json!({}))
            .map_err(|err| runner_failure(RunnerEffectKind::TelemetryWrite, err.to_string()))?;
    }

    let effective_config_path = run_artifacts_dir.join("effective_config.json");
    if !effective_config_path.exists() {
        let payload = serde_json::json!({
            "command": step.command.template,
            "image": step.image,
            "resources": step.resources,
        });
        bijux_dna_infra::atomic_write_json(&effective_config_path, &payload)
            .map_err(|err| runner_failure(RunnerEffectKind::TelemetryWrite, err.to_string()))?;
    }

    let tool_invocation_path = run_artifacts_dir.join("tool_invocation.json");
    if !tool_invocation_path.exists() {
        write_tool_invocation(
            step,
            runner,
            input_hashes,
            output_hashes,
            command,
            &tool_invocation_path,
        )?;
    }

    let command_path = run_artifacts_dir.join("command.txt");
    if !command_path.exists() {
        bijux_dna_infra::write_bytes(&command_path, command.as_bytes())
            .map_err(|err| runner_failure(RunnerEffectKind::TelemetryWrite, err.to_string()))?;
    }

    let stdout_log_path = run_artifacts_dir.join("stdout.log");
    if !stdout_log_path.exists() {
        bijux_dna_infra::write_bytes(&stdout_log_path, stdout.as_bytes())
            .map_err(|err| runner_failure(RunnerEffectKind::TelemetryWrite, err.to_string()))?;
    }

    let stderr_log_path = run_artifacts_dir.join("stderr.log");
    if !stderr_log_path.exists() {
        bijux_dna_infra::write_bytes(&stderr_log_path, stderr.as_bytes())
            .map_err(|err| runner_failure(RunnerEffectKind::TelemetryWrite, err.to_string()))?;
    }

    let working_directory_path = run_artifacts_dir.join("working_directory.txt");
    if !working_directory_path.exists() {
        let working_directory = step
            .out_dir
            .canonicalize()
            .unwrap_or_else(|_| step.out_dir.clone())
            .display()
            .to_string();
        bijux_dna_infra::write_bytes(&working_directory_path, working_directory.as_bytes())
            .map_err(|err| runner_failure(RunnerEffectKind::TelemetryWrite, err.to_string()))?;
    }

    let stage_report_path = run_artifacts_dir.join("stage_report.json");
    if !stage_report_path.exists() {
        let inferred_tool_version = infer_tool_version_from_image(&step.image.image);
        let payload = serde_json::json!({
            "schema_version": "bijux.stage_report.v1",
            "stage_id": step.stage_id.to_string(),
            "stage_version": 1,
            "tool_id": step.image.image.clone(),
            "tool_version": inferred_tool_version,
            "metrics_path": metrics_path.display().to_string(),
            "tool_invocation_path": tool_invocation_path.display().to_string(),
            "effective_config_path": effective_config_path.display().to_string(),
            "effective_config_hash": null,
            "facts_row_id": null,
            "summary": {
                "metric_provenance": {
                    "run_id": run_id,
                    "stage_id": step.stage_id.to_string(),
                    "tool_id": step.image.image.clone(),
                    "tool_version": inferred_tool_version,
                    "params_hash": params_fingerprint,
                    "input_artifact_hashes": input_hashes,
                }
            },
            "warnings": [],
            "errors": [],
            "invariants": [],
            "verdict": null,
            "outputs": step
                .io
                .outputs
                .iter()
                .map(|output| output.path.display().to_string())
                .collect::<Vec<_>>(),
            "subreports": [],
            "log_paths": [stdout_log_path, stderr_log_path],
        });
        bijux_dna_infra::atomic_write_json(&stage_report_path, &payload)
            .context("write stage_report.json")?;
    }

    Ok(())
}

fn write_tool_invocation(
    step: &ExecutionStep,
    runner: RuntimeKind,
    input_hashes: &[String],
    output_hashes: &[String],
    command: &str,
    tool_invocation_path: &Path,
) -> Result<()> {
    let inferred_tool_version = infer_tool_version_from_image(&step.image.image);
    let parameters_json = serde_json::json!({ "command": step.command.template });
    let params_provenance = serde_json::json!({
        "tool_params": parameters_json,
        "defaults": serde_json::json!({}),
        "overrides": serde_json::json!({}),
        "effective_params": serde_json::json!({}),
    });
    let params_provenance_normalized =
        bijux_dna_core::contract::canonical::canonicalize_json_value(&params_provenance);
    let invocation = ToolInvocationV1::new(ToolInvocationSpecV1 {
        schema_version: "bijux.tool_invocation.v1".to_string(),
        contract_version: bijux_dna_core::contract::ContractVersion::v1(),
        stage_id: step.stage_id.clone(),
        tool_id: bijux_dna_core::ids::ToolId::new(step.image.image.clone()),
        tool_version: inferred_tool_version.clone(),
        resolved_tool_version: None,
        image_digest: step.image.digest.clone().unwrap_or_else(|| step.image.image.clone()),
        runner_kind: format!("{runner:?}"),
        platform: runtime_platform_identity(runner),
        parameters_json: parameters_json.clone(),
        parameters_json_normalized: parameters_json,
        effective_params_json: serde_json::json!({}),
        effective_params_json_normalized: serde_json::json!({}),
        params_provenance,
        params_provenance_normalized,
        resources: step.resources.clone(),
        environment: selected_environment(&step.out_dir),
        input_hashes: input_hashes.to_vec(),
        output_hashes: output_hashes.to_vec(),
        executed_command: Some(command.to_string()),
    });
    bijux_dna_infra::atomic_write_json(tool_invocation_path, &invocation)
        .context("write tool_invocation.json")
}
