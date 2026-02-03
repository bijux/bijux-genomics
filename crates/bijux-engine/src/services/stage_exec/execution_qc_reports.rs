
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
            std::fs::write(&path, serde_json::to_vec_pretty(&payload)?)
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
            std::fs::write(&path, serde_json::to_vec_pretty(&suggested_payload)?)
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
