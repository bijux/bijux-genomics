pub fn write_stage_event_jsonl(
    run_artifacts_dir: &Path,
    event: &bijux_core::TelemetryEventV1,
) -> Result<PathBuf> {
    let path = run_artifacts_dir.join("stage_events.jsonl");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).context("create stage_events dir")?;
    }
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .context("open stage_events.jsonl")?;
    writeln!(file, "{}", serde_json::to_string(event)?)?;
    Ok(path)
}

pub fn write_progress_event_jsonl(
    run_artifacts_dir: &Path,
    event: &ProgressEventV1,
) -> Result<PathBuf> {
    let path = run_artifacts_dir.join("progress.jsonl");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).context("create progress dir")?;
    }
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .context("open progress.jsonl")?;
    writeln!(file, "{}", serde_json::to_string(event)?)?;
    Ok(path)
}

pub fn write_runs_export_jsonl(run_artifacts_dir: &Path, row: &RunsExportRowV1) -> Result<PathBuf> {
    let dashboard_dir = run_artifacts_dir.join("dashboard");
    std::fs::create_dir_all(&dashboard_dir).context("create dashboard dir")?;
    let path = dashboard_dir.join("runs.jsonl");
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .context("open runs.jsonl")?;
    writeln!(file, "{}", serde_json::to_string(row)?)?;
    Ok(path)
}

#[allow(clippy::too_many_arguments)]
pub fn write_stage_report_v1(
    run_artifacts_dir: &Path,
    stage_id: &str,
    stage_version: i32,
    tool_id: &str,
    tool_version: &str,
    metrics_path: &Path,
    tool_invocation_path: &Path,
    effective_config_path: &Path,
    facts_row_id: Option<&str>,
    outputs: &[PathBuf],
    subreports: &[PathBuf],
    log_paths: &[PathBuf],
    warnings: &[String],
    errors: &[String],
    invariants: &[bijux_core::InvariantResultV1],
    verdict: Option<&bijux_core::StageVerdictV1>,
) -> Result<PathBuf> {
    let effective_config_hash =
        crate::services::observer::hash_file_sha256(effective_config_path).ok();
    let payload = StageReportV1 {
        schema_version: "bijux.stage_report.v1".to_string(),
        stage_id: stage_id.to_string(),
        stage_version,
        tool_id: tool_id.to_string(),
        tool_version: tool_version.to_string(),
        metrics_path: metrics_path.display().to_string(),
        tool_invocation_path: tool_invocation_path.display().to_string(),
        effective_config_path: effective_config_path.display().to_string(),
        effective_config_hash,
        facts_row_id: facts_row_id.map(str::to_string),
        summary: serde_json::json!({
            "outputs": outputs.iter().map(|p| p.display().to_string()).collect::<Vec<_>>()
        }),
        warnings: warnings.to_vec(),
        errors: errors.to_vec(),
        invariants: invariants.to_vec(),
        verdict: verdict.cloned(),
        outputs: outputs.iter().map(|p| p.display().to_string()).collect(),
        subreports: subreports.iter().map(|p| p.display().to_string()).collect(),
        log_paths: log_paths.iter().map(|p| p.display().to_string()).collect(),
    };
    let path = run_artifacts_dir.join("stage_report.json");
    bijux_io::atomic_write_json(&path, &payload).context("write stage_report.json")?;
    Ok(path)
}

#[allow(clippy::too_many_arguments)]
pub fn write_retention_report_v1(
    run_artifacts_dir: &Path,
    stage_id: &str,
    tool_id: &str,
    tool_version: &str,
    condition: &serde_json::Value,
    parameters_json: &serde_json::Value,
    reads_in: u64,
    reads_out: u64,
    bases_in: u64,
    bases_out: u64,
) -> Result<PathBuf> {
    let reports_dir = run_artifacts_dir.join("reports");
    std::fs::create_dir_all(&reports_dir).context("create reports dir")?;
    let file_name = format!("{stage_id}.retention.json");
    let payload = RetentionReportV1 {
        schema_version: "bijux.retention_report.v1".to_string(),
        stage_id: stage_id.to_string(),
        tool_id: tool_id.to_string(),
        tool_version: tool_version.to_string(),
        boundary: "pre/post".to_string(),
        numerator: serde_json::json!({
            "reads_out": reads_out,
            "bases_out": bases_out,
        }),
        denominator: serde_json::json!({
            "reads_in": reads_in,
            "bases_in": bases_in,
        }),
        scope: "reads+bases".to_string(),
        condition: condition.clone(),
        parameters_json: parameters_json.clone(),
        retention: Some(bijux_core::RetentionReportMetricV1 {
            #[allow(clippy::cast_precision_loss)]
            value: if reads_in > 0 {
                (reads_out as f64) / (reads_in as f64)
            } else {
                0.0
            },
            numerator_reads: reads_out,
            denominator_reads: reads_in,
            numerator_bases: bases_out,
            denominator_bases: bases_in,
            definition: "reads_out / reads_in".to_string(),
            stage_boundary: stage_id.to_string(),
            conditions: serde_json::json!({
                "condition": condition.clone(),
                "parameters": parameters_json.clone(),
            }),
        }),
    };
    let path = reports_dir.join(file_name);
    bijux_io::atomic_write_json(&path, &payload).context("write retention report")?;
    Ok(path)
}

