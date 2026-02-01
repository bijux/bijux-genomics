use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use bijux_engine::api::StageResultV1;

pub(super) fn write_run_summary(
    out_dir: &Path,
    stage_runs: &[StageExecutionSummary],
    failures: &[bijux_stages_fastq::RawFailure],
    merge_decision: Option<&bijux_stages_fastq::fastq::preprocess::MergeDecisionTrace>,
) -> Result<()> {
    let root = out_dir.join("run_artifacts");
    fs::create_dir_all(&root).context("create run summary artifacts dir")?;
    let run_id = stage_runs
        .first()
        .map(|entry| entry.result.run_id.clone())
        .unwrap_or_default();
    let stages: Vec<serde_json::Value> = stage_runs
        .iter()
        .map(|entry| {
            let artifacts_dir = entry.plan.out_dir.join("run_artifacts");
            let metrics_path = artifacts_dir.join("metrics_envelope.json");
            let metrics =
                read_json_if_exists(&metrics_path).and_then(|value| value.get("metrics").cloned());
            let stage_report_path = artifacts_dir.join("stage_report.json");
            let retention_report_path = artifacts_dir
                .join("reports")
                .join(format!("{}.retention.json", entry.plan.stage_id.0));
            serde_json::json!({
                "stage_id": entry.plan.stage_id.0,
                "tool_id": entry.plan.tool_id.0,
                "exit_code": entry.result.exit_code,
                "runtime_s": entry.result.runtime_s,
                "memory_mb": entry.result.memory_mb,
                "out_dir": entry.plan.out_dir,
                "artifacts": {
                    "metrics_envelope": metrics_path,
                    "stage_report": stage_report_path,
                    "retention_report": retention_report_path
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
                "reason": failure.reason
            })
        })
        .collect();
    let summary = serde_json::json!({
        "schema_version": "bijux.run_summary.v1",
        "run_id": run_id,
        "total_runtime_s": total_runtime_s,
        "stages": stages,
        "failures": failures_json,
        "pipeline_decisions": {
            "merge": merge_decision
        }
    });
    let summary_path = root.join("run_summary.json");
    fs::write(&summary_path, serde_json::to_vec_pretty(&summary)?)
        .context("write run_summary.json")?;
    let html_path = root.join("run_summary.html");
    let html = render_run_summary_html(&summary);
    fs::write(&html_path, html).context("write run_summary.html")?;
    write_run_manifest(out_dir, stage_runs, failures)?;
    Ok(())
}

#[allow(clippy::too_many_lines)]
pub(super) fn write_run_manifest(
    out_dir: &Path,
    stage_runs: &[StageExecutionSummary],
    failures: &[bijux_stages_fastq::RawFailure],
) -> Result<()> {
    let run_id = stage_runs
        .first()
        .map(|entry| entry.result.run_id.clone())
        .unwrap_or_default();
    let stages: Vec<serde_json::Value> = stage_runs
        .iter()
        .map(|entry| {
            let artifacts_dir = entry.plan.out_dir.join("run_artifacts");
            let mut artifacts = Vec::new();
            let add_artifact = |artifacts: &mut Vec<serde_json::Value>, name: &str, path: &Path| {
                if path.exists() {
                    if let Ok(hash) = bijux_engine::api::hash_file_sha256(path) {
                        artifacts.push(serde_json::json!({
                            "name": name,
                            "path": path,
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
                    .join(format!("{}.retention.json", entry.plan.stage_id.0)),
            );
            add_artifact(
                &mut artifacts,
                "telemetry_events",
                &artifacts_dir.join("telemetry").join("events.jsonl"),
            );
            let mut primary_outputs = Vec::new();
            if entry.plan.stage_id.0 == "fastq.qc_post" {
                let qc_report = artifacts_dir
                    .join("reports")
                    .join(format!("{}.qc_post_report.json", entry.plan.stage_id.0));
                if qc_report.exists() {
                    primary_outputs.push(qc_report);
                }
                let multiqc_report = entry.plan.out_dir.join("multiqc_report.html");
                if multiqc_report.exists() {
                    primary_outputs.push(multiqc_report);
                }
                let multiqc_data = entry.plan.out_dir.join("multiqc_data");
                if multiqc_data.exists() {
                    primary_outputs.push(multiqc_data);
                }
            }
            serde_json::json!({
                "stage_id": entry.plan.stage_id.0,
                "tool_id": entry.plan.tool_id.0,
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
                "reason": failure.reason
            })
        })
        .collect();
    let manifest = serde_json::json!({
        "schema_version": "bijux.run_manifest.v1",
        "run_id": run_id,
        "stages": stages,
        "failures": failures_json,
        "telemetry": {
            "events": stage_runs.iter().map(|entry| {
                entry.plan.out_dir
                    .join("run_artifacts")
                    .join("telemetry")
                    .join("events.jsonl")
            }).collect::<Vec<_>>()
        }
    });
    let path = out_dir.join("run_manifest.json");
    fs::write(&path, serde_json::to_vec_pretty(&manifest)?).context("write run_manifest.json")?;
    Ok(())
}

fn read_json_if_exists(path: &Path) -> Option<serde_json::Value> {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
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
pub(super) struct StageExecutionSummary {
    pub plan: bijux_core::StagePlanV1,
    pub result: StageResultV1,
}
