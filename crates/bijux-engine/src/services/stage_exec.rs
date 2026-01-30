use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;

use anyhow::{anyhow, Context, Result};
use bijux_environment::api::{ResolvedImage, RunnerKind};
use chrono::Utc;
use uuid::Uuid;

use crate::api::{
    cleanup_execution, execution_memory_mb, hash_file_sha256, run_merge_execution,
    run_multiqc_execution, run_tool_execution, run_validate_execution,
};
use crate::services::run_artifacts::{
    default_trace_ids, params_hash, run_artifacts_dir_for_out, write_facts_jsonl,
    write_metrics_envelope, write_observability_manifest, write_plan_artifacts,
    write_retention_report_v1, write_stage_report_v1, write_telemetry_event,
};
use bijux_core::run_index::{insert_stage_row, StageIndexRow};
use bijux_core::{canonicalize_json_value, FactsRowV1, StageObservabilityContextV1};

#[derive(Debug, Clone)]
pub struct StagePlan {
    pub stage_id: String,
    pub stage_version: i32,
    pub tool: String,
    pub tool_version: String,
    pub image: ResolvedImage,
    pub runner: RunnerKind,
    pub inputs: Vec<PathBuf>,
    pub out_dir: PathBuf,
    pub outputs: Vec<PathBuf>,
    pub params: serde_json::Value,
    pub aux_images: HashMap<String, ResolvedImage>,
}

#[derive(Debug, Clone)]
pub struct StageResultV1 {
    pub run_id: String,
    pub exit_code: i32,
    pub runtime_s: f64,
    pub memory_mb: f64,
    pub outputs: Vec<PathBuf>,
    pub metrics_path: Option<PathBuf>,
    pub stdout: String,
    pub stderr: String,
    pub command: String,
}

#[derive(Debug)]
struct ExecutionEnvelope {
    exit_code: i32,
    stdout: String,
    stderr: String,
    command: String,
}

