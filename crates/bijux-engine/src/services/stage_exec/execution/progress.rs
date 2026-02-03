
#[allow(clippy::too_many_arguments)]
fn write_progress_and_runs(
    plan: &StagePlanV1,
    run_id: &str,
    run_artifacts_dir: &Path,
    started_at: chrono::DateTime<Utc>,
    runtime_s: f64,
    memory_mb: f64,
    execution: &ExecutionEnvelope,
    params_hash: &str,
    input_hash: &str,
    metrics_envelope_path: &Path,
    outputs: &[PathBuf],
) -> Result<()> {
    let finished_at = Utc::now();
    let progress_status = if execution.exit_code == 0 { "ok" } else { "error" };
    write_progress_event_jsonl(
        run_artifacts_dir,
        &crate::services::run_artifacts::ProgressEventV1 {
            schema_version: "bijux.progress.v1",
            stage_id: plan.stage_id.0.clone(),
            tool_id: plan.tool_id.0.clone(),
            status: progress_status.to_string(),
            started_at: started_at.to_rfc3339(),
            finished_at: finished_at.to_rfc3339(),
            outputs: outputs
                .iter()
                .map(|path| path.display().to_string())
                .collect(),
            metrics_path: Some(metrics_envelope_path.display().to_string()),
        },
    )?;
    write_runs_export_jsonl(
        run_artifacts_dir,
        &crate::services::run_artifacts::RunsExportRowV1 {
            schema_version: "bijux.runs_export.v1",
            run_id: run_id.to_string(),
            stage_id: plan.stage_id.0.clone(),
            tool_id: plan.tool_id.0.clone(),
            tool_version: plan.tool_version.clone(),
            started_at: started_at.to_rfc3339(),
            finished_at: finished_at.to_rfc3339(),
            runtime_s,
            memory_mb,
            exit_code: execution.exit_code,
            params_hash: params_hash.to_string(),
            input_hash: input_hash.to_string(),
            metrics_path: Some(metrics_envelope_path.display().to_string()),
        },
    )?;

    Ok(())
}
