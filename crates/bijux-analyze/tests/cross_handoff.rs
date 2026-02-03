use anyhow::Result;
use bijux_analyze::write_run_report_from_facts;
use tempfile::TempDir;

#[test]
fn cross_domain_handoff_section_is_emitted() -> Result<()> {
    if std::env::var("BIJUX_E2E").is_err() {
        return Ok(());
    }
    let temp = TempDir::new()?;
    let base = temp.path();
    let manifest = serde_json::json!({
        "schema_version": "bijux.run_manifest.v2",
        "run_id": "run-1",
        "profile_id": "fastq-to-bam__default__v1",
        "domains": ["Fastq", "Cross", "Bam"],
        "stages": [],
        "domain_transitions": [{
            "from": "fastq",
            "to": "bam",
            "boundary": "run_artifacts/boundaries/alignment_boundary.json"
        }],
        "boundaries": [{
            "name": "alignment_boundary",
            "path": "run_artifacts/boundaries/alignment_boundary.json",
            "sha256": "sha256:dummy"
        }]
    });
    std::fs::write(
        base.join("run_manifest.json"),
        serde_json::to_vec_pretty(&manifest)?,
    )?;
    let report_path = write_run_report_from_facts(base, &[])?;
    let report_raw = std::fs::read_to_string(&report_path)?;
    let report_json: serde_json::Value = serde_json::from_str(&report_raw)?;
    let sections = report_json
        .get("sections")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| anyhow::anyhow!("missing report sections"))?;
    assert!(sections.contains_key("handoff"));
    Ok(())
}