#[allow(clippy::too_many_arguments)]
pub fn write_trim_report_v1(
    run_artifacts_dir: &Path,
    stage_id: &str,
    tool_id: &str,
    reads_in: u64,
    reads_out: u64,
    bases_in: u64,
    bases_out: u64,
    adapter_preset: Option<String>,
    adapter_bank_id: Option<String>,
    adapter_bank_hash: Option<String>,
    adapter_overrides: Option<serde_json::Value>,
) -> Result<PathBuf> {
    let reports_dir = run_artifacts_dir.join("reports");
    std::fs::create_dir_all(&reports_dir).context("create reports dir")?;
    let file_name = format!("{stage_id}.trim_report.json");
    let payload = TrimReportV1 {
        schema_version: "bijux.trim_report.v1".to_string(),
        stage_id: stage_id.to_string(),
        tool_id: tool_id.to_string(),
        reads_in,
        reads_out,
        bases_in,
        bases_out,
        bases_trimmed: bases_in.saturating_sub(bases_out),
        per_adapter_counts: std::collections::BTreeMap::new(),
        adapter_preset,
        adapter_bank_id,
        adapter_bank_hash,
        adapter_overrides,
    };
    let path = reports_dir.join(file_name);
    bijux_io::atomic_write_json(&path, &payload).context("write trim report")?;
    Ok(path)
}

pub fn write_validate_report_v1(
    run_artifacts_dir: &Path,
    stage_id: &str,
    tool_id: &str,
    reads_total: u64,
    reads_valid: u64,
    reads_invalid: u64,
) -> Result<PathBuf> {
    let reports_dir = run_artifacts_dir.join("reports");
    std::fs::create_dir_all(&reports_dir).context("create reports dir")?;
    let file_name = format!("{stage_id}.validate_report.json");
    let payload = ValidateReportV1 {
        schema_version: "bijux.validate_report.v1".to_string(),
        stage_id: stage_id.to_string(),
        tool_id: tool_id.to_string(),
        reads_total,
        reads_valid,
        reads_invalid,
        integrity_ok: reads_invalid == 0,
    };
    let path = reports_dir.join(file_name);
    bijux_io::atomic_write_json(&path, &payload).context("write validate report")?;
    Ok(path)
}

#[allow(clippy::too_many_arguments)]
pub fn write_filter_report_v1(
    run_artifacts_dir: &Path,
    stage_id: &str,
    tool_id: &str,
    reads_in: u64,
    reads_out: u64,
    reads_removed_total: u64,
    reads_removed_by_n: u64,
    reads_removed_by_entropy: u64,
    reads_removed_low_complexity: u64,
    reads_removed_by_kmer: u64,
    reads_removed_contaminant_kmer: u64,
    reads_removed_by_length: u64,
    entropy_distribution: serde_json::Value,
    conditions: serde_json::Value,
    redundant_filters: Vec<String>,
) -> Result<PathBuf> {
    let reports_dir = run_artifacts_dir.join("reports");
    std::fs::create_dir_all(&reports_dir).context("create reports dir")?;
    let file_name = format!("{stage_id}.filter_report.json");
    let payload = FilterReportV1 {
        schema_version: "bijux.filter_report.v1".to_string(),
        stage_id: stage_id.to_string(),
        tool_id: tool_id.to_string(),
        reads_in,
        reads_out,
        reads_removed_total,
        reads_removed_by_n,
        reads_removed_by_entropy,
        reads_removed_low_complexity,
        reads_removed_by_kmer,
        reads_removed_contaminant_kmer,
        reads_removed_by_length,
        entropy_distribution,
        conditions,
        redundant_filters,
    };
    let path = reports_dir.join(file_name);
    bijux_io::atomic_write_json(&path, &payload).context("write filter report")?;
    Ok(path)
}

#[allow(clippy::too_many_arguments)]
pub fn write_merge_report_v1(
    run_artifacts_dir: &Path,
    stage_id: &str,
    tool_id: &str,
    reads_r1: u64,
    reads_r2: u64,
    reads_merged: u64,
    reads_unmerged: u64,
    merge_rate: f64,
) -> Result<PathBuf> {
    let reports_dir = run_artifacts_dir.join("reports");
    std::fs::create_dir_all(&reports_dir).context("create reports dir")?;
    let file_name = format!("{stage_id}.merge_report.json");
    let payload = MergeReportV1 {
        schema_version: "bijux.merge_report.v1".to_string(),
        stage_id: stage_id.to_string(),
        tool_id: tool_id.to_string(),
        reads_r1,
        reads_r2,
        reads_merged,
        reads_unmerged,
        merge_rate,
    };
    let path = reports_dir.join(file_name);
    bijux_io::atomic_write_json(&path, &payload).context("write merge report")?;
    Ok(path)
}

