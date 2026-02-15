use std::path::PathBuf;

use anyhow::Result;
use bijux_dna_analyze::load::load_facts;
use bijux_dna_analyze::report::write_run_report_from_facts;
use bijux_dna_runtime::ReportSchemaV1;

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("pipelines")
        .join("fastq-to-fastq__default__v1")
}

#[test]
fn analyze_accepts_runtime_manifest_and_matches_report_schema() -> Result<()> {
    let root = fixture_root();
    let facts_path = root.join("facts.jsonl");
    let manifest_path = root.join("run_manifest.json");

    if !manifest_path.exists() {
        let manifest = serde_json::json!({
            "schema_version": "bijux.run_manifest.v3",
            "run_id": "fastq-to-fastq__default__v1",
            "pipeline_id": "fastq-to-fastq__default__v1",
            "profile_id": "fastq-to-fastq__default__v1",
            "graph_hash": "sha256:graph",
            "dataset_fingerprints": ["sha256:input"],
            "stage_contracts": {
                "fastq.trim": "sha256:contract"
            }
        });
        bijux_dna_infra::write_bytes(&manifest_path, serde_json::to_vec_pretty(&manifest)?)?;
    }

    let facts = load_facts(&facts_path).map_err(|err| anyhow::anyhow!(err.to_string()))?;
    let report_path = write_run_report_from_facts(&root, &facts)?;
    let report_raw = std::fs::read_to_string(report_path)?;
    let _report: ReportSchemaV1 = serde_json::from_str(&report_raw)?;
    Ok(())
}
