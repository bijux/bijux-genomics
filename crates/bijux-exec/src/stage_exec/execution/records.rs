
#[allow(clippy::too_many_arguments)]
fn write_facts_and_records(
    plan: &StagePlanV1,
    run_id: &str,
    run_artifacts_dir: &Path,
    trace_id: &str,
    span_id: &str,
    params_hash: &str,
    input_hash: &str,
    output_hashes: &[String],
    runtime_s: f64,
    memory_mb: f64,
    execution: &ExecutionEnvelope,
    stage_metrics: &serde_json::Value,
    metrics_envelope_path: &Path,
    log_paths: &[PathBuf],
    report_parts: &serde_json::Value,
    plugin_artifacts: &serde_json::Value,
    warnings: &[String],
    invariants: &[bijux_core::InvariantResultV1],
    verdict: Option<&bijux_core::StageVerdictV1>,
    reads_in: u64,
    reads_out: u64,
    bases_in: u64,
    bases_out: u64,
    pairs_in: u64,
    pairs_out: u64,
) -> Result<()> {
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
            bank_hashes: serde_json::json!({}),
            reads_in: Some(reads_in),
            reads_out: Some(reads_out),
            bases_in: Some(bases_in),
            bases_out: Some(bases_out),
            pairs_in: Some(pairs_in),
            pairs_out: Some(pairs_out),
            metrics: stage_metrics.clone(),
            reports: serde_json::json!({
                "parts": report_parts,
                "warnings": warnings,
                "invariants": invariants,
                "verdict": verdict,
                "scientific_preset": scientific_preset,
            }),
            artifacts: serde_json::json!({
                "metrics_envelope": metrics_envelope_path.display().to_string(),
                "stage_metrics": run_artifacts_dir
                    .join("stage_metrics.json")
                    .display()
                    .to_string(),
                "execution_logs": log_paths
                    .iter()
                    .map(|path| path.display().to_string())
                    .collect::<Vec<_>>(),
                "plugin_artifacts": plugin_artifacts,
            }),
        },
    )?;

    Ok(())
}