#[allow(clippy::too_many_arguments)]
pub fn write_qc_post_report_v1(
    run_artifacts_dir: &Path,
    stage_id: &str,
    tool_id: &str,
    raw_fastqc_dir: Option<&Path>,
    trimmed_fastqc_dir: Option<&Path>,
    multiqc_report: Option<&Path>,
    multiqc_data: Option<&Path>,
    fastqc_raw_modules: serde_json::Value,
    fastqc_trimmed_modules: serde_json::Value,
    fastqc_metrics_v2_path: Option<&Path>,
    suggested_adapters_path: Option<&Path>,
    suggested_preset: Option<&str>,
) -> Result<PathBuf> {
    let reports_dir = run_artifacts_dir.join("reports");
    std::fs::create_dir_all(&reports_dir).context("create reports dir")?;
    let file_name = format!("{stage_id}.qc_post_report.json");
    let payload = QcPostReportV1 {
        schema_version: "bijux.qc_post_report.v1".to_string(),
        stage_id: stage_id.to_string(),
        tool_id: tool_id.to_string(),
        raw_fastqc_dir: raw_fastqc_dir.map(|path| path.display().to_string()),
        trimmed_fastqc_dir: trimmed_fastqc_dir.map(|path| path.display().to_string()),
        multiqc_report: multiqc_report.map(|path| path.display().to_string()),
        multiqc_data: multiqc_data.map(|path| path.display().to_string()),
        fastqc_raw_modules,
        fastqc_trimmed_modules,
        fastqc_metrics_v2_path: fastqc_metrics_v2_path.map(|path| path.display().to_string()),
        suggested_adapters_path: suggested_adapters_path.map(|path| path.display().to_string()),
        suggested_preset: suggested_preset.map(str::to_string),
    };
    let path = reports_dir.join(file_name);
    bijux_io::atomic_write_json(&path, &payload).context("write qc_post report")?;
    Ok(path)
}

pub fn write_telemetry_event(path: &Path, event: &TelemetryEventV1) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).context("create telemetry dir")?;
    }
    let line = serde_json::to_string(event)?;
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .context("open telemetry jsonl")?
        .write_all(format!("{line}\n").as_bytes())
        .context("append telemetry jsonl")?;
    Ok(())
}

pub fn write_facts_jsonl(path: &Path, fact: &FactsRowV1) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).context("create dashboard dir")?;
    }
    let line = serde_json::to_string(fact)?;
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .context("open facts jsonl")?
        .write_all(format!("{line}\n").as_bytes())
        .context("append facts jsonl")?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn write_observability_manifest(
    run_artifacts_dir: &Path,
    stage_id: &str,
    tool_id: &str,
    plan_path: &Path,
    effective_config_path: &Path,
    stage_config_path: &Path,
    tool_invocation_path: &Path,
    metrics_envelope_path: &Path,
    stage_metrics_path: &Path,
    stage_report_path: &Path,
    retention_report_path: Option<&Path>,
    extra_artifacts: &[serde_json::Value],
) -> Result<PathBuf> {
    let mut artifacts = vec![
        serde_json::json!({
            "name": "plan",
            "path": plan_path,
        }),
        serde_json::json!({
            "name": "effective_config",
            "path": effective_config_path,
        }),
        serde_json::json!({
            "name": "stage_config",
            "path": stage_config_path,
        }),
        serde_json::json!({
            "name": "tool_invocation",
            "path": tool_invocation_path,
        }),
        serde_json::json!({
            "name": "metrics_envelope",
            "path": metrics_envelope_path,
        }),
        serde_json::json!({
            "name": "stage_metrics",
            "path": stage_metrics_path,
        }),
        serde_json::json!({
            "name": "stage_report",
            "path": stage_report_path,
        }),
    ];
    if let Some(path) = retention_report_path {
        artifacts.push(serde_json::json!({
            "name": "retention_report",
            "path": path,
        }));
    }
    if !extra_artifacts.is_empty() {
        artifacts.extend(extra_artifacts.iter().cloned());
    }
    let payload = ObservabilityManifestV1 {
        schema_version: "bijux.observability_manifest.v1",
        stage_id: stage_id.to_string(),
        tool_id: tool_id.to_string(),
        artifacts,
    };
    let path = run_artifacts_dir.join("observability_manifest.json");
    bijux_io::atomic_write_json(&path, &payload).context("write observability_manifest.json")?;
    Ok(path)
}
