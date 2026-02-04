use bijux_stages_bam::domain_bam::metrics::{
    evaluate_bam_invariants, BamInvariantThresholds, BamMetricsV1,
};
use bijux_stages_fastq::metrics::{
    filter_removals_for_plan, retention_conditions_from_effective, stats_or_zero,
};

struct ReportArtifacts {
    subreports: Vec<PathBuf>,
    assertion_results: Vec<bijux_core::InvariantResultV1>,
    invariant_verdict: bijux_core::StageVerdictV1,
    retention_report_path: Option<PathBuf>,
    effective_adapters_path: Option<PathBuf>,
    stage_report_path: PathBuf,
    quality_gate: Option<serde_json::Value>,
    adapter_validation: Option<serde_json::Value>,
    contaminant_action: bool,
}

struct InvariantOutcome {
    results: Vec<bijux_core::InvariantResultV1>,
    verdict: bijux_core::StageVerdictV1,
}

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
fn build_stage_reports_and_warnings(
    plan: &StagePlanV1,
    run_id: &str,
    run_artifacts_dir: &Path,
    trace_id: &str,
    span_id: &str,
    canonical_params: &serde_json::Value,
    adapter_bank: Option<&AdapterBankProvenanceV1>,
    bank_assets: Option<&serde_json::Value>,
    banks_json: Option<&serde_json::Value>,
    input_paths: &[PathBuf],
    outputs: &[PathBuf],
    stage_metrics: &serde_json::Value,
    metrics_path: &Path,
    tool_invocation_path: &Path,
    plan_artifacts: &bijux_engine::services::run_artifacts::PlanArtifacts,
    facts_row_id: &str,
    log_paths: &[PathBuf],
    params_hash: &str,
    input_hash: &str,
    output_hashes: &[String],
    metrics_envelope_path: &Path,
    stage_metrics_path: &Path,
    emit_event: &dyn Fn(&bijux_core::TelemetryEventV1) -> Result<()>,
    emit_artifact: &dyn Fn(&str, &Path) -> Result<()>,
) -> Result<ReportArtifacts> {
    let mut subreports: Vec<PathBuf> = Vec::new();
    let mut extra_warnings: Vec<String> = Vec::new();
    let mut contaminant_action = false;
    if let Some(banks_value) = banks_json.and_then(|value| value.as_object()) {
        let mut banks_report = serde_json::Map::new();
        for (bank_name, bank_value) in banks_value {
            let entries = bank_entries_from_value(bank_value);
            let references = bank_references_from_value(bank_value);
            let assets_for_bank = bank_assets
                .and_then(|assets| assets.get(bank_name))
                .cloned();
            let bank_entry_report = serde_json::json!({
                "bank_id": bank_value.get("bank_id"),
                "bank_hash": bank_value.get("bank_hash"),
                "preset": bank_value.get("preset"),
                "preset_hash": bank_value.get("preset_hash"),
                "enabled_entries": entries.iter().map(|entry| {
                    serde_json::json!({
                        "id": entry.id,
                        "rationale": entry.rationale,
                        "source": entry.source,
                    })
                }).collect::<Vec<_>>(),
                "references": references.iter().map(|reference| {
                    serde_json::json!({
                        "id": reference.id,
                        "file": reference.file,
                        "sha256": reference.sha256,
                        "rationale": reference.rationale,
                        "source": reference.source,
                    })
                }).collect::<Vec<_>>(),
                "assets": assets_for_bank,
            });
            banks_report.insert(bank_name.clone(), bank_entry_report);
        }
        let report_payload = serde_json::json!({
            "schema_version": "bijux.bank_report.v1",
            "stage_id": plan.stage_id.0,
            "tool_id": plan.tool_id.0,
            "banks": banks_report,
        });
        let reports_dir = run_artifacts_dir.join("reports");
        bijux_infra::ensure_dir(&reports_dir).context("create reports dir")?;
        let report_path = reports_dir.join("bank_report.json");
        bijux_infra::atomic_write_json(&report_path, &report_payload)
            .context("write bank_report.json")?;
        emit_artifact("bank_report", &report_path)?;
        subreports.push(report_path);
    }
    if plan.stage_id.0 == "fastq.trim" {
        let input = stats_or_zero(input_paths.first().map(PathBuf::as_path))?;
        let output = stats_or_zero(outputs.first().map(PathBuf::as_path))?;
        let adapter_bank = canonical_params.get("adapter_bank");
        let adapter_preset = adapter_bank
            .and_then(|value| value.get("preset"))
            .and_then(|value| value.as_str())
            .map(str::to_string);
        let adapter_bank_id = adapter_bank
            .and_then(|value| value.get("bank_id"))
            .and_then(|value| value.as_str())
            .map(str::to_string);
        let adapter_bank_hash = adapter_bank
            .and_then(|value| value.get("bank_hash"))
            .and_then(|value| value.as_str())
            .map(str::to_string);
        let adapter_overrides = canonical_params.get("adapter_overrides").cloned();
        let report_path = write_trim_report_v1(
            run_artifacts_dir,
            &plan.stage_id.0,
            &plan.tool_id.0,
            input.reads,
            output.reads,
            input.bases,
            output.bases,
            adapter_preset,
            adapter_bank_id,
            adapter_bank_hash,
            adapter_overrides,
        )?;
        emit_artifact("trim_report", &report_path)?;
        subreports.push(report_path);
    }
    if plan.stage_id.0 == "fastq.validate_pre" {
        let input = stats_or_zero(input_paths.first().map(PathBuf::as_path))?;
        let report_path = write_validate_report_v1(
            run_artifacts_dir,
            &plan.stage_id.0,
            &plan.tool_id.0,
            input.reads,
            input.reads,
            0,
        )?;
        emit_artifact("validate_report", &report_path)?;
        subreports.push(report_path);
    }
    if plan.stage_id.0 == "fastq.detect_adapters" {
        let (suggested_payload, suggested_preset) = adapter_suggestions_from_fastqc(&plan.out_dir);
        if suggested_payload
            .as_object()
            .is_some_and(|map| !map.is_empty())
        {
            let reports_dir = run_artifacts_dir.join("reports");
            bijux_infra::ensure_dir(&reports_dir).context("create reports dir")?;
            let path = reports_dir.join("adapter_candidates.json");
            let payload = serde_json::json!({
                "schema_version": "bijux.adapter_candidates.v1",
                "stage_id": plan.stage_id.0,
                "tool_id": plan.tool_id.0,
                "suggested_preset": suggested_preset,
                "candidates": suggested_payload
                    .get("candidates")
                    .cloned()
                    .unwrap_or(serde_json::json!([])),
            });
            bijux_infra::atomic_write_json(&path, &payload)
                .context("write adapter_candidates.json")?;
            emit_artifact("adapter_candidates", &path)?;
            subreports.push(path);
        }
    }
    if plan.stage_id.0 == "fastq.filter" {
        let input = stats_or_zero(input_paths.first().map(PathBuf::as_path))?;
        let output = stats_or_zero(outputs.first().map(PathBuf::as_path))?;
        let redundant_filters = canonical_params
            .get("redundant_filters")
            .and_then(|value| value.as_array())
            .map(|values| {
                values
                    .iter()
                    .filter_map(|value| value.as_str().map(str::to_string))
                    .collect::<Vec<String>>()
            })
            .unwrap_or_default();
        let removals =
            filter_removals_for_plan(plan.tool_id.0.as_str(), &plan.out_dir, canonical_params);
        let report_path = write_filter_report_v1(
            run_artifacts_dir,
            &plan.stage_id.0,
            &plan.tool_id.0,
            input.reads,
            output.reads,
            input.reads.saturating_sub(output.reads),
            removals.by_n,
            removals.by_entropy,
            removals.by_low_complexity,
            removals.by_kmer,
            removals.by_contaminant_kmer,
            removals.by_length,
            serde_json::json!({}),
            retention_conditions_from_effective(
                &plan.stage_id.0,
                &plan.effective_params,
                canonical_params,
            ),
            redundant_filters,
        )?;
        emit_artifact("filter_report", &report_path)?;
        subreports.push(report_path);
    }
    if plan.stage_id.0 == "fastq.merge" {
        let r1 = stats_or_zero(input_paths.first().map(PathBuf::as_path))?;
        let r2 = stats_or_zero(input_paths.get(1).map(PathBuf::as_path))?;
        let merged = stats_or_zero(outputs.first().map(PathBuf::as_path))?;
        let unmerged_r1 = stats_or_zero(outputs.get(1).map(PathBuf::as_path))?;
        let unmerged_r2 = stats_or_zero(outputs.get(2).map(PathBuf::as_path))?;
        let reads_unmerged = unmerged_r1.reads.min(unmerged_r2.reads);
        let min_reads = r1.reads.min(r2.reads);
        let merge_rate = if min_reads > 0 {
            f64_from_u64(merged.reads) / f64_from_u64(min_reads)
        } else {
            0.0
        };
        let report_path = write_merge_report_v1(
            run_artifacts_dir,
            &plan.stage_id.0,
            &plan.tool_id.0,
            r1.reads,
            r2.reads,
            merged.reads,
            reads_unmerged,
            merge_rate,
        )?;
        emit_artifact("merge_report", &report_path)?;
        subreports.push(report_path);
    }
    let adapter_validation = write_qc_post_reports(
        plan,
        run_id,
        run_artifacts_dir,
        trace_id,
        span_id,
        canonical_params,
        emit_event,
        emit_artifact,
        &mut subreports,
        &mut extra_warnings,
    )?;

    let mut warnings = warnings_for_plan(plan, canonical_params);
    warnings.extend(extra_warnings);
    let (reads_in, reads_out, ..) = extract_io_deltas(stage_metrics);
    let invariant_eval = if plan.stage_id.0.starts_with("bam.") {
        let bam_metrics: BamMetricsV1 = serde_json::from_value(stage_metrics.clone())?;
        let thresholds = BamInvariantThresholds::default();
        let outcome = evaluate_bam_invariants(&plan.stage_id.0, &bam_metrics, &thresholds);
        InvariantOutcome {
            results: outcome.results,
            verdict: outcome.verdict,
        }
    } else {
        let thresholds = thresholds_from_env();
        let outcome = evaluate_invariants(
            &plan.stage_id.0,
            stage_metrics,
            &plan.effective_params,
            &thresholds,
        )
        ;
        InvariantOutcome {
            results: outcome.results,
            verdict: outcome.verdict,
        }
    };
    let assertion_results = invariant_eval.results.clone();
    if plan.stage_id.0 == "fastq.filter" && canonical_params.get("kmer_ref").is_some() {
        contaminant_action = true;
        emit_event(&bijux_core::TelemetryEventV1 {
            schema_version: "bijux.telemetry.v1".to_string(),
            run_id: run_id.to_string(),
            stage_id: plan.stage_id.0.clone(),
            tool_id: plan.tool_id.0.clone(),
            event_name: "contaminant_action".to_string(),
            timestamp: Utc::now().to_rfc3339(),
            duration_ms: None,
            status: "ok".to_string(),
            trace_id: trace_id.to_string(),
            span_id: span_id.to_string(),
            attrs: serde_json::json!({
                "enabled": true,
                "kmer_ref": canonical_params.get("kmer_ref"),
            }),
        })?;
    }
    let quality_gate =
        quality_gate_decision(&plan.stage_id.0, stage_metrics, reads_in, reads_out);
    if let Some(decision) = quality_gate.as_ref() {
        let status = decision
            .get("status")
            .and_then(|value| value.as_str())
            .unwrap_or("pass");
        emit_event(&bijux_core::TelemetryEventV1 {
            schema_version: "bijux.telemetry.v1".to_string(),
            run_id: run_id.to_string(),
            stage_id: plan.stage_id.0.clone(),
            tool_id: plan.tool_id.0.clone(),
            event_name: "quality_gate".to_string(),
            timestamp: Utc::now().to_rfc3339(),
            duration_ms: None,
            status: status.to_string(),
            trace_id: trace_id.to_string(),
            span_id: span_id.to_string(),
            attrs: decision.clone(),
        })?;
        if status != "pass" {
            warnings.push(format!(
                "warning: quality gate {} for stage {}",
                status, plan.stage_id.0
            ));
        }
    }

    let stage_report_path = write_stage_report_v1(
        run_artifacts_dir,
        &plan.stage_id.0,
        stage_version_i32(plan.stage_version),
        &plan.tool_id.0,
        &plan.tool_version,
        metrics_path,
        tool_invocation_path,
        &plan_artifacts.effective_config_path,
        Some(facts_row_id),
        outputs,
        &subreports,
        log_paths,
        &warnings,
        &[],
        &assertion_results,
        Some(&invariant_eval.verdict),
    )?;
    emit_artifact("stage_report", &stage_report_path)?;
    for warning in &warnings {
        emit_event(&bijux_core::TelemetryEventV1 {
            schema_version: "bijux.telemetry.v1".to_string(),
            run_id: run_id.to_string(),
            stage_id: plan.stage_id.0.clone(),
            tool_id: plan.tool_id.0.clone(),
            event_name: "warn".to_string(),
            timestamp: Utc::now().to_rfc3339(),
            duration_ms: None,
            status: "warn".to_string(),
            trace_id: trace_id.to_string(),
            span_id: span_id.to_string(),
            attrs: serde_json::json!({
                "message": warning,
            }),
        })?;
    }
    let retention_report_path = if is_retention_stage(&plan.stage_id.0) {
        retention_counts_for_plan(input_paths, outputs)?.map(|counts| {
            write_retention_report_v1(
                run_artifacts_dir,
                &plan.stage_id.0,
                &plan.tool_id.0,
                &plan.tool_version,
                &retention_conditions_from_effective(
                    &plan.stage_id.0,
                    &plan.effective_params,
                    canonical_params,
                ),
                canonical_params,
                counts.reads_in,
                counts.reads_out,
                counts.bases_in,
                counts.bases_out,
            )
        })
    } else {
        None
    }
    .transpose()?;
    if let Some(retention_path) = retention_report_path.as_ref() {
        emit_artifact("retention_report", retention_path)?;
    }
    let effective_adapters_path = match adapter_bank {
        Some(bank) => write_effective_adapters_from_provenance(run_artifacts_dir, bank)?,
        None => None,
    };
    if let Some(path) = effective_adapters_path.as_ref() {
        emit_artifact("effective_adapters", path)?;
    }
    let mut extra_manifest_artifacts = Vec::new();
    if let Some(path) = effective_adapters_path.as_ref() {
        extra_manifest_artifacts.push(serde_json::json!({
            "name": "effective_adapters",
            "path": path,
        }));
    }
    if !log_paths.is_empty() {
        extra_manifest_artifacts.push(serde_json::json!({
            "name": "execution_logs",
            "paths": log_paths
                .iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>(),
        }));
    }
    let _observability_manifest = write_observability_manifest(
        run_artifacts_dir,
        &plan.stage_id.0,
        &plan.tool_id.0,
        &plan_artifacts.plan_path,
        &plan_artifacts.effective_config_path,
        &plan_artifacts.stage_config_path,
        tool_invocation_path,
        metrics_envelope_path,
        stage_metrics_path,
        &stage_report_path,
        retention_report_path.as_deref(),
        &extra_manifest_artifacts,
    )?;
    let _ = insert_stage_row(
        &run_artifacts_dir.join("run_index.jsonl"),
        &StageIndexRow {
            run_id: run_id.to_string(),
            stage_id: plan.stage_id.0.clone(),
            tool_id: plan.tool_id.0.clone(),
            params_hash: params_hash.to_string(),
            input_hash: input_hash.to_string(),
            output_hashes: output_hashes.to_vec(),
            artifacts: serde_json::json!({
                "plan": plan_artifacts.plan_path.display().to_string(),
                "effective_config": plan_artifacts.effective_config_path.display().to_string(),
                "stage_config": plan_artifacts.stage_config_path.display().to_string(),
                "metrics_envelope": metrics_envelope_path.display().to_string(),
                "stage_report": stage_report_path.display().to_string(),
                "retention_report": retention_report_path
                    .as_ref()
                    .map(|path| path.display().to_string()),
                "effective_adapters": effective_adapters_path
                    .as_ref()
                    .map(|path| path.display().to_string()),
            }),
        },
    );

    Ok(ReportArtifacts {
        subreports,
        assertion_results,
        invariant_verdict: invariant_eval.verdict,
        retention_report_path,
        effective_adapters_path,
        stage_report_path,
        quality_gate,
        adapter_validation,
        contaminant_action,
    })
}

struct RetentionCounts {
    reads_in: u64,
    reads_out: u64,
    bases_in: u64,
    bases_out: u64,
}

fn retention_counts_for_plan(
    input_paths: &[PathBuf],
    outputs: &[PathBuf],
) -> Result<Option<RetentionCounts>> {
    let input = stats_or_zero(input_paths.first().map(PathBuf::as_path))?;
    let output = stats_or_zero(outputs.first().map(PathBuf::as_path))?;
    Ok(Some(RetentionCounts {
        reads_in: input.reads,
        reads_out: output.reads,
        bases_in: input.bases,
        bases_out: output.bases,
    }))
}
