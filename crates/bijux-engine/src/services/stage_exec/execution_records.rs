
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
