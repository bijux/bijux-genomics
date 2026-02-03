// Consolidated execution helpers to keep stage_exec dir within module-count guardrails.

// --- execution_progress.rs ---

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

// --- execution_records.rs ---

#[allow(clippy::too_many_arguments)]
fn write_facts_and_records(
    plan: &StagePlanV1,
    run_id: &str,
    run_artifacts_dir: &Path,
    trace_id: &str,
    span_id: &str,
    canonical_params: &serde_json::Value,
    params_hash: &str,
    input_hash: &str,
    output_hashes: &[String],
    runtime_s: f64,
    memory_mb: f64,
    execution: &ExecutionEnvelope,
    stage_metrics: &serde_json::Value,
    metrics_envelope_path: &Path,
    stage_report_path: &Path,
    retention_report_path: Option<&Path>,
    effective_adapters_path: Option<&Path>,
    log_paths: &[PathBuf],
    quality_gate: Option<&serde_json::Value>,
    adapter_validation: Option<&serde_json::Value>,
    contaminant_action: bool,
    assertion_results: &[bijux_core::InvariantResultV1],
    _invariant_verdict: &bijux_core::StageVerdictV1,
    subreports: &[PathBuf],
    reads_in: u64,
    reads_out: u64,
    bases_in: u64,
    bases_out: u64,
    pairs_in: u64,
    pairs_out: u64,
) -> Result<()> {
    let assertions_payload = serde_json::json!({
        "schema_version": "bijux.assertions.v1",
        "results": assertion_results,
    });
    let scientific_preset = std::env::var("BIJUX_SCIENTIFIC_PRESET").ok();
    write_facts_jsonl(
        &run_artifacts_dir.join("dashboard").join("facts.jsonl"),
        &FactsRowV1 {
            schema_version: "bijux.facts.v1".to_string(),
            run_id: run_id.to_string(),
            stage_id: plan.stage_id.0.clone(),
            tool_id: plan.tool_id.0.clone(),
            tool_version: plan.tool_version.clone(),
            image_digest: plan.image.digest.clone(),
            trace_id: trace_id.to_string(),
            span_id: span_id.to_string(),
            params_hash: params_hash.to_string(),
            input_hash: input_hash.to_string(),
            output_hashes: output_hashes.to_vec(),
            runtime_s,
            memory_mb,
            exit_code: execution.exit_code,
            bank_hashes: bank_refs_from_params(canonical_params),
            reads_in: Some(reads_in),
            reads_out: Some(reads_out),
            bases_in: Some(bases_in),
            bases_out: Some(bases_out),
            pairs_in: Some(pairs_in),
            pairs_out: Some(pairs_out),
            metrics: stage_metrics.clone(),
            reports: serde_json::json!({
                "stage_report": stage_report_path.display().to_string(),
                "retention_report": retention_report_path
                    .as_ref()
                    .map(|path| path.display().to_string()),
                "bank_report": subreports
                    .iter()
                    .find(|path| path.ends_with("bank_report.json"))
                    .map(|path| path.display().to_string()),
                "qc_post_report": subreports
                    .iter()
                    .find(|path| path.ends_with("qc_post_report.json"))
                    .map(|path| path.display().to_string()),
                "filter_report": subreports
                    .iter()
                    .find(|path| path.ends_with("filter_report.json"))
                    .map(|path| path.display().to_string()),
                "adapter_candidates": subreports
                    .iter()
                    .find(|path| path.ends_with("adapter_candidates.json"))
                    .map(|path| path.display().to_string()),
                "fastqc_metrics_v2": subreports
                    .iter()
                    .find(|path| path.ends_with("fastqc_metrics_v2.json"))
                    .map(|path| path.display().to_string()),
                "quality_gate": quality_gate.cloned(),
                "adapter_validation": adapter_validation.cloned(),
                "contaminant_action": contaminant_action,
                "assertions": assertions_payload,
                "scientific_preset": scientific_preset,
            }),
            artifacts: serde_json::json!({
                "metrics_envelope": metrics_envelope_path.display().to_string(),
                "stage_report": stage_report_path.display().to_string(),
                "retention_report": retention_report_path
                    .as_ref()
                    .map(|path| path.display().to_string()),
                "effective_adapters": effective_adapters_path
                    .as_ref()
                    .map(|path| path.display().to_string()),
                "execution_logs": log_paths
                    .iter()
                    .map(|path| path.display().to_string())
                    .collect::<Vec<_>>(),
            }),
        },
    )?;

    Ok(())
}