/// Execute a single stage plan.
///
/// # Errors
/// Returns an error if the execution fails or the plan is invalid.
#[allow(clippy::too_many_lines)]
pub fn execute_stage_plan(plan: &StagePlan) -> Result<StageResultV1> {
    let run_id = Uuid::new_v4().to_string();
    let (r1, r2) = match plan.inputs.as_slice() {
        [] => (None, None),
        [r1] => (Some(r1.as_path()), None),
        [r1, r2, ..] => (Some(r1.as_path()), Some(r2.as_path())),
    };
    let r1 = r1.ok_or_else(|| anyhow!("plan inputs missing r1"))?;
    let r1_dir = r1
        .parent()
        .ok_or_else(|| anyhow!("input r1 has no parent directory"))?;
    let container_name = format!("bijux-stage-{}-{}", plan.stage_id, Uuid::new_v4());
    let run_artifacts_dir = run_artifacts_dir_for_out(&plan.out_dir);
    std::fs::create_dir_all(&run_artifacts_dir).context("create run_artifacts dir")?;
    let (trace_id, span_id) = default_trace_ids();
    let telemetry_path = std::env::var("BIJUX_TELEMETRY_JSONL").map_or_else(
        |_| run_artifacts_dir.join("telemetry").join("events.jsonl"),
        PathBuf::from,
    );
    let canonical_params = canonicalize_json_value(&plan.params);
    let params_hash = params_hash(&canonical_params)?;
    let input_hash = hash_inputs(&plan.inputs)?;
    let plan_artifacts = write_plan_artifacts(
        &run_artifacts_dir,
        &plan.stage_id,
        plan.stage_version,
        &plan.tool,
        &plan.tool_version,
        &plan.inputs,
        &plan.outputs,
        &canonical_params,
    )?;
    write_telemetry_event(
        &telemetry_path,
        &bijux_core::TelemetryEventV1 {
            run_id: run_id.clone(),
            stage_id: plan.stage_id.clone(),
            tool_id: plan.tool.clone(),
            event_name: "stage_start".to_string(),
            timestamp: Utc::now().to_rfc3339(),
            duration_ms: None,
            status: "ok".to_string(),
            trace_id: trace_id.clone(),
            span_id: span_id.clone(),
            attrs: serde_json::json!({
                "params_hash": &params_hash,
                "input_hash": &input_hash,
                "runner": format!("{:?}", plan.runner),
                "image": plan.image.full_name.clone(),
                "tool_version": plan.tool_version.clone(),
            }),
        },
    )?;
    write_telemetry_event(
        &telemetry_path,
        &bijux_core::TelemetryEventV1 {
            run_id: run_id.clone(),
            stage_id: plan.stage_id.clone(),
            tool_id: plan.tool.clone(),
            event_name: "tool_start".to_string(),
            timestamp: Utc::now().to_rfc3339(),
            duration_ms: None,
            status: "ok".to_string(),
            trace_id: trace_id.clone(),
            span_id: span_id.clone(),
            attrs: serde_json::json!({
                "params_hash": &params_hash,
                "input_hash": &input_hash,
                "runner": format!("{:?}", plan.runner),
                "image": plan.image.full_name.clone(),
                "tool_version": plan.tool_version.clone(),
            }),
        },
    )?;
    let start = Instant::now();
    let mut outputs_override: Option<Vec<PathBuf>> = None;
    let execution = match plan.stage_id.as_str() {
        "fastq.merge" => {
            let r2 = r2.ok_or_else(|| anyhow!("merge requires r2 input"))?;
            let exec = run_merge_execution(
                &plan.tool,
                &plan.image,
                r1_dir,
                r1,
                r2,
                &plan.out_dir,
                &container_name,
            )?;
            outputs_override = Some(vec![
                exec.merged_fastq.clone(),
                exec.unmerged_r1.clone(),
                exec.unmerged_r2.clone(),
            ]);
            ExecutionEnvelope {
                exit_code: exec.exit_code,
                stdout: exec.stdout,
                stderr: exec.stderr,
                command: exec.command,
            }
        }
        "fastq.qc_post" if plan.tool == "multiqc" => {
            let fastqc_image = plan
                .aux_images
                .get("fastqc")
                .ok_or_else(|| anyhow!("fastqc image missing for multiqc qc_post"))?;
            let fastqc_dir = plan.out_dir.join("fastqc");
            std::fs::create_dir_all(&fastqc_dir)?;
            let fastqc_container = format!("bijux-stage-fastqc-{}", Uuid::new_v4());
            let fastqc_exec = run_validate_execution(
                "fastqc",
                fastqc_image,
                r1_dir,
                r1,
                &fastqc_dir,
                &fastqc_container,
            )?;
            cleanup_execution(&fastqc_container)?;
            if fastqc_exec.exit_code != 0 {
                return Err(anyhow!("fastqc exit code {}", fastqc_exec.exit_code));
            }
            let exec =
                run_multiqc_execution(&plan.image, &fastqc_dir, &plan.out_dir, &container_name)?;
            ExecutionEnvelope {
                exit_code: exec.exit_code,
                stdout: exec.stdout,
                stderr: exec.stderr,
                command: exec.command,
            }
        }
        "fastq.validate_pre" | "fastq.qc_post" => {
            let exec = run_validate_execution(
                &plan.tool,
                &plan.image,
                r1_dir,
                r1,
                &plan.out_dir,
                &container_name,
            )?;
            ExecutionEnvelope {
                exit_code: exec.exit_code,
                stdout: exec.stdout,
                stderr: exec.stderr,
                command: exec.command,
            }
        }
        _ => {
            let exec = run_tool_execution(
                &plan.tool,
                &plan.image,
                r1_dir,
                r1,
                &plan.out_dir,
                &container_name,
            )?;
            ExecutionEnvelope {
                exit_code: exec.exit_code,
                stdout: exec.stdout,
                stderr: exec.stderr,
                command: exec.command,
            }
        }
    };
    let runtime_s = start.elapsed().as_secs_f64();
    let memory_mb = execution_memory_mb(&container_name)?;
    let duration_ms = {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        {
            (runtime_s * 1000.0).max(0.0) as u64
        }
    };
    cleanup_execution(&container_name)?;
    let outputs = outputs_override.unwrap_or_else(|| plan.outputs.clone());
    let output_hashes = hash_outputs(&outputs)?;
    let ctx = StageObservabilityContextV1 {
        stage_id: plan.stage_id.clone(),
        stage_version: plan.stage_version,
        tool_id: plan.tool.clone(),
        tool_version: plan.tool_version.clone(),
        input_hash: input_hash.clone(),
        params_hash: params_hash.clone(),
        parameters_json: canonical_params.clone(),
    };
    let execution_metrics = bijux_core::measure::ExecutionMetrics {
        runtime_s,
        memory_mb,
        exit_code: execution.exit_code,
    };
    let metrics_envelope_path = write_metrics_envelope(
        &run_artifacts_dir,
        &ctx,
        &execution_metrics,
        &serde_json::json!({}),
        &output_hashes,
    )?;
    let stage_report_path = write_stage_report_v1(
        &run_artifacts_dir,
        &plan.stage_id,
        plan.stage_version,
        &plan.tool,
        &plan.tool_version,
        &outputs,
    )?;
    let retention_report_path = if is_retention_stage(&plan.stage_id) {
        Some(write_retention_report_v1(
            &run_artifacts_dir,
            &plan.stage_id,
            &plan.tool,
            &plan.tool_version,
            &canonical_params,
        )?)
    } else {
        None
    };
    let _observability_manifest = write_observability_manifest(
        &run_artifacts_dir,
        &plan.stage_id,
        &plan.tool,
        &plan_artifacts.plan_path,
        &plan_artifacts.effective_config_path,
        &plan_artifacts.stage_config_path,
        &metrics_envelope_path,
        &stage_report_path,
        retention_report_path.as_deref(),
    )?;
    let _ = insert_stage_row(
        &run_artifacts_dir.join("run_index.jsonl"),
        &StageIndexRow {
            run_id: run_id.clone(),
            stage_id: plan.stage_id.clone(),
            tool_id: plan.tool.clone(),
            params_hash: params_hash.clone(),
            input_hash: input_hash.clone(),
            output_hashes: output_hashes.clone(),
            artifacts: serde_json::json!({
                "plan": plan_artifacts.plan_path.display().to_string(),
                "effective_config": plan_artifacts.effective_config_path.display().to_string(),
                "stage_config": plan_artifacts.stage_config_path.display().to_string(),
                "metrics_envelope": metrics_envelope_path.display().to_string(),
                "stage_report": stage_report_path.display().to_string(),
                "retention_report": retention_report_path.as_ref().map(|path| path.display().to_string()),
            }),
        },
    );
    write_facts_jsonl(
        &run_artifacts_dir.join("dashboard").join("facts.jsonl"),
        &FactsRowV1 {
            schema_version: "bijux.facts_row.v1".to_string(),
            run_id: run_id.clone(),
            stage_id: plan.stage_id.clone(),
            tool_id: plan.tool.clone(),
            params_hash: params_hash.clone(),
            input_hash: input_hash.clone(),
            output_hashes: output_hashes.clone(),
            runtime_s,
            memory_mb,
            exit_code: execution.exit_code,
            metrics: serde_json::json!({}),
            artifacts: serde_json::json!({
                "metrics_envelope": metrics_envelope_path.display().to_string(),
                "stage_report": stage_report_path.display().to_string(),
                "retention_report": retention_report_path.as_ref().map(|path| path.display().to_string()),
            }),
        },
    )?;
    write_telemetry_event(
        &telemetry_path,
        &bijux_core::TelemetryEventV1 {
            run_id: run_id.clone(),
            stage_id: plan.stage_id.clone(),
            tool_id: plan.tool.clone(),
            event_name: "artifact_written".to_string(),
            timestamp: Utc::now().to_rfc3339(),
            duration_ms: None,
            status: "ok".to_string(),
            trace_id: trace_id.clone(),
            span_id: span_id.clone(),
            attrs: serde_json::json!({
                "params_hash": &params_hash,
                "input_hash": &input_hash,
                "output_hashes": &output_hashes,
                "runner": format!("{:?}", plan.runner),
                "image": plan.image.full_name.clone(),
                "metrics_envelope": metrics_envelope_path.display().to_string(),
                "stage_report": stage_report_path.display().to_string(),
                "retention_report": retention_report_path.as_ref().map(|path| path.display().to_string()),
            }),
        },
    )?;
    let marker_path = plan.out_dir.join("engine_execution.json");
    let marker = serde_json::json!({
        "schema_version": "bijux.engine_execution.v1",
        "stage": plan.stage_id,
        "tool": plan.tool,
    });
    std::fs::write(&marker_path, serde_json::to_vec_pretty(&marker)?)
        .context("write engine execution marker")?;
    write_telemetry_event(
        &telemetry_path,
        &bijux_core::TelemetryEventV1 {
            run_id: run_id.clone(),
            stage_id: plan.stage_id.clone(),
            tool_id: plan.tool.clone(),
            event_name: "tool_end".to_string(),
            timestamp: Utc::now().to_rfc3339(),
            duration_ms: Some(duration_ms),
            status: if execution.exit_code == 0 {
                "ok".to_string()
            } else {
                "error".to_string()
            },
            trace_id: trace_id.clone(),
            span_id: span_id.clone(),
            attrs: serde_json::json!({
                "exit_code": execution.exit_code,
                "params_hash": &params_hash,
                "input_hash": &input_hash,
                "output_hashes": &output_hashes,
                "runner": format!("{:?}", plan.runner),
                "image": plan.image.full_name.clone(),
            }),
        },
    )?;
    if execution.exit_code != 0 {
        write_telemetry_event(
            &telemetry_path,
            &bijux_core::TelemetryEventV1 {
                run_id: run_id.clone(),
                stage_id: plan.stage_id.clone(),
                tool_id: plan.tool.clone(),
                event_name: "error".to_string(),
                timestamp: Utc::now().to_rfc3339(),
                duration_ms: None,
                status: "error".to_string(),
                trace_id: trace_id.clone(),
                span_id: span_id.clone(),
                attrs: serde_json::json!({
                    "exit_code": execution.exit_code,
                    "params_hash": &params_hash,
                    "input_hash": &input_hash,
                    "output_hashes": &output_hashes,
                    "runner": format!("{:?}", plan.runner),
                    "image": plan.image.full_name.clone(),
                }),
            },
        )?;
    }
    write_telemetry_event(
        &telemetry_path,
        &bijux_core::TelemetryEventV1 {
            run_id: run_id.clone(),
            stage_id: plan.stage_id.clone(),
            tool_id: plan.tool.clone(),
            event_name: "stage_end".to_string(),
            timestamp: Utc::now().to_rfc3339(),
            duration_ms: Some(duration_ms),
            status: if execution.exit_code == 0 {
                "ok".to_string()
            } else {
                "error".to_string()
            },
            trace_id: trace_id.clone(),
            span_id: span_id.clone(),
            attrs: serde_json::json!({
                "exit_code": execution.exit_code,
                "params_hash": &params_hash,
                "input_hash": &input_hash,
                "output_hashes": &output_hashes,
                "runner": format!("{:?}", plan.runner),
                "image": plan.image.full_name.clone(),
            }),
        },
    )?;
    Ok(StageResultV1 {
        run_id,
        exit_code: execution.exit_code,
        runtime_s,
        memory_mb,
        outputs,
        metrics_path: Some(metrics_envelope_path),
        stdout: execution.stdout,
        stderr: execution.stderr,
        command: execution.command,
    })
}

fn hash_inputs(inputs: &[PathBuf]) -> Result<String> {
    if inputs.is_empty() {
        return Ok("none".to_string());
    }
    let mut hashes = Vec::new();
    for input in inputs {
        hashes.push(hash_file_sha256(input)?);
    }
    Ok(hashes.join(","))
}

fn hash_outputs(outputs: &[PathBuf]) -> Result<Vec<String>> {
    let mut hashes = Vec::new();
    for output in outputs {
        if output.exists() {
            hashes.push(hash_file_sha256(output)?);
        }
    }
    Ok(hashes)
}

fn is_retention_stage(stage_id: &str) -> bool {
    matches!(
        stage_id,
        "fastq.trim"
            | "fastq.filter"
            | "fastq.merge"
            | "fastq.correct"
            | "fastq.umi"
            | "fastq.preprocess"
    )
}
