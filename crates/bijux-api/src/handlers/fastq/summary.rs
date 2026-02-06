use std::fs;
use std::path::Path;

use super::STAGE_QC_POST;
use anyhow::{Context, Result};
use bijux_core::metrics::ToolInvocationV1;
use bijux_core::plan::execution_graph::ExecutionStep;
use bijux_planner_fastq::{CorrectDecisionTrace, MergeDecisionTrace};
use bijux_runner::primitives::StageResultV1;

pub(super) fn write_run_summary(
    out_dir: &Path,
    stage_runs: &[StageExecutionSummary],
    failures: &[bijux_planner_fastq::stage_api::RawFailure],
    merge_decision: Option<&MergeDecisionTrace>,
    correct_decision: Option<&CorrectDecisionTrace>,
    adapter_inference: Option<&serde_json::Value>,
    stage_skips: &[serde_json::Value],
) -> Result<()> {
    let root = bijux_runtime::recording::run_artifacts_dir_for_out(out_dir);
    bijux_infra::ensure_dir(&root).context("create run summary artifacts dir")?;
    let run_id = stage_runs
        .first()
        .map(|entry| entry.result.run_id.clone())
        .unwrap_or_default();
    let stages: Vec<serde_json::Value> = stage_runs
        .iter()
        .map(|entry| {
            let artifacts_dir =
                bijux_runtime::recording::run_artifacts_dir_for_out(&entry.plan.out_dir);
            let metrics_path = artifacts_dir.join("metrics_envelope.json");
            let metrics =
                read_json_if_exists(&metrics_path).and_then(|value| value.get("metrics").cloned());
            let stage_report_path = artifacts_dir.join("stage_report.json");
            let retention_report_path = artifacts_dir
                .join("reports")
                .join(format!("{}.retention.json", entry.plan.step_id.0));
            serde_json::json!({
                "stage_id": entry.plan.step_id.0,
                "tool_id": entry.plan.image.image,
                "exit_code": entry.result.exit_code,
                "runtime_s": entry.result.runtime_s,
                "memory_mb": entry.result.memory_mb,
                "out_dir": relative_path_string(out_dir, &entry.plan.out_dir),
                "artifacts": {
                    "metrics_envelope": relative_path_string(out_dir, &metrics_path),
                    "stage_report": relative_path_string(out_dir, &stage_report_path),
                    "retention_report": relative_path_string(out_dir, &retention_report_path)
                },
                "metrics": metrics.unwrap_or(serde_json::Value::Null)
            })
        })
        .collect();
    let total_runtime_s: f64 = stage_runs.iter().map(|entry| entry.result.runtime_s).sum();
    let failures_json: Vec<serde_json::Value> = failures
        .iter()
        .map(|failure| {
            serde_json::json!({
                "stage": failure.stage,
                "tool": failure.tool,
                "reason": failure.reason,
                "category": format!("{:?}", failure.category),
            })
        })
        .collect();
    let run_provenance = run_provenance_from_stage_runs(out_dir, stage_runs);
    let summary = serde_json::json!({
        "schema_version": "bijux.run_summary.v1",
        "run_id": run_id,
        "total_runtime_s": total_runtime_s,
        "stages": stages,
        "failures": failures_json,
        "run_provenance": run_provenance,
        "pipeline_decisions": {
            "merge": merge_decision,
            "correct": correct_decision,
            "adapter_inference": adapter_inference,
            "stage_skips": stage_skips
        }
    });
    let summary_path = root.join("run_summary.json");
    bijux_infra::atomic_write_json(&summary_path, &summary)
        .with_context(|| "write run_summary.json")?;
    let html_path = root.join("run_summary.html");
    let html = render_run_summary_html(&summary);
    bijux_infra::atomic_write_bytes(&html_path, html.as_bytes())
        .context("write run_summary.html")?;
    write_run_manifest(out_dir, stage_runs, failures)?;
    write_scientific_provenance(out_dir, stage_runs)?;
    Ok(())
}