// --- execution_qc_reports.rs ---

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
fn write_qc_post_reports(
    plan: &StagePlanV1,
    run_id: &str,
    run_artifacts_dir: &Path,
    trace_id: &str,
    span_id: &str,
    canonical_params: &serde_json::Value,
    emit_event: &dyn Fn(&bijux_core::TelemetryEventV1) -> Result<()>,
    emit_artifact: &dyn Fn(&str, &Path) -> Result<()>,
    subreports: &mut Vec<PathBuf>,
    extra_warnings: &mut Vec<String>,
) -> Result<Option<serde_json::Value>> {
    let mut adapter_validation: Option<serde_json::Value> = None;
    if plan.stage_id.0 == "fastq.qc_post" {
        let raw_dir = plan.out_dir.join("fastqc_raw");
        let trimmed_dir = plan.out_dir.join("fastqc_trimmed");
        let raw_modules = fastqc_modules_from_dir(&raw_dir);
        let trimmed_modules = fastqc_modules_from_dir(&trimmed_dir);
        let raw_dir_opt = if raw_dir.exists() {
            Some(raw_dir.as_path())
        } else {
            None
        };
        let trimmed_dir_opt = if trimmed_dir.exists() {
            Some(trimmed_dir.as_path())
        } else {
            None
        };
        let multiqc_report = plan.out_dir.join("multiqc_report.html");
        let multiqc_data = plan.out_dir.join("multiqc_data");
        let raw_metrics_v2 = fastqc_metrics_v2_from_dir(&raw_dir);
        let trimmed_metrics_v2 = fastqc_metrics_v2_from_dir(&trimmed_dir);
        let fastqc_metrics_path = if raw_metrics_v2.is_some() || trimmed_metrics_v2.is_some() {
            let reports_dir = run_artifacts_dir.join("reports");
            std::fs::create_dir_all(&reports_dir).context("create reports dir")?;
            let payload = serde_json::json!({
                "schema_version": "bijux.fastqc_metrics.v2",
                "raw": raw_metrics_v2,
                "trimmed": trimmed_metrics_v2,
            });
            let path = reports_dir.join("fastqc_metrics_v2.json");
            bijux_io::atomic_write_json(&path, &payload)
                .context("write fastqc_metrics_v2.json")?;
            emit_artifact("fastqc_metrics_v2", &path)?;
            subreports.push(path.clone());
            Some(path)
        } else {
            None
        };
        let (suggested_payload, suggested_preset) = if raw_dir.exists() {
            adapter_suggestions_from_fastqc(&raw_dir)
        } else {
            adapter_suggestions_from_fastqc(&trimmed_dir)
        };
        if let Some(preset) = suggested_preset.as_deref() {
            let current = canonical_params
                .get("adapter_bank")
                .and_then(|value| value.get("preset"))
                .and_then(|value| value.as_str());
            if current == Some("illumina-default") {
                adapter_validation = Some(serde_json::json!({
                    "current_preset": current,
                    "suggested_preset": preset,
                    "status": "warn",
                }));
                extra_warnings.push(format!(
                    "warning: adapter signal detected; consider preset '{preset}'"
                ));
                emit_event(&bijux_core::TelemetryEventV1 {
                    schema_version: "bijux.telemetry.v1".to_string(),
                    run_id: run_id.to_string(),
                    stage_id: plan.stage_id.0.clone(),
                    tool_id: plan.tool_id.0.clone(),
                    event_name: "adapter_validation".to_string(),
                    timestamp: Utc::now().to_rfc3339(),
                    duration_ms: None,
                    status: "warn".to_string(),
                    trace_id: trace_id.to_string(),
                    span_id: span_id.to_string(),
                    attrs: serde_json::json!({
                        "current_preset": current,
                        "suggested_preset": preset,
                    }),
                })?;
            }
        }
        let suggested_path = if suggested_payload
            .as_object()
            .is_none_or(serde_json::Map::is_empty)
        {
            None
        } else {
            let reports_dir = run_artifacts_dir.join("reports");
            std::fs::create_dir_all(&reports_dir).context("create reports dir")?;
            let path = reports_dir.join("suggested_adapters.json");
            bijux_io::atomic_write_json(&path, &suggested_payload)
                .context("write suggested adapters")?;
            emit_artifact("suggested_adapters", &path)?;
            Some(path)
        };

        let report_path = write_qc_post_report_v1(
            run_artifacts_dir,
            &plan.stage_id.0,
            &plan.tool_id.0,
            raw_dir_opt,
            trimmed_dir_opt,
            multiqc_report.exists().then_some(multiqc_report.as_path()),
            multiqc_data.exists().then_some(multiqc_data.as_path()),
            raw_modules,
            trimmed_modules,
            fastqc_metrics_path.as_deref(),
            suggested_path.as_deref(),
            suggested_preset.as_deref(),
        )?;
        emit_artifact("qc_post_report", &report_path)?;
        subreports.push(report_path);
    }

    Ok(adapter_validation)
}
