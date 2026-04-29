use std::path::PathBuf;

use anyhow::Result;
use bijux_dna_api::v1::api::report::{render_report, RenderReportRequest};
use bijux_dna_runtime::FactsRowV1;

fn fact() -> FactsRowV1 {
    FactsRowV1 {
        schema_version: "bijux.facts.v1".to_string(),
        run_id: "run-1".to_string(),
        stage_id: "fastq.trim_reads".to_string(),
        tool_id: "fastp".to_string(),
        tool_version: "0.23.4".to_string(),
        image_digest: Some("sha256:img".to_string()),
        trace_id: "trace-1".to_string(),
        span_id: "span-1".to_string(),
        params_hash: "sha256:param".to_string(),
        input_hash: "sha256:input".to_string(),
        output_hashes: vec!["sha256:output".to_string()],
        runtime_s: 1.0,
        memory_mb: 16.0,
        exit_code: 0,
        bank_hashes: serde_json::json!({"adapter_bank_hash": "sha256:adapter"}),
        reads_in: Some(10),
        reads_out: Some(9),
        bases_in: Some(100),
        bases_out: Some(90),
        pairs_in: None,
        pairs_out: None,
        metrics: serde_json::json!({}),
        reports: serde_json::json!({
            "stage_report": "stage_report.json"
        }),
        artifacts: serde_json::json!({}),
    }
}

#[test]
fn render_report_emits_evidence_bundle() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let facts_path = temp.path().join("facts.jsonl");
    let payload = format!("{}\n", serde_json::to_string(&fact())?);
    bijux_dna_infra::write_bytes(&facts_path, payload)?;
    bijux_dna_infra::atomic_write_json(
        &temp.path().join("run_manifest.json"),
        &serde_json::json!({
            "schema_version": "bijux.run_manifest.v3",
            "run_id": "run-1",
            "correlation_id": "corr-run-1",
            "graph_hash": "sha256:graph",
            "dataset_fingerprints": ["sha256:input"],
            "stages": [{ "stage_id": "fastq.trim_reads" }],
            "output_artifacts": []
        }),
    )?;
    bijux_dna_infra::atomic_write_json(
        &temp.path().join("defaults_ledger.json"),
        &serde_json::json!({
            "pipeline_id": "fastq-to-fastq__default__v1",
            "tools": {},
            "params": {},
            "thresholds": {},
            "tool_provenance": {},
            "param_provenance": {},
            "assumptions": [],
            "citations": {}
        }),
    )?;
    let response = render_report(&RenderReportRequest {
        base_dir: PathBuf::from(temp.path()),
        facts_path: facts_path.clone(),
    })?;
    assert!(response.report_path.exists());
    assert!(response.evidence_bundle_path.exists());
    Ok(())
}