#[allow(clippy::too_many_lines)]
pub(super) fn write_run_manifest(
    out_dir: &Path,
    stage_runs: &[StageExecutionSummary],
    failures: &[bijux_planner_fastq::stage_api::RawFailure],
) -> Result<()> {
    let run_id = stage_runs
        .first()
        .map(|entry| entry.result.run_id.clone())
        .unwrap_or_default();
    let stages: Vec<serde_json::Value> = stage_runs
        .iter()
        .map(|entry| {
            let artifacts_dir =
                bijux_runtime::recording::run_artifacts_dir_for_out(&entry.plan.out_dir);
            let mut artifacts = Vec::new();
            let add_artifact = |artifacts: &mut Vec<serde_json::Value>, name: &str, path: &Path| {
                if path.exists() {
                    if let Ok(hash) = bijux_infra::hash_file_sha256(path) {
                        artifacts.push(serde_json::json!({
                            "name": name,
                            "path": relative_path_string(out_dir, path),
                            "sha256": hash,
                        }));
                    }
                }
            };
            add_artifact(
                &mut artifacts,
                "metrics_envelope",
                &artifacts_dir.join("metrics_envelope.json"),
            );
            add_artifact(
                &mut artifacts,
                "metrics",
                &artifacts_dir.join("metrics.json"),
            );
            add_artifact(
                &mut artifacts,
                "stage_metrics",
                &artifacts_dir.join("stage_metrics.json"),
            );
            add_artifact(
                &mut artifacts,
                "stage_report",
                &artifacts_dir.join("stage_report.json"),
            );
            add_artifact(
                &mut artifacts,
                "effective_config",
                &artifacts_dir.join("effective_config.json"),
            );
            add_artifact(
                &mut artifacts,
                "retention_report",
                &artifacts_dir
                    .join("reports")
                    .join(format!("{}.retention.json", entry.plan.step_id.0)),
            );
            add_artifact(
                &mut artifacts,
                "telemetry_events",
                &artifacts_dir.join("telemetry").join("events.jsonl"),
            );
            let mut primary_outputs = Vec::new();
            if entry.plan.step_id.as_str() == STAGE_QC_POST.as_str() {
                let qc_report = artifacts_dir
                    .join("reports")
                    .join(format!("{}.qc_post_report.json", entry.plan.step_id.0));
                if qc_report.exists() {
                    primary_outputs.push(relative_path_string(out_dir, &qc_report));
                }
                let multiqc_report = entry.plan.out_dir.join("multiqc_report.html");
                if multiqc_report.exists() {
                    primary_outputs.push(relative_path_string(out_dir, &multiqc_report));
                }
                let multiqc_data = entry.plan.out_dir.join("multiqc_data");
                if multiqc_data.exists() {
                    primary_outputs.push(relative_path_string(out_dir, &multiqc_data));
                }
            }
            serde_json::json!({
                "stage_id": entry.plan.step_id.0,
                "tool_id": entry.plan.image.image,
                "artifacts": artifacts,
                "primary_outputs": primary_outputs,
            })
        })
        .collect();
    let failures_json: Vec<serde_json::Value> = failures
        .iter()
        .map(|failure| {
            serde_json::json!({
                "stage": failure.stage,
                "tool": failure.tool,
                "reason": failure.reason,
                "category": format!("{:?}", failure.category),
            })
        })
        .collect();
    let defaults_path = out_dir.join("defaults_ledger.json");
    let defaults_hash =
        bijux_infra::hash_file_sha256(&defaults_path).context("hash defaults_ledger.json")?;
    let _pipeline_id = std::env::var("BIJUX_PIPELINE_ID").unwrap_or_else(|_| "unknown".to_string());
    let _git_commit = std::env::var("BIJUX_GIT_COMMIT").unwrap_or_else(|_| "unknown".to_string());
    let _build_profile =
        std::env::var("BIJUX_BUILD_PROFILE").unwrap_or_else(|_| "unknown".to_string());
    let run_provenance = run_provenance_from_stage_runs(out_dir, stage_runs);
    let manifest = serde_json::json!({
        "schema_version": "bijux.run_manifest.v1",
        "run_id": run_id,
        "stages": stages,
        "failures": failures_json,
        "defaults_ledger": relative_path_string(out_dir, &defaults_path),
        "defaults_ledger_sha256": defaults_hash,
        "run_provenance": run_provenance,
        "telemetry": {
            "events": stage_runs.iter().map(|entry| {
                relative_path_string(
                    out_dir,
                    &bijux_runtime::recording::run_artifacts_dir_for_out(&entry.plan.out_dir)
                        .join("telemetry")
                        .join("events.jsonl"),
                )
            }).collect::<Vec<_>>()
        }
    });
    let path = out_dir.join("run_manifest.json");
    bijux_infra::atomic_write_json(&path, &manifest).context("write run_manifest.json")?;
    Ok(())
}

fn write_scientific_provenance(out_dir: &Path, stage_runs: &[StageExecutionSummary]) -> Result<()> {
    let defaults_path = out_dir.join("defaults_ledger.json");
    let (pipeline_id, planner_version) = if defaults_path.exists() {
        let raw = fs::read_to_string(&defaults_path)?;
        let value: serde_json::Value = serde_json::from_str(&raw)?;
        let pipeline_id = value
            .get("pipeline_id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        let planner_version =
            std::env::var("BIJUX_PLANNER_VERSION").unwrap_or_else(|_| "unknown".to_string());
        (pipeline_id, planner_version)
    } else {
        ("unknown".to_string(), "unknown".to_string())
    };
    let mut invocations = Vec::new();
    let mut params_hashes = std::collections::BTreeMap::new();
    for entry in stage_runs {
        let artifacts_dir =
            bijux_runtime::recording::run_artifacts_dir_for_out(&entry.plan.out_dir);
        let invocation_path = artifacts_dir
            .join("invocations")
            .join(format!("{}.tool_invocation.json", entry.plan.step_id.0));
        if invocation_path.exists() {
            let raw = fs::read_to_string(&invocation_path)?;
            let invocation: ToolInvocationV1 = serde_json::from_str(&raw)?;
            let key = format!("{}:{}", invocation.stage_id, invocation.tool_id);
            let metrics_path = artifacts_dir.join("metrics_envelope.json");
            if metrics_path.exists() {
                let metrics_raw = fs::read_to_string(&metrics_path)?;
                if let Ok(metrics) = serde_json::from_str::<serde_json::Value>(&metrics_raw) {
                    if let Some(params_hash) = metrics
                        .get("context")
                        .and_then(|ctx| ctx.get("params_hash"))
                        .and_then(|v| v.as_str())
                    {
                        params_hashes.insert(key, params_hash.to_string());
                    }
                }
            }
            invocations.push(invocation);
        }
    }
    let provenance = bijux_runtime::provenance::build_scientific_provenance(
        pipeline_id,
        planner_version,
        &params_hashes,
        &invocations,
    );
    bijux_runtime::recording::write_scientific_provenance(out_dir, &provenance)?;
    Ok(())
}

fn read_json_if_exists(path: &Path) -> Option<serde_json::Value> {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
}

fn relative_path_string(base: &Path, path: &Path) -> String {
    path.strip_prefix(base)
        .unwrap_or(path)
        .to_string_lossy()
        .to_string()
}

fn run_provenance_from_stage_runs(
    _out_dir: &Path,
    stage_runs: &[StageExecutionSummary],
) -> serde_json::Value {
    let mut params_by_stage = std::collections::BTreeMap::new();
    let mut input_hashes = Vec::new();
    let mut tool_versions = std::collections::BTreeSet::new();
    let mut image_digests = std::collections::BTreeSet::new();
    for entry in stage_runs {
        tool_versions.insert("unknown".to_string());
        if let Some(digest) = entry.plan.image.digest.clone() {
            image_digests.insert(digest);
        }
        let envelope_path =
            bijux_runtime::recording::run_artifacts_dir_for_out(&entry.plan.out_dir)
                .join("metrics_envelope.json");
        if let Some(value) = read_json_if_exists(&envelope_path) {
            if let Some(hash) = value.get("input_hash").and_then(serde_json::Value::as_str) {
                input_hashes.push(hash.to_string());
            }
            if let Some(hash) = value.get("params_hash").and_then(serde_json::Value::as_str) {
                params_by_stage.insert(entry.plan.step_id.to_string(), hash.to_string());
            }
        }
    }
    input_hashes.sort();
    input_hashes.dedup();
    let params_hash = bijux_core::params_hash(&serde_json::json!(params_by_stage))
        .unwrap_or_else(|_| "unknown".to_string());
    let tool_version = if tool_versions.len() == 1 {
        tool_versions
            .into_iter()
            .next()
            .unwrap_or_else(|| "unknown".to_string())
    } else {
        "multiple".to_string()
    };
    let tool_image_digest = if image_digests.len() == 1 {
        image_digests.into_iter().next()
    } else {
        None
    };
    let pipeline_id = std::env::var("BIJUX_PIPELINE_ID").unwrap_or_else(|_| "unknown".to_string());
    let git_commit = std::env::var("BIJUX_GIT_COMMIT").unwrap_or_else(|_| "unknown".to_string());
    let build_profile =
        std::env::var("BIJUX_BUILD_PROFILE").unwrap_or_else(|_| "unknown".to_string());
    let reference_genome = std::env::var("BIJUX_REFERENCE_GENOME").ok();
    let plan_hash = std::env::var("BIJUX_PLAN_HASH").ok();
    serde_json::json!({
        "schema_version": "bijux.run_provenance.v1",
        "tool_image_digest": tool_image_digest,
        "tool_version": tool_version,
        "params_hash": params_hash,
        "input_hashes": input_hashes,
        "reference_genome": reference_genome,
        "pipeline_id": pipeline_id,
        "git_commit": git_commit,
        "build_profile": build_profile,
        "plan_hash": plan_hash,
    })
}

fn render_run_summary_html(summary: &serde_json::Value) -> String {
    let pretty = serde_json::to_string_pretty(summary).unwrap_or_else(|_| "{}".to_string());
    format!(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <title>bijux run summary</title>
  <style>
    body {{
      font-family: system-ui, -apple-system, sans-serif;
      margin: 2rem;
      line-height: 1.4;
      background: #f7f7f9;
      color: #111;
    }}
    pre {{
      padding: 1rem;
      background: #fff;
      border-radius: 8px;
      overflow: auto;
      box-shadow: 0 1px 4px rgba(0,0,0,0.08);
    }}
  </style>
</head>
<body>
  <h1>Run summary</h1>
  <pre>{pretty}</pre>
</body>
</html>
"#
    )
}

#[derive(Clone)]
/// Summarized stage execution for run summaries.
///
/// Stability: v1 (stable).
pub struct StageExecutionSummary {
    pub plan: ExecutionStep,
    pub result: StageResultV1,
}

#[cfg(test)]
mod tests {
    use super::StageExecutionSummary;
    use super::{write_run_manifest, write_scientific_provenance};
    use bijux_core::metrics::{AdapterBankProvenanceV1, ToolInvocationV1};
    use bijux_core::{
        CommandSpecV1, ContainerImageRefV1, StageIO, StagePlanV1, StageVersion, ToolConstraints,
        ToolId,
    };
    use bijux_planner_fastq::stage_api::STAGE_TRIM;
    use bijux_runner::primitives::StageResultV1;
    use std::path::PathBuf;

    #[test]
    fn run_manifest_includes_defaults_ledger() -> anyhow::Result<()> {
        let temp = bijux_infra::temp_dir("bijux-run-manifest")?;
        let out_dir = temp.path();
        let defaults = serde_json::json!({
            "pipeline_id": "fastq-to-fastq__default__v1",
            "tools": {},
            "params": {},
            "thresholds": {},
            "tool_provenance": {},
            "param_provenance": {},
            "assumptions": [],
            "citations": {},
        });
        bijux_infra::write_bytes(
            out_dir.join("defaults_ledger.json"),
            serde_json::to_vec_pretty(&defaults)?,
        )?;

        let stage_out = out_dir.join("stage");
        bijux_infra::ensure_dir(&stage_out)?;
        let plan = StagePlanV1 {
            stage_id: STAGE_TRIM.clone(),
            stage_version: StageVersion(1),
            tool_id: ToolId::from_static("fastp"),
            tool_version: "0.0.0".to_string(),
            image: ContainerImageRefV1 {
                image: "tool:latest".to_string(),
                digest: None,
            },
            command: CommandSpecV1 { template: vec![] },
            resources: ToolConstraints {
                runtime: "1h".to_string(),
                mem_gb: 1,
                tmp_gb: 1,
                threads: 1,
            },
            io: StageIO {
                inputs: Vec::new(),
                outputs: Vec::new(),
            },
            out_dir: stage_out,
            params: serde_json::json!({}),
            effective_params: serde_json::json!({}),
            aux_images: std::collections::BTreeMap::new(),
            reason: bijux_core::plan::stage_plan::PlanDecisionReason::default(),
        };
        let result = StageResultV1 {
            run_id: "run-1".to_string(),
            exit_code: 0,
            runtime_s: 1.0,
            memory_mb: 1.0,
            outputs: Vec::new(),
            metrics_path: None,
            stdout: String::new(),
            stderr: String::new(),
            command: String::new(),
        };
        let stage_runs = vec![StageExecutionSummary {
            plan: bijux_core::plan::execution_graph::ExecutionStep::from(&plan),
            result,
        }];
        write_run_manifest(out_dir, &stage_runs, &[])?;
        let manifest_raw = std::fs::read_to_string(out_dir.join("run_manifest.json"))?;
        let manifest: serde_json::Value = serde_json::from_str(&manifest_raw)?;
        assert!(manifest.get("defaults_ledger").is_some());
        assert!(manifest.get("defaults_ledger_sha256").is_some());
        assert_no_absolute_paths(&manifest);
        Ok(())
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn scientific_provenance_contract_is_written() -> anyhow::Result<()> {
        let temp = bijux_infra::temp_dir("bijux-scientific-provenance")?;
        let out_dir = temp.path();
        let defaults = serde_json::json!({
            "pipeline_id": "fastq-to-fastq__default__v1",
            "tools": {},
            "params": {},
            "thresholds": {},
            "tool_provenance": {},
            "param_provenance": {},
            "assumptions": [],
            "citations": {},
        });
        bijux_infra::write_bytes(
            out_dir.join("defaults_ledger.json"),
            serde_json::to_vec_pretty(&defaults)?,
        )?;
        std::env::set_var("BIJUX_PLANNER_VERSION", "planner.v1");

        let stage_out = out_dir.join("stage");
        let artifacts = bijux_runtime::recording::run_artifacts_dir_for_out(&stage_out);
        let invocations = artifacts.join("invocations");
        bijux_infra::ensure_dir(&invocations)?;
        let plan = StagePlanV1 {
            stage_id: STAGE_TRIM.clone(),
            stage_version: StageVersion(1),
            tool_id: ToolId::from_static("fastp"),
            tool_version: "0.0.0".to_string(),
            image: ContainerImageRefV1 {
                image: "tool:latest".to_string(),
                digest: Some("sha256:img".to_string()),
            },
            command: CommandSpecV1 {
                template: vec!["fastp".to_string()],
            },
            resources: ToolConstraints {
                runtime: "1h".to_string(),
                mem_gb: 1,
                tmp_gb: 1,
                threads: 1,
            },
            io: StageIO {
                inputs: vec![bijux_core::plan::stage_plan::ArtifactRef {
                    name: "input".to_string(),
                    path: PathBuf::from("input.fastq.gz"),
                }],
                outputs: vec![bijux_core::plan::stage_plan::ArtifactRef {
                    name: "output".to_string(),
                    path: PathBuf::from("output.fastq.gz"),
                }],
            },
            out_dir: stage_out.clone(),
            params: serde_json::json!({"sample_id":"s1"}),
            effective_params: serde_json::json!({}),
            aux_images: std::collections::BTreeMap::new(),
            reason: bijux_core::plan::stage_plan::PlanDecisionReason::default(),
        };
        let invocation = ToolInvocationV1 {
            schema_version: "bijux.tool_invocation.v1".to_string(),
            stage_id: plan.stage_id.to_string(),
            tool_id: plan.tool_id.to_string(),
            tool_version: plan.tool_version.clone(),
            resolved_tool_version: Some(plan.tool_version.clone()),
            image_digest: "sha256:img".to_string(),
            runner_kind: "docker".to_string(),
            platform: "test".to_string(),
            parameters_json: serde_json::json!({"min_len": 10}),
            parameters_json_normalized: serde_json::json!({"min_len": 10}),
            effective_params_json: serde_json::json!({}),
            effective_params_json_normalized: serde_json::json!({}),
            adapter_bank: Some(AdapterBankProvenanceV1 {
                bank_id: "bank".to_string(),
                bank_version: "v1".to_string(),
                bank_hash: "sha256:bank".to_string(),
                presets_hash: "sha256:presets".to_string(),
                preset: "default".to_string(),
                preset_hash: "sha256:preset".to_string(),
                enabled_categories: Vec::new(),
                disabled_categories: Vec::new(),
                enable_adapters: Vec::new(),
                disable_adapters: Vec::new(),
                enabled_entries: Vec::new(),
            }),
            banks: None,
            bank_assets: None,
            resources: plan.resources.clone(),
            environment: std::collections::BTreeMap::new(),
            input_hashes: vec!["sha256:input".to_string()],
            output_hashes: vec!["sha256:output".to_string()],
            executed_command: Some("fastp".to_string()),
        };
        bijux_infra::atomic_write_json(
            &invocations.join(format!("{}.tool_invocation.json", plan.stage_id.0)),
            &invocation,
        )?;
        let metrics_envelope = serde_json::json!({
            "context": {
                "params_hash": "params"
            }
        });
        bijux_infra::atomic_write_json(
            &artifacts.join("metrics_envelope.json"),
            &metrics_envelope,
        )?;

        let summary = StageExecutionSummary {
            plan: bijux_core::plan::execution_graph::ExecutionStep::from(&plan),
            result: StageResultV1 {
                run_id: "run-1".to_string(),
                exit_code: 0,
                runtime_s: 1.0,
                memory_mb: 1.0,
                outputs: Vec::new(),
                metrics_path: None,
                stdout: String::new(),
                stderr: String::new(),
                command: "fastp".to_string(),
            },
        };
        write_scientific_provenance(out_dir, &[summary])?;
        let raw = std::fs::read_to_string(out_dir.join("scientific_provenance.json"))?;
        let payload: serde_json::Value = serde_json::from_str(&raw)?;
        assert_eq!(
            payload.get("pipeline_id").and_then(|v| v.as_str()),
            Some("fastq-to-fastq__default__v1")
        );
        assert_eq!(
            payload.get("planner_version").and_then(|v| v.as_str()),
            Some("planner.v1")
        );
        insta::assert_json_snapshot!(payload);
        Ok(())
    }

    fn assert_no_absolute_paths(value: &serde_json::Value) {
        match value {
            serde_json::Value::String(s) => {
                assert!(
                    !s.starts_with('/') || s.starts_with("//"),
                    "absolute path found: {s}"
                );
                assert!(!s.contains(":\\"), "windows absolute path found: {s}");
            }
            serde_json::Value::Array(items) => {
                for item in items {
                    assert_no_absolute_paths(item);
                }
            }
            serde_json::Value::Object(map) => {
                for (_, value) in map {
                    assert_no_absolute_paths(value);
                }
            }
            _ => {}
        }
    }
}
